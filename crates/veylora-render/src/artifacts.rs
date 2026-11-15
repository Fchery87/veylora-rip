//! Disk-backed, write-once stage artifacts. Files are created exclusively and never overwritten,
//! hashed as they are written, and removed together if a render does not complete.
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use veylora_domain::provenance::{ContentHash, ContentHasher};

#[derive(Debug)]
pub enum ArtifactError {
    /// Names are one path component of ASCII letters, digits, `-`, `_` and `.`.
    InvalidName,
    /// The artifact already exists. Artifacts are write-once.
    Exists,
    /// The byte quota was exceeded or the device is full.
    NoSpace,
    Io(std::io::Error),
}
impl std::fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName => f.write_str("artifact.invalid_name"),
            Self::Exists => f.write_str("artifact.exists"),
            Self::NoSpace => f.write_str("artifact.no_space"),
            Self::Io(e) => write!(f, "artifact.io: {e}"),
        }
    }
}
impl std::error::Error for ArtifactError {}
impl From<std::io::Error> for ArtifactError {
    fn from(e: std::io::Error) -> Self {
        // Full-disk conditions surface as a distinct, actionable error.
        match e.kind() {
            std::io::ErrorKind::StorageFull | std::io::ErrorKind::QuotaExceeded => Self::NoSpace,
            _ => Self::Io(e),
        }
    }
}

/// One retained file: exact bytes, with the hash of those bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactFile {
    pub name: String,
    pub bytes: u64,
    pub hash: ContentHash,
}

pub trait ArtifactSink {
    fn write(&mut self, bytes: &[u8]) -> Result<(), ArtifactError>;
    /// Flushes and returns the retained file's size and hash.
    fn finish(self: Box<Self>) -> Result<ArtifactFile, ArtifactError>;
}
pub trait ArtifactStore {
    /// Creates a new write-once artifact.
    fn create(&mut self, name: &str) -> Result<Box<dyn ArtifactSink>, ArtifactError>;
    /// Bytes the store can still accept, if it knows. The operating system's free space is not
    /// queried (the standard library has no portable call), so a plain directory store only
    /// knows a configured quota.
    fn remaining_bytes(&self) -> Option<u64>;
    /// Removes every artifact created through this store.
    fn rollback(&mut self);
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && !name.starts_with('.')
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

/// Stores artifacts as files directly inside an existing directory.
pub struct DirectoryStore {
    root: PathBuf,
    quota: Option<u64>,
    used: Arc<AtomicU64>,
    created: Vec<PathBuf>,
}
impl DirectoryStore {
    pub fn new(root: impl Into<PathBuf>, quota_bytes: Option<u64>) -> Self {
        Self {
            root: root.into(),
            quota: quota_bytes,
            used: Arc::new(AtomicU64::new(0)),
            created: Vec::new(),
        }
    }
    pub fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
}
struct FileSink {
    file: BufWriter<File>,
    hasher: ContentHasher,
    name: String,
    bytes: u64,
    quota: Option<u64>,
    used: Arc<AtomicU64>,
}
impl ArtifactSink for FileSink {
    fn write(&mut self, bytes: &[u8]) -> Result<(), ArtifactError> {
        let len = bytes.len() as u64;
        let total = self.used.fetch_add(len, Ordering::SeqCst) + len;
        if self.quota.is_some_and(|q| total > q) {
            self.used.fetch_sub(len, Ordering::SeqCst);
            return Err(ArtifactError::NoSpace);
        }
        self.file.write_all(bytes)?;
        self.hasher.update(bytes);
        self.bytes += len;
        Ok(())
    }
    fn finish(mut self: Box<Self>) -> Result<ArtifactFile, ArtifactError> {
        self.file.flush()?;
        self.file.get_ref().sync_all()?;
        Ok(ArtifactFile {
            name: self.name,
            bytes: self.bytes,
            hash: self.hasher.finalize(),
        })
    }
}
impl ArtifactStore for DirectoryStore {
    fn create(&mut self, name: &str) -> Result<Box<dyn ArtifactSink>, ArtifactError> {
        if !valid_name(name) {
            return Err(ArtifactError::InvalidName);
        }
        let path = self.root.join(name);
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::AlreadyExists => ArtifactError::Exists,
                _ => ArtifactError::from(e),
            })?;
        self.created.push(path);
        Ok(Box::new(FileSink {
            file: BufWriter::new(file),
            hasher: ContentHasher::new(),
            name: name.to_owned(),
            bytes: 0,
            quota: self.quota,
            used: Arc::clone(&self.used),
        }))
    }
    fn remaining_bytes(&self) -> Option<u64> {
        self.quota
            .map(|q| q.saturating_sub(self.used.load(Ordering::SeqCst)))
    }
    fn rollback(&mut self) {
        for path in self.created.drain(..) {
            let _ = std::fs::remove_file(path);
        }
        self.used.store(0, Ordering::SeqCst);
    }
}

/// Sample encoding of a stage file: little-endian, row-major, one plane per file, no header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SampleFormat {
    /// Normalized coverage, 0.0..=1.0.
    F32Le,
    /// Device raster codes (0 or the dot level).
    U16Le,
}
impl SampleFormat {
    pub const fn bytes(self) -> u64 {
        match self {
            Self::F32Le => 4,
            Self::U16Le => 2,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    /// C, M, Y, K and white after separation, white geometry and clear mask, before calibration.
    LogicalPlanes,
    /// Calibrated planes allocated to physical slots, addressed by protocol channel.
    PhysicalPlanes,
    /// Binary device raster codes after screening.
    ScreenedPlanes,
}
/// All files of one stage, with a combined hash over the ordered (name, hash) pairs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageArtifact {
    pub stage: Stage,
    pub width: usize,
    pub height: usize,
    pub format: SampleFormat,
    pub files: Vec<ArtifactFile>,
    pub hash: ContentHash,
}
impl StageArtifact {
    pub(crate) fn new(
        stage: Stage,
        width: usize,
        height: usize,
        format: SampleFormat,
        files: Vec<ArtifactFile>,
    ) -> Self {
        let mut h = ContentHasher::new();
        h.update(b"veylora-render-stage-v1\0");
        for f in &files {
            h.update(&(f.name.len() as u64).to_le_bytes());
            h.update(f.name.as_bytes());
            h.update(&f.bytes.to_le_bytes());
            h.update(f.hash.to_string().as_bytes());
        }
        Self {
            stage,
            width,
            height,
            format,
            files,
            hash: h.finalize(),
        }
    }
}
