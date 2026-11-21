//! Process start-up for the three modes (`--print-public-key`, `--headless`, window).
//!
//! Window mode: open the store, load the identity, start the core and the web server on a
//! random free port, and hand back the URL the window opens (`http://127.0.0.1:<port>/#token=`).
//! Headless mode starts only the network and no server.

use crate::app::memnet::MemNetwork;
use crate::app::{CoreDeps, LiveCore, system_clock};
use crate::cli::{ExportArgs, KeyCommand};
use crate::config::Settings;
use crate::dungeons::TopicId;
use crate::engine::{Event, Message};
use crate::engine::{NetError, Network};
use crate::identity::{Identity, PeerId, Signer};
use crate::keys::{self, KeyError, OldKey};
use crate::net::{IrohConfig, IrohNet, KnownPeers, TrustedPeers, live::LiveNet, trusted};
use crate::server::core::{BootstrapSource, NetworkView};
use crate::server::static_files::Assets;
use crate::server::{Server, ServerConfig};
use crate::store::{DataDirLock, Store, VaraqStore};
use std::io::Write;
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
    /// Where the network publishes peer count and bootstrap source.
    pub status: watch::Sender<NetworkView>,
}

/// Known-peers cache size for a headless peer (it is a relay, so it remembers many).
pub const HEADLESS_KNOWN_PEERS: usize = 5000;

/// The network the app runs on: the real iroh network, or the in-memory one that hears nobody
/// (tests, and `WOWFT_NET=memory` for running the UI without a network).
pub enum AnyNet {
    Live(Box<LiveNet<IrohNet>>),
    Mem(MemNetwork),
}

macro_rules! each {
    ($s:expr, $n:ident => $e:expr) => {
        match $s {
            AnyNet::Live($n) => $e,
            AnyNet::Mem($n) => $e,
        }
    };
}

impl Network for AnyNet {
    async fn join(&mut self, t: TopicId) -> Result<(), NetError> {
        each!(self, n => n.join(t).await)
    }
    async fn leave(&mut self, t: TopicId) -> Result<(), NetError> {
        each!(self, n => n.leave(t).await)
    }
    async fn broadcast(&mut self, t: TopicId, m: Message) -> Result<(), NetError> {
        each!(self, n => n.broadcast(t, m).await)
    }
    async fn send_direct(&mut self, p: PeerId, t: TopicId, m: Message) -> Result<(), NetError> {
        each!(self, n => n.send_direct(p, t, m).await)
    }
    /// Cancel-safe in both variants.
    async fn next_event(&mut self) -> Result<Event, NetError> {
        each!(self, n => n.next_event().await)
    }
    async fn dial(&mut self, p: PeerId) -> Result<(), NetError> {
        match self {
            AnyNet::Live(n) => Network::dial(&mut **n, p).await,
            AnyNet::Mem(n) => Network::dial(n, p).await,
        }
    }
}

/// `WOWFT_NET=memory` selects the in-memory network.
fn use_memory_net() -> bool {
    std::env::var("WOWFT_NET").is_ok_and(|v| v == "memory")
}

/// Build the network: iroh with bootstrap (remembered peers, peer exchange, then the trusted
/// peers from the built-in list and settings), publishing `NetworkView` on `ctx.status`.
/// `next_event` is cancel-safe, as `LiveCore` requires.
pub async fn build_network(ctx: NetContext) -> Result<AnyNet, NetError> {
    if use_memory_net() {
        let (net, _handle) = MemNetwork::new();
        return Ok(AnyNet::Mem(net));
    }
    let settings = Settings::load(ctx.store.as_ref()).unwrap_or_default();
    let cap = if ctx.headless {
        HEADLESS_KNOWN_PEERS
    } else {
        settings.known_peers_cache
    };
    let known =
        KnownPeers::load(cap, ctx.store.clone()).map_err(|e| NetError::Io(e.to_string()))?;
    let (trusted, bad) = TrustedPeers::from_settings(trusted::BUILTIN, &settings);
    for b in bad {
        tracing::warn!("ignoring trusted peer entry that is not a key: {b}");
    }
    let identity = Arc::new(
        Identity::load_or_create(ctx.store.as_ref()).map_err(|e| NetError::Io(e.to_string()))?,
    );
    let iroh = IrohNet::start(identity, known, IrohConfig::Default).await?;
    Ok(AnyNet::Live(Box::new(LiveNet::new(
        iroh,
        ctx.me,
        trusted,
        ctx.headless,
        ctx.status,
    ))))
}

