//! Static files: first from the `WOWFT_WEB_DIR` folder (when set), then the embedded web app.
//! `/api` is never served from here.
//!
//! The embedded files come through the `Assets` trait. The desktop app implements it over
//! Tauri's asset resolver (the built `dist` is embedded by `generate_context!`, so no second
//! embedding crate is needed). Tests use a map.

use super::auth::error_response;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

pub struct Asset {
    pub bytes: Vec<u8>,
    pub mime: String,
}

/// The embedded files. `path` has no leading slash; the empty string is the root.
pub trait Assets: Send + Sync + 'static {
    fn get(&self, path: &str) -> Option<Asset>;
}

/// A fixed set of files, for tests and for builds without a window.
#[derive(Default)]
pub struct MapAssets(pub BTreeMap<String, Vec<u8>>);

impl Assets for MapAssets {
    fn get(&self, path: &str) -> Option<Asset> {
        self.0.get(path).map(|b| Asset {
            bytes: b.clone(),
            mime: mime_for(path).to_owned(),
        })
    }
}

pub fn mime_for(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "webmanifest" => "application/manifest+json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "txt" => "text/plain; charset=utf-8",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

fn percent_decode(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            let hex = std::str::from_utf8(b.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// Turn a request path into safe relative segments. `None` for anything that tries to leave
/// the folder: `..`, a backslash, a NUL, a drive or an absolute component, or an empty-looking
/// path after decoding. A `.` or empty segment is skipped.
pub fn normalize(request_path: &str) -> Option<Vec<String>> {
    let decoded = percent_decode(request_path)?;
    if decoded.contains('\0') || decoded.contains('\\') {
        return None;
    }
    let mut segs = Vec::new();
    for s in decoded.split('/') {
        match s {
            "" | "." => {}
            ".." => return None,
            s if s.contains(':') => return None,
            s => segs.push(s.to_owned()),
        }
    }
    Some(segs)
}

/// Read `segs` from inside `root`. The canonical path must stay under the canonical root, so
/// symlinks cannot lead out of the folder either.
fn read_override(root: &Path, segs: &[String]) -> Option<Vec<u8>> {
    let mut p = PathBuf::from(root);
    for s in segs {
        // Every segment is a plain name (normalize refused the rest).
        if Path::new(s)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        {
            return None;
        }
        p.push(s);
    }
    let canon = p.canonicalize().ok()?;
    let root = root.canonicalize().ok()?;
    if !canon.starts_with(&root) || !canon.is_file() {
        return None;
    }
    std::fs::read(canon).ok()
}

fn serve(asset: Asset, head: bool) -> Response {
    let mut r = Response::new(if head {
        Body::empty()
    } else {
        Body::from(asset.bytes)
    });
    let h = r.headers_mut();
    h.insert(header::CONTENT_TYPE, asset.mime.parse().unwrap());
    h.insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    h.insert(header::REFERRER_POLICY, "no-referrer".parse().unwrap());
    h.insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());
    r
}

/// Look up one file: override folder first, then embedded.
fn lookup(web_dir: Option<&Path>, assets: &dyn Assets, segs: &[String]) -> Option<Asset> {
    let rel = segs.join("/");
    if let Some(root) = web_dir
        && let Some(bytes) = read_override(root, segs)
    {
        return Some(Asset {
            bytes,
            mime: mime_for(&rel).to_owned(),
        });
    }
    assets.get(&rel)
}

/// Serve a GET or HEAD for a non-API path. A path that is not a file and has no extension
/// falls back to `index.html`, so the single-page app can own its routes.
pub fn serve_path(
    method: &Method,
    request_path: &str,
    web_dir: Option<&Path>,
    assets: &dyn Assets,
) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return error_response(
            StatusCode::METHOD_NOT_ALLOWED,
            "method_not_allowed",
            "GET only",
        );
    }
    let head = method == Method::HEAD;
    let Some(mut segs) = normalize(request_path) else {
        return error_response(StatusCode::BAD_REQUEST, "bad_path", "Bad path");
    };
    // `/api` is never a file, however it is spelled (`/%61pi`, `//api`, `/./api`).
    if segs.first().is_some_and(|s| s.eq_ignore_ascii_case("api")) {
        return error_response(StatusCode::NOT_FOUND, "not_found", "No such route");
    }
    if segs.is_empty() {
        segs.push("index.html".into());
    }
    if let Some(a) = lookup(web_dir, assets, &segs) {
        return serve(a, head);
    }
    let last = segs.last().map(String::as_str).unwrap_or("");
    if !last.contains('.')
        && let Some(a) = lookup(web_dir, assets, &["index.html".to_owned()])
    {
        return serve(a, head);
    }
    error_response(StatusCode::NOT_FOUND, "not_found", "No such file")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_refuses_escapes() {
        assert_eq!(normalize("/a/b.js"), Some(vec!["a".into(), "b.js".into()]));
        assert_eq!(normalize("//a/./b"), Some(vec!["a".into(), "b".into()]));
        assert_eq!(normalize("/"), Some(vec![]));
        assert!(normalize("/../secret").is_none());
        assert!(normalize("/a/../../secret").is_none());
        assert!(normalize("/%2e%2e/secret").is_none());
        assert!(normalize("/%2E%2e%2fsecret").is_none());
        assert!(normalize("/a%5c..%5csecret").is_none());
        assert!(normalize("/a\\b").is_none());
        assert!(normalize("/a%00b").is_none());
        assert!(normalize("/C:/x").is_none());
        assert!(normalize("/%zz").is_none());
    }

    #[test]
    fn mime_types() {
        assert!(mime_for("a/b.js").starts_with("text/javascript"));
        assert_eq!(mime_for("x.png"), "image/png");
        assert_eq!(mime_for("noext"), "application/octet-stream");
    }
}
