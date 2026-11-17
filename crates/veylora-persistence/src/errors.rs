//! Errors returned by the durable store. Every variant is explicit: a caller
//! inspecting the error must be able to decide whether the state is safe to
//! continue from.

use std::{fmt, io};

/// Alias for persistence results.
pub type PersistenceResult<T> = Result<T, PersistenceError>;

/// Classified persistence error. Variants are narrow on purpose: callers must
/// handle each failure mode distinctly and must never auto-retry an ambiguous
/// submission.
#[derive(Debug)]
pub enum PersistenceError {
    /// The store directory exists but holds a newer or older schema than this
    /// build understands. Migrations advance forwards only.
    SchemaOutOfRange { found: u32, supported: u32 },

    /// A stored file or log entry failed structural validation. The specific
    /// record is identified for diagnostics; no bytes are truncated by the
    /// store itself.
    Corrupt { detail: String },

    /// The host filesystem rejected the write because no space was left.
    /// The store treats this as a hard failure; callers decide whether to
    /// retry after reclaiming space.
    DiskFull { detail: String },

    /// A write attempted to replace an immutable record.
    ImmutableRecordExists { id: String },

    /// An append-only attempt record would violate the contract (duplicate
    /// sequence, snapshot-hash drift, or out-of-order history).
    AttemptContractViolation { detail: String },

    /// A lifecycle event is not legal in the job's current state.
    LifecycleViolation { detail: String },

    /// A follow-up attempt was requested for a job whose latest attempt is
    /// ambiguous. The store refuses to record automatic retries.
    AmbiguousNoRetry { job_id: String },

    /// Record not found.
    NotFound { id: String },

    /// Other I/O failure, with the underlying error preserved.
    Io(io::Error),

    /// JSON encode/decode failure that is not a schema-version problem.
    Serde(serde_json::Error),
}

impl PersistenceError {
    pub(crate) fn from_io(err: io::Error) -> Self {
        if is_disk_full(&err) {
            Self::DiskFull {
                detail: err.to_string(),
            }
        } else {
            Self::Io(err)
        }
    }
}

#[cfg(windows)]
fn is_disk_full(err: &io::Error) -> bool {
    // ERROR_DISK_FULL and ERROR_HANDLE_DISK_FULL on Windows.
    matches!(err.raw_os_error(), Some(112) | Some(39))
}

#[cfg(not(windows))]
fn is_disk_full(err: &io::Error) -> bool {
    matches!(err.raw_os_error(), Some(code) if code == 28)
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SchemaOutOfRange { found, supported } => write!(
                f,
                "persistence schema version {found} is outside this build's supported range (<= {supported})"
            ),
            Self::Corrupt { detail } => write!(f, "persistence record is corrupt: {detail}"),
            Self::DiskFull { detail } => write!(f, "persistence write failed, disk is full: {detail}"),
            Self::ImmutableRecordExists { id } => {
                write!(f, "refusing to overwrite immutable record {id}")
            }
            Self::AttemptContractViolation { detail } => {
                write!(f, "append-only attempt log violated: {detail}")
            }
            Self::LifecycleViolation { detail } => {
                write!(f, "job lifecycle violated: {detail}")
            }
            Self::AmbiguousNoRetry { job_id } => write!(
                f,
                "job {job_id} has an ambiguous submission; manual review required before another attempt"
            ),
            Self::NotFound { id } => write!(f, "persistence record {id} not found"),
            Self::Io(err) => write!(f, "persistence I/O error: {err}"),
            Self::Serde(err) => write!(f, "persistence serde error: {err}"),
        }
    }
}

impl std::error::Error for PersistenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Serde(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for PersistenceError {
    fn from(err: io::Error) -> Self {
        Self::from_io(err)
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serde(err)
    }
}
