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

/// Built-in trusted peer keys. Empty until the author runs the always-on peer with
/// `--print-public-key` and commits its key.
pub const BUILTIN: &[[u8; 32]] = &[];

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
    fn built_in_list_is_empty_until_the_author_supplies_a_key() {
        let (t, bad) = TrustedPeers::from_settings(BUILTIN, &Settings::default());
        assert!(t.is_empty() && bad.is_empty());
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
