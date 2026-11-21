//! Local storage. `Store` is a small synchronous key-value trait. `VaraqStore` is the real
//! backend, `MemStore` is for tests.
//!
//! Key layout (plan item 24): `identity/secret`, `known_peers/<peerId hex>`, `settings`,
//! `api/token`, `character`.

mod mem;
mod varaq_store;

pub use mem::MemStore;
pub use varaq_store::VaraqStore;

use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

pub const KEY_IDENTITY_SECRET: &str = "identity/secret";
pub const KEY_SETTINGS: &str = "settings";
pub const KEY_API_TOKEN: &str = "api/token";
pub const KEY_CHARACTER: &str = "character";
pub const PREFIX_KNOWN_PEERS: &str = "known_peers/";

pub fn known_peer_key(peer_hex: &str) -> String {
    format!("{PREFIX_KNOWN_PEERS}{peer_hex}")
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("storage backend: {0}")]
    Backend(String),
    #[error("stored value could not be decoded: {0}")]
    Decode(String),
}

pub trait Store: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError>;
    fn put(&self, key: &str, value: &[u8]) -> Result<(), StoreError>;
    /// Returns whether the key existed.
    fn delete(&self, key: &str) -> Result<bool, StoreError>;
    /// All entries whose key starts with `prefix`, in key order.
    fn list(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>, StoreError>;
}

pub fn get_json<T: DeserializeOwned>(s: &dyn Store, key: &str) -> Result<Option<T>, StoreError> {
    match s.get(key)? {
        None => Ok(None),
        Some(b) => serde_json::from_slice(&b)
            .map(Some)
            .map_err(|e| StoreError::Decode(format!("{key}: {e}"))),
    }
}

pub fn put_json<T: Serialize>(s: &dyn Store, key: &str, v: &T) -> Result<(), StoreError> {
    let b = serde_json::to_vec(v).map_err(|e| StoreError::Decode(format!("{key}: {e}")))?;
    s.put(key, &b)
}

#[cfg(test)]
pub(crate) mod conformance {
    use super::*;

    /// Behaviour every backend must have.
    pub fn check(s: &dyn Store) {
        assert_eq!(s.get("a").unwrap(), None);
        s.put("a", b"1").unwrap();
        s.put("known_peers/bb", b"2").unwrap();
        s.put("known_peers/aa", b"3").unwrap();
        s.put("settings", b"4").unwrap();
        assert_eq!(s.get("a").unwrap(), Some(b"1".to_vec()));
        s.put("a", b"replaced").unwrap();
        assert_eq!(s.get("a").unwrap(), Some(b"replaced".to_vec()));
        let l = s.list(PREFIX_KNOWN_PEERS).unwrap();
        let keys: Vec<&str> = l.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["known_peers/aa", "known_peers/bb"]);
        assert_eq!(l[0].1, b"3");
        assert!(s.delete("a").unwrap());
        assert!(!s.delete("a").unwrap());
        assert_eq!(s.get("a").unwrap(), None);
        put_json(s, "j", &vec![1u8, 2]).unwrap();
        assert_eq!(get_json::<Vec<u8>>(s, "j").unwrap(), Some(vec![1, 2]));
        s.put("bad", b"not json").unwrap();
        assert!(matches!(
            get_json::<Vec<u8>>(s, "bad"),
            Err(StoreError::Decode(_))
        ));
    }
}
