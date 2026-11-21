use super::*;
use crate::server::static_files::MapAssets;
use std::io::{Read, Write};

fn temp(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("wowft-runtime-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
fn public_key_is_created_once_and_stable() {
    let d = temp("key");
    let a = public_key(&d).unwrap();
    let b = public_key(&d).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.len(), 64);
    assert!(PeerId::from_hex(&a).is_some());
    let other = temp("key2");
    assert_ne!(a, public_key(&other).unwrap());
    let _ = std::fs::remove_dir_all(d);
    let _ = std::fs::remove_dir_all(other);
}

fn get(port: u16, path: &str, token: Option<&str>) -> String {
    let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
    let auth = token
        .map(|t| format!("Authorization: Bearer {t}\r\n"))
        .unwrap_or_default();
    write!(
        s,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{auth}Connection: close\r\n\r\n"
    )
    .unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    out
}

#[test]
fn window_mode_starts_the_server_on_a_free_port_with_the_token_in_the_url() {
    // No sockets to n0 from a unit test.
    unsafe { std::env::set_var("WOWFT_NET", "memory") };
    let d = temp("window");
    let assets = Arc::new(MapAssets(
        [("index.html".to_owned(), b"<p>hi</p>".to_vec())]
            .into_iter()
            .collect(),
    ));
    let r = start_window(&d, assets).unwrap();
    let port = r.server.port();
    let prefix = format!("http://127.0.0.1:{port}/#token=");
    let token = r
        .window_url
        .strip_prefix(&prefix)
        .expect("url shape")
        .to_owned();
    assert_eq!(token.len(), 64);
    assert!(get(port, "/", None).ends_with("<p>hi</p>"));
    assert!(get(port, "/api/state", None).starts_with("HTTP/1.1 401"));
    let ok = get(port, "/api/state", Some(&token));
    assert!(ok.starts_with("HTTP/1.1 200"), "{ok}");
    assert!(ok.contains("\"serverNow\""));
    // The token survives a restart of the app.
    drop(r);
    let again = start_window(&d, Arc::new(MapAssets::default())).unwrap();
    assert!(again.window_url.ends_with(&token));
    drop(again);
    let _ = std::fs::remove_dir_all(d);
}
