// Prevents an extra console window on Windows in release.
// Only the windowed build; a headless-only build keeps its console.
#![cfg_attr(
    all(not(debug_assertions), feature = "ui"),
    windows_subsystem = "windows"
)]

fn main() {
    app_lib::run()
}
