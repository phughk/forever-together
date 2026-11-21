//! Server tests from the verification plan: 401 without a token, 403 for a wrong Host, Origin
//! or non-loopback caller on a loopback route, override priority, path escape refused, `/api`
//! not overridable, token reset, the pairing URL, event streaming. (Command-line flags are
//! tested in `cli`.)

use super::core::*;
use super::static_files::MapAssets;
use super::*;
use crate::store::MemStore;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Method, Request, StatusCode};
use std::sync::Mutex;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tower::ServiceExt;

pub(crate) fn sample_state() -> AppState {
    AppState {
        server_now: 0,
        character: None,
        search: SearchView {
            status: SearchStatus::Idle,
            dungeons: vec![],
            started_at: None,
            stage: 0,
            stage_clock_started_at: None,
            searching: None,
            topics: 0,
            diagnosis: None,
        },
        party: None,
        network: NetworkView {
            peers: 0,
            bootstrap: BootstrapSource::None,
        },
        settings: SettingsView {
            lan_mode: false,
            lan_address: None,
            known_peers_size: 100,
            trusted_peers: vec![],
        },
    }
}

/// A core that records calls and keeps its state in a watch channel.
pub(crate) struct FakeCore {
    pub tx: watch::Sender<AppState>,
    pub calls: Mutex<Vec<String>>,
    pub addon: Mutex<Option<String>>,
}

impl FakeCore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            tx: watch::channel(sample_state()).0,
            calls: Mutex::new(vec![]),
            addon: Mutex::new(None),
        })
    }
    fn log(&self, s: impl Into<String>) {
        self.calls.lock().unwrap().push(s.into());
    }
    pub fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

impl AppCore for FakeCore {
    fn state(&self) -> AppState {
        self.tx.borrow().clone()
    }
    fn subscribe(&self) -> watch::Receiver<AppState> {
        self.tx.subscribe()
    }
    async fn set_character(&self, code: String) -> Result<CharacterView, CoreError> {
        self.log(format!("character {code}"));
        if code == "bad" {
            return Err(CoreError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "BadCheck",
                "typo",
            ));
        }
        Ok(CharacterView {
            first_name: "Aldric".into(),
            surname: "Thornwood".into(),
            faction: FactionView::Alliance,
            class_name: None,
            race_name: None,
            level: 24,
            item_level: 120,
            role: RoleView::Tank,
            realm: "5".into(),
            realm_name: "realm 5".into(),
            region: 3,
            region_name: "EU".into(),
            zone: "Westfall".into(),
            zone_known: true,
            created_at: 1,
        })
    }
    async fn clear_character(&self) -> Result<(), CoreError> {
        self.log("clear_character");
        Ok(())
    }
    async fn clear_history(&self) -> Result<(), CoreError> {
        self.log("clear_history");
        Ok(())
    }
    async fn dungeons(&self) -> DungeonsResponse {
        DungeonsResponse {
            dungeons: vec![],
            recommended: vec![],
        }
    }
    async fn start_search(&self, d: Vec<String>) -> Result<(), CoreError> {
        self.log(format!("start {d:?}"));
        Ok(())
    }
    async fn stop_search(&self) -> Result<(), CoreError> {
        self.log("stop");
        Ok(())
    }
    async fn confirm_party(&self) -> Result<(), CoreError> {
        self.log("confirm");
        Ok(())
    }
    async fn decline_party(&self) -> Result<(), CoreError> {
        self.log("decline");
        Ok(())
    }
    fn settings(&self) -> SettingsView {
        self.tx.borrow().settings.clone()
    }
    async fn update_settings(&self, p: SettingsPatch) -> Result<SettingsView, CoreError> {
        self.log(format!("settings {p:?}"));
        self.tx.send_modify(|s| {
            if let Some(l) = p.lan_mode {
                s.settings.lan_mode = l;
            }
            if let Some(k) = p.known_peers_size {
                s.settings.known_peers_size = k;
            }
            if let Some(t) = p.trusted_peers {
                s.settings.trusted_peers = t;
            }
        });
        Ok(self.settings())
    }
    async fn addon_dir(&self) -> Option<String> {
        self.addon.lock().unwrap().clone()
    }
    async fn set_addon_dir(&self, dir: Option<String>) -> Result<(), CoreError> {
        self.log(format!("addon_dir {dir:?}"));
        *self.addon.lock().unwrap() = dir;
        Ok(())
    }
}

const PORT: u16 = 4000;

struct H {
    router: Router,
    shared: Arc<Shared<FakeCore>>,
    core: Arc<FakeCore>,
    token: String,
}

fn harness_with(web_dir: Option<PathBuf>, assets: &[(&str, &str)]) -> H {
    harness_full(web_dir, assets, Vec::new())
}

fn harness_full(web_dir: Option<PathBuf>, assets: &[(&str, &str)], cands: Vec<PathBuf>) -> H {
    let core = FakeCore::new();
    let store: Arc<dyn Store> = Arc::new(MemStore::new());
    let guard = Arc::new(Guard::new(TokenManager::load_or_create(store).unwrap()));
    guard.port.store(PORT, Ordering::Relaxed);
    let (ctl, _rx) = mpsc::unbounded_channel();
    let token = guard.tokens.current();
    let shared = Arc::new(Shared {
        core: core.clone(),
        guard,
        assets: Arc::new(MapAssets(
            assets
                .iter()
                .map(|(k, v)| (k.to_string(), v.as_bytes().to_vec()))
                .collect(),
        )),
        web_dir,
        heartbeat: Duration::from_millis(50),
        lan_ips: Arc::new(|| vec![Ipv4Addr::new(192, 168, 1, 20), Ipv4Addr::new(10, 0, 0, 7)]),
        addon_candidates: Arc::new(move || cands.clone()),
        addon_lock: Arc::new(std::sync::Mutex::new(())),
        listener_generation: watch::channel(0).0,
        ctl,
    });
    H {
        router: routes::build_router(shared.clone()),
        shared,
        core,
        token,
    }
}

