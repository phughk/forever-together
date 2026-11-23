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
