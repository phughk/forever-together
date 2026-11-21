use super::{Store, StoreError};
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::sync::Mutex;
use varaq::WorkerId;
use varaq::io::devices::{Devices, Refusal};
use varaq::tree::scan::Step;
use varaq::txn::db::Db;

const POOL_PAGES: usize = 256;
const RING_BYTES: usize = 4 << 20;
const WORKER: WorkerId = WorkerId(0);

/// Persistent store over `varaq` (0.1.x: the on-disk format may change between versions).
/// One worker, so operations are serialised by a mutex.
pub struct VaraqStore {
    db: Mutex<Db>,
}

fn be(e: impl std::fmt::Display) -> StoreError {
    StoreError::Backend(e.to_string())
}

fn devices(dir: &Path) -> std::io::Result<Devices<File>> {
    let file = |name: &str| {
        let mut o = OpenOptions::new();
        o.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            o.mode(0o600);
        }
        o.open(dir.join(name))
    };
    Ok(Devices::new(
        file("catalog")?,
        file("log")?,
        file("data")?,
        file("versions")?,
    ))
}

impl VaraqStore {
    /// Open the store in `dir`, creating it (and the directory, mode 0700) if it is missing.
    pub fn open(dir: &Path) -> Result<Self, StoreError> {
        std::fs::create_dir_all(dir).map_err(be)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700)).map_err(be)?;
        }
        let db = match devices(dir).map_err(be)?.open(POOL_PAGES) {
            Ok(db) => db,
            Err(Refusal::NoSuchStore) => devices(dir)
                .map_err(be)?
                .create(1, POOL_PAGES, RING_BYTES)
                .map_err(|e| StoreError::Backend(format!("{e:?}")))?,
            Err(e) => return Err(StoreError::Backend(format!("{e:?}"))),
        };
        Ok(Self { db: Mutex::new(db) })
    }
}

impl Store for VaraqStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.db
            .lock()
            .unwrap()
            .get(WORKER, key.as_bytes())
            .map_err(be)
    }
    fn put(&self, key: &str, value: &[u8]) -> Result<(), StoreError> {
        self.db
            .lock()
            .unwrap()
            .insert(WORKER, key.as_bytes(), value)
            .map_err(be)
    }
    fn delete(&self, key: &str) -> Result<bool, StoreError> {
        self.db
            .lock()
            .unwrap()
            .remove(WORKER, key.as_bytes())
            .map_err(be)
    }
    fn list(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>, StoreError> {
        let db = self.db.lock().unwrap();
        let tx = db.begin(WORKER).map_err(be)?;
        let mut out = Vec::new();
        tx.scan(prefix.as_bytes(), |k, v| {
            if !k.starts_with(prefix.as_bytes()) {
                return Step::Stop;
            }
            out.push((String::from_utf8_lossy(k).into_owned(), v.to_vec()));
            Step::Continue
        })
        .map_err(be)?;
        tx.commit().map_err(be)?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("wowft-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn conforms() {
        let d = tmp("varaq-conf");
        super::super::conformance::check(&VaraqStore::open(&d).unwrap());
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn survives_restart_and_files_are_private() {
        let d = tmp("varaq-restart");
        {
            let s = VaraqStore::open(&d).unwrap();
            s.put("identity/secret", &[7u8; 32]).unwrap();
        }
        let s = VaraqStore::open(&d).unwrap();
        assert_eq!(s.get("identity/secret").unwrap(), Some(vec![7u8; 32]));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for e in std::fs::read_dir(&d).unwrap() {
                let m = e.unwrap().metadata().unwrap().permissions().mode() & 0o777;
                assert_eq!(m & 0o077, 0, "file readable by others: {m:o}");
            }
        }
        drop(s);
        let _ = std::fs::remove_dir_all(d);
    }
}
