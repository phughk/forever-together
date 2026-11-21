//! Process start-up for the three modes (`--print-public-key`, `--headless`, window).
//!
//! Window mode: open the store, load the identity, start the core and the web server on a
//! random free port, and hand back the URL the window opens (`http://127.0.0.1:<port>/#token=`).
//! Headless mode starts only the network and no server.

use crate::app::memnet::MemNetwork;
use crate::app::{CoreDeps, LiveCore, system_clock};
use crate::engine::{NetError, Network};
use crate::identity::{Identity, PeerId, Signer};
use crate::server::core::{BootstrapSource, NetworkView};
use crate::server::static_files::Assets;
use crate::server::{Server, ServerConfig};
use crate::store::{Store, VaraqStore};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::watch;

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;

pub type AnyError = Box<dyn std::error::Error + Send + Sync>;

/// What the network layer needs to start.
pub struct NetContext {
    pub me: PeerId,
    pub store: Arc<dyn Store>,
    /// A headless peer keeps a large known-peers cache and has no search of its own.
    pub headless: bool,
}

/// SEAM: the `net` module plugs in here. Replace the body with the iroh-backed network
/// (`net::IrohNetwork::start(ctx).await`) and change the return type; nothing else in the app
/// names the concrete type. Until then the app runs on the in-memory network, which accepts
/// every join and broadcast and hears nobody. Requirements: `next_event` must be cancel-safe,
/// and the network publishes `NetworkView` (peer count, bootstrap source) on the `watch`
/// sender that `start_window` creates (hand it to the network through `NetContext`).
pub async fn build_network(_ctx: NetContext) -> Result<MemNetwork, NetError> {
    let (net, _handle) = MemNetwork::new();
    Ok(net)
}

pub fn open_store(data_dir: &Path) -> Result<Arc<dyn Store>, AnyError> {
    std::fs::create_dir_all(data_dir)?;
    Ok(Arc::new(VaraqStore::open(data_dir)?))
}

/// `--print-public-key`: the peer ID, created and saved on first use.
pub fn public_key(data_dir: &Path) -> Result<String, AnyError> {
    let store = open_store(data_dir)?;
    Ok(Identity::load_or_create(store.as_ref())?.peer_id().to_hex())
}

/// Everything started for window mode. Keep it alive for the life of the app.
pub struct Running {
    pub runtime: tokio::runtime::Runtime,
    pub server: Server,
    pub core: Arc<LiveCore>,
    pub window_url: String,
    _net_status: watch::Sender<NetworkView>,
}

pub fn start_window(data_dir: &Path, assets: Arc<dyn Assets>) -> Result<Running, AnyError> {
    let store = open_store(data_dir)?;
    let me = Identity::load_or_create(store.as_ref())?.peer_id();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .thread_name("wowft-server")
        .build()?;
    let (status_tx, status_rx) = watch::channel(NetworkView {
        peers: 0,
        bootstrap: BootstrapSource::None,
    });
    let ctx = NetContext {
        me,
        store: store.clone(),
        headless: false,
    };
    let core = Arc::new(LiveCore::spawn(
        CoreDeps {
            store: store.clone(),
            me,
            clock: system_clock(),
            net_status: status_rx,
        },
        move || build_network(ctx),
    ));
    let lan = crate::config::Settings::load(store.as_ref())
        .map(|s| s.lan_mode)
        .unwrap_or(false);
    let server = runtime.block_on(Server::start(
        core.clone(),
        store,
        assets,
        ServerConfig::from_env(lan),
    ))?;
    let window_url = server.window_url();
    // The port is printed, the token never is.
    eprintln!(
        "forever-together: web server on http://127.0.0.1:{}/",
        server.port()
    );
    Ok(Running {
        runtime,
        server,
        core,
        window_url,
        _net_status: status_tx,
    })
}

/// `--headless`: join the network and relay until interrupted. No web server, no window.
pub fn run_headless(data_dir: &Path) -> Result<(), AnyError> {
    let store = open_store(data_dir)?;
    let me = Identity::load_or_create(store.as_ref())?.peer_id();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        let mut net = build_network(NetContext {
            me,
            store,
            headless: true,
        })
        .await?;
        tracing::info!("headless peer {me} running; Ctrl-C to stop");
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => break,
                ev = net.next_event() => {
                    if let Err(NetError::Closed) = ev {
                        return Err("the network closed".into());
                    }
                }
            }
        }
        Ok::<(), AnyError>(())
    })
}