fn harness() -> H {
    harness_with(
        None,
        &[("index.html", "<html>embedded</html>"), ("app.js", "js")],
    )
}

const LOCAL: &str = "127.0.0.1:50000";
const REMOTE: &str = "192.168.1.9:50000";

struct Rq {
    method: Method,
    path: String,
    host: String,
    origin: Option<String>,
    token: Option<String>,
    body: String,
    peer: &'static str,
}

fn rq(method: Method, path: &str) -> Rq {
    Rq {
        method,
        path: path.into(),
        host: format!("127.0.0.1:{PORT}"),
        origin: None,
        token: None,
        body: String::new(),
        peer: LOCAL,
    }
}

impl Rq {
    fn token(mut self, t: &str) -> Self {
        self.token = Some(t.into());
        self
    }
    fn host(mut self, h: &str) -> Self {
        self.host = h.into();
        self
    }
    fn origin(mut self, o: &str) -> Self {
        self.origin = Some(o.into());
        self
    }
    fn body(mut self, b: impl Into<String>) -> Self {
        self.body = b.into();
        self
    }
    fn peer(mut self, p: &'static str) -> Self {
        self.peer = p;
        self
    }
}

async fn send(h: &H, r: Rq) -> (StatusCode, String, axum::http::HeaderMap) {
    let mut b = Request::builder()
        .method(r.method)
        .uri(r.path)
        .header("host", r.host);
    if let Some(o) = r.origin {
        b = b.header("origin", o);
    }
    if let Some(t) = r.token {
        b = b.header("authorization", format!("Bearer {t}"));
    }
    let mut req = b.body(Body::from(r.body)).unwrap();
    req.extensions_mut()
        .insert(ConnectInfo(r.peer.parse::<SocketAddr>().unwrap()));
    let resp = h.router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let headers = resp.headers().clone();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    (
        status,
        String::from_utf8_lossy(&bytes).into_owned(),
        headers,
    )
}

fn err_code(body: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(body).unwrap();
    assert!(v["message"].is_string(), "error body has a message: {body}");
    v["error"].as_str().unwrap().to_owned()
}

// ---- auth -----------------------------------------------------------------------------

