pub mod addon_install;
pub mod app;
pub mod cli;
pub mod code;
pub mod config;
pub mod dungeons;
pub mod engine;
pub mod history;
pub mod identity;
#[cfg(test)]
mod integration_tests;
pub mod keys;
pub mod net;
pub mod runtime;
pub mod server;
pub mod store;

use clap::Parser;
use cli::{Cli, Mode};
use server::static_files::{Asset, Assets};
use std::sync::Arc;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

/// The embedded web app, read through Tauri's asset resolver (`generate_context!` embeds the
/// built `dist`).
struct TauriAssets(tauri::AssetResolver<tauri::Wry>);

impl Assets for TauriAssets {
    fn get(&self, path: &str) -> Option<Asset> {
        let a = self.0.get(path.to_owned())?;
        Some(Asset {
            bytes: a.bytes,
            mime: a.mime_type,
        })
    }
}

fn fail(msg: impl std::fmt::Display) -> ! {
    eprintln!("forever-together: {msg}");
    std::process::exit(1)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cli = Cli::parse();
    let data_dir = cli.resolve_data_dir();
    match cli.mode() {
        Mode::Key(cmd) => {
            let confirm = |question: &str| {
                use std::io::{BufRead, Write};
                eprint!("{question}");
                let _ = std::io::stderr().flush();
                let mut line = String::new();
                std::io::stdin().lock().read_line(&mut line).is_ok()
                    && matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes")
            };
            if let Err(e) = runtime::run_key_command(
                &data_dir,
                &cmd,
                &mut std::io::stdout(),
                &mut std::io::stderr(),
                confirm,
            ) {
                fail(e);
            }
        }
        Mode::Headless => {
            if let Err(e) = runtime::run_headless(&data_dir) {
                fail(e);
            }
        }
        Mode::Window => run_window(data_dir),
    }
}

fn run_window(data_dir: std::path::PathBuf) {
    tauri::Builder::default()
        .setup(move |app| {
            let assets = Arc::new(TauriAssets(app.asset_resolver()));
            let running = runtime::start_window(&data_dir, assets).map_err(|e| e.to_string())?;
            // The window is created here, not in tauri.conf.json: it loads the local server and
            // has no IPC capability, so the page can only do what the web API allows.
            let mut window = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External(running.window_url.parse()?),
            )
            .title("Forever Together")
            .inner_size(1000.0, 720.0);
            // The app icon from tauri.conf.json (macOS takes its Dock icon from the bundle).
            if let Some(icon) = app.default_window_icon() {
                window = window.icon(icon.clone())?;
            }
            window.build()?;
            app.manage(running);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
