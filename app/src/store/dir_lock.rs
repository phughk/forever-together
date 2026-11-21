//! An exclusive lock on a data directory (ADR 0039, decision 9).
//!
//! Step 0 of ticket 0043 showed that `varaq` does not refuse a second opener: two processes can
//! open the same directory and write to it, and neither sees the other's writes. The app takes
//! this lock for as long as it runs, and the `key` commands take it too, so only one process
//! uses a data directory at a time. The operating system drops the lock if the process dies.

use std::fs::{File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use thiserror::Error;

const LOCK_FILE: &str = "lock";

#[derive(Debug, Error)]
pub enum LockError {
    #[error(
        "another forever-together process is using {0}. Stop it first, or use a different --data-dir"
    )]
    Held(PathBuf),
    #[error("data directory lock: {0}")]
    Io(String),
}

/// Held until dropped.
#[derive(Debug)]
pub struct DataDirLock {
    _file: File,
}

impl DataDirLock {
    /// Create the directory if needed and take the lock, or fail at once if another process
    /// holds it.
    pub fn acquire(dir: &Path) -> Result<Self, LockError> {
        std::fs::create_dir_all(dir).map_err(|e| LockError::Io(e.to_string()))?;
        let mut o = OpenOptions::new();
        o.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            o.mode(0o600);
        }
        let file = o
            .open(dir.join(LOCK_FILE))
            .map_err(|e| LockError::Io(e.to_string()))?;
        match file.try_lock() {
            Ok(()) => Ok(Self { _file: file }),
            Err(TryLockError::WouldBlock) => Err(LockError::Held(dir.to_path_buf())),
            Err(TryLockError::Error(e)) => Err(LockError::Io(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("wowft-lock-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn second_holder_is_refused_until_the_first_drops() {
        let d = tmp("two");
        let first = DataDirLock::acquire(&d).unwrap();
        assert!(matches!(
            DataDirLock::acquire(&d),
            Err(LockError::Held(p)) if p == d
        ));
        drop(first);
        let again = DataDirLock::acquire(&d).unwrap();
        drop(again);
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn different_directories_do_not_conflict() {
        let a = tmp("a");
        let b = tmp("b");
        let _la = DataDirLock::acquire(&a).unwrap();
        let _lb = DataDirLock::acquire(&b).unwrap();
        let _ = std::fs::remove_dir_all(a);
        let _ = std::fs::remove_dir_all(b);
    }

    #[cfg(unix)]
    #[test]
    fn the_lock_file_is_private() {
        use std::os::unix::fs::PermissionsExt;
        let d = tmp("mode");
        let _l = DataDirLock::acquire(&d).unwrap();
        let m = std::fs::metadata(d.join(LOCK_FILE))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(m & 0o077, 0);
        let _ = std::fs::remove_dir_all(d);
    }
}
