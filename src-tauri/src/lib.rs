pub mod app;
pub mod cli;
pub mod code;
pub mod config;
pub mod dungeons;
pub mod engine;
pub mod identity;
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
        Mode::PrintPublicKey => match runtime::public_key(&data_dir) {
            Ok(k) => println!("{k}"),
            Err(e) => fail(e),
        },
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
            WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External(running.window_url.parse()?),
            )
            .title("Forever Together")
            .inner_size(1000.0, 720.0)
            .build()?;
            app.manage(running);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
