//! Durable, content-addressed storage for print recipes and calibration records.
//!
//! Layout under `<root>`: `VERSION`, `recipes/<hash>.json`, `calibrations/<hash>.json`. The file
//! name is the record's own content hash, so a record can never be replaced by different content:
//! writing the same record again is a no-op, and a file whose content no longer matches its name
//! is reported as corrupt on read. Writes go through a temporary file, are fsynced, then renamed.
//! Nothing here promotes a recipe or marks a calibration validated.

use crate::{
    errors::{PersistenceError, PersistenceResult},
    jobs::{atomic_write, read_schema_version},
};
use std::path::{Path, PathBuf};
use veylora_color::CalibrationRecord;
use veylora_domain::{provenance::ContentHash, recipe::PrintRecipe};

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug)]
pub struct FileRecordStore {
    root: PathBuf,
}

impl FileRecordStore {
    pub fn open(root: impl Into<PathBuf>) -> PersistenceResult<Self> {
        let root = root.into();
        std::fs::create_dir_all(root.join("recipes"))?;
        std::fs::create_dir_all(root.join("calibrations"))?;
        match read_schema_version(&root)? {
            0 => atomic_write(&root.join("VERSION"), SCHEMA_VERSION.to_string().as_bytes())?,
            found if found > SCHEMA_VERSION => {
                return Err(PersistenceError::SchemaOutOfRange {
                    found,
                    supported: SCHEMA_VERSION,
                })
            }
            _ => {}
        }
        Ok(Self { root })
    }

    fn path(&self, dir: &str, hash: &ContentHash) -> PathBuf {
        self.root.join(dir).join(format!("{hash}.json"))
    }

    pub fn put_recipe(&self, recipe: &PrintRecipe) -> PersistenceResult<ContentHash> {
        let hash = recipe.content_hash();
        self.put(&self.path("recipes", &hash), &serde_json::to_vec(recipe)?)?;
        Ok(hash)
    }

    pub fn get_recipe(&self, hash: &ContentHash) -> PersistenceResult<PrintRecipe> {
        let recipe: PrintRecipe = self.read(&self.path("recipes", hash), hash)?;
        if recipe.content_hash() != *hash {
            return Err(corrupt(hash, "recipe content does not match its name"));
        }
        Ok(recipe)
    }

    pub fn list_recipes(&self) -> PersistenceResult<Vec<ContentHash>> {
        self.list("recipes")
    }

    /// Refuses a record that fails its own integrity hash.
    pub fn put_calibration(&self, record: &CalibrationRecord) -> PersistenceResult<ContentHash> {
        let hash = record.content_hash;
        if hash != record.compute_hash() {
            return Err(corrupt(&hash, "calibration record does not match its hash"));
        }
        self.put(
            &self.path("calibrations", &hash),
            &serde_json::to_vec(record)?,
        )?;
        Ok(hash)
    }

    pub fn get_calibration(&self, hash: &ContentHash) -> PersistenceResult<CalibrationRecord> {
        let record: CalibrationRecord = self.read(&self.path("calibrations", hash), hash)?;
        if record.content_hash != *hash || record.compute_hash() != *hash {
            return Err(corrupt(hash, "calibration content does not match its name"));
        }
        Ok(record)
    }

    pub fn list_calibrations(&self) -> PersistenceResult<Vec<ContentHash>> {
        self.list("calibrations")
    }

    /// Write-once. Same name means same content hash, so an existing file must hold identical
    /// bytes. A file that differs is damaged: it is reported and never overwritten.
    fn put(&self, path: &Path, bytes: &[u8]) -> PersistenceResult<()> {
        match std::fs::read(path) {
            Ok(existing) if existing == bytes => Ok(()),
            Ok(_) => Err(PersistenceError::Corrupt {
                detail: format!("{} exists with different content", path.display()),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => atomic_write(path, bytes),
            Err(e) => Err(PersistenceError::from_io(e)),
        }
    }

    fn read<T: serde::de::DeserializeOwned>(
        &self,
        path: &Path,
        hash: &ContentHash,
    ) -> PersistenceResult<T> {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(PersistenceError::NotFound {
                    id: hash.to_string(),
                })
            }
            Err(e) => return Err(PersistenceError::from_io(e)),
        };
        serde_json::from_slice(&bytes).map_err(|e| corrupt(hash, &e.to_string()))
    }

    fn list(&self, dir: &str) -> PersistenceResult<Vec<ContentHash>> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(self.root.join(dir))? {
            let name = entry?.file_name();
            let Some(stem) = name.to_str().and_then(|n| n.strip_suffix(".json")) else {
                continue;
            };
            if let Ok(hash) = stem.parse() {
                out.push(hash);
            }
        }
        out.sort();
        Ok(out)
    }
}

fn corrupt(hash: &ContentHash, why: &str) -> PersistenceError {
    PersistenceError::Corrupt {
        detail: format!("{hash}: {why}"),
    }
}