/// Take the data directory lock, then open the store. The lock must outlive the store: keep it
/// for as long as the process uses the directory (ADR 0039, decision 9).
pub fn open_store(data_dir: &Path) -> Result<(DataDirLock, Arc<dyn Store>), AnyError> {
    let lock = DataDirLock::acquire(data_dir)?;
    Ok((lock, Arc::new(VaraqStore::open(data_dir)?)))
}

fn show_old(old: &OldKey) -> String {
    match old {
        OldKey::None => "none".to_owned(),
        OldKey::Id(p) => p.to_hex(),
        OldKey::Unreadable => "unreadable".to_owned(),
    }
}

const CHANGED_WARNING: &str = "Other nodes now see a new peer ID. If this is a trusted peer, its entry in net::trusted::BUILTIN needs a new release.";

/// The `key` subcommands (ADR 0039). Takes the data directory lock first, so it refuses while the
/// app (or another `key` command) uses the directory. Output goes to `out` (data) and `err`
/// (notes), and `confirm` answers the reset prompt, so tests can drive it.
pub fn run_key_command<O: Write, E: Write>(
    data_dir: &Path,
    cmd: &KeyCommand,
    out: &mut O,
    err: &mut E,
    confirm: impl FnOnce(&str) -> bool,
) -> Result<(), AnyError> {
    let _lock = DataDirLock::acquire(data_dir)?;
    let store = VaraqStore::open(data_dir)?;
    let key_error = |e: KeyError| -> AnyError {
        match e {
            KeyError::WouldReplace(r) => format!(
                "a key is already stored (peer {}); the file holds peer {}. Pass --force to replace it",
                show_old(&r.old),
                r.new
            )
            .into(),
            other => other.into(),
        }
    };
    match cmd {
        KeyCommand::Show => {
            let info = keys::show(&store).map_err(key_error)?;
            writeln!(out, "{}\n{}", info.peer_id, info.openssh_public)?;
        }
        KeyCommand::Export(ExportArgs { stdout: true, .. }) => {
            let (peer, text) = keys::export_text(&store).map_err(key_error)?;
            write!(out, "{text}")?;
            writeln!(err, "peer {peer}")?;
        }
        KeyCommand::Export(ExportArgs {
            file: Some(path), ..
        }) => {
            let peer = keys::export_to_file(&store, path).map_err(key_error)?;
            writeln!(out, "wrote the key for peer {peer} to {}", path.display())?;
        }
        KeyCommand::Export(_) => return Err("give a file, or --stdout".into()),
        KeyCommand::Import { file, force } => {
            let r = keys::import_from_file(&store, file, *force).map_err(key_error)?;
            writeln!(
                out,
                "imported the key; peer {} -> {}",
                show_old(&r.old),
                r.new
            )?;
            if r.old != OldKey::None {
                writeln!(err, "{CHANGED_WARNING}")?;
            }
        }
        KeyCommand::Reset { yes } => {
            let old = keys::old_key(&store).map_err(key_error)?;
            if old != OldKey::None && !yes {
                let q = format!(
                    "Replace this install's key (peer {}) with a new one? {} [y/N] ",
                    show_old(&old),
                    "Other nodes will see a new peer."
                );
                if !confirm(&q) {
                    return Err("not confirmed; nothing changed".into());
                }
            }
            let r = keys::reset(&store).map_err(key_error)?;
            writeln!(out, "new key; peer {} -> {}", show_old(&r.old), r.new)?;
            if r.old != OldKey::None {
                writeln!(err, "{CHANGED_WARNING}")?;
            }
        }
    }
    Ok(())
}

/// Everything started for window mode. Keep it alive for the life of the app.
pub struct Running {
    pub runtime: tokio::runtime::Runtime,
    pub server: Server,
    pub core: Arc<LiveCore>,
    pub window_url: String,
    /// Held so no other process (a second app, a `key` command) uses this data directory.
    /// Declared last so it is released after everything that uses the store.
    pub _lock: DataDirLock,
}

pub fn start_window(data_dir: &Path, assets: Arc<dyn Assets>) -> Result<Running, AnyError> {
    let (lock, store) = open_store(data_dir)?;
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
        status: status_tx,
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
        _lock: lock,
        runtime,
        server,
        core,
        window_url,
    })
}

/// `--headless`: join the network and relay until interrupted. No web server, no window.
pub fn run_headless(data_dir: &Path) -> Result<(), AnyError> {
    let (_lock, store) = open_store(data_dir)?;
    let me = Identity::load_or_create(store.as_ref())?.peer_id();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        let (status, _rx) = watch::channel(NetworkView {
            peers: 0,
            bootstrap: BootstrapSource::None,
        });
        let mut net = build_network(NetContext {
            me,
            store,
            headless: true,
            status,
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
