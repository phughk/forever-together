//! Ticket 0004: a run on the REAL iroh network (n0's relays and DNS), not the in-memory one.
//! Ignored by default because it needs the internet and takes minutes:
//!
//! `cargo test --lib live_tests -- --ignored --nocapture`
//!
//! Three apps run in this one process on one machine, each with its own data directory and key:
//! T (the trusted peer, never searches), A and B. A and B know T only by its public key in their
//! `trusted_peers` setting, so reaching it needs n0's address lookup. One machine means one
//! network and one NAT: this does NOT show hole punching between different networks, which stays
//! a two-network manual check (see docs/egm/0004-verify-iroh-live-network/design_draft.md).

use crate::code::Role;
use crate::code::test_encoder::{encode, sample_character};
use crate::config::Settings;
use crate::identity::Signer;
use crate::runtime::{Running, open_store, start_window};
use crate::server::core::{AppState, BootstrapSource};
use crate::server::static_files::MapAssets;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("wowft-live-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn http(r: &Running, method: &str, path: &str, body: &str) -> (u16, String) {
    let port = r.server.port();
    let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(
        s,
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {}\r\n\
         Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        r.server.token(),
        body.len()
    )
    .unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    (
        out[9..12].parse().unwrap(),
        out.split("\r\n\r\n").nth(1).unwrap_or("").to_owned(),
    )
}

fn state(r: &Running) -> AppState {
    let (s, b) = http(r, "GET", "/api/state", "");
    assert_eq!(s, 200, "{b}");
    serde_json::from_str(&b).unwrap()
}

fn start(d: &Path) -> Running {
    start_window(d, Arc::new(MapAssets::default())).unwrap()
}

/// Write `trusted` into the data directory's settings before the app starts.
fn trust(d: &Path, trusted: &str) {
    let (lock, store) = open_store(d).unwrap();
    let s = Settings {
        trusted_peers: vec![trusted.to_owned()],
        ..Settings::default()
    };
    s.save(store.as_ref()).unwrap();
    drop((lock, store));
}

fn search(r: &Running, name: &str, role: Role) {
    let mut c = sample_character();
    c.role = Some(role);
    c.first_name = name.to_owned();
    c.surname = format!("{name}son");
    let body = serde_json::json!({ "code": encode(&c) }).to_string();
    assert_eq!(http(r, "POST", "/api/character", &body).0, 200);
    let body = serde_json::json!({ "dungeons": ["the-deadmines"] }).to_string();
    assert_eq!(http(r, "POST", "/api/search", &body).0, 204);
}

