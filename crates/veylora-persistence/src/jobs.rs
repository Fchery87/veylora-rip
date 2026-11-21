//! Durable, append-only job store.
//!
//! Snapshots are write-once JSON files; attempts are appended to a JSON-Lines
//! event log. Reads reconstruct the latest state per attempt by replaying the
//! log in order. The store never rewrites or truncates a log; a corrupt tail is
//! surfaced as [`PersistenceError::Corrupt`] and the caller decides how to
//! quarantine the file.

use crate::{
    errors::{PersistenceError, PersistenceResult},
    migrations::{shipped_migrations, CURRENT_SCHEMA_VERSION},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
};
use veylora_domain::{
    ids::{AttemptId, JobId},
    job::{AttemptState, JobSnapshot, SubmissionAttempt},
    provenance::ContentHash,
};

/// Snapshot of the most recent known state of an attempt, derived by replaying
/// the append-only log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptView {
    pub attempt_id: AttemptId,
    pub job_id: JobId,
    pub snapshot_hash: ContentHash,
    pub sequence: u32,
    pub state: AttemptState,
    pub bytes_written: u64,
    pub note: String,
    pub recorded_at: String,
}

impl From<&SubmissionAttempt> for AttemptView {
    fn from(a: &SubmissionAttempt) -> Self {
        Self {
            attempt_id: a.attempt_id,
            job_id: a.job_id,
            snapshot_hash: a.snapshot_hash,
            sequence: a.sequence,
            state: a.state,
            bytes_written: a.bytes_written,
            note: a.note.clone(),
            recorded_at: a.recorded_at.clone(),
        }
    }
}

/// Minimal append-only job store contract.
pub trait JobStore {
    fn create_snapshot(&self, job: JobSnapshot) -> PersistenceResult<()>;
    fn get(&self, id: JobId) -> PersistenceResult<JobSnapshot>;
    fn append_attempt(&self, attempt: SubmissionAttempt) -> PersistenceResult<()>;
    fn list_attempts(&self, id: JobId) -> PersistenceResult<Vec<AttemptView>>;
}

/// Filesystem implementation.
#[derive(Debug)]
pub struct FileJobStore {
    root: PathBuf,
}

impl FileJobStore {
    /// Open or initialize a store at `root`, running shipped migrations.
    pub fn open(root: impl Into<PathBuf>) -> PersistenceResult<Self> {
        let root = root.into();
        std::fs::create_dir_all(&root)?;
        let found = read_schema_version(&root)?;
        if found > CURRENT_SCHEMA_VERSION {
            return Err(PersistenceError::SchemaOutOfRange {
                found,
                supported: CURRENT_SCHEMA_VERSION,
            });
        }
        let migrations = shipped_migrations()?;
        let new_version = migrations.run(&root, found)?;
        if new_version != found {
            write_schema_version(&root, new_version)?;
        }
        Ok(Self { root })
    }

    /// Return the on-disk schema version. Used by restart recovery tests.
    pub fn schema_version(&self) -> PersistenceResult<u32> {
        read_schema_version(&self.root)
    }

    fn job_path(&self, id: JobId) -> PathBuf {
        self.root.join("jobs").join(format!("{id}.json"))
    }

    /// Every raw record for `id` in the order it was appended, including the opening
    /// `Submitting` record of each attempt. `list_attempts` only shows each attempt's latest state.
    pub fn attempt_history(&self, id: JobId) -> PersistenceResult<Vec<SubmissionAttempt>> {
        Ok(self
            .read_attempts()?
            .into_iter()
            .filter(|r| r.job_id == id)
            .collect())
    }

    fn log_path(&self) -> PathBuf {
        self.root.join("attempts.log")
    }

