//! The local web server (ADR 0012, plan "Local web API").
//!
//! One axum server on a random free port serves the embedded web app and the REST + event
//! stream API, to the desktop window on `127.0.0.1` and, in LAN mode, to phones. `AppCore`
//! (see `core`) is everything it needs from the app. Turning LAN mode on or off restarts the
//! listener on the same port (`127.0.0.1` <-> `0.0.0.0`).

pub mod auth;
pub mod core;
pub mod events;
pub mod pairing;
pub mod routes;
pub mod static_files;

#[cfg(test)]
mod tests;

use crate::store::Store;
use auth::{Guard, TokenManager};
use axum::Router;
use core::AppCore;
use static_files::Assets;
use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::task::JoinHandle;

/// Environment variable naming a folder whose files take priority over the embedded ones.
pub const WEB_DIR_ENV: &str = "WOWFT_WEB_DIR";

/// How long an old listener may take to wind down before it is aborted.
const STOP_GRACE: Duration = Duration::from_secs(2);

pub struct ServerConfig {
    pub web_dir: Option<PathBuf>,
    pub heartbeat: Duration,
    pub lan: bool,
    /// 0 picks a random free port.
    pub port: u16,
    pub lan_ips: Arc<dyn Fn() -> Vec<Ipv4Addr> + Send + Sync>,
}

impl ServerConfig {
    /// Production defaults: the override folder from `WOWFT_WEB_DIR`, a 15 s heartbeat and the
    /// detected LAN address.
    pub fn from_env(lan: bool) -> Self {
        Self {
            web_dir: std::env::var_os(WEB_DIR_ENV)
                .filter(|v| !v.is_empty())
                .map(PathBuf::from),
            heartbeat: events::HEARTBEAT,
            lan,
            port: 0,
            lan_ips: Arc::new(pairing::detect_lan_ips),
        }
    }
}

/// Everything a handler can reach.
pub struct Shared<C> {
    pub core: Arc<C>,
    pub guard: Arc<Guard>,
    pub assets: Arc<dyn Assets>,
    pub web_dir: Option<PathBuf>,
    pub heartbeat: Duration,
    pub lan_ips: Arc<dyn Fn() -> Vec<Ipv4Addr> + Send + Sync>,
    /// Bumped when the listener restarts, which ends open event streams.
    pub listener_generation: watch::Sender<u64>,
    ctl: mpsc::UnboundedSender<Ctl>,
}

impl<C> Shared<C> {
    /// Ask the listener controller to switch LAN mode. Delayed a moment so the response that
    /// triggered it reaches the caller before the old listener goes away.
    pub fn request_lan(&self, lan: bool) {
        let ctl = self.ctl.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(150)).await;
            let _ = ctl.send(Ctl::Lan(lan));
        });
    }
}

enum Ctl {
    Lan(bool),
    Stop(oneshot::Sender<()>),
}

struct Live {
    stop: oneshot::Sender<()>,
    task: JoinHandle<()>,
}

fn bind_addr(lan: bool, port: u16) -> SocketAddr {
    let ip = if lan {
        Ipv4Addr::UNSPECIFIED
    } else {
        Ipv4Addr::LOCALHOST
    };
    SocketAddr::from((ip, port))
}

fn spawn_listener(listener: TcpListener, router: Router) -> Live {
    let (stop, rx) = oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        let app = router.into_make_service_with_connect_info::<SocketAddr>();
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = rx.await;
            })
            .await;
    });
    Live { stop, task }
}

async fn stop_listener(live: Live, generation: &watch::Sender<u64>) {
    // End open event streams first, or graceful shutdown would wait for them.
    generation.send_modify(|g| *g += 1);
    let _ = live.stop.send(());
    let mut task = live.task;
    if tokio::time::timeout(STOP_GRACE, &mut task).await.is_err() {
        task.abort();
        let _ = task.await;
    }
}

async fn bind_with_retry(addr: SocketAddr) -> io::Result<TcpListener> {
    let mut last = None;
    for _ in 0..20 {
        match TcpListener::bind(addr).await {
            Ok(l) => return Ok(l),
            Err(e) => {
                last = Some(e);
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }
    Err(last.unwrap())
}

/// A running server. Dropping it does not stop it; call `shutdown`.
pub struct Server {
    guard: Arc<Guard>,
    ctl: mpsc::UnboundedSender<Ctl>,
    port: u16,
}

impl Server {
    pub async fn start<C: AppCore>(
        core: Arc<C>,
        store: Arc<dyn Store>,
        assets: Arc<dyn Assets>,
        cfg: ServerConfig,
    ) -> io::Result<Server> {
        let tokens = TokenManager::load_or_create(store).map_err(io::Error::other)?;
        let guard = Arc::new(Guard::new(tokens));
        let listener = TcpListener::bind(bind_addr(cfg.lan, cfg.port)).await?;
        let port = listener.local_addr()?.port();
        guard.port.store(port, Ordering::Relaxed);
        guard.lan.store(cfg.lan, Ordering::Relaxed);

        let (ctl, mut ctl_rx) = mpsc::unbounded_channel();
        let (generation, _) = watch::channel(0u64);
        let shared = Arc::new(Shared {
            core,
            guard: guard.clone(),
            assets,
            web_dir: cfg.web_dir,
            heartbeat: cfg.heartbeat,
            lan_ips: cfg.lan_ips,
            listener_generation: generation,
            ctl: ctl.clone(),
        });
        let router = routes::build_router(shared.clone());
        let mut live = spawn_listener(listener, router.clone());
        let mut lan = cfg.lan;
        let g = guard.clone();

        tokio::spawn(async move {
            while let Some(msg) = ctl_rx.recv().await {
                match msg {
                    Ctl::Lan(want) if want != lan => {
                        stop_listener(live, &shared.listener_generation).await;
                        let (l, now_lan) = match bind_with_retry(bind_addr(want, port)).await {
                            Ok(l) => (l, want),
                            Err(e) => {
                                tracing::error!("could not rebind for LAN mode {want}: {e}");
                                match bind_with_retry(bind_addr(lan, port)).await {
                                    Ok(l) => (l, lan),
                                    Err(e) => {
                                        tracing::error!("could not rebind at all: {e}");
                                        return;
                                    }
                                }
                            }
                        };
                        lan = now_lan;
                        g.lan.store(lan, Ordering::Relaxed);
                        live = spawn_listener(l, router.clone());
                    }
                    Ctl::Lan(_) => {}
                    Ctl::Stop(done) => {
                        stop_listener(live, &shared.listener_generation).await;
                        let _ = done.send(());
                        return;
                    }
                }
            }
            stop_listener(live, &shared.listener_generation).await;
        });

        Ok(Server { guard, ctl, port })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn lan(&self) -> bool {
        self.guard.lan.load(Ordering::Relaxed)
    }

    /// The current API token (the desktop window gets it in the URL fragment).
    pub fn token(&self) -> String {
        self.guard.tokens.current()
    }

    /// `http://127.0.0.1:<port>/#token=<token>`, the URL the desktop window opens.
    pub fn window_url(&self) -> String {
        format!("http://127.0.0.1:{}/#token={}", self.port, self.token())
    }

    pub async fn shutdown(self) {
        let (tx, rx) = oneshot::channel();
        if self.ctl.send(Ctl::Stop(tx)).is_ok() {
            let _ = rx.await;
        }
    }
}
