//! Token, Host and Origin checks, loopback detection and the per-token rate limit.
//!
//! Order of checks on an API request (the first failure wins):
//! 1. Host and Origin (403). Applies to every request, static files included, so a page on
//!    another site cannot reach the server by DNS rebinding.
//! 2. Bearer token (401), constant-time compare.
//! 3. Rate limit, 20 requests per second per token (429).
//! 4. Loopback-only routes (403): decided by the socket's peer address.

use crate::store::{KEY_API_TOKEN, Store, StoreError};
use axum::extract::{ConnectInfo, Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use tokio::sync::watch;

/// Requests per second one token may make.
pub const RATE_LIMIT_PER_SECOND: u32 = 20;

/// A JSON error body: `{ "error": "<code>", "message": "<text>" }`.
pub fn error_response(status: StatusCode, code: &str, message: &str) -> Response {
    let body = serde_json::json!({ "error": code, "message": message });
    let mut r = (status, axum::Json(body)).into_response();
    r.headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    r
}

/// Constant-time equality. Both sides are hashed first so the length does not leak either.
pub fn ct_eq(a: &str, b: &str) -> bool {
    let ha = Sha256::digest(a.as_bytes());
    let hb = Sha256::digest(b.as_bytes());
    ha.iter()
        .zip(hb.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

fn new_token() -> String {
    let mut b = [0u8; 32];
    rand::fill(&mut b);
    data_encoding::HEXLOWER.encode(&b)
}

/// The per-install API token: 32 random bytes as hex, kept in the store.
pub struct TokenManager {
    store: Arc<dyn Store>,
    token: RwLock<String>,
    generation: watch::Sender<u64>,
}

impl TokenManager {
    pub fn load_or_create(store: Arc<dyn Store>) -> Result<Self, StoreError> {
        let existing = store
            .get(KEY_API_TOKEN)?
            .and_then(|b| String::from_utf8(b).ok())
            .filter(|t| t.len() == 64 && t.bytes().all(|c| c.is_ascii_hexdigit()));
        let token = match existing {
            Some(t) => t,
            None => {
                let t = new_token();
                store.put(KEY_API_TOKEN, t.as_bytes())?;
                t
            }
        };
        Ok(Self {
            store,
            token: RwLock::new(token),
            generation: watch::channel(0).0,
        })
    }

    pub fn current(&self) -> String {
        self.token.read().unwrap().clone()
    }

    pub fn verify(&self, candidate: &str) -> bool {
        ct_eq(&self.token.read().unwrap(), candidate)
    }

    /// Make a new token. The old one stops working at once, which un-pairs every phone.
    pub fn reset(&self) -> Result<String, StoreError> {
        let t = new_token();
        self.store.put(KEY_API_TOKEN, t.as_bytes())?;
        *self.token.write().unwrap() = t.clone();
        self.generation.send_modify(|g| *g += 1);
        Ok(t)
    }

    /// Changes whenever the token is reset, so open streams can end.
    pub fn watch_generation(&self) -> watch::Receiver<u64> {
        self.generation.subscribe()
    }
}

/// Token bucket per token: capacity and refill are both `RATE_LIMIT_PER_SECOND`.
#[derive(Default)]
pub struct RateLimiter {
    buckets: Mutex<HashMap<[u8; 32], (f64, Instant)>>,
}

impl RateLimiter {
    pub fn check(&self, token: &str, now: Instant) -> bool {
        let key: [u8; 32] = Sha256::digest(token.as_bytes()).into();
        let cap = RATE_LIMIT_PER_SECOND as f64;
        let mut m = self.buckets.lock().unwrap();
        let (tokens, last) = m.entry(key).or_insert((cap, now));
        let elapsed = now.saturating_duration_since(*last).as_secs_f64();
        *tokens = (*tokens + elapsed * cap).min(cap);
        *last = now;
        if *tokens >= 1.0 {
            *tokens -= 1.0;
            true
        } else {
            false
        }
    }

    pub fn clear(&self) {
        self.buckets.lock().unwrap().clear();
    }
}

/// What the middleware needs to know. Shared with the listener controller, which updates
/// `port` and `lan`.
pub struct Guard {
    pub tokens: TokenManager,
    pub limiter: RateLimiter,
    pub port: AtomicU16,
    pub lan: AtomicBool,
}

impl Guard {
    pub fn new(tokens: TokenManager) -> Self {
        Self {
            tokens,
            limiter: RateLimiter::default(),
            port: AtomicU16::new(0),
            lan: AtomicBool::new(false),
        }
    }
}

fn split_host_port(authority: &str) -> Option<(&str, Option<u16>)> {
    if let Some(rest) = authority.strip_prefix('[') {
        let (host, after) = rest.split_once(']')?;
        return match after {
            "" => Some((host, None)),
            _ => Some((host, Some(after.strip_prefix(':')?.parse().ok()?))),
        };
    }
    match authority.rsplit_once(':') {
        Some((h, p)) => Some((h, Some(p.parse().ok()?))),
        None => Some((authority, None)),
    }
}

/// Whether the `Host` header is acceptable: a loopback name or address, or in LAN mode any IP
/// literal (a name could be a DNS-rebinding trick), on the port the server is bound to.
pub fn host_allowed(authority: &str, port: u16, lan: bool) -> bool {
    let Some((host, p)) = split_host_port(authority) else {
        return false;
    };
    if p.unwrap_or(80) != port {
        return false;
    }
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    match host.parse::<IpAddr>() {
        Ok(ip) => ip.is_loopback() || lan,
        Err(_) => false,
    }
}

/// An Origin, when present, must be exactly this server (same scheme `http`, same authority as
/// the Host header). `null` and any other origin are refused.
pub fn origin_allowed(origin: &str, host: &str) -> bool {
    origin
        .strip_prefix("http://")
        .is_some_and(|a| a.eq_ignore_ascii_case(host))
}

pub fn is_loopback_peer(addr: &SocketAddr) -> bool {
    match addr.ip() {
        IpAddr::V4(v4) => v4.is_loopback(),
        IpAddr::V6(v6) => v6.is_loopback() || v6.to_ipv4_mapped().is_some_and(|v| v.is_loopback()),
    }
}

fn header_str(h: &HeaderMap, name: header::HeaderName) -> Option<&str> {
    h.get(name).and_then(|v| v.to_str().ok())
}

/// Middleware: Host and Origin.
pub async fn host_origin(State(g): State<Arc<Guard>>, req: Request, next: Next) -> Response {
    let port = g.port.load(Ordering::Relaxed);
    let lan = g.lan.load(Ordering::Relaxed);
    let host = header_str(req.headers(), header::HOST);
    let Some(host) = host.filter(|h| host_allowed(h, port, lan)) else {
        return error_response(StatusCode::FORBIDDEN, "bad_host", "Host not allowed");
    };
    if let Some(o) = req.headers().get(header::ORIGIN) {
        let ok = o.to_str().ok().is_some_and(|o| origin_allowed(o, host));
        if !ok {
            return error_response(StatusCode::FORBIDDEN, "bad_origin", "Origin not allowed");
        }
    }
    next.run(req).await
}

/// The bearer token from the request, if the header is well formed.
pub fn bearer(h: &HeaderMap) -> Option<&str> {
    let v = header_str(h, header::AUTHORIZATION)?;
    let (scheme, tok) = v.split_once(' ')?;
    scheme.eq_ignore_ascii_case("bearer").then_some(tok.trim())
}

/// Middleware for `/api`: token and rate limit.
pub async fn require_token(State(g): State<Arc<Guard>>, req: Request, next: Next) -> Response {
    let Some(tok) = bearer(req.headers()) else {
        return error_response(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "Bearer token required",
        );
    };
    if !g.tokens.verify(tok) {
        return error_response(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "Bearer token is wrong",
        );
    }
    if !g.limiter.check(tok, Instant::now()) {
        let mut r = error_response(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limited",
            "At most 20 requests per second per token",
        );
        r.headers_mut()
            .insert(header::RETRY_AFTER, "1".parse().unwrap());
        return r;
    }
    next.run(req).await
}

/// Middleware for loopback-only routes.
pub async fn require_loopback(req: Request, next: Next) -> Response {
    let ok = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .is_some_and(|c| is_loopback_peer(&c.0));
    if !ok {
        return error_response(
            StatusCode::FORBIDDEN,
            "loopback_only",
            "This can only be done from the computer running the app",
        );
    }
    next.run(req).await
}

/// Used by handlers that decide loopback per request (settings writes).
pub fn caller_is_loopback(req_ext: &axum::http::Extensions) -> bool {
    req_ext
        .get::<ConnectInfo<SocketAddr>>()
        .is_some_and(|c| is_loopback_peer(&c.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemStore;
    use std::time::Duration;

    #[test]
    fn constant_time_compare_semantics() {
        assert!(ct_eq("abc", "abc"));
        assert!(!ct_eq("abc", "abd"));
        assert!(!ct_eq("abc", "abcd"));
        assert!(!ct_eq("", "a"));
    }

    #[test]
    fn token_is_64_hex_persisted_and_reset() {
        let store: Arc<dyn Store> = Arc::new(MemStore::new());
        let t = TokenManager::load_or_create(store.clone()).unwrap();
        let first = t.current();
        assert_eq!(first.len(), 64);
        assert!(first.bytes().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(
            TokenManager::load_or_create(store.clone())
                .unwrap()
                .current(),
            first
        );
        let mut generation = t.watch_generation();
        let second = t.reset().unwrap();
        assert_ne!(first, second);
        assert!(!t.verify(&first));
        assert!(t.verify(&second));
        assert!(generation.has_changed().unwrap());
        generation.borrow_and_update();
        assert_eq!(
            TokenManager::load_or_create(store).unwrap().current(),
            second
        );
    }

    #[test]
    fn host_rules() {
        assert!(host_allowed("127.0.0.1:4000", 4000, false));
        assert!(host_allowed("localhost:4000", 4000, false));
        assert!(host_allowed("LOCALHOST:4000", 4000, false));
        assert!(host_allowed("[::1]:4000", 4000, false));
        assert!(!host_allowed("127.0.0.1:4001", 4000, false));
        assert!(!host_allowed("127.0.0.1", 4000, false));
        assert!(!host_allowed("evil.example:4000", 4000, true));
        assert!(!host_allowed("localhost.evil.example:4000", 4000, false));
        assert!(!host_allowed("192.168.1.5:4000", 4000, false));
        assert!(host_allowed("192.168.1.5:4000", 4000, true));
        assert!(!host_allowed("", 4000, true));
    }

    #[test]
    fn origin_rules() {
        assert!(origin_allowed("http://127.0.0.1:4000", "127.0.0.1:4000"));
        assert!(!origin_allowed("http://evil.example", "127.0.0.1:4000"));
        assert!(!origin_allowed("https://127.0.0.1:4000", "127.0.0.1:4000"));
        assert!(!origin_allowed("null", "127.0.0.1:4000"));
    }

    #[test]
    fn rate_limit_allows_a_burst_of_twenty_then_refills() {
        let l = RateLimiter::default();
        let t0 = Instant::now();
        for _ in 0..20 {
            assert!(l.check("tok", t0));
        }
        assert!(!l.check("tok", t0));
        // Another token has its own budget.
        assert!(l.check("other", t0));
        // Half a second refills ten requests.
        let t1 = t0 + Duration::from_millis(500);
        for _ in 0..10 {
            assert!(l.check("tok", t1));
        }
        assert!(!l.check("tok", t1));
    }

    #[test]
    fn loopback_detection() {
        assert!(is_loopback_peer(&"127.0.0.1:1".parse().unwrap()));
        assert!(is_loopback_peer(&"[::1]:1".parse().unwrap()));
        assert!(is_loopback_peer(&"[::ffff:127.0.0.1]:1".parse().unwrap()));
        assert!(!is_loopback_peer(&"192.168.1.9:1".parse().unwrap()));
    }
}
