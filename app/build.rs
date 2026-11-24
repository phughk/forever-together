//! Build script: runs the Tauri build, then bundles `../addon/ForeverTogether/` into
//! `$OUT_DIR/addon_bundle.rs` so the app can install its own copy of the addon.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const ADDON_DIR: &str = "../addon/ForeverTogether";

fn main() {
    #[cfg(feature = "ui")]
    tauri_build::build();
    bundle_addon();
}

/// Collects `(relative path with '/', absolute path)` for every non-dot file under `dir`.
fn walk(dir: &Path, prefix: &str, out: &mut Vec<(String, PathBuf)>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.expect("read addon dir entry");
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let rel = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}/{name}")
        };
        let path = entry.path();
        if path.is_dir() {
            walk(&path, &rel, out);
        } else {
            out.push((rel, path));
        }
    }
}

/// First `## Version:` line of a `.toc`, or `None`.
fn toc_version(text: &str) -> Option<String> {
    let text = text.trim_start_matches('\u{feff}');
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("##") else {
            continue;
        };
        let Some((key, value)) = rest.split_once(':') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("version") {
            let v = value.trim();
            return if v.is_empty() {
                None
            } else {
                Some(v.to_string())
            };
        }
    }
    None
}

fn bundle_addon() {
    println!("cargo:rerun-if-changed={ADDON_DIR}");
    let root = Path::new(&std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"))
        .join(ADDON_DIR);
    let mut files = Vec::new();
    walk(&root, "", &mut files);
    files.sort();
    let toc = files
        .iter()
        .find(|(rel, _)| rel == "ForeverTogether.toc")
        .unwrap_or_else(|| panic!("{} has no ForeverTogether.toc", root.display()));
    let text = std::fs::read_to_string(&toc.1).expect("read ForeverTogether.toc as UTF-8");
    let version = toc_version(&text).expect("ForeverTogether.toc has no `## Version:` line");
    let dotted = version
        .split('.')
        .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
    assert!(
        dotted,
        "addon version {version:?} is not a dotted number such as 0.1.0"
    );

    let mut code = String::new();
    writeln!(code, "pub static BUNDLE: &[BundledFile] = &[").unwrap();
    for (rel, abs) in &files {
        let abs = abs.to_string_lossy().into_owned();
        writeln!(
            code,
            "    BundledFile {{ path: {rel:?}, bytes: include_bytes!({abs:?}) }},"
        )
        .unwrap();
    }
    writeln!(code, "];").unwrap();
    writeln!(code, "pub const BUNDLED_VERSION: &str = {version:?};").unwrap();
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR")).join("addon_bundle.rs");
    std::fs::write(out, code).expect("write addon_bundle.rs");
}
