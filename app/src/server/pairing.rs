//! The pairing URL: `http://<lan-ip>:<port>/#token=<token>`. The address is one of the machine's
//! private IPv4 addresses; the one the system routes through comes first and is the default.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

/// RFC 1918 ranges.
pub fn is_private_v4(ip: &Ipv4Addr) -> bool {
    ip.is_private()
}

/// The first private IPv4 address in `candidates`.
pub fn first_private(candidates: &[Ipv4Addr]) -> Option<Ipv4Addr> {
    candidates.iter().copied().find(is_private_v4)
}

/// The address of the interface the system would use to reach the internet. Connecting a UDP
/// socket sends nothing: it only makes the system pick a route, and `local_addr` then shows
/// the interface's address. Returns it only when it is private.
pub fn detect_route_ip() -> Option<Ipv4Addr> {
    let sock = UdpSocket::bind(("0.0.0.0", 0)).ok()?;
    // TEST-NET-1: never routed, no packet is sent.
    sock.connect(SocketAddr::from(([192, 0, 2, 1], 9))).ok()?;
    match sock.local_addr().ok()?.ip() {
        IpAddr::V4(v4) => first_private(&[v4]),
        IpAddr::V6(_) => None,
    }
}

/// Every private IPv4 address on an up, non-loopback interface, with the routed one (if any) first.
pub fn detect_lan_ips() -> Vec<Ipv4Addr> {
    let found: Vec<Ipv4Addr> = if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter(|i| !i.is_loopback())
        .filter_map(|i| match i.ip() {
            IpAddr::V4(v4) => Some(v4),
            IpAddr::V6(_) => None,
        })
        .collect();
    order_candidates(&found, detect_route_ip())
}

/// Keep the private addresses of `found`, without duplicates, and put `preferred` first.
pub fn order_candidates(found: &[Ipv4Addr], preferred: Option<Ipv4Addr>) -> Vec<Ipv4Addr> {
    let mut out: Vec<Ipv4Addr> = Vec::new();
    for ip in preferred.iter().chain(found.iter()) {
        if is_private_v4(ip) && !out.contains(ip) {
            out.push(*ip);
        }
    }
    out
}

pub fn pairing_url(ip: Ipv4Addr, port: u16, token: &str) -> String {
    format!("http://{ip}:{port}/#token={token}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_private_address_wins() {
        let list = [
            Ipv4Addr::new(8, 8, 8, 8),
            Ipv4Addr::new(169, 254, 1, 1),
            Ipv4Addr::new(192, 168, 1, 20),
            Ipv4Addr::new(10, 0, 0, 5),
        ];
        assert_eq!(first_private(&list), Some(Ipv4Addr::new(192, 168, 1, 20)));
        assert_eq!(first_private(&list[..2]), None);
        assert!(is_private_v4(&Ipv4Addr::new(172, 16, 0, 1)));
        assert!(!is_private_v4(&Ipv4Addr::new(172, 32, 0, 1)));
    }

    #[test]
    fn candidates_are_private_unique_and_route_first() {
        let found = [
            Ipv4Addr::new(10, 0, 0, 5),
            Ipv4Addr::new(8, 8, 8, 8),
            Ipv4Addr::new(192, 168, 1, 20),
            Ipv4Addr::new(10, 0, 0, 5),
        ];
        assert_eq!(
            order_candidates(&found, Some(Ipv4Addr::new(192, 168, 1, 20))),
            vec![Ipv4Addr::new(192, 168, 1, 20), Ipv4Addr::new(10, 0, 0, 5)]
        );
        assert_eq!(
            order_candidates(&found, None),
            vec![Ipv4Addr::new(10, 0, 0, 5), Ipv4Addr::new(192, 168, 1, 20)]
        );
        assert!(order_candidates(&[Ipv4Addr::new(8, 8, 8, 8)], None).is_empty());
    }

    #[test]
    fn url_shape() {
        assert_eq!(
            pairing_url(Ipv4Addr::new(192, 168, 1, 20), 4321, "abc"),
            "http://192.168.1.20:4321/#token=abc"
        );
    }
}
