//! Plan step 7, automated part: several app instances (each a `LiveCore` with its own store and
//! its own web server) on one shared in-memory network. Everything a player does goes through
//! the HTTP API: paste a code, search, Confirm. The test walks widening, a committed party,
//! Ready, Confirm and the name reveal, and checks that a Horde character never matches.
//!
//! The clock is the real clock running `SCALE` times faster, so the 60 s widening steps and the
//! 10 s epochs pass in a few seconds. Real phones, a real window and the live game are manual
//! checks.

use crate::app::{CoreDeps, LiveCore};
use crate::code::test_encoder::{encode, sample_character};
use crate::code::{Faction, Role};
use crate::engine::NetError;
use crate::identity::{Identity, Signer};
use crate::net::FakeHub;
use crate::server::core::*;
use crate::server::static_files::MapAssets;
use crate::server::{Server, ServerConfig};
use crate::store::MemStore;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::watch;

const DUNGEON: &str = "the-deadmines";

struct Instance {
    name: &'static str,
    peer: crate::identity::PeerId,
    server: Server,
    token: String,
    _core: Arc<LiveCore>,
    store: Arc<dyn crate::store::Store>,
    _status: watch::Sender<NetworkView>,
}

/// A raw HTTP/1.1 call with a bearer token. Returns the status and the body.
async fn call(i: &Instance, method: &str, path: &str, body: Option<&str>) -> (u16, String) {
    let port = i.server.port();
    let mut s = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let body = body.unwrap_or("");
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {}\r\n\
         Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        i.token,
        body.len()
    );
    s.write_all(req.as_bytes()).await.unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).await.unwrap();
    let status = out[9..12].parse().unwrap();
    let body = out.split("\r\n\r\n").nth(1).unwrap_or("").to_owned();
    (status, body)
}

async fn state(i: &Instance) -> AppState {
    let (s, b) = call(i, "GET", "/api/state", None).await;
    assert_eq!(s, 200, "{b}");
    serde_json::from_str(&b).unwrap_or_else(|e| panic!("{e}: {b}"))
}

async fn spawn_instance(
    hub: &FakeHub,
    clock: crate::app::Clock,
    name: &'static str,
    role: Role,
    faction: Faction,
    item_level: u16,
) -> (Instance, String) {
    let id = Arc::new(Identity::generate());
    let store: Arc<dyn crate::store::Store> = Arc::new(MemStore::new());
    let net = hub.connect(id.clone() as Arc<dyn Signer>);
    let (tx, rx) = watch::channel(NetworkView {
        peers: 0,
        bootstrap: BootstrapSource::None,
    });
    let core = Arc::new(LiveCore::spawn(
        CoreDeps {
            store: store.clone(),
            me: id.peer_id(),
            clock,
            net_status: rx,
        },
        move || async move { Ok::<_, NetError>(net) },
    ));
    let server = Server::start(
        core.clone(),
        store.clone(),
        Arc::new(MapAssets::default()),
        ServerConfig {
            web_dir: None,
            heartbeat: Duration::from_secs(15),
            lan: false,
            port: 0,
            lan_ips: Arc::new(Vec::new),
        },
    )
    .await
    .unwrap();
    let token = server.token();
    let mut c = sample_character();
    c.role = Some(role);
    c.faction = faction;
    c.item_level = item_level;
    c.first_name = name.to_owned();
    c.surname = format!("{name}son");
    (
        Instance {
            name,
            peer: id.peer_id(),
            server,
            token,
            store,
            _core: core,
            _status: tx,
        },
        encode(&c),
    )
}

