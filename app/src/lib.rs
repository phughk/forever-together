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
#[cfg(test)]
mod live_tests;
pub mod net;
pub mod runtime;
pub mod server;
pub mod store;

use clap::Parser;
use cli::{Cli, Mode};
#[cfg(feature = "ui")]
use server::static_files::{Asset, Assets};
#[cfg(feature = "ui")]
use std::sync::Arc;
#[cfg(feature = "ui")]
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

/// The embedded web app, read through Tauri's asset resolver (`generate_context!` embeds the
/// built `dist`).
#[cfg(feature = "ui")]
struct TauriAssets(tauri::AssetResolver<tauri::Wry>);

#[cfg(feature = "ui")]
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

/// Send `tracing` output to stderr. `RUST_LOG` picks the filter (for example
/// `RUST_LOG=app_lib=debug,iroh=info`); without it only warnings and errors show, so a
/// player's terminal stays quiet. `WOWFT_LOG_FORMAT=json` switches to JSON lines. Safe to call twice (the second call is ignored).
fn init_logging() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr);
    // WOWFT_LOG_FORMAT=json: one JSON object per line (for log shippers); anything else is text.
    let _ = if std::env::var("WOWFT_LOG_FORMAT").is_ok_and(|v| v.eq_ignore_ascii_case("json")) {
        builder.json().try_init()
    } else {
        builder.try_init()
    };
}

#[cfg_attr(feature = "ui", cfg_attr(mobile, tauri::mobile_entry_point))]
pub fn run() {
    let cli = Cli::parse();
    let data_dir = cli.resolve_data_dir();
    // Key commands print results on stdout and stderr and never log.
    if !matches!(cli.mode(), Mode::Key(_)) {
        init_logging();
    }
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
        Mode::Headless { relay } => {
            if let Err(e) = runtime::run_headless(&data_dir, relay) {
                fail(e);
            }
        }
        #[cfg(feature = "ui")]
        Mode::Window => run_window(data_dir),
        #[cfg(not(feature = "ui"))]
        Mode::Window => fail(
            "this build has no window (built without the `ui` feature); run it with --headless",
        ),
    }
}

#[cfg(feature = "ui")]
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
