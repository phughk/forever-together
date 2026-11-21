//! The REST routes (plan "Local web API") and the router that wires the checks around them.

use super::Shared;
use super::auth::{
    Guard, caller_is_loopback, error_response, host_origin, require_loopback, require_token,
};
use super::core::{AppCore, CoreError, SettingsPatch};
use super::events::{event_body, fresh};
use super::pairing::pairing_url;
use super::static_files::serve_path;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::{from_fn, from_fn_with_state};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::sync::atomic::Ordering;

/// Largest request body.
pub const MAX_BODY: usize = 8 * 1024;

type S<C> = State<Arc<Shared<C>>>;

fn core_err(e: CoreError) -> Response {
    error_response(e.status, &e.code, &e.message)
}

fn no_content() -> Response {
    StatusCode::NO_CONTENT.into_response()
}

fn json<T: serde::Serialize>(v: &T) -> Response {
    let mut r = axum::Json(v).into_response();
    r.headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    r
}

// The error is a ready-made response, returned straight to the caller.
#[allow(clippy::result_large_err)]
async fn read_json<T: DeserializeOwned>(body: Body) -> Result<T, Response> {
    let bytes = to_bytes(body, MAX_BODY).await.map_err(|_| {
        error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            "payload_too_large",
            "Request body is larger than 8 KiB",
        )
    })?;
    serde_json::from_slice(&bytes)
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, "bad_json", &e.to_string()))
}

async fn get_state<C: AppCore>(State(s): S<C>) -> Response {
    json(&fresh(s.core.state()))
}

async fn get_events<C: AppCore>(State(s): S<C>) -> Response {
    let body = event_body(
        s.core.subscribe(),
        s.heartbeat,
        s.guard.tokens.watch_generation(),
        s.listener_generation.subscribe(),
    );
    let mut r = Response::new(body);
    let h = r.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        "application/x-ndjson".parse().unwrap(),
    );
    h.insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    h.insert("x-accel-buffering", "no".parse().unwrap());
    r
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CodeBody {
    code: String,
}

async fn post_character<C: AppCore>(State(s): S<C>, req: Request) -> Response {
    let body: CodeBody = match read_json(req.into_body()).await {
        Ok(b) => b,
        Err(r) => return r,
    };
    match s.core.set_character(body.code).await {
        Ok(c) => json(&c),
        Err(e) => core_err(e),
    }
}

async fn delete_character<C: AppCore>(State(s): S<C>) -> Response {
    match s.core.clear_character().await {
        Ok(()) => no_content(),
        Err(e) => core_err(e),
    }
}

async fn get_dungeons<C: AppCore>(State(s): S<C>) -> Response {
    json(&s.core.dungeons().await)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SearchBody {
    dungeons: Vec<String>,
}

async fn post_search<C: AppCore>(State(s): S<C>, req: Request) -> Response {
    let body: SearchBody = match read_json(req.into_body()).await {
        Ok(b) => b,
        Err(r) => return r,
    };
    match s.core.start_search(body.dungeons).await {
        Ok(()) => no_content(),
        Err(e) => core_err(e),
    }
}

async fn delete_search<C: AppCore>(State(s): S<C>) -> Response {
    match s.core.stop_search().await {
        Ok(()) => no_content(),
        Err(e) => core_err(e),
    }
}

async fn post_confirm<C: AppCore>(State(s): S<C>) -> Response {
    match s.core.confirm_party().await {
        Ok(()) => no_content(),
        Err(e) => core_err(e),
    }
}

async fn post_decline<C: AppCore>(State(s): S<C>) -> Response {
    match s.core.decline_party().await {
        Ok(()) => no_content(),
        Err(e) => core_err(e),
    }
}

async fn get_settings<C: AppCore>(State(s): S<C>) -> Response {
    json(&s.core.settings())
}

async fn put_settings<C: AppCore>(State(s): S<C>, req: Request) -> Response {
    let loopback = caller_is_loopback(req.extensions());
    let patch: SettingsPatch = match read_json(req.into_body()).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    if patch.touches_loopback_only() && !loopback {
        return error_response(
            StatusCode::FORBIDDEN,
            "loopback_only",
            "LAN mode and trusted peers can only be changed from the computer running the app",
        );
    }
    let before = s.core.settings().lan_mode;
    match s.core.update_settings(patch).await {
        Ok(new) => {
            if new.lan_mode != before {
                s.request_lan(new.lan_mode);
            }
            json(&new)
        }
        Err(e) => core_err(e),
    }
}

async fn get_pairing<C: AppCore>(State(s): S<C>) -> Response {
    if !s.core.settings().lan_mode {
        return error_response(
            StatusCode::CONFLICT,
            "lan_mode_off",
            "Turn on LAN mode to pair a phone",
        );
    }
    let Some(ip) = (s.lan_ip)() else {
        return error_response(
            StatusCode::CONFLICT,
            "no_lan_address",
            "No private network address was found",
        );
    };
    let port = s.guard.port.load(Ordering::Relaxed);
    json(&serde_json::json!({ "url": pairing_url(ip, port, &s.guard.tokens.current()) }))
}

async fn post_token_reset<C: AppCore>(State(s): S<C>) -> Response {
    match s.guard.tokens.reset() {
        Ok(t) => {
            s.guard.limiter.clear();
            json(&serde_json::json!({ "token": t }))
        }
        Err(e) => core_err(CoreError::internal(e.to_string())),
    }
}

async fn api_not_found() -> Response {
    error_response(StatusCode::NOT_FOUND, "not_found", "No such route")
}

async fn static_handler<C: AppCore>(State(s): S<C>, req: Request) -> Response {
    serve_path(
        req.method(),
        req.uri().path(),
        s.web_dir.as_deref(),
        s.assets.as_ref(),
    )
}

pub fn build_router<C: AppCore>(shared: Arc<Shared<C>>) -> Router {
    let guard: Arc<Guard> = shared.guard.clone();
    let api = Router::new()
        .route("/state", get(get_state::<C>))
        .route("/events", get(get_events::<C>))
        .route(
            "/character",
            post(post_character::<C>).delete(delete_character::<C>),
        )
        .route("/dungeons", get(get_dungeons::<C>))
        .route("/search", post(post_search::<C>))
        .route("/search", delete(delete_search::<C>))
        .route("/party/confirm", post(post_confirm::<C>))
        .route("/party/decline", post(post_decline::<C>))
        .route("/settings", get(get_settings::<C>).put(put_settings::<C>))
        .route(
            "/pairing",
            get(get_pairing::<C>).route_layer(from_fn(require_loopback)),
        )
        .route(
            "/token/reset",
            post(post_token_reset::<C>).route_layer(from_fn(require_loopback)),
        )
        .fallback(api_not_found)
        .layer(from_fn_with_state(guard.clone(), require_token));
    Router::new()
        .nest("/api", api)
        .fallback(static_handler::<C>)
        .layer(from_fn_with_state(guard, host_origin))
        .with_state(shared)
}
