use super::*;
use crate::cli::{ExportArgs, KeyCommand};
use crate::server::static_files::MapAssets;
use std::io::{Read, Write};

fn temp(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("wowft-runtime-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// Run a key command against `d`, returning (stdout, stderr) or the error message.
fn key(d: &std::path::Path, cmd: KeyCommand, confirm: bool) -> Result<(String, String), String> {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    run_key_command(d, &cmd, &mut out, &mut err, |_| confirm).map_err(|e| e.to_string())?;
    Ok((
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    ))
}

fn show_json(show_output: &str) -> serde_json::Value {
    serde_json::from_str(show_output).unwrap()
}

fn peer_of(show_output: &str) -> String {
    show_json(show_output)["peer_id"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn key_show_creates_a_key_once_and_prints_the_peer_id_then_the_public_line() {
    let d = temp("show");
    let (a, _) = key(&d, KeyCommand::Show, false).unwrap();
    let (b, _) = key(&d, KeyCommand::Show, false).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.lines().count(), 1, "one line of JSON");
    let v = show_json(&a);
    assert_eq!(v.as_object().unwrap().len(), 2);
    let peer = v["peer_id"].as_str().unwrap();
    assert_eq!(peer.len(), 64);
    assert!(PeerId::from_hex(peer).is_some());
    assert!(
        v["public_key"]
            .as_str()
            .unwrap()
            .starts_with("ssh-ed25519 AAAA")
    );
    let other = temp("show2");
    assert_ne!(a, key(&other, KeyCommand::Show, false).unwrap().0);
    let _ = std::fs::remove_dir_all(d);
    let _ = std::fs::remove_dir_all(other);
}

#[test]
fn import_before_first_start_gives_the_node_that_identity() {
    // The deploy path: a cloud node gets its key by `key import`, then starts.
    let src = temp("deploy-src");
    let dst = temp("deploy-dst");
    let file = std::env::temp_dir().join(format!("wowft-deploy-{}.key", std::process::id()));
    let _ = std::fs::remove_file(&file);
    let wanted = peer_of(&key(&src, KeyCommand::Show, false).unwrap().0);
    key(
        &src,
        KeyCommand::Export(ExportArgs {
            file: Some(file.clone()),
            stdout: false,
        }),
        false,
    )
    .unwrap();
    key(
        &dst,
        KeyCommand::Import {
            file: file.clone(),
            force: false,
        },
        false,
    )
    .unwrap();
    // What the app loads at start is that key.
    let (lock, store) = open_store(&dst).unwrap();
    let me = Identity::load_or_create(store.as_ref()).unwrap().peer_id();
    assert_eq!(me.to_hex(), wanted);
    drop((lock, store));
    assert_eq!(
        peer_of(&key(&dst, KeyCommand::Show, false).unwrap().0),
        wanted
    );
    let _ = std::fs::remove_file(file);
    let _ = std::fs::remove_dir_all(src);
    let _ = std::fs::remove_dir_all(dst);
}

#[test]
fn a_key_made_by_show_needs_force_before_import_replaces_it() {
    let d = temp("force");
    let file = std::env::temp_dir().join(format!("wowft-force-{}.key", std::process::id()));
    let _ = std::fs::remove_file(&file);
    let other = temp("force-src");
    key(
        &other,
        KeyCommand::Export(ExportArgs {
            file: Some(file.clone()),
            stdout: false,
        }),
        false,
    )
    .unwrap();
    let first = peer_of(&key(&d, KeyCommand::Show, false).unwrap().0);
    let import = |force| KeyCommand::Import {
        file: file.clone(),
        force,
    };
    let e = key(&d, import(false), false).unwrap_err();
    assert!(e.contains("--force") && e.contains(&first), "{e}");
    assert_eq!(peer_of(&key(&d, KeyCommand::Show, false).unwrap().0), first);
    let (out, err) = key(&d, import(true), false).unwrap();
    assert!(out.contains(&first));
    assert!(err.contains("BUILTIN"));
    assert_ne!(peer_of(&key(&d, KeyCommand::Show, false).unwrap().0), first);
    let _ = std::fs::remove_file(file);
    let _ = std::fs::remove_dir_all(d);
    let _ = std::fs::remove_dir_all(other);
}

#[test]
fn export_to_stdout_prints_the_key_and_notes_the_peer_on_stderr() {
    let d = temp("stdout");
    let peer = peer_of(&key(&d, KeyCommand::Show, false).unwrap().0);
    let (out, err) = key(
        &d,
        KeyCommand::Export(ExportArgs {
            file: None,
            stdout: true,
        }),
        false,
    )
    .unwrap();
    assert!(out.starts_with("-----BEGIN OPENSSH PRIVATE KEY-----"));
    assert!(!err.contains("BEGIN"));
    assert!(err.contains(&peer));
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn reset_asks_first_and_only_a_yes_changes_the_key() {
    let d = temp("reset");
    let first = peer_of(&key(&d, KeyCommand::Show, false).unwrap().0);
    let e = key(&d, KeyCommand::Reset { yes: false }, false).unwrap_err();
    assert!(e.contains("nothing changed"), "{e}");
    assert_eq!(peer_of(&key(&d, KeyCommand::Show, false).unwrap().0), first);
    // Answering yes at the prompt, or passing --yes, replaces the key.
    key(&d, KeyCommand::Reset { yes: false }, true).unwrap();
    let second = peer_of(&key(&d, KeyCommand::Show, false).unwrap().0);
    assert_ne!(second, first);
    let (out, err) = key(&d, KeyCommand::Reset { yes: true }, false).unwrap();
    assert_eq!(
        out,
        format!(
            "old: {second}\nnew: {}\n",
            peer_of(&key(&d, KeyCommand::Show, false).unwrap().0)
        )
    );
    assert!(err.is_empty(), "{err}");
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn reset_on_an_empty_directory_needs_no_confirmation() {
    let d = temp("reset-empty");
    let (out, err) = key(&d, KeyCommand::Reset { yes: false }, false).unwrap();
    assert!(out.starts_with("old: none\nnew: "));
    assert!(err.is_empty());
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn key_commands_are_refused_while_another_process_holds_the_data_directory() {
    let d = temp("locked");
    let held = DataDirLock::acquire(&d).unwrap();
    for cmd in [
        KeyCommand::Show,
        KeyCommand::Reset { yes: true },
        KeyCommand::Export(ExportArgs {
            file: None,
            stdout: true,
        }),
    ] {
        let e = key(&d, cmd, true).unwrap_err();
        assert!(e.contains("another forever-together process"), "{e}");
    }
    drop(held);
    key(&d, KeyCommand::Show, false).unwrap();
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn the_running_app_holds_the_directory_so_key_commands_wait_for_it() {
    unsafe { std::env::set_var("WOWFT_NET", "memory") };
    let d = temp("app-lock");
    let running = start_window(&d, Arc::new(MapAssets::default())).unwrap();
    assert!(key(&d, KeyCommand::Reset { yes: true }, true).is_err());
    // A second app on the same directory is refused too.
    assert!(start_window(&d, Arc::new(MapAssets::default())).is_err());
    drop(running);
    key(&d, KeyCommand::Show, false).unwrap();
    let _ = std::fs::remove_dir_all(d);
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
