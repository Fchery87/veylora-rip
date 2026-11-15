//! Durable storage for device setups and print modes.
//!
//! Layout under `<root>`: `VERSION`, `modes/<hash>.json`, and `setups/<hash>/<nnnnnnnn>.json`.
//!
//! A mode's hash covers its status, so every promotion is a different, immutable file. A setup's
//! hash does not cover its verification state, which changes when evidence is recorded, so each
//! setup keeps an append-only run of numbered state files and the latest one is current. Earlier
//! states are never rewritten. Every read re-validates the record through the device crate, and a
//! record filed under the wrong name or edited by hand is reported as corrupt.

use crate::{
    errors::{PersistenceError, PersistenceResult},
    jobs::{atomic_write, read_schema_version},
};
use std::path::{Path, PathBuf};
use veylora_device::{
    mode::{ModeRecord, PrintMode},
    setup::{DeviceSetup, SetupRecord, VerificationState},
};
use veylora_domain::provenance::ContentHash;

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug)]
pub struct FileDeviceStore {
    root: PathBuf,
}

impl FileDeviceStore {
    pub fn open(root: impl Into<PathBuf>) -> PersistenceResult<Self> {
        let root = root.into();
        std::fs::create_dir_all(root.join("setups"))?;
        std::fs::create_dir_all(root.join("modes"))?;
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

    fn setup_dir(&self, hash: &ContentHash) -> PathBuf {
        self.root.join("setups").join(hash.to_string())
    }
    fn mode_path(&self, hash: &ContentHash) -> PathBuf {
        self.root.join("modes").join(format!("{hash}.json"))
    }

    /// Records the setup's current state. An unchanged state is not written again.
    pub fn put_setup(&self, setup: &DeviceSetup) -> PersistenceResult<ContentHash> {
        let hash = setup.content_hash();
        let dir = self.setup_dir(&hash);
        std::fs::create_dir_all(&dir)?;
        let record = setup.to_record();
        let states = state_files(&dir)?;
        if let Some((_, latest)) = states.last() {
            if read_setup_record(latest, &hash)? == record {
                return Ok(hash);
            }
        }
        let next = states.last().map_or(1, |(n, _)| n + 1);
        atomic_write(
            &dir.join(format!("{next:08}.json")),
            &serde_json::to_vec(&record)?,
        )?;
        Ok(hash)
    }

    /// The latest state of the setup with this content hash.
    pub fn get_setup(&self, hash: &ContentHash) -> PersistenceResult<DeviceSetup> {
        let dir = self.setup_dir(hash);
        let states = state_files(&dir)?;
        let (_, latest) = states.last().ok_or_else(|| PersistenceError::NotFound {
            id: hash.to_string(),
        })?;
        load_setup(latest, hash)
    }

    /// Verification state at each recorded step, oldest first.
    pub fn setup_states(&self, hash: &ContentHash) -> PersistenceResult<Vec<VerificationState>> {
        state_files(&self.setup_dir(hash))?
            .iter()
            .map(|(_, path)| Ok(load_setup(path, hash)?.verification().clone()))
            .collect()
    }

    pub fn list_setups(&self) -> PersistenceResult<Vec<ContentHash>> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(self.root.join("setups"))? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(hash) = entry.file_name().to_str().and_then(|n| n.parse().ok()) {
                    out.push(hash);
                }
            }
        }
        out.sort();
        Ok(out)
    }

    /// Write-once. An existing file must hold identical bytes; a differing one is damaged and is
    /// reported, never overwritten.
    pub fn put_mode(&self, mode: &PrintMode) -> PersistenceResult<ContentHash> {
        let hash = mode.content_hash();
        let path = self.mode_path(&hash);
        let bytes = serde_json::to_vec(&mode.to_record())?;
        match std::fs::read(&path) {
            Ok(existing) if existing == bytes => {}
            Ok(_) => return Err(corrupt(&hash, "mode file exists with different content")),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => atomic_write(&path, &bytes)?,
            Err(e) => return Err(PersistenceError::from_io(e)),
        }
        Ok(hash)
    }

    pub fn get_mode(&self, hash: &ContentHash) -> PersistenceResult<PrintMode> {
        let bytes = match std::fs::read(self.mode_path(hash)) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(PersistenceError::NotFound {
                    id: hash.to_string(),
                })
            }
            Err(e) => return Err(PersistenceError::from_io(e)),
        };
        let record: ModeRecord =
            serde_json::from_slice(&bytes).map_err(|e| corrupt(hash, &e.to_string()))?;
        let mode = PrintMode::from_record(record).map_err(|f| corrupt(hash, &f.message))?;
        if mode.content_hash() != *hash {
            return Err(corrupt(hash, "mode content does not match its name"));
        }
        Ok(mode)
    }

    pub fn list_modes(&self) -> PersistenceResult<Vec<ContentHash>> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(self.root.join("modes"))? {
            let name = entry?.file_name();
            if let Some(hash) = name
                .to_str()
                .and_then(|n| n.strip_suffix(".json"))
                .and_then(|n| n.parse().ok())
            {
                out.push(hash);
            }
        }
        out.sort();
        Ok(out)
    }
}

fn state_files(dir: &Path) -> PersistenceResult<Vec<(u32, PathBuf)>> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(PersistenceError::from_io(e)),
    };
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        if let Some(n) = name
            .to_str()
            .and_then(|n| n.strip_suffix(".json"))
            .and_then(|n| n.parse::<u32>().ok())
        {
            out.push((n, entry.path()));
        }
    }
    out.sort();
    Ok(out)
}

fn read_setup_record(path: &Path, hash: &ContentHash) -> PersistenceResult<SetupRecord> {
    let bytes = std::fs::read(path).map_err(PersistenceError::from_io)?;
    serde_json::from_slice(&bytes).map_err(|e| corrupt(hash, &e.to_string()))
}

fn load_setup(path: &Path, hash: &ContentHash) -> PersistenceResult<DeviceSetup> {
    let setup = DeviceSetup::from_record(read_setup_record(path, hash)?)
        .map_err(|f| corrupt(hash, &f.message))?;
    if setup.content_hash() != *hash {
        return Err(corrupt(hash, "setup content does not match its name"));
    }
    Ok(setup)
}

fn corrupt(hash: &ContentHash, why: &str) -> PersistenceError {
    PersistenceError::Corrupt {
        detail: format!("{hash}: {why}"),
    }
}
