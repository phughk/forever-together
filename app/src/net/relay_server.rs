//! The hole-punching relay a headless node can run (ADR 0041): the embedded `iroh-relay`
//! server on plain HTTP, no TLS, no QUIC endpoint, and a low per-client rate limit.
//!
//! A relay that helps hole punching carries the first packets of every connection (iroh 1.5.0
//! does hole punching inside a QUIC connection that starts on the relay), so it cannot be made
//! to carry nothing. The rate limit is what keeps it from being used as a traffic relay: a
//! handshake and the NAT-traversal frames fit inside it, sustained traffic does not.

use crate::engine::NetError;
use iroh_relay::server::{ClientRateLimit, RelayConfig, Server, ServerConfig};
use std::net::SocketAddr;
use std::num::NonZeroU32;

/// Bytes per second one client may send to the relay. Chosen from the measurement in
/// `relay_rate_tests` and ticket 0049; revisit with the live two-network numbers.
pub const CLIENT_BYTES_PER_SECOND: u32 = 2_000;
/// Burst a client may send at once (a QUIC handshake is a few packets of up to 1.3 kB each).
/// Loopback measurements (ticket 0049): at 2 kB/s with this burst a direct path forms in about
/// 11 ms, and a 20 kB transfer takes about 7 s (sustained throughput is held to the rate).
pub const CLIENT_BURST_BYTES: u32 = 8 * 1024;

/// A running relay. Dropping it stops the server.
pub struct RelayServer {
    server: Server,
}

impl RelayServer {
    /// Serve on `bind` with the standard limit.
    pub async fn start(bind: SocketAddr) -> Result<Self, NetError> {
        Self::start_with_rate(bind, CLIENT_BYTES_PER_SECOND, CLIENT_BURST_BYTES).await
    }

    /// Serve with an explicit rate (tests and measurements).
    pub async fn start_with_rate(
        bind: SocketAddr,
        bytes_per_second: u32,
        burst_bytes: u32,
    ) -> Result<Self, NetError> {
        let mut relay = RelayConfig::new(bind);
        let mut limit = ClientRateLimit::new(
            NonZeroU32::new(bytes_per_second).ok_or_else(|| NetError::Io("rate is zero".into()))?,
        );
        limit.max_burst_bytes = NonZeroU32::new(burst_bytes);
        relay.limits.client_rx = Some(limit);
        let mut config = ServerConfig::default();
        config.relay = Some(relay);
        let server = Server::spawn(config)
            .await
            .map_err(|e| NetError::Io(format!("relay server: {e}")))?;
        Ok(Self { server })
    }

    /// The address the server listens on (useful when `bind` used port 0).
    pub fn addr(&self) -> Option<SocketAddr> {
        self.server.http_addr()
    }