/// Poll until `done` is true; print the elapsed time; return it (or None at the timeout).
fn wait(what: &str, limit: Duration, mut done: impl FnMut() -> bool) -> Option<Duration> {
    let t = Instant::now();
    while t.elapsed() < limit {
        if done() {
            eprintln!("LIVE {what}: {:.1} s", t.elapsed().as_secs_f32());
            return Some(t.elapsed());
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    eprintln!("LIVE {what}: NOT within {} s", limit.as_secs());
    None
}

fn others_seen(r: &Running) -> u32 {
    state(r).search.diagnosis.map_or(0, |d| d.seen.players)
}

#[test]
#[ignore = "needs the internet; run with --ignored --nocapture"]
fn real_network_bootstrap_remembered_peers_exchange_and_gossip() {
    crate::init_logging();
    let (dt, da, db) = (dir("t"), dir("a"), dir("b"));
    // T: an ordinary peer that everyone knows by key only.
    let t_key = {
        let (lock, store) = open_store(&dt).unwrap();
        let k = crate::identity::Identity::load_or_create(store.as_ref())
            .unwrap()
            .peer_id()
            .to_hex();
        drop((lock, store));
        k
    };
    let t = start(&dt);
    state(&t);
    // LIVE_T_SEARCHES=1: T joins the same topic as a searching player would.
    if std::env::var("LIVE_T_SEARCHES").is_ok() {
        search(&t, "Testt", Role::Damage);
    }
    eprintln!("LIVE T up, waiting 10 s for its address to be published");
    std::thread::sleep(Duration::from_secs(10));

    // Step 1: a fresh A reaches T by key through n0.
    trust(&da, &t_key);
    let a = start(&da);
    search(&a, "Testa", Role::Tank);
    let s1 = wait(
        "step 1 A bootstrap via trusted",
        Duration::from_secs(180),
        || state(&a).network.bootstrap == BootstrapSource::Trusted,
    );

    // Step 2: restart A; remembered peers come first.
    drop(a);
    let a = start(&da);
    // Bootstrap runs per topic, so A must be searching before it reports a source.
    search(&a, "Testa", Role::Tank);
    let s2 = wait(
        "step 2 A bootstrap via remembered",
        Duration::from_secs(90),
        || state(&a).network.bootstrap == BootstrapSource::Remembered,
    );

    // Step 3 and 4: a fresh B joins through T; A and B see each other's ads (signed gossip).
    trust(&db, &t_key);
    let b = start(&db);
    search(&b, "Testb", Role::Healer);
    let mut n = 0;
    let s3 = wait(
        "step 3/4 A and B see each other",
        Duration::from_secs(90),
        || {
            n += 1;
            let (sa, sb) = (state(&a), state(&b));
            if n % 20 == 1 {
                let sit = |s: &AppState| s.search.diagnosis.as_ref().map(|d| d.situation.clone());
                eprintln!(
                    "LIVE   A {:?} peers {} {:?} seen {} | B {:?} peers {} {:?} seen {}",
                    sa.network.bootstrap,
                    sa.network.peers,
                    sit(&sa),
                    others_seen(&a),
                    sb.network.bootstrap,
                    sb.network.peers,
                    sit(&sb),
                    others_seen(&b)
                );
            }
            others_seen(&a) >= 1 && others_seen(&b) >= 1
        },
    );

    // Step 3b: T goes away; A and B must keep seeing each other.
    drop(t);
    std::thread::sleep(Duration::from_secs(30));
    let (pa, pb) = (state(&a).network.peers, state(&b).network.peers);
    eprintln!(
        "LIVE after T stopped: A peers {pa}, B peers {pb}, seen A {} B {}",
        others_seen(&a),
        others_seen(&b)
    );

    for d in [dt, da, db] {
        let _ = std::fs::remove_dir_all(d);
    }
    assert!(s1.is_some(), "step 1: A never reached T by key");
    assert!(s2.is_some(), "step 2: remembered peers not used first");
    assert!(s3.is_some(), "step 3/4: A and B never saw each other");
    assert!(
        pa >= 1 && pb >= 1,
        "step 3b: A and B lost each other without T"
    );
}

/// Ticket 0049, measurement: time for a node that knows only a key to connect, with n0 DNS
/// only, the Mainline DHT only, and both (ADR 0041). Two endpoints on this machine, so the
/// connection itself is quick; what is measured is publish plus resolve. Run:
/// `cargo test --lib live_tests::lookup -- --ignored --nocapture`
#[test]
#[ignore = "needs the internet; run with --ignored --nocapture"]
fn lookup_time_for_dns_dht_and_both() {
    use iroh::address_lookup::{AddrFilter, DnsAddressLookup, PkarrPublisher, PkarrResolver};
    use iroh::endpoint::presets;
    use iroh::{Endpoint, RelayMode};
    use iroh_mainline_address_lookup::DhtAddressLookup;

    const ALPN: &[u8] = b"wowft/test/lookup";
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        for (name, dns, dht) in [
            ("dns", true, false),
            ("dht", false, true),
            ("both", true, true),
        ] {
            let build = || async {
                let mut b = Endpoint::builder(presets::Minimal)
                    .relay_mode(RelayMode::Default)
                    .alpns(vec![ALPN.to_vec()]);
                if dns {
                    b = b
                        .address_lookup(
                            PkarrPublisher::n0_dns().addr_filter(AddrFilter::unfiltered()),
                        )
                        .address_lookup(PkarrResolver::n0_dns())
                        .address_lookup(DnsAddressLookup::n0_dns());
                }
                if dht {
                    b = b.address_lookup(
                        DhtAddressLookup::builder().addr_filter(AddrFilter::unfiltered()),
                    );
                }
                b.bind().await.unwrap()
            };
            let (a, b) = (build().await, build().await);
            let a_id = a.id();
            let acc = tokio::spawn({
                let a = a.clone();
                async move {
                    while let Some(inc) = a.accept().await {
                        if let Ok(c) = inc.await {
                            c.closed().await;
                        }
                    }
                }
            });
            let t0 = Instant::now();
            let mut tries = 0;
            let ok = loop {
                tries += 1;
                let r = tokio::time::timeout(Duration::from_secs(10), b.connect(a_id, ALPN)).await;
                if let Ok(Ok(c)) = r {
                    c.close(0u32.into(), b"x");
                    break true;
                }
                if t0.elapsed() > Duration::from_secs(120) {
                    break false;
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            };
            eprintln!(
                "LIVE lookup {name}: connected={ok} after {:.1} s ({tries} tries)",
                t0.elapsed().as_secs_f32()
            );
            acc.abort();
            let _ = tokio::time::timeout(Duration::from_secs(3), a.close()).await;
            let _ = tokio::time::timeout(Duration::from_secs(3), b.close()).await;
        }
    });
}

/// Two `IrohNet`s on the real network with the app's `Default` configuration: B dials A by key.
#[test]
#[ignore = "needs the internet; run with --ignored --nocapture"]
fn default_config_dials_by_key() {
    use crate::identity::Identity;
    use crate::net::bootstrap::Dialer;
    use crate::net::{IrohConfig, IrohNet, KnownPeers};
    use crate::store::MemStore;
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let known = || KnownPeers::new(100, Arc::new(MemStore::new()));
        let ia = Arc::new(Identity::generate());
        let ib = Arc::new(Identity::generate());
        let a = IrohNet::start(ia.clone(), known(), IrohConfig::Default)
            .await
            .unwrap();
        let mut b = IrohNet::start(ib, known(), IrohConfig::Default)
            .await
            .unwrap();
        let t0 = Instant::now();
        let mut ok = false;
        while t0.elapsed() < Duration::from_secs(60) {
            if Dialer::dial(&mut b, ia.peer_id()).await.is_ok() {
                ok = true;
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        eprintln!(
            "LIVE default dial by key: ok={ok} after {:.1} s",
            t0.elapsed().as_secs_f32()
        );
        drop(a);
    });
}