#[tokio::test]
async fn api_without_or_with_wrong_token_is_401() {
    let h = harness();
    for path in [
        "/api/state",
        "/api/dungeons",
        "/api/settings",
        "/api/events",
        "/api/nothing",
    ] {
        let (s, b, _) = send(&h, rq(Method::GET, path)).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED, "{path}");
        assert_eq!(err_code(&b), "unauthorized");
        let (s, _, _) = send(&h, rq(Method::GET, path).token("nope")).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED, "{path} wrong token");
    }
    // Same length, one character off.
    let mut almost = h.token.clone();
    almost.replace_range(0..1, if h.token.starts_with('0') { "1" } else { "0" });
    let (s, _, _) = send(&h, rq(Method::GET, "/api/state").token(&almost)).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    // The token in a query string or a non-Bearer scheme does not count.
    let (s, _, _) = send(
        &h,
        rq(Method::GET, &format!("/api/state?token={}", h.token)),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn valid_token_reads_state_in_the_client_shape() {
    let h = harness();
    let (s, b, _) = send(&h, rq(Method::GET, "/api/state").token(&h.token)).await;
    assert_eq!(s, StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&b).unwrap();
    assert!(v["serverNow"].as_u64().unwrap() > 1_000_000_000_000);
    assert_eq!(v["search"]["status"], "idle");
    assert_eq!(v["settings"]["lan_mode"], false);
    assert_eq!(v["settings"]["known_peers_size"], 100);
    assert_eq!(v["network"]["bootstrap"], "none");
    assert!(v["character"].is_null() && v["party"].is_null());
}

#[tokio::test]
async fn search_diagnosis_has_the_contract_shape() {
    let mut state = sample_state();
    assert!(
        serde_json::to_value(&state).unwrap()["search"]["diagnosis"].is_null(),
        "idle: null"
    );
    state.search.status = SearchStatus::LimitReached;
    state.search.diagnosis = Some(DiagnosisView {
        situation: "no_tank".into(),
        outlook: None,
        ended: true,
        seen: SeenView {
            players: 8,
            tanks: 0,
            healers: 3,
            damage: 5,
        },
        own_role: Some("damage".into()),
        sample_full: false,
        network_peers: 4,
        attempts: AttemptsView {
            invites_sent: 9,
            accepts_got: 0,
            invites_got: 2,
            taken_seen: 3,
            drops: 1,
            declines: 1,
        },
    });
    let v = serde_json::to_value(&state).unwrap();
    assert_eq!(
        v["search"]["diagnosis"],
        serde_json::json!({
            "situation": "no_tank",
            "outlook": null,
            "ended": true,
            "seen": {"players": 8, "tanks": 0, "healers": 3, "damage": 5},
            "ownRole": "damage",
            "sampleFull": false,
            "networkPeers": 4,
            "attempts": {
                "invitesSent": 9, "acceptsGot": 0, "invitesGot": 2,
                "takenSeen": 3, "drops": 1, "declines": 1
            }
        })
    );
    // The outlook is one of three codes, camelCase-free, and a payload without it parses as none.
    let mut with = state.clone();
    with.search.diagnosis.as_mut().unwrap().outlook = Some("sometimes_found".into());
    let w = serde_json::to_value(&with).unwrap();
    assert_eq!(w["search"]["diagnosis"]["outlook"], "sometimes_found");
    let mut without = w.clone();
    without["search"]["diagnosis"]
        .as_object_mut()
        .unwrap()
        .remove("outlook");
    let back: AppState = serde_json::from_value(without).unwrap();
    assert_eq!(back.search.diagnosis.unwrap().outlook, None);
    // A diagnosis from before the counters parses with zeros.
    let mut older = v.clone();
    older["search"]["diagnosis"]
        .as_object_mut()
        .unwrap()
        .remove("attempts");
    let back: AppState = serde_json::from_value(older).unwrap();
    assert_eq!(
        back.search.diagnosis.unwrap().attempts,
        AttemptsView::default()
    );
    // An old client payload without the field still parses.
    let mut old = serde_json::to_value(sample_state()).unwrap();
    old["search"].as_object_mut().unwrap().remove("diagnosis");
    let back: AppState = serde_json::from_value(old).unwrap();
    assert!(back.search.diagnosis.is_none());
}

#[tokio::test]
async fn wrong_host_or_origin_is_403_even_with_a_token() {
    let h = harness();
    for host in [
        "evil.example:4000",
        "127.0.0.1:9999",
        "192.168.1.9:4000",
        "localhost",
    ] {
        let (s, b, _) = send(&h, rq(Method::GET, "/api/state").token(&h.token).host(host)).await;
        assert_eq!(s, StatusCode::FORBIDDEN, "{host}");
        assert_eq!(err_code(&b), "bad_host");
    }
    // Static files are checked too.
    let (s, _, _) = send(&h, rq(Method::GET, "/").host("evil.example:4000")).await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    for origin in [
        "http://evil.example",
        "null",
        "https://127.0.0.1:4000",
        "http://localhost:4000",
    ] {
        let (s, b, _) = send(
            &h,
            rq(Method::GET, "/api/state").token(&h.token).origin(origin),
        )
        .await;
        assert_eq!(s, StatusCode::FORBIDDEN, "{origin}");
        assert_eq!(err_code(&b), "bad_origin");
    }
    let (s, _, _) = send(
        &h,
        rq(Method::GET, "/api/state")
            .token(&h.token)
            .origin("http://127.0.0.1:4000"),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
}

#[tokio::test]
async fn no_cors_headers_are_sent() {
    let h = harness();
    let (_, _, headers) = send(&h, rq(Method::GET, "/api/state").token(&h.token)).await;
    assert!(
        headers
            .keys()
            .all(|k| !k.as_str().starts_with("access-control-"))
    );
}

#[tokio::test]
async fn loopback_only_routes_refuse_a_remote_caller() {
    let h = harness();
    h.shared.guard.lan.store(true, Ordering::Relaxed);
    let remote = |m: Method, p: &str| {
        rq(m, p)
            .token(&h.token)
            .host(&format!("192.168.1.20:{PORT}"))
            .peer(REMOTE)
    };
    for (m, p) in [
        (Method::GET, "/api/pairing"),
        (Method::POST, "/api/token/reset"),
        (Method::DELETE, "/api/history"),
    ] {
        let (s, b, _) = send(&h, remote(m, p)).await;
        assert_eq!(s, StatusCode::FORBIDDEN, "{p}");
        assert_eq!(err_code(&b), "loopback_only");
    }
    let old = h.token.clone();
    assert!(h.shared.guard.tokens.verify(&old), "token untouched");
    for body in [r#"{"lan_mode":true}"#, r#"{"trusted_peers":[]}"#] {
        let (s, b, _) = send(&h, remote(Method::PUT, "/api/settings").body(body)).await;
        assert_eq!(s, StatusCode::FORBIDDEN, "{body}");
        assert_eq!(err_code(&b), "loopback_only");
    }
    assert!(h.core.calls().is_empty());
    // A remote caller may change the other settings and read everything else.
    let (s, _, _) = send(
        &h,
        remote(Method::PUT, "/api/settings").body(r#"{"known_peers_size":50}"#),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let (s, _, _) = send(&h, remote(Method::GET, "/api/state")).await;
    assert_eq!(s, StatusCode::OK);
    // Token is checked before the loopback rule.
    let (s, _, _) = send(
        &h,
        rq(Method::GET, "/api/pairing")
            .host(&format!("192.168.1.20:{PORT}"))
            .peer(REMOTE),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_history_is_loopback_only_and_needs_a_token() {
    let h = harness();
    h.shared.guard.lan.store(true, Ordering::Relaxed);
    let (s, _, _) = send(&h, rq(Method::DELETE, "/api/history")).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    let (s, b, _) = send(
        &h,
        rq(Method::DELETE, "/api/history")
            .token(&h.token)
            .host(&format!("192.168.1.20:{PORT}"))
            .peer(REMOTE),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    assert_eq!(err_code(&b), "loopback_only");
    assert!(h.core.calls().is_empty());
    let (s, b, _) = send(&h, rq(Method::DELETE, "/api/history").token(&h.token)).await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    assert!(b.is_empty());
    assert_eq!(h.core.calls(), ["clear_history"]);
}

#[tokio::test]
async fn token_reset_invalidates_the_old_token() {
    let h = harness();
    let (s, b, _) = send(&h, rq(Method::POST, "/api/token/reset").token(&h.token)).await;
    assert_eq!(s, StatusCode::OK);
    let new = serde_json::from_str::<serde_json::Value>(&b).unwrap()["token"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(new.len(), 64);
    assert_ne!(new, h.token);
    let (s, _, _) = send(&h, rq(Method::GET, "/api/state").token(&h.token)).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    let (s, _, _) = send(&h, rq(Method::GET, "/api/state").token(&new)).await;
    assert_eq!(s, StatusCode::OK);
}

#[tokio::test]
async fn rate_limit_is_twenty_per_second_per_token() {
    let h = harness();
    let mut ok = 0;
    let mut limited = 0;
    for _ in 0..60 {
        let (s, b, _) = send(&h, rq(Method::GET, "/api/state").token(&h.token)).await;
        match s {
            StatusCode::OK => ok += 1,
            StatusCode::TOO_MANY_REQUESTS => {
                assert_eq!(err_code(&b), "rate_limited");
                limited += 1;
            }
            other => panic!("unexpected {other}"),
        }
    }
    assert!((20..=24).contains(&ok), "burst allowed {ok}");
    assert!(limited >= 36);
}

// ---- commands ---------------------------------------------------------------------------

#[tokio::test]
async fn commands_reach_the_core_and_errors_use_the_error_body() {
    let h = harness();
    let t = h.token.clone();
    let (s, b, _) = send(
        &h,
        rq(Method::POST, "/api/character")
            .token(&t)
            .body(r#"{"code":"WFT1:X"}"#),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(b.contains("\"firstName\":\"Aldric\"") && b.contains("\"zoneKnown\":true"));
    let (s, b, _) = send(
        &h,
        rq(Method::POST, "/api/character")
            .token(&t)
            .body(r#"{"code":"bad"}"#),
    )
    .await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(err_code(&b), "BadCheck");
    for (m, p) in [
        (Method::DELETE, "/api/character"),
        (Method::DELETE, "/api/search"),
        (Method::POST, "/api/party/confirm"),
        (Method::POST, "/api/party/decline"),
    ] {
        let (s, _, _) = send(&h, rq(m, p).token(&t)).await;
        assert_eq!(s, StatusCode::NO_CONTENT, "{p}");
    }
    let (s, _, _) = send(
        &h,
        rq(Method::POST, "/api/search")
            .token(&t)
            .body(r#"{"dungeons":["a","b"]}"#),
    )
    .await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    assert_eq!(
        h.core.calls(),
        [
            "character WFT1:X",
            "character bad",
            "clear_character",
            "stop",
            "confirm",
            "decline",
            "start [\"a\", \"b\"]"
        ]
    );
    let (s, _, _) = send(&h, rq(Method::GET, "/api/dungeons").token(&t)).await;
    assert_eq!(s, StatusCode::OK);
}

#[tokio::test]
async fn bodies_over_8_kib_are_refused_and_bad_json_is_400() {
    let h = harness();
    let t = h.token.clone();
    let big = format!(r#"{{"code":"{}"}}"#, "A".repeat(9000));
    let (s, b, _) = send(&h, rq(Method::POST, "/api/character").token(&t).body(big)).await;
    assert_eq!(s, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(err_code(&b), "payload_too_large");
    let just_under = format!(r#"{{"code":"{}"}}"#, "A".repeat(8000));
    let (s, _, _) = send(
        &h,
        rq(Method::POST, "/api/character")
            .token(&t)
            .body(just_under),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let (s, b, _) = send(
        &h,
        rq(Method::POST, "/api/character").token(&t).body("{nope"),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert_eq!(err_code(&b), "bad_json");
    let (s, _, _) = send(
        &h,
        rq(Method::PUT, "/api/settings")
            .token(&t)
            .body(r#"{"surprise":1}"#),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert_eq!(h.core.calls().len(), 1);
}

// ---- pairing ----------------------------------------------------------------------------

#[tokio::test]
async fn pairing_url_uses_the_lan_address_port_and_token() {
    let h = harness();
    // LAN mode off: nothing to pair with.
    let (s, b, _) = send(&h, rq(Method::GET, "/api/pairing").token(&h.token)).await;
    assert_eq!(s, StatusCode::CONFLICT);
    assert_eq!(err_code(&b), "lan_mode_off");
    h.core.tx.send_modify(|s| s.settings.lan_mode = true);
    let (s, b, _) = send(&h, rq(Method::GET, "/api/pairing").token(&h.token)).await;
    assert_eq!(s, StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&b).unwrap();
    assert_eq!(
        v["url"],
        format!("http://192.168.1.20:{PORT}/#token={}", h.token)
    );
    assert_eq!(v["address"], "192.168.1.20");
    assert_eq!(
        v["addresses"],
        serde_json::json!(["192.168.1.20", "10.0.0.7"])
    );

    // A listed address can be chosen; an unknown one falls back to the default.
    let (_, b, _) = send(
        &h,
        rq(Method::GET, "/api/pairing?ip=10.0.0.7").token(&h.token),
    )
    .await;
    let v: serde_json::Value = serde_json::from_str(&b).unwrap();
    assert_eq!(
        v["url"],
        format!("http://10.0.0.7:{PORT}/#token={}", h.token)
    );
    let (_, b, _) = send(
        &h,
        rq(Method::GET, "/api/pairing?ip=8.8.8.8").token(&h.token),
    )
    .await;
    let v: serde_json::Value = serde_json::from_str(&b).unwrap();
    assert_eq!(v["address"], "192.168.1.20");
}

// ---- static files -----------------------------------------------------------------------

fn temp_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("wowft-server-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[tokio::test]
async fn embedded_files_need_no_token_and_unknown_routes_fall_back_to_index() {
    let h = harness();
    let (s, b, hd) = send(&h, rq(Method::GET, "/")).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(b, "<html>embedded</html>");
    assert!(
        hd["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/html")
    );
    assert_eq!(hd["x-content-type-options"], "nosniff");
    let (s, b, hd) = send(&h, rq(Method::GET, "/app.js")).await;
    assert_eq!((s, b.as_str()), (StatusCode::OK, "js"));
    assert!(
        hd["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/javascript")
    );
    let (s, b, _) = send(&h, rq(Method::GET, "/settings")).await;
    assert_eq!((s, b.as_str()), (StatusCode::OK, "<html>embedded</html>"));
    let (s, _, _) = send(&h, rq(Method::GET, "/missing.png")).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    let (s, b, _) = send(&h, rq(Method::HEAD, "/app.js")).await;
    assert_eq!((s, b.as_str()), (StatusCode::OK, ""));
    let (s, _, _) = send(&h, rq(Method::POST, "/app.js")).await;
    assert_eq!(s, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn override_folder_wins_over_embedded_files() {
    let dir = temp_dir("override");
    std::fs::write(dir.join("app.js"), "overridden").unwrap();
    std::fs::create_dir_all(dir.join("themes")).unwrap();
    std::fs::write(dir.join("themes/dark.css"), "body{}").unwrap();
    let h = harness_with(
        Some(dir.clone()),
        &[
            ("index.html", "embedded"),
            ("app.js", "js"),
            ("other.js", "other"),
        ],
    );
    let (_, b, _) = send(&h, rq(Method::GET, "/app.js")).await;
    assert_eq!(b, "overridden");
    let (_, b, hd) = send(&h, rq(Method::GET, "/themes/dark.css")).await;
    assert_eq!(b, "body{}");
    assert!(hd["content-type"].to_str().unwrap().starts_with("text/css"));
    // Files the folder does not have still come from the embedded set.
    let (_, b, _) = send(&h, rq(Method::GET, "/other.js")).await;
    assert_eq!(b, "other");
    let (_, b, _) = send(&h, rq(Method::GET, "/")).await;
    assert_eq!(b, "embedded");
    // A folder index.html takes over the root.
    std::fs::write(dir.join("index.html"), "custom").unwrap();
    let (_, b, _) = send(&h, rq(Method::GET, "/")).await;
    assert_eq!(b, "custom");
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn paths_cannot_escape_the_override_folder() {
    let base = temp_dir("escape");
    let dir = base.join("web");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(base.join("secret.txt"), "TOP SECRET").unwrap();
    std::fs::write(dir.join("ok.txt"), "fine").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(base.join("secret.txt"), dir.join("link.txt")).unwrap();
    let h = harness_with(Some(dir), &[("index.html", "embedded")]);
    for p in [
        "/../secret.txt",
        "/%2e%2e/secret.txt",
        "/%2E%2E%2Fsecret.txt",
        "/a/../../secret.txt",
        "/..%2fsecret.txt",
        "/%5c..%5csecret.txt",
        "/..\\secret.txt",
        "//etc/passwd",
        "/link.txt",
    ] {
        let (_, b, _) = send(&h, rq(Method::GET, p)).await;
        assert!(!b.contains("TOP SECRET"), "{p} leaked");
        assert!(!b.contains("root:"), "{p} leaked");
    }
    let (_, b, _) = send(&h, rq(Method::GET, "/ok.txt")).await;
    assert_eq!(b, "fine");
    let _ = std::fs::remove_dir_all(base);
}

#[tokio::test]
async fn api_is_never_overridable() {
    let dir = temp_dir("api");
    std::fs::create_dir_all(dir.join("api")).unwrap();
    std::fs::write(dir.join("api/state"), "FAKE STATE").unwrap();
    std::fs::write(dir.join("api/events"), "FAKE").unwrap();
    let h = harness_with(Some(dir.clone()), &[("index.html", "embedded")]);
    // The real route answers, and still wants a token.
    let (s, b, _) = send(&h, rq(Method::GET, "/api/state")).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    assert!(!b.contains("FAKE"));
    let (_, b, _) = send(&h, rq(Method::GET, "/api/state").token(&h.token)).await;
    assert!(!b.contains("FAKE") && b.contains("serverNow"));
    // Other spellings never reach the folder either.
    for p in [
        "/%61pi/state",
        "//api/state",
        "/./api/state",
        "/API/state",
        "/api/%2e%2e/api/state",
    ] {
        let (_, b, _) = send(&h, rq(Method::GET, p).token(&h.token)).await;
        assert!(!b.contains("FAKE"), "{p} served an override");
    }
    let _ = std::fs::remove_dir_all(dir);
}

// ---- real sockets -----------------------------------------------------------------------

async fn start_server(lan: bool, heartbeat_ms: u64) -> (Server, Arc<FakeCore>) {
    let core = FakeCore::new();
    let store: Arc<dyn Store> = Arc::new(MemStore::new());
    let server = Server::start(
        core.clone(),
        store,
        Arc::new(MapAssets(
            [("index.html".to_owned(), b"hello".to_vec())]
                .into_iter()
                .collect(),
        )),
        ServerConfig {
            web_dir: None,
            heartbeat: Duration::from_millis(heartbeat_ms),
            lan,
            port: 0,
            lan_ips: Arc::new(|| vec![Ipv4Addr::new(192, 168, 1, 20), Ipv4Addr::new(10, 0, 0, 7)]),
            addon_candidates: Arc::new(Vec::new),
        },
    )
    .await
    .unwrap();
    (server, core)
}

/// One HTTP/1.1 request with `Connection: close`; returns status and body text.
async fn http(
    port: u16,
    method: &str,
    path: &str,
    host: &str,
    token: Option<&str>,
    body: &str,
) -> (u16, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let auth = token
        .map(|t| format!("Authorization: Bearer {t}\r\n"))
        .unwrap_or_default();
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}\r\n{auth}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    s.write_all(req.as_bytes()).await.unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).await.unwrap();
    let status = out.split(' ').nth(1).unwrap().parse().unwrap();
    let body = out.split("\r\n\r\n").nth(1).unwrap_or("").to_owned();
    (status, body)
}

#[tokio::test]
async fn server_picks_a_random_port_and_builds_the_window_url() {
    let (server, _) = start_server(false, 10_000).await;
    assert_ne!(server.port(), 0);
    let url = server.window_url();
    assert_eq!(
        url,
        format!(
            "http://127.0.0.1:{}/#token={}",
            server.port(),
            server.token()
        )
    );
    let host = format!("127.0.0.1:{}", server.port());
    let (s, b) = http(server.port(), "GET", "/", &host, None, "").await;
    assert_eq!((s, b.as_str()), (200, "hello"));
    let (s, _) = http(server.port(), "GET", "/api/state", &host, None, "").await;
    assert_eq!(s, 401);
    let (s, b) = http(
        server.port(),
        "GET",
        "/api/state",
        &host,
        Some(&server.token()),
        "",
    )
    .await;
    assert_eq!(s, 200);
    assert!(b.contains("serverNow"));
    server.shutdown().await;
}

/// Read NDJSON lines from a chunked response, skipping chunk framing.
async fn next_json(r: &mut BufReader<TcpStream>) -> Option<serde_json::Value> {
    loop {
        let mut line = String::new();
        let n = tokio::time::timeout(Duration::from_secs(5), r.read_line(&mut line))
            .await
            .expect("stream stalled")
            .unwrap();
        if n == 0 || line == "0\r\n" {
            return None;
        }
        if line.starts_with('{') {
            return Some(serde_json::from_str(line.trim()).unwrap());
        }
    }
}

#[tokio::test]
async fn event_stream_sends_state_changes_heartbeats_and_ends_on_token_reset() {
    let (server, core) = start_server(false, 80).await;
    let port = server.port();
    let mut conn = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let req = format!(
        "GET /api/events HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {}\r\n\r\n",
        server.token()
    );
    conn.write_all(req.as_bytes()).await.unwrap();
    let mut r = BufReader::new(conn);
    let mut head = String::new();
    loop {
        let mut l = String::new();
        r.read_line(&mut l).await.unwrap();
        if l == "\r\n" {
            break;
        }
        head.push_str(&l.to_ascii_lowercase());
    }
    assert!(head.starts_with("http/1.1 200"));
    assert!(head.contains("content-type: application/x-ndjson"));

    let first = next_json(&mut r).await.unwrap();
    assert_eq!(first["type"], "state");
    assert_eq!(first["state"]["search"]["status"], "idle");
    assert!(first["state"]["serverNow"].as_u64().unwrap() > 0);

    let hb = next_json(&mut r).await.unwrap();
    assert_eq!(hb, serde_json::json!({ "type": "heartbeat" }));

    core.tx.send_modify(|s| s.search.stage = 3);
    loop {
        let ev = next_json(&mut r).await.unwrap();
        if ev["type"] == "state" {
            assert_eq!(ev["state"]["search"]["stage"], 3);
            break;
        }
    }

    // Resetting the token ends the stream for the old token.
    let host = format!("127.0.0.1:{port}");
    let (s, _) = http(
        port,
        "POST",
        "/api/token/reset",
        &host,
        Some(&server.token()),
        "",
    )
    .await;
    assert_eq!(s, 200);
    let mut ended = false;
    for _ in 0..20 {
        if next_json(&mut r).await.is_none() {
            ended = true;
            break;
        }
    }
    assert!(ended, "stream did not end after a token reset");
    server.shutdown().await;
}

#[tokio::test]
async fn lan_mode_restarts_the_listener_on_the_same_port() {
    let (server, core) = start_server(false, 10_000).await;
    let port = server.port();
    let token = server.token();
    let lan_host = format!("192.168.1.20:{port}");
    assert!(!server.lan());
    // Off: a LAN Host is refused.
    let (s, _) = http(port, "GET", "/api/state", &lan_host, Some(&token), "").await;
    assert_eq!(s, 403);

    let host = format!("127.0.0.1:{port}");
    let (s, b) = http(
        port,
        "PUT",
        "/api/settings",
        &host,
        Some(&token),
        r#"{"lan_mode":true}"#,
    )
    .await;
    assert_eq!(s, 200, "{b}");
    assert!(b.contains("\"lan_mode\":true"));
    for _ in 0..100 {
        if server.lan() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(server.lan(), "listener never restarted");
    let (s, _) = http(port, "GET", "/api/state", &lan_host, Some(&token), "").await;
    assert_eq!(s, 200);
    let (s, b) = http(port, "GET", "/api/pairing", &host, Some(&token), "").await;
    assert_eq!(s, 200);
    assert!(b.contains(&format!("http://192.168.1.20:{port}/#token={token}")));

    // And back off.
    core.tx.send_modify(|_| {});
    let (s, _) = http(
        port,
        "PUT",
        "/api/settings",
        &host,
        Some(&token),
        r#"{"lan_mode":false}"#,
    )
    .await;
    assert_eq!(s, 200);
    for _ in 0..100 {
        if !server.lan() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(!server.lan());
    let (s, _) = http(port, "GET", "/api/state", &lan_host, Some(&token), "").await;
    assert_eq!(s, 403);
    let (s, _) = http(port, "GET", "/api/state", &host, Some(&token), "").await;
    assert_eq!(s, 200);
    server.shutdown().await;
}

// ---- addon installer routes ------------------------------------------------------------

/// A fake game tree `<tmp>/Interface/AddOns`; returns the AddOns folder (canonical).
fn fake_addons(name: &str) -> PathBuf {
    let root = temp_dir(name);
    let addons = root.join("Interface").join("AddOns");
    std::fs::create_dir_all(&addons).unwrap();
    addons.canonicalize().unwrap()
}

fn json_of(b: &str) -> serde_json::Value {
    serde_json::from_str(b).unwrap()
}

async fn put_addon_folder(h: &H, folder: &str) -> (StatusCode, String) {
    let body = serde_json::json!({ "folder": folder }).to_string();
    let (s, b, _) = send(
        h,
        rq(Method::PUT, "/api/addon/folder")
            .token(&h.token)
            .body(body),
    )
    .await;
    (s, b)
}

async fn addon_get(h: &H) -> serde_json::Value {
    let (s, b, _) = send(h, rq(Method::GET, "/api/addon").token(&h.token)).await;
    assert_eq!(s, StatusCode::OK, "{b}");
    json_of(&b)
}

fn files_under(dir: &std::path::Path) -> Vec<String> {
    fn walk(base: &std::path::Path, d: &std::path::Path, out: &mut Vec<String>) {
        for e in std::fs::read_dir(d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(base, &p, out);
            } else {
                out.push(
                    p.strip_prefix(base)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    let mut out = vec![];
    walk(dir, dir, &mut out);
    out.sort();
    out
}

#[tokio::test]
async fn addon_routes_need_a_token_and_a_loopback_caller() {
    let h = harness();
    for (m, path, body) in [
        (Method::GET, "/api/addon", ""),
        (Method::PUT, "/api/addon/folder", r#"{"folder":"/x"}"#),
        (Method::POST, "/api/addon/install", ""),
    ] {
        let (s, b, _) = send(&h, rq(m.clone(), path).body(body)).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED, "{path}");
        assert_eq!(err_code(&b), "unauthorized");
        let (s, b, _) = send(&h, rq(m, path).token(&h.token).body(body).peer(REMOTE)).await;
        assert_eq!(s, StatusCode::FORBIDDEN, "{path}");
        assert_eq!(err_code(&b), "loopback_only");
    }
    assert!(h.core.calls().iter().all(|c| !c.starts_with("addon_dir")));
}

#[tokio::test]
async fn put_addon_folder_rejects_bad_json_and_unknown_fields() {
    let h = harness();
    for body in ["nope", r#"{"folder":"/x","extra":1}"#, "{}"] {
        let (s, b, _) = send(
            &h,
            rq(Method::PUT, "/api/addon/folder")
                .token(&h.token)
                .body(body),
        )
        .await;
        assert_eq!(s, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(err_code(&b), "bad_json");
    }
}

#[tokio::test]
async fn get_addon_without_a_folder_is_no_folder_and_reports_the_bundle() {
    let h = harness();
    let v = addon_get(&h).await;
    assert_eq!(v["status"], "no_folder");
    assert_eq!(v["folder"], serde_json::Value::Null);
    assert_eq!(v["folder_source"], serde_json::Value::Null);
    assert_eq!(
        v["bundled_version"],
        crate::addon_install::bundled_version()
    );
    assert!(v["candidates"].is_array());
    assert_eq!(v["installed"]["state"], "missing");
}

#[tokio::test]
async fn put_folder_saves_it_and_get_then_shows_saved() {
    let h = harness();
    let addons = fake_addons("addon-put");
    let (s, b) = put_addon_folder(&h, addons.to_str().unwrap()).await;
    assert_eq!(s, StatusCode::OK, "{b}");
    let v = json_of(&b);
    assert_eq!(v["folder"], addons.to_str().unwrap());
    assert_eq!(v["folder_source"], "saved");
    assert_eq!(v["status"], "not_installed");
    assert_eq!(
        h.core.addon.lock().unwrap().as_deref(),
        addons.to_str(),
        "saved as the canonical path"
    );
    let v = addon_get(&h).await;
    assert_eq!(v["folder_source"], "saved");
    assert_eq!(v["status"], "not_installed");
    assert_eq!(v["installed"]["state"], "missing");
}

#[tokio::test]
async fn put_folder_refuses_a_folder_that_is_not_an_addons_folder() {
    let h = harness();
    let dir = temp_dir("addon-bad");
    let (s, b) = put_addon_folder(&h, dir.to_str().unwrap()).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "{b}");
    assert_eq!(err_code(&b), "addon_folder_invalid");
    let v = json_of(&b);
    assert!(
        v["message"]
            .as_str()
            .unwrap()
            .contains(dir.to_str().unwrap())
    );
    assert!(v["detail"].is_string());
    assert_eq!(*h.core.addon.lock().unwrap(), None, "nothing saved");
}

#[tokio::test]
async fn install_without_a_saved_folder_is_409_unset() {
    let h = harness();
    let (s, b, _) = send(&h, rq(Method::POST, "/api/addon/install").token(&h.token)).await;
    assert_eq!(s, StatusCode::CONFLICT);
    assert_eq!(err_code(&b), "addon_folder_unset");
}

#[tokio::test]
async fn install_writes_exactly_the_bundle_and_then_the_status_is_up_to_date() {
    let h = harness();
    let addons = fake_addons("addon-install");
    // A neighbour addon is left alone.
    std::fs::create_dir_all(addons.join("Other")).unwrap();
    std::fs::write(addons.join("Other").join("Other.toc"), "x").unwrap();
    put_addon_folder(&h, addons.to_str().unwrap()).await;
    let (s, b, _) = send(&h, rq(Method::POST, "/api/addon/install").token(&h.token)).await;
    assert_eq!(s, StatusCode::OK, "{b}");
    let v = json_of(&b);
    assert_eq!(v["status"], "up_to_date");
    assert_eq!(v["folder_source"], "saved");
    assert_eq!(v["installed"]["state"], "present");
    assert_eq!(
        v["installed"]["version"],
        crate::addon_install::bundled_version()
    );
    let mut want: Vec<String> = crate::addon_install::bundle()
        .iter()
        .map(|f| f.path.to_owned())
        .collect();
    want.sort();
    assert_eq!(files_under(&addons.join("ForeverTogether")), want);
    assert!(addons.join("Other").join("Other.toc").exists());
    assert_eq!(addon_get(&h).await["status"], "up_to_date");
}

#[tokio::test]
async fn every_status_is_reachable_through_a_temp_tree() {
    let h = harness();
    let addons = fake_addons("addon-status");
    put_addon_folder(&h, addons.to_str().unwrap()).await;
    assert_eq!(addon_get(&h).await["status"], "not_installed");
    let (s, _, _) = send(&h, rq(Method::POST, "/api/addon/install").token(&h.token)).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(addon_get(&h).await["status"], "up_to_date");
    let toc = addons.join("ForeverTogether").join("ForeverTogether.toc");
    let text = std::fs::read_to_string(&toc).unwrap();
    // Same version, different bytes.
    std::fs::write(&toc, format!("{text}\n## X-Local: edit\n")).unwrap();
    assert_eq!(addon_get(&h).await["status"], "files_differ");
    // Older installed version.
    let older: String = text
        .lines()
        .map(|l| {
            if l.to_ascii_lowercase().starts_with("## version:") {
                "## Version: 0.0.1".to_owned()
            } else {
                l.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&toc, older).unwrap();
    let v = addon_get(&h).await;
    assert_eq!(v["status"], "update_available");
    assert_eq!(v["installed"]["version"], "0.0.1");
    // The saved folder disappears.
    std::fs::remove_dir_all(&addons).unwrap();
    let v = addon_get(&h).await;
    assert_eq!(v["status"], "folder_invalid");
    assert_eq!(v["folder"], addons.to_str().unwrap());
    assert_eq!(v["folder_source"], "saved");
}

#[tokio::test]
async fn a_single_detected_candidate_is_shown_but_never_saved() {
    let addons = fake_addons("addon-detect");
    let h = harness_full(None, &[], vec![addons.clone()]);
    let v = addon_get(&h).await;
    assert_eq!(v["folder_source"], "detected");
    assert_eq!(v["folder"], addons.to_str().unwrap());
    assert_eq!(v["status"], "not_installed");
    assert_eq!(v["candidates"][0], addons.to_str().unwrap());
    assert_eq!(*h.core.addon.lock().unwrap(), None);
    assert!(h.core.calls().iter().all(|c| !c.starts_with("addon_dir")));
    // Install still needs the folder to be saved.
    let (s, b, _) = send(&h, rq(Method::POST, "/api/addon/install").token(&h.token)).await;
    assert_eq!(s, StatusCode::CONFLICT);
    assert_eq!(err_code(&b), "addon_folder_unset");
    // Two candidates: no automatic choice.
    let other = fake_addons("addon-detect2");
    let h = harness_full(None, &[], vec![addons, other]);
    let v = addon_get(&h).await;
    assert_eq!(v["folder_source"], serde_json::Value::Null);
    assert_eq!(v["status"], "no_folder");
    assert_eq!(v["candidates"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn a_saved_folder_wins_over_a_detected_one() {
    let detected = fake_addons("addon-win-a");
    let saved = fake_addons("addon-win-b");
    let h = harness_full(None, &[], vec![detected]);
    put_addon_folder(&h, saved.to_str().unwrap()).await;
    let v = addon_get(&h).await;
    assert_eq!(v["folder_source"], "saved");
    assert_eq!(v["folder"], saved.to_str().unwrap());
}

#[tokio::test]
async fn install_into_a_vanished_folder_names_it_in_the_error() {
    let h = harness();
    let addons = fake_addons("addon-gone");
    put_addon_folder(&h, addons.to_str().unwrap()).await;
    std::fs::remove_dir_all(&addons).unwrap();
    let (s, b, _) = send(&h, rq(Method::POST, "/api/addon/install").token(&h.token)).await;
    assert!(s.is_client_error(), "{s} {b}");
    let v = json_of(&b);
    assert!(v["error"].as_str().unwrap().starts_with("addon_"), "{b}");
    assert!(
        v["message"]
            .as_str()
            .unwrap()
            .contains(addons.to_str().unwrap())
    );
    assert!(v["detail"].is_string());
}

#[cfg(unix)]
#[tokio::test]
async fn install_into_a_read_only_folder_is_403_and_names_the_folder() {
    use std::os::unix::fs::PermissionsExt;
    let h = harness();
    let addons = fake_addons("addon-ro");
    put_addon_folder(&h, addons.to_str().unwrap()).await;
    std::fs::set_permissions(&addons, std::fs::Permissions::from_mode(0o555)).unwrap();
    let probe = addons.join(".probe");
    let writable = std::fs::write(&probe, b"").is_ok();
    let _ = std::fs::remove_file(&probe);
    let (s, b, _) = send(&h, rq(Method::POST, "/api/addon/install").token(&h.token)).await;
    std::fs::set_permissions(&addons, std::fs::Permissions::from_mode(0o755)).unwrap();
    if writable {
        return; // running as root: permissions do not apply
    }
    assert_eq!(s, StatusCode::FORBIDDEN, "{b}");
    assert_eq!(err_code(&b), "addon_permission_denied");
    let v = json_of(&b);
    assert!(
        v["message"]
            .as_str()
            .unwrap()
            .contains(addons.to_str().unwrap())
    );
    assert!(v["detail"].is_string());
    assert!(!addons.join("ForeverTogether").exists());
}

#[test]
fn each_addon_error_maps_to_its_http_status() {
    use crate::addon_install::AddonError as E;
    let f = || "/g/Interface/AddOns".to_owned();
    let d = || "raw".to_owned();
    let cases = [
        (E::FolderUnset, 409, "addon_folder_unset"),
        (
            E::FolderInvalid {
                folder: f(),
                reason: d(),
            },
            400,
            "addon_folder_invalid",
        ),
        (
            E::FolderMissing {
                folder: f(),
                detail: d(),
            },
            404,
            "addon_folder_missing",
        ),
        (
            E::PermissionDenied {
                folder: f(),
                detail: d(),
            },
            403,
            "addon_permission_denied",
        ),
        (
            E::ReadOnly {
                folder: f(),
                detail: d(),
            },
            409,
            "addon_read_only",
        ),
        (
            E::NoSpace {
                folder: f(),
                detail: d(),
            },
            507,
            "addon_no_space",
        ),
        (
            E::Locked {
                folder: f(),
                detail: d(),
            },
            409,
            "addon_locked",
        ),
        (
            E::InstallFailed {
                folder: f(),
                detail: d(),
            },
            500,
            "addon_install_failed",
        ),
    ];
    for (e, status, code) in cases {
        let ce = super::addon_routes::addon_err(&e);
        assert_eq!(ce.status.as_u16(), status, "{code}");
        assert_eq!(ce.code, code);
        if !matches!(e, E::FolderUnset) {
            assert!(ce.message.contains("/g/Interface/AddOns"), "{code}");
        }
    }
}

#[tokio::test]
async fn a_phone_cannot_change_the_addon_folder_and_settings_have_no_such_field() {
    let h = harness();
    let addons = fake_addons("addon-phone");
    let body = serde_json::json!({ "folder": addons.to_str().unwrap() }).to_string();
    let (s, _, _) = send(
        &h,
        rq(Method::PUT, "/api/addon/folder")
            .token(&h.token)
            .body(body)
            .peer(REMOTE),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    assert_eq!(*h.core.addon.lock().unwrap(), None);
    let (s, b, _) = send(
        &h,
        rq(Method::PUT, "/api/settings")
            .token(&h.token)
            .body(r#"{"addon_dir":"/x"}"#),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "{b}");
}