    fn read_attempts(&self) -> PersistenceResult<Vec<SubmissionAttempt>> {
        let path = self.log_path();
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(PersistenceError::from_io(err)),
        };
        let reader = BufReader::new(file);
        let mut out = Vec::new();
        for (line_no, line) in reader.lines().enumerate() {
            let line = line.map_err(PersistenceError::from_io)?;
            if line.trim().is_empty() {
                continue;
            }
            let record: SubmissionAttempt =
                serde_json::from_str(&line).map_err(|err| PersistenceError::Corrupt {
                    detail: format!("attempts.log line {}: {err}", line_no + 1),
                })?;
            out.push(record);
        }
        Ok(out)
    }

    fn validate_append(
        &self,
        history: &[SubmissionAttempt],
        snapshot: &JobSnapshot,
        incoming: &SubmissionAttempt,
    ) -> PersistenceResult<()> {
        if incoming.snapshot_hash != snapshot.content_hash() {
            return Err(PersistenceError::AttemptContractViolation {
                detail: "attempt snapshot hash does not match stored snapshot".into(),
            });
        }
        if incoming.recorded_at.trim().is_empty() {
            return Err(PersistenceError::AttemptContractViolation {
                detail: "attempt requires a recorded_at timestamp".into(),
            });
        }
        let mut per_attempt: BTreeMap<AttemptId, &SubmissionAttempt> = BTreeMap::new();
        for record in history.iter().filter(|r| r.job_id == incoming.job_id) {
            per_attempt
                .entry(record.attempt_id)
                .and_modify(|current| {
                    if record.sequence > current.sequence {
                        *current = record;
                    }
                })
                .or_insert(record);
        }

        // Ambiguous attempts prohibit automatic follow-on records. Only a
        // terminal transition on the same attempt (operator-confirmed-finished)
        // is allowed.
        if let Some(latest) = per_attempt
            .values()
            .max_by_key(|a| a.sequence)
            .filter(|a| a.state.is_ambiguous())
        {
            let allowed_follow_up = incoming.attempt_id == latest.attempt_id
                && matches!(incoming.state, AttemptState::OperatorConfirmedFinished);
            if !allowed_follow_up {
                return Err(PersistenceError::AmbiguousNoRetry {
                    job_id: incoming.job_id.to_string(),
                });
            }
        }

        match per_attempt.get(&incoming.attempt_id) {
            None => {
                // Fresh attempt: first event must be Submitting.
                if !matches!(incoming.state, AttemptState::Submitting) {
                    return Err(PersistenceError::AttemptContractViolation {
                        detail: "a new attempt must open in the submitting state".into(),
                    });
                }
                let expected_sequence =
                    per_attempt.values().map(|a| a.sequence).max().unwrap_or(0) + 1;
                if incoming.sequence != expected_sequence {
                    return Err(PersistenceError::AttemptContractViolation {
                        detail: format!(
                            "new attempt sequence must be {expected_sequence}, got {}",
                            incoming.sequence
                        ),
                    });
                }
            }
            Some(previous) => {
                let is_operator_confirm_after_ambiguous = previous.state.is_ambiguous()
                    && matches!(incoming.state, AttemptState::OperatorConfirmedFinished);
                if previous.state.is_terminal() && !is_operator_confirm_after_ambiguous {
                    return Err(PersistenceError::AttemptContractViolation {
                        detail: "cannot append more events to a terminal attempt".into(),
                    });
                }
                if incoming.sequence <= previous.sequence {
                    return Err(PersistenceError::AttemptContractViolation {
                        detail: format!(
                            "attempt sequence must strictly increase (was {}, got {})",
                            previous.sequence, incoming.sequence
                        ),
                    });
                }
                if incoming.snapshot_hash != previous.snapshot_hash {
                    return Err(PersistenceError::AttemptContractViolation {
                        detail: "attempt snapshot hash changed between events".into(),
                    });
                }
            }
        }
        Ok(())
    }
}

impl JobStore for FileJobStore {
    fn create_snapshot(&self, job: JobSnapshot) -> PersistenceResult<()> {
        let path = self.job_path(job.job_id());
        if path.exists() {
            return Err(PersistenceError::ImmutableRecordExists {
                id: job.job_id().to_string(),
            });
        }
        let payload = serde_json::to_vec_pretty(&SnapshotEnvelope {
            schema_version: CURRENT_SCHEMA_VERSION,
            snapshot_hash: job.content_hash(),
            snapshot: job,
        })?;
        atomic_write(&path, &payload)?;
        Ok(())
    }

