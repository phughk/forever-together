//! The addon installer routes (ADR 0028). All three are loopback only, including the GET,
//! because they expose and change folder paths. The module `addon_install` does the work; these
//! routes translate HTTP to it. Filesystem work runs on the blocking pool. The install route
//! never takes a path: it uses only the folder saved by `PUT /api/addon/folder`.

use super::Shared;
use super::core::{AppCore, CoreError};
use super::routes::{core_err, json, read_json};
use crate::addon_install::{self, AddonError, AddonReport};
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::response::Response;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

type S<C> = State<Arc<Shared<C>>>;

/// The production candidate provider: real environment, real home, current OS.
pub fn detect_real_candidates() -> Vec<PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from);
    addon_install::detect_candidates(
        &|k| std::env::var(k).ok(),
        home.as_deref(),
        addon_install::current_os(),
    )
}

pub(super) fn addon_err(e: &AddonError) -> CoreError {
    let status = StatusCode::from_u16(e.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    CoreError::new(status, e.code(), e.message()).with_detail(e.detail())
}

fn view(report: &AddonReport, source: Option<&str>, candidates: &[PathBuf]) -> serde_json::Value {
    serde_json::json!({
        "bundled_version": report.bundled_version,
        "folder": report.folder.as_ref().map(|p| p.to_string_lossy().into_owned()),
        "folder_source": source,
        "candidates": candidates
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        "installed": {
            "state": report.installed.state.as_str(),
            "version": report.installed.version,
        },
        "status": report.status.as_str(),
    })
}

// The error is a ready-made response, returned straight to the caller.
#[allow(clippy::result_large_err)]
async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> Result<T, Response> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| core_err(CoreError::internal(e.to_string())))
}

/// The report for the saved folder, or for the single detected candidate, or for none.
#[allow(clippy::result_large_err)]
async fn current_view<C: AppCore>(s: &Shared<C>) -> Result<serde_json::Value, Response> {
    let saved = s.core.addon_dir().await;
    let provider = s.addon_candidates.clone();
    blocking(move || {
        let candidates = provider();
        let (folder, source): (Option<PathBuf>, Option<&str>) = match saved {
            Some(p) => (Some(PathBuf::from(p)), Some("saved")),
            None if candidates.len() == 1 => (Some(candidates[0].clone()), Some("detected")),
            None => (None, None),
        };
        let report = addon_install::inspect(folder.as_deref());
        view(&report, source, &candidates)
    })
    .await
}

pub async fn get_addon<C: AppCore>(State(s): S<C>) -> Response {
    match current_view(&s).await {
        Ok(v) => json(&v),
        Err(r) => r,
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FolderBody {
    folder: String,
}

pub async fn put_folder<C: AppCore>(State(s): S<C>, req: Request) -> Response {
    let body: FolderBody = match read_json(req.into_body()).await {
        Ok(b) => b,
        Err(r) => return r,
    };
    let checked = match blocking(move || addon_install::validate_folder(&body.folder, true)).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let canon = match checked {
        Ok(p) => p,
        Err(e) => return core_err(addon_err(&e)),
    };
    if let Err(e) = s
        .core
        .set_addon_dir(Some(canon.to_string_lossy().into_owned()))
        .await
    {
        return core_err(e);
    }
    match current_view(&s).await {
        Ok(v) => json(&v),
        Err(r) => r,
    }
}

pub async fn post_install<C: AppCore>(State(s): S<C>) -> Response {
    let Some(saved) = s.core.addon_dir().await else {
        return core_err(addon_err(&AddonError::FolderUnset));
    };
    let lock = s.addon_lock.clone();
    let provider = s.addon_candidates.clone();
    let done = blocking(move || {
        // One install at a time. A poisoned lock only means an earlier install panicked.
        let _guard = lock.lock().unwrap_or_else(|p| p.into_inner());
        let folder: &Path = Path::new(&saved);
        addon_install::install(folder).map(|r| view(&r, Some("saved"), &provider()))
    })
    .await;
    match done {
        Ok(Ok(v)) => json(&v),
        Ok(Err(e)) => core_err(addon_err(&e)),
        Err(r) => r,
    }
}
