//! Epoch ordering hash (spec section 3): the first 8 bytes of
//! `SHA-256(epoch as 8 bytes big-endian || peer ID)`.

use crate::identity::PeerId;
use sha2::{Digest, Sha256};

pub fn rank(epoch: u64, peer: &PeerId) -> u64 {
    let mut h = Sha256::new();
    h.update(epoch.to_be_bytes());
    h.update(peer.0);
    let d = h.finalize();
    u64::from_be_bytes(d[..8].try_into().unwrap())
}

/// Sort key: rank, ties broken by lowest peer ID.
pub fn order_key(epoch: u64, peer: &PeerId) -> (u64, PeerId) {
    (rank(epoch, peer), *peer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_epoch_dependent() {
        let p = PeerId([3; 32]);
        assert_eq!(rank(7, &p), rank(7, &p));
        assert_ne!(rank(7, &p), rank(8, &p));
        assert_ne!(rank(7, &p), rank(7, &PeerId([4; 32])));
    }

    #[test]
    fn matches_the_spec_definition() {
        let p = PeerId([0; 32]);
        let mut input = 5u64.to_be_bytes().to_vec();
        input.extend_from_slice(&[0; 32]);
        let d = Sha256::digest(&input);
        assert_eq!(rank(5, &p), u64::from_be_bytes(d[..8].try_into().unwrap()));
    }
}
