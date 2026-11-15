//! Content-addressed storage for imported source bytes (artwork and masks).
//!
//! Files are named by the SHA-256 of their exact bytes, so references from a job snapshot do not
//! depend on where the file originally lived. Reads re-hash; a file whose bytes no longer match
//! its name is reported as corrupt.

use crate::{
    errors::{PersistenceError, PersistenceResult},
    jobs::atomic_write,
};
use std::path::PathBuf;
use veylora_domain::provenance::ContentHash;

#[derive(Debug)]
pub struct FileAssetStore {
    root: PathBuf,
}

impl FileAssetStore {
    pub fn open(root: impl Into<PathBuf>) -> PersistenceResult<Self> {
        let root = root.into();
        std::fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    fn path(&self, hash: &ContentHash) -> PathBuf {
        self.root.join(hash.to_string())
    }

    /// Stores the bytes and returns their hash. Identical bytes are stored once.
    pub fn put(&self, bytes: &[u8]) -> PersistenceResult<ContentHash> {
        let hash = ContentHash::digest(bytes);
        let path = self.path(&hash);
        match std::fs::read(&path) {
            Ok(existing) if existing == bytes => {}
            Ok(_) => {
                return Err(PersistenceError::Corrupt {
                    detail: format!("{hash}: stored asset differs from its name"),
                })
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => atomic_write(&path, bytes)?,
            Err(e) => return Err(PersistenceError::from_io(e)),
        }
        Ok(hash)
    }

    pub fn get(&self, hash: &ContentHash) -> PersistenceResult<Vec<u8>> {
        let bytes = match std::fs::read(self.path(hash)) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(PersistenceError::NotFound {
                    id: hash.to_string(),
                })
            }
            Err(e) => return Err(PersistenceError::from_io(e)),
        };
        if ContentHash::digest(&bytes) != *hash {
            return Err(PersistenceError::Corrupt {
                detail: format!("{hash}: stored asset differs from its name"),
            });
        }
        Ok(bytes)
    }
}