    pub async fn shutdown(self) {
        let _ = self.server.shutdown().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iroh::address_lookup::MemoryLookup;
    use iroh::endpoint::presets;
    use iroh::{Endpoint, RelayMode, RelayUrl};
    use std::time::{Duration, Instant};

    const ALPN: &[u8] = b"wowft/test/relay";

    async fn endpoint(relay: &RelayUrl, lookup: &MemoryLookup) -> Endpoint {
        Endpoint::builder(presets::Minimal)
            .relay_mode(RelayMode::custom([relay.clone()]))
            .clear_address_lookup()
            .address_lookup(lookup.clone())
            .clear_ip_transports()
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await
            .unwrap()
    }

    /// Wait until the endpoint has reported a home relay, then publish its address.
    async fn publish(ep: &Endpoint, lookup: &MemoryLookup) {
        let t = Instant::now();
        loop {
            let a = ep.addr();
            if a.relay_urls().next().is_some() {
                lookup.add_endpoint_info(a);
                return;
            }
            assert!(t.elapsed() < Duration::from_secs(15), "no home relay");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    /// Two endpoints with no direct path, only a relay with `rate`. Returns the time the
    /// connection took to open, and the time to send `bytes` from one to the other.
    async fn through_relay(rate: u32, burst: u32, bytes: usize) -> (Duration, Option<Duration>) {
        let relay = RelayServer::start_with_rate("127.0.0.1:0".parse().unwrap(), rate, burst)
            .await
            .unwrap();
        let url: RelayUrl = format!("http://{}", relay.addr().unwrap()).parse().unwrap();
        let lookup = MemoryLookup::new();
        let a = endpoint(&url, &lookup).await;
        let b = endpoint(&url, &lookup).await;
        publish(&a, &lookup).await;
        publish(&b, &lookup).await;
        let a_id = a.id();
        let server = tokio::spawn(async move {
            let incoming = a.accept().await.unwrap();
            let conn = incoming.await.unwrap();
            let (mut tx, mut rx) = conn.accept_bi().await.unwrap();
            let got = rx.read_to_end(usize::MAX).await.unwrap();
            tx.write_all(b"ok").await.unwrap();
            tx.finish().unwrap();
            conn.closed().await;
            (got.len(), a)
        });
        let t0 = Instant::now();
        let conn = tokio::time::timeout(Duration::from_secs(60), b.connect(a_id, ALPN))
            .await
            .expect("connect timed out")
            .unwrap();
        let connect = t0.elapsed();
        eprintln!("RELAY ({rate} B/s) connected in {connect:?}");
        let (mut tx, mut rx) = conn.open_bi().await.unwrap();
        let t1 = Instant::now();
        tx.write_all(&vec![7u8; bytes]).await.unwrap();
        tx.finish().unwrap();
        let reply = tokio::time::timeout(Duration::from_secs(15), rx.read_to_end(8)).await;
        let transfer = match reply {
            Ok(r) => {
                assert_eq!(r.unwrap(), b"ok");
                Some(t1.elapsed())
            }
            Err(_) => None,
        };
        conn.close(0u32.into(), b"done");
        server.abort();
        let _ = tokio::time::timeout(Duration::from_secs(2), b.close()).await;
        let _ = tokio::time::timeout(Duration::from_secs(2), relay.shutdown()).await;
        (connect, transfer)
    }

    /// With a generous limit the connection and a 20 kB transfer are quick; with the project's
    /// limit the connection still opens, but the same transfer is held to the rate (about 2 kB/s
    /// sustained, so 20 kB takes several seconds). This is the observable effect of the limit
    /// that ADR 0041 depends on. A rate limit caps throughput; it cannot make it zero.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn the_rate_limit_lets_a_handshake_through_and_starves_bulk_traffic() {
        let bytes = 20_000;
        let (fast_connect, fast) = through_relay(50_000_000, 50_000_000, bytes).await;
        let (slow_connect, slow) =
            through_relay(CLIENT_BYTES_PER_SECOND, CLIENT_BURST_BYTES, bytes).await;
        eprintln!(
            "RELAY generous: connect {fast_connect:?}, 20 kB in {fast:?} | project limit: connect {slow_connect:?}, 20 kB in {slow:?}"
        );
        assert!(fast.is_some(), "an unlimited relay carries 20 kB");
        assert!(
            slow_connect < Duration::from_secs(10),
            "the handshake must still pass"
        );
        let slow = slow.expect("20 kB still arrives at the limited rate");
        assert!(
            slow > fast.unwrap() * 100 && slow > Duration::from_secs(3),
            "the project limit must hold bulk traffic to the rate: {slow:?} vs {fast:?}"
        );
    }

    /// Measurement for ticket 0049 (loopback only: it shows the effect of the limit, not NAT
    /// behaviour). Run: `cargo test --lib relay_server::tests::measure -- --ignored --nocapture`.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "measurement"]
    async fn measure_how_low_the_rate_can_go() {
        for (rate, burst) in [
            (500, 2_000),
            (2_000, 8_000),
            (5_000, 16_000),
            (20_000, 65_536),
        ] {
            let (c, t) = through_relay(rate, burst, 20_000).await;
            eprintln!("RELAY rate {rate} B/s burst {burst}: connect {c:?}, 20 kB in {t:?}");
        }
    }

    /// Two endpoints that know each other only by a relay URL (so the connection starts on the
    /// relay) but can also reach each other directly on loopback. Returns how long it took for
    /// the connection to gain a direct path, or None if it did not within `wait`.
    async fn time_to_direct(rate: u32, burst: u32, wait: Duration) -> Option<Duration> {
        let relay = RelayServer::start_with_rate("127.0.0.1:0".parse().unwrap(), rate, burst)
            .await
            .unwrap();
        let url: RelayUrl = format!("http://{}", relay.addr().unwrap()).parse().unwrap();
        let build = || async {
            Endpoint::builder(presets::Minimal)
                .relay_mode(RelayMode::custom([url.clone()]))
                .clear_address_lookup()
                .bind_addr("127.0.0.1:0".parse::<std::net::SocketAddr>().unwrap())
                .unwrap()
                .alpns(vec![ALPN.to_vec()])
                .bind()
                .await
                .unwrap()
        };
        let (a, b) = (build().await, build().await);
        let t = Instant::now();
        while a.addr().relay_urls().next().is_none() {
            assert!(t.elapsed() < Duration::from_secs(15), "no home relay");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let a_id = a.id();
        let accept = tokio::spawn(async move {
            let conn = a.accept().await.unwrap().await.unwrap();
            conn.closed().await;
            a
        });
        // Only the relay is known: no direct address in the target.
        let target = iroh::EndpointAddr::new(a_id).with_relay_url(url.clone());
        let t0 = Instant::now();
        let conn = b.connect(target, ALPN).await.unwrap();
        let mut paths = conn.paths_stream();
        let direct = tokio::time::timeout(wait, paths.wait_any(|p| p.is_ip()))
            .await
            .ok()
            .flatten()
            .map(|_| t0.elapsed());
        conn.close(0u32.into(), b"done");
        let a = accept.await.unwrap();
        a.close().await;
        b.close().await;
        relay.shutdown().await;
        direct
    }

    /// Hole punching (the connection moving from the relay to a direct path) still completes
    /// under the limit. Loopback only: it checks that the limit does not starve the NAT
    /// traversal frames, not how real NATs behave.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_direct_path_still_forms_under_the_limit() {
        let d = time_to_direct(
            CLIENT_BYTES_PER_SECOND,
            CLIENT_BURST_BYTES,
            Duration::from_secs(20),
        )
        .await;
        eprintln!("RELAY direct path under the project limit: {d:?}");
        assert!(d.is_some(), "the connection never moved to a direct path");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "measurement"]
    async fn measure_time_to_direct_at_each_rate() {
        for (rate, burst) in [
            (500, 2_000),
            (1_000, 4_000),
            (2_000, 8_000),
            (5_000, 16_000),
            (20_000, 65_536),
            (50_000_000, 50_000_000),
        ] {
            let d = time_to_direct(rate, burst, Duration::from_secs(20)).await;
            eprintln!("RELAY rate {rate} B/s burst {burst}: direct path after {d:?}");
        }
    }
}
