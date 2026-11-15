//! Printer-independent submission contracts. This crate deliberately does not encode device protocol values.

use serde::{Deserialize, Serialize};
use std::{io, path::Path};
use thiserror::Error;
use uuid::Uuid;
use veylora_domain::provenance::ContentHash;

pub mod fake;
mod file_sink;
pub mod windows;

pub use file_sink::{FileSink, FileSinkError};

/// Exact bytes produced by an upstream encoder or artifact stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrinterArtifact {
    bytes: Vec<u8>,
    hash: ContentHash,
}

impl PrinterArtifact {
    pub fn from_encoded_bytes(bytes: Vec<u8>) -> Self {
        let hash = ContentHash::digest(&bytes);
        Self { bytes, hash }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn len(&self) -> u64 {
        self.bytes.len() as u64
    }
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
    pub fn hash(&self) -> ContentHash {
        self.hash
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportStatus {
    Started,
    Writing,
    Finished,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportOutcome {
    Submitted,
    Cancelled,
    Failed,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportResult {
    pub attempt_id: Uuid,
    pub outcome: TransportOutcome,
    pub bytes_intended: u64,
    pub bytes_written: u64,
}

pub trait SubmissionObserver {
    fn on_bytes(&mut self, count: u64);
    fn on_status(&mut self, status: TransportStatus);
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("transport failed before writing")]
    BeforeWrite,
    #[error("transport failed after writing {bytes_written} of {bytes_intended} bytes")]
    DuringWrite {
        bytes_written: u64,
        bytes_intended: u64,
    },
    #[error("transport result is ambiguous after {bytes_written} of {bytes_intended} bytes")]
    Ambiguous {
        bytes_written: u64,
        bytes_intended: u64,
    },
    #[error("submission cancelled after {bytes_written} of {bytes_intended} bytes")]
    Cancelled {
        bytes_written: u64,
        bytes_intended: u64,
    },
    #[error("transport I/O error: {0}")]
    Io(#[from] io::Error),
}

pub trait Cancellation: Send + Sync {
    fn is_cancelled(&self) -> bool;
}

pub trait PrinterTransport {
    fn submit(
        &self,
        artifact: &PrinterArtifact,
        observer: &mut dyn SubmissionObserver,
    ) -> Result<TransportResult, TransportError>;
}

/// Append-only facts for every submission attempt. It intentionally has no retry method.
#[derive(Debug, Default)]
pub struct SubmissionAttempts {
    facts: Vec<TransportResult>,
}
impl SubmissionAttempts {
    pub fn record(&mut self, result: TransportResult) {
        self.facts.push(result);
    }
    pub fn record_error(&mut self, attempt_id: Uuid, error: &TransportError) {
        let (outcome, bytes_written, bytes_intended) = match error {
            TransportError::BeforeWrite => (TransportOutcome::Failed, 0, 0),
            TransportError::DuringWrite {
                bytes_written,
                bytes_intended,
            } => (TransportOutcome::Failed, *bytes_written, *bytes_intended),
            TransportError::Ambiguous {
                bytes_written,
                bytes_intended,
            } => (TransportOutcome::Ambiguous, *bytes_written, *bytes_intended),
            TransportError::Cancelled {
                bytes_written,
                bytes_intended,
            } => (TransportOutcome::Cancelled, *bytes_written, *bytes_intended),
            TransportError::Io(_) => (TransportOutcome::Failed, 0, 0),
        };
        self.record(TransportResult {
            attempt_id,
            outcome,
            bytes_intended,
            bytes_written,
        });
    }
    pub fn facts(&self) -> &[TransportResult] {
        &self.facts
    }
}

pub(crate) fn ensure_parent(path: &Path) -> Result<(), FileSinkError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}
