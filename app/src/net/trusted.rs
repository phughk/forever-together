//! Trusted peers (ADR 0009, 0023): ordinary peers (the same app, headless) listed by public key
//! only. They are the last resort of bootstrap, never a hub: they learn their own peers through
//! peer exchange like everyone else.
//!
//! The built-in list is empty until the author supplies a key (plan, prerequisites). The type
//! takes any `Vec<PeerId>`, so tests pass their own.
//!
//! S0 row "iroh address lookup resolves a trusted peer from its public key": checked from
//! source only. A key alone is dialable only if an address lookup service can resolve it (the
//! default `N0` preset does, through the n0 DNS/pkarr service), see `docs/spike-s0.md`.

use crate::config::Settings;
use crate::identity::PeerId;

/// Built-in trusted peer keys.
/// The project's hole-punching relays (ADR 0041): plain `http://` URLs with an IP and port, added
/// to the relay map next to n0's. Run by a headless node started with `--relay <addr:port>`.
pub const RELAYS: &[&str] = &["http://89.40.6.71:3340"];

pub const BUILTIN: &[[u8; 32]] = &[
    [
        0x7c, 0x4b, 0x43, 0x0a, 0x31, 0x76, 0xb8, 0xb4, 0x8d, 0xca, 0xd1, 0x7d, 0x33, 0xd1, 0xe7,
        0x30, 0xc7, 0xa0, 0x42, 0x0b, 0xec, 0x20, 0xaf, 0x15, 0xa8, 0xda, 0x27, 0x96, 0xeb, 0x28,
        0x68, 0x0e,
    ],
    [
        0x7b, 0x69, 0x49, 0x95, 0xa6, 0x7d, 0x74, 0xe8, 0x2e, 0xbd, 0x85, 0x51, 0x2b, 0xf8, 0xd9,
        0x64, 0x58, 0xf5, 0x11, 0xd9, 0xa2, 0x29, 0x28, 0xf1, 0x22, 0x4a, 0x43, 0x11, 0xcb, 0x79,
        0x1d, 0x3a,
    ],
];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrustedPeers {
    peers: Vec<PeerId>,
}

impl TrustedPeers {
    /// The given keys, in order, without duplicates.
    pub fn new(peers: Vec<PeerId>) -> Self {
        let mut out: Vec<PeerId> = Vec::with_capacity(peers.len());
        for p in peers {
            if !out.contains(&p) {
                out.push(p);
            }
        }
        Self { peers: out }
    }

    /// The built-in keys plus the user's extra keys (hex). Returns the list and the entries of
    /// the setting that were not valid keys, so the caller can tell the user.
    pub fn from_settings(builtin: &[[u8; 32]], settings: &Settings) -> (Self, Vec<String>) {
        let mut all: Vec<PeerId> = builtin.iter().map(|b| PeerId(*b)).collect();
        let mut bad = Vec::new();
        for s in &settings.trusted_peers {
            match PeerId::from_hex(s.trim()) {
                Some(p) => all.push(p),
                None => bad.push(s.clone()),
            }
        }
        (Self::new(all), bad)
    }

    pub fn peers(&self) -> &[PeerId] {
        &self.peers
    }

    pub fn is_trusted(&self, peer: &PeerId) -> bool {
        self.peers.contains(peer)
    }

    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_list_is_trusted1_then_trusted2() {
        let (t, bad) = TrustedPeers::from_settings(BUILTIN, &Settings::default());
        assert!(bad.is_empty());
        let hex: Vec<String> = t.peers().iter().map(|p| p.to_hex()).collect();
        assert_eq!(
            hex,
            [
                "7c4b430a3176b8b48dcad17d33d1e730c7a0420bec20af15a8da2796eb28680e",
                "7b694995a67d74e82ebd85512bf8d96458f511d9a22928f1224a4311cb791d3a",
            ]
        );
    }

    #[test]
    fn settings_extend_the_list_and_bad_entries_are_reported() {
        let s = Settings {
            trusted_peers: vec![
                PeerId([2; 32]).to_hex(),
                "nonsense".into(),
                format!("  {}  ", PeerId([3; 32]).to_hex()),
                PeerId([2; 32]).to_hex(),
            ],
            ..Settings::default()
        };
        let (t, bad) = TrustedPeers::from_settings(&[[1; 32]], &s);
        assert_eq!(
            t.peers(),
            &[PeerId([1; 32]), PeerId([2; 32]), PeerId([3; 32])]
        );
        assert_eq!(bad, vec!["nonsense".to_string()]);
        assert!(t.is_trusted(&PeerId([3; 32])) && !t.is_trusted(&PeerId([4; 32])));
    }

    #[test]
    fn new_takes_any_vec_and_dedups() {
        let t = TrustedPeers::new(vec![PeerId([1; 32]), PeerId([1; 32])]);
        assert_eq!(t.peers().len(), 1);
    }
}