/// A shared network and a clock running `scale` times faster than the real one.
fn world(scale: u32) -> (FakeHub, crate::app::Clock) {
    let base = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let t0 = Instant::now();
    let clock: crate::app::Clock =
        Arc::new(move || Duration::from_secs_f64(base + t0.elapsed().as_secs_f64() * scale as f64));
    (FakeHub::new(), clock)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn five_alliance_players_form_a_party_and_the_horde_player_does_not() {
    let (hub, clock) = world(12);

    // The last damage dealer's item level is 25% off, so he only fits at stage 2 (30%).
    let cast = [
        ("Tank", Role::Tank, Faction::Alliance, 120),
        ("Healer", Role::Healer, Faction::Alliance, 120),
        ("Dpsone", Role::Damage, Faction::Alliance, 120),
        ("Dpstwo", Role::Damage, Faction::Alliance, 120),
        ("Dpsthree", Role::Damage, Faction::Alliance, 150),
        ("Horde", Role::Damage, Faction::Horde, 120),
    ];
    let mut all = Vec::new();
    for (name, role, faction, ilvl) in cast {
        let (inst, code) = spawn_instance(&hub, clock.clone(), name, role, faction, ilvl).await;
        let body = serde_json::json!({ "code": code }).to_string();
        let (s, b) = call(&inst, "POST", "/api/character", Some(&body)).await;
        assert_eq!(s, 200, "{name}: {b}");
        all.push(inst);
    }
    let body = serde_json::json!({ "dungeons": [DUNGEON] }).to_string();
    for i in &all {
        let (s, b) = call(i, "POST", "/api/search", Some(&body)).await;
        assert_eq!(s, 204, "{}: {b}", i.name);
    }
    let alliance = 5;

    // Walk widening, wait for a committed party, and Confirm as each player would.
    let deadline = Instant::now() + Duration::from_secs(90);
    let mut max_stage_without_party = 0;
    let mut confirmed = vec![false; alliance];
    let mut saw_ready_all = false;
    let mut finals: Vec<Option<PartyView>> = vec![None; alliance];
    while Instant::now() < deadline {
        for (n, i) in all.iter().take(alliance).enumerate() {
            let s = state(i).await;
            match s.party {
                None => max_stage_without_party = max_stage_without_party.max(s.search.stage),
                Some(p) if p.phase == PartyPhase::Confirming => {
                    assert_eq!(p.members.len(), 5);
                    if p.members.iter().all(|m| m.ready) {
                        saw_ready_all = true;
                    }
                    if !confirmed[n] {
                        let (st, b) = call(i, "POST", "/api/party/confirm", None).await;
                        assert_eq!(st, 204, "{}: {b}", i.name);
                        confirmed[n] = true;
                    }
                }
                Some(p) if p.phase == PartyPhase::Final => finals[n] = Some(p),
                Some(p) => panic!("{} saw an unexpected party phase {:?}", i.name, p.phase),
            }
        }
        if finals.iter().all(Option::is_some) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert!(
        finals.iter().all(Option::is_some),
        "no final party before the deadline; confirmed {confirmed:?}"
    );
    assert!(
        max_stage_without_party >= 2,
        "the party formed before widening reached stage 2 (saw stage {max_stage_without_party})"
    );
    assert!(saw_ready_all, "never saw every member Ready");

    // Names are revealed in the final party, and every member sees the same five.
    let mut expected: Vec<String> = all
        .iter()
        .take(alliance)
        .map(|i| format!("{}/{}son", i.name, i.name))
        .collect();
    expected.sort();
    for (n, f) in finals.iter().enumerate() {
        let p = f.as_ref().unwrap();
        let mut names: Vec<String> = p
            .members
            .iter()
            .map(|m| {
                format!(
                    "{}/{}",
                    m.first_name
                        .as_deref()
                        .expect("first name shown when final"),
                    m.surname.as_deref().expect("surname shown when final")
                )
            })
            .collect();
        names.sort();
        assert_eq!(names, expected, "member {}", all[n].name);
        assert_eq!(p.members.iter().filter(|m| m.is_self).count(), 1);
        assert!(p.members.iter().all(|m| m.confirmed));
    }
    // Other members' role and level come from their ads.
    let tank_view = finals[0].as_ref().unwrap();
    let healer = tank_view
        .members
        .iter()
        .find(|m| m.first_name.as_deref() == Some("Healer"))
        .unwrap();
    assert_eq!(healer.role, RoleView::Healer);
    assert_eq!(healer.level, 24);
    assert_eq!(healer.item_level, 120);

    // The Horde player is on other topics: still searching, never in a party.
    let h = state(&all[5]).await;
    assert!(h.party.is_none());
    assert_eq!(h.search.status, SearchStatus::Searching);
}

/// A member that answered Ready and then went silent is found by the host's heartbeat: the
/// others see the party dropped as unreachable. Real-speed clock, so the 4 s window is real.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_member_that_goes_silent_is_dropped_by_heartbeat() {
    let (hub, clock) = world(1);
    let cast = [
        ("Tank", Role::Tank),
        ("Healer", Role::Healer),
        ("Dpsone", Role::Damage),
        ("Dpstwo", Role::Damage),
        ("Dpsthree", Role::Damage),
    ];
    let mut all = Vec::new();
    for (name, role) in cast {
        let (inst, code) =
            spawn_instance(&hub, clock.clone(), name, role, Faction::Alliance, 120).await;
        let body = serde_json::json!({ "code": code }).to_string();
        assert_eq!(
            call(&inst, "POST", "/api/character", Some(&body)).await.0,
            200
        );
        all.push(inst);
    }
    let body = serde_json::json!({ "dungeons": [DUNGEON] }).to_string();
    for i in &all {
        assert_eq!(call(i, "POST", "/api/search", Some(&body)).await.0, 204);
    }
    // Wait until all five see everyone Ready, then the last one disappears.
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        assert!(Instant::now() < deadline, "no committed party");
        let mut n = 0;
        for i in &all {
            if let Some(p) = state(i).await.party
                && p.phase == PartyPhase::Confirming
                && p.members.iter().all(|m| m.ready)
            {
                n += 1;
            }
        }
        if n == 5 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let dead = all.pop().unwrap();
    let dead_name = dead.name;
    hub.disconnect(dead.peer);
    let mut reasons = vec![None; all.len()];
    let deadline = Instant::now() + Duration::from_secs(14);
    while Instant::now() < deadline && reasons.iter().any(Option::is_none) {
        for (n, i) in all.iter().enumerate() {
            if let Some(p) = state(i).await.party
                && p.phase == PartyPhase::Dropped
            {
                reasons[n] = p.drop_reason;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        reasons.iter().all(|r| *r == Some(DropReason::Unreachable)),
        "{dead_name} went silent but the drop reasons were {reasons:?}"
    );
}

/// A search that runs to the limit leaves one history entry, and `DELETE /api/history` removes it
/// (ADR 0027). The clock runs 60 times faster than the real one.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_expired_search_is_recorded_and_delete_history_clears_it() {
    let (hub, clock) = world(60);
    let (i, code) = spawn_instance(&hub, clock, "Testa", Role::Tank, Faction::Alliance, 120).await;
    let body = serde_json::json!({ "code": code }).to_string();
    assert_eq!(call(&i, "POST", "/api/character", Some(&body)).await.0, 200);
    let body = serde_json::json!({ "dungeons": [DUNGEON] }).to_string();
    assert_eq!(call(&i, "POST", "/api/search", Some(&body)).await.0, 204);
    let deadline = Instant::now() + Duration::from_secs(40);
    while state(&i).await.search.status != SearchStatus::LimitReached {
        assert!(Instant::now() < deadline, "the search never expired");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let rows = crate::history::load(i.store.as_ref()).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].1.outcome, crate::history::Outcome::Expired);
    assert_eq!(rows[0].1.dungeon, DUNGEON);
    assert_eq!(call(&i, "DELETE", "/api/history", None).await.0, 204);
    assert!(crate::history::load(i.store.as_ref()).unwrap().is_empty());
}
