//! The peer's permanent identity. Keys are plain ed25519 behind the `Signer` trait, so the
//! `net` module can adapt iroh's secret key to the same trait later. A `PeerId` is the 32-byte
//! ed25519 public key (the same bytes as an iroh node ID).

use crate::store::{KEY_IDENTITY_SECRET, Store, StoreError};
use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PeerId(pub [u8; 32]);

impl PeerId {
    pub fn to_hex(&self) -> String {
        data_encoding::HEXLOWER.encode(&self.0)
    }
    pub fn from_hex(s: &str) -> Option<PeerId> {
        let b = data_encoding::HEXLOWER
            .decode(s.to_ascii_lowercase().as_bytes())
            .ok()?;
        Some(PeerId(b.try_into().ok()?))
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PeerId({}..)", &self.to_hex()[..8])
    }
}

pub type SignatureBytes = [u8; 64];

/// Something that can sign as one peer.
pub trait Signer: Send + Sync {
    fn peer_id(&self) -> PeerId;
    fn sign(&self, msg: &[u8]) -> SignatureBytes;
}

/// Check a signature made by `peer`. False for any malformed key or signature.
pub fn verify(peer: &PeerId, msg: &[u8], sig: &SignatureBytes) -> bool {
    let Ok(vk) = VerifyingKey::from_bytes(&peer.0) else {
        return false;
    };
    vk.verify_strict(msg, &Signature::from_bytes(sig)).is_ok()
}

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("stored secret key has the wrong length")]
    BadStoredKey,
}

pub struct Identity {
    key: SigningKey,
}

impl Identity {
    pub fn from_secret(secret: [u8; 32]) -> Self {
        Self {
            key: SigningKey::from_bytes(&secret),
        }
    }

    pub fn generate() -> Self {
        let mut secret = [0u8; 32];
        rand::fill(&mut secret);
        Self::from_secret(secret)
    }

    /// Load the stored key, or create and store one. The key is created only if missing.
    pub fn load_or_create(store: &dyn Store) -> Result<Self, IdentityError> {
        match store.get(KEY_IDENTITY_SECRET)? {
            Some(b) => {
                let secret: [u8; 32] = b.try_into().map_err(|_| IdentityError::BadStoredKey)?;
                Ok(Self::from_secret(secret))
            }
            None => {
                let mut secret = [0u8; 32];
                rand::fill(&mut secret);
                store.put(KEY_IDENTITY_SECRET, &secret)?;
                Ok(Self::from_secret(secret))
            }
        }
    }
}

impl Signer for Identity {
    fn peer_id(&self) -> PeerId {
        PeerId(self.key.verifying_key().to_bytes())
    }
    fn sign(&self, msg: &[u8]) -> SignatureBytes {
        use ed25519_dalek::Signer as _;
        self.key.sign(msg).to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{MemStore, VaraqStore};

    #[test]
    fn sign_and_verify() {
        let id = Identity::generate();
        let sig = id.sign(b"hello");
        assert!(verify(&id.peer_id(), b"hello", &sig));
        assert!(!verify(&id.peer_id(), b"hellO", &sig));
        let other = Identity::generate();
        assert!(!verify(&other.peer_id(), b"hello", &sig));
        let mut bad = sig;
        bad[0] ^= 1;
        assert!(!verify(&id.peer_id(), b"hello", &bad));
    }

    #[test]
    fn known_vector_from_secret() {
        // The same secret always gives the same peer ID.
        let a = Identity::from_secret([9; 32]);
        let b = Identity::from_secret([9; 32]);
        assert_eq!(a.peer_id(), b.peer_id());
        assert_ne!(a.peer_id(), Identity::from_secret([8; 32]).peer_id());
    }

    #[test]
    fn created_once_and_kept() {
        let s = MemStore::new();
        let first = Identity::load_or_create(&s).unwrap().peer_id();
        let second = Identity::load_or_create(&s).unwrap().peer_id();
        assert_eq!(first, second);
        assert_eq!(s.get(KEY_IDENTITY_SECRET).unwrap().unwrap().len(), 32);
    }

    #[test]
    fn corrupt_stored_key_is_an_error_not_a_new_identity() {
        let s = MemStore::new();
        s.put(KEY_IDENTITY_SECRET, b"short").unwrap();
        assert!(matches!(
            Identity::load_or_create(&s),
            Err(IdentityError::BadStoredKey)
        ));
    }

    #[test]
    fn persists_across_restart_on_varaq() {
        let d = std::env::temp_dir().join(format!("wowft-identity-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let first = Identity::load_or_create(&VaraqStore::open(&d).unwrap())
            .unwrap()
            .peer_id();
        let second = Identity::load_or_create(&VaraqStore::open(&d).unwrap())
            .unwrap()
            .peer_id();
        assert_eq!(first, second);
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn hex_round_trip() {
        let p = Identity::generate().peer_id();
        assert_eq!(PeerId::from_hex(&p.to_hex()), Some(p));
        assert_eq!(PeerId::from_hex("zz"), None);
    }
}
