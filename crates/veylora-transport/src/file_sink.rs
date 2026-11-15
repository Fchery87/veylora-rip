use crate::{ensure_parent, PrinterArtifact};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileSinkError {
    #[error("output already exists: {0}")]
    AlreadyExists(PathBuf),
    #[error("file sink I/O error: {0}")]
    Io(#[from] io::Error),
}

/// Writes exact upstream artifact bytes once, without encoding or protocol knowledge.
#[derive(Debug, Clone)]
pub struct FileSink {
    path: PathBuf,
}

impl FileSink {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn write(&self, artifact: &PrinterArtifact) -> Result<(), FileSinkError> {
        ensure_parent(&self.path)?;
        if self.path.exists() {
            return Err(FileSinkError::AlreadyExists(self.path.clone()));
        }
        let temporary = self.path.with_extension("tmp");
        let result = (|| {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            io::Write::write_all(&mut file, artifact.bytes())?;
            file.sync_all()?;
            fs::rename(&temporary, &self.path)?;
            Ok::<_, io::Error>(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result.map_err(FileSinkError::Io)
    }
}