    fn get(&self, id: JobId) -> PersistenceResult<JobSnapshot> {
        let path = self.job_path(id);
        let mut file = File::open(&path).map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                PersistenceError::NotFound { id: id.to_string() }
            } else {
                PersistenceError::from_io(err)
            }
        })?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(PersistenceError::from_io)?;
        let envelope: SnapshotEnvelope =
            serde_json::from_slice(&bytes).map_err(|err| PersistenceError::Corrupt {
                detail: format!("snapshot {id}: {err}"),
            })?;
        if envelope.snapshot_hash != envelope.snapshot.content_hash() {
            return Err(PersistenceError::Corrupt {
                detail: format!("snapshot {id} hash does not match stored contents"),
            });
        }
        if envelope.schema_version > CURRENT_SCHEMA_VERSION {
            return Err(PersistenceError::SchemaOutOfRange {
                found: envelope.schema_version,
                supported: CURRENT_SCHEMA_VERSION,
            });
        }
        Ok(envelope.snapshot)
    }

    fn append_attempt(&self, attempt: SubmissionAttempt) -> PersistenceResult<()> {
        let snapshot = self.get(attempt.job_id)?;
        let history = self.read_attempts()?;
        self.validate_append(&history, &snapshot, &attempt)?;
        let line = serde_json::to_string(&attempt)?;
        let path = self.log_path();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(PersistenceError::from_io)?;
        file.write_all(line.as_bytes())
            .map_err(PersistenceError::from_io)?;
        file.write_all(b"\n").map_err(PersistenceError::from_io)?;
        file.sync_all().map_err(PersistenceError::from_io)?;
        sync_dir(&self.root)?;
        Ok(())
    }

    fn list_attempts(&self, id: JobId) -> PersistenceResult<Vec<AttemptView>> {
        let history = self.read_attempts()?;
        let mut per_attempt: BTreeMap<AttemptId, SubmissionAttempt> = BTreeMap::new();
        for record in history.into_iter().filter(|r| r.job_id == id) {
            per_attempt
                .entry(record.attempt_id)
                .and_modify(|current| {
                    if record.sequence > current.sequence {
                        *current = record.clone();
                    }
                })
                .or_insert(record);
        }
        let mut views: Vec<AttemptView> = per_attempt.values().map(AttemptView::from).collect();
        views.sort_by_key(|v| v.sequence);
        Ok(views)
    }
}

#[derive(Serialize, Deserialize)]
struct SnapshotEnvelope {
    schema_version: u32,
    snapshot_hash: ContentHash,
    snapshot: JobSnapshot,
}

pub(crate) fn create_empty_log(path: &Path) -> PersistenceResult<()> {
    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(PersistenceError::from_io)?;
    file.sync_all().map_err(PersistenceError::from_io)?;
    Ok(())
}

pub(crate) fn read_schema_version(root: &Path) -> PersistenceResult<u32> {
    let path = root.join("VERSION");
    match std::fs::read_to_string(&path) {
        Ok(text) => text
            .trim()
            .parse()
            .map_err(|err| PersistenceError::Corrupt {
                detail: format!("VERSION file: {err}"),
            }),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(err) => Err(PersistenceError::from_io(err)),
    }
}

fn write_schema_version(root: &Path, version: u32) -> PersistenceResult<()> {
    atomic_write(&root.join("VERSION"), version.to_string().as_bytes())?;
    sync_dir(root)?;
    Ok(())
}

pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> PersistenceResult<()> {
    let tmp = tmp_path(path);
    {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp)
            .map_err(PersistenceError::from_io)?;
        file.write_all(bytes).map_err(PersistenceError::from_io)?;
        file.sync_all().map_err(PersistenceError::from_io)?;
    }
    std::fs::rename(&tmp, path).map_err(PersistenceError::from_io)?;
    if let Some(parent) = path.parent() {
        sync_dir(parent)?;
    }
    Ok(())
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

pub(crate) fn sync_dir(path: &Path) -> PersistenceResult<()> {
    // On Windows a directory File cannot be fsynced; on Unix the open+sync
    // ensures directory entries are durable. Soft-fail when the platform does
    // not support it.
    match File::open(path) {
        Ok(file) => match file.sync_all() {
            Ok(()) => Ok(()),
            Err(err) if is_unsupported(&err) => Ok(()),
            Err(err) => Err(PersistenceError::from_io(err)),
        },
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => Ok(()),
        Err(err) => Err(PersistenceError::from_io(err)),
    }
}

fn is_unsupported(err: &std::io::Error) -> bool {
    matches!(
        err.kind(),
        std::io::ErrorKind::Unsupported | std::io::ErrorKind::InvalidInput
    )
}
