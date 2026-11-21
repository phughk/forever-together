//! `GET /api/events`: newline-delimited JSON, `{"type":"state","state":...}` on every change
//! and `{"type":"heartbeat"}` on a fixed interval (15 s in production). The first line is the
//! current state. The stream ends when the token is reset or the listener restarts, so a
//! revoked phone does not keep receiving updates; the client then reconnects.

use super::core::AppState;
use axum::body::{Body, Bytes};
use futures_util::stream;
use serde::Serialize;
use std::convert::Infallible;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::watch;
use tokio::time::{Instant, Interval, MissedTickBehavior, interval_at};

pub const HEARTBEAT: Duration = Duration::from_secs(15);

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Line<'a> {
    State { state: &'a AppState },
    Heartbeat,
}

fn line(l: &Line<'_>) -> Bytes {
    let mut v = serde_json::to_vec(l).unwrap_or_default();
    v.push(b'\n');
    Bytes::from(v)
}

/// A snapshot with the server clock refreshed.
pub fn fresh(mut s: AppState) -> AppState {
    s.server_now = now_ms();
    s
}

struct St {
    state: watch::Receiver<AppState>,
    ticker: Interval,
    stops: [watch::Receiver<u64>; 2],
    first: bool,
}

pub fn event_body(
    state: watch::Receiver<AppState>,
    heartbeat: Duration,
    token_generation: watch::Receiver<u64>,
    listener_generation: watch::Receiver<u64>,
) -> Body {
    let mut ticker = interval_at(Instant::now() + heartbeat, heartbeat);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut token_generation = token_generation;
    let mut listener_generation = listener_generation;
    token_generation.borrow_and_update();
    listener_generation.borrow_and_update();
    let st = St {
        state,
        ticker,
        stops: [token_generation, listener_generation],
        first: true,
    };
    let s = stream::unfold(st, |mut st| async move {
        if st.first {
            st.first = false;
            let b = line(&Line::State {
                state: &fresh(st.state.borrow_and_update().clone()),
            });
            return Some((Ok::<_, Infallible>(b), st));
        }
        let [tok, lis] = &mut st.stops;
        tokio::select! {
            r = st.state.changed() => {
                if r.is_err() { return None; }
                let b = line(&Line::State { state: &fresh(st.state.borrow_and_update().clone()) });
                Some((Ok(b), st))
            }
            _ = st.ticker.tick() => Some((Ok(line(&Line::Heartbeat)), st)),
            _ = tok.changed() => None,
            _ = lis.changed() => None,
        }
    });
    Body::from_stream(s)
}
