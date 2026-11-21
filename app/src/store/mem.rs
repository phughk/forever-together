use super::{Store, StoreError};
use std::collections::BTreeMap;
use std::sync::Mutex;

/// In-memory store for tests.
#[derive(Default)]
pub struct MemStore {
    map: Mutex<BTreeMap<String, Vec<u8>>>,
}

impl MemStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Store for MemStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        Ok(self.map.lock().unwrap().get(key).cloned())
    }
    fn put(&self, key: &str, value: &[u8]) -> Result<(), StoreError> {
        self.map
            .lock()
            .unwrap()
            .insert(key.to_owned(), value.to_vec());
        Ok(())
    }
    fn delete(&self, key: &str) -> Result<bool, StoreError> {
        Ok(self.map.lock().unwrap().remove(key).is_some())
    }
    fn list(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>, StoreError> {
        Ok(self
            .map
            .lock()
            .unwrap()
            .range(prefix.to_owned()..)
            .take_while(|(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conforms() {
        super::super::conformance::check(&MemStore::new());
    }
}
