//! Durable, append-only job lifecycle history.
//!
//! Layout: `<root>/lifecycle/<job>/<nnnnnnnn>.json`, one immutable file per event. A job with no
//! events is a draft. State is never stored; it is the replay of the events through the domain
//! state machine, and every read re-checks the whole history (contiguous sequence numbers, each
//! event starting from the state the previous one ended in, each result matching the machine).
//! Damaged history is reported as corrupt, never repaired or skipped.
//!
//! Each event is written to a temporary file and hard-linked to its final name. The link fails
//! when the name exists, so two writers racing for the same sequence number cannot both succeed,
//! and a crash leaves at most an ignored `.tmp` file.

use crate::{
    errors::{PersistenceError, PersistenceResult},
    jobs::{read_schema_version, sync_dir},
};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use veylora_domain::{
    ids::{AttemptId, JobId},
    lifecycle::{transition, JobState, LifecycleEvent},
};

const SCHEMA_VERSION: u32 = 1;
/// How many times a lost race is retried before giving up. A lost race means another writer
/// advanced the job, so each retry re-checks legality against the new state.
const MAX_RACES: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleRecord {
    pub schema_version: u32,
    pub job_id: JobId,
    pub sequence: u32,
    pub event: LifecycleEvent,
    pub from: JobState,
    pub to: JobState,
    pub recorded_at: String,
    pub note: String,
    /// The submission attempt this event concerns, when it concerns one.
    pub attempt_id: Option<AttemptId>,
}

#[derive(Debug)]
pub struct FileLifecycleStore {
    root: PathBuf,
}

impl FileLifecycleStore {
    pub fn open(root: impl Into<PathBuf>) -> PersistenceResult<Self> {
        let root = root.into();
        std::fs::create_dir_all(root.join("lifecycle"))?;
        match read_schema_version(&root)? {
            0 => write_version(&root)?,
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

    fn job_dir(&self, job: JobId) -> PathBuf {
        self.root.join("lifecycle").join(job.to_string())
    }

    /// The validated history, oldest first.
    pub fn events(&self, job: JobId) -> PersistenceResult<Vec<LifecycleRecord>> {
        let dir = self.job_dir(job);
        let mut files = Vec::new();
        match std::fs::read_dir(&dir) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry?;
                    if let Some(n) = entry
                        .file_name()
                        .to_str()
                        .and_then(|n| n.strip_suffix(".json"))
                        .and_then(|n| n.parse::<u32>().ok())
                    {
                        files.push((n, entry.path()));
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(PersistenceError::from_io(e)),
        }
        files.sort();
        let mut out: Vec<LifecycleRecord> = Vec::with_capacity(files.len());
        for (n, path) in files {
            let bytes = std::fs::read(&path).map_err(PersistenceError::from_io)?;
            let record: LifecycleRecord = serde_json::from_slice(&bytes)
                .map_err(|e| corrupt(job, n, &format!("unreadable record: {e}")))?;
            let previous = out.last().map_or(JobState::Draft, |r| r.to);
            if record.schema_version != SCHEMA_VERSION {
                return Err(corrupt(job, n, "unsupported record schema"));
            }
            if record.job_id != job || record.sequence != n || n as usize != out.len() + 1 {
                return Err(corrupt(
                    job,
                    n,
                    "record is filed under the wrong job or sequence",
                ));
            }
            if record.from != previous {
                return Err(corrupt(
                    job,
                    n,
                    "record does not start from the previous state",
                ));
            }
            match transition(record.from, record.event) {
                Ok(to) if to == record.to => {}
                _ => {
                    return Err(corrupt(
                        job,
                        n,
                        "recorded result is not what the state machine allows",
                    ))
                }
            }
            out.push(record);
        }
        Ok(out)
    }

    /// Writes a whole history for a job that has none, all or nothing. The records are written to a
    /// staging directory that readers ignore and moved into place in one step, so a failure leaves
    /// no partial history. Refuses a job that already has a history, however small.
    pub fn import_events(&self, job: JobId, records: &[LifecycleRecord]) -> PersistenceResult<()> {
        let dir = self.job_dir(job);
        if dir.exists() {
            return Err(PersistenceError::LifecycleViolation {
                detail: format!("job {job} already has a history; an import never merges into it"),
            });
        }
        static STAGE: AtomicU64 = AtomicU64::new(0);
        let staging = self.root.join("lifecycle").join(format!(
            "staging-{}-{}",
            std::process::id(),
            STAGE.fetch_add(1, Ordering::SeqCst)
        ));
        let result = (|| {
            std::fs::create_dir_all(&staging).map_err(PersistenceError::from_io)?;
            for r in records {
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(staging.join(format!("{:08}.json", r.sequence)))
                    .map_err(PersistenceError::from_io)?;
                file.write_all(&serde_json::to_vec(r)?)
                    .map_err(PersistenceError::from_io)?;
                file.sync_all().map_err(PersistenceError::from_io)?;
            }
            sync_dir(&staging)?;
            std::fs::rename(&staging, &dir).map_err(PersistenceError::from_io)?;
            sync_dir(&self.root.join("lifecycle"))
        })();
        if result.is_err() {
            let _ = std::fs::remove_dir_all(&staging);
        }
        result
    }

    /// Removes a history that [`Self::import_events`] just wrote, when a later step of the same
    /// import failed. Only for a history the caller created itself.
    pub fn discard_imported_events(&self, job: JobId) -> PersistenceResult<()> {
        match std::fs::remove_dir_all(self.job_dir(job)) {
            Ok(()) => sync_dir(&self.root.join("lifecycle")),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(PersistenceError::from_io(e)),
        }
    }

    pub fn state(&self, job: JobId) -> PersistenceResult<JobState> {
        Ok(self.events(job)?.last().map_or(JobState::Draft, |r| r.to))
    }

    /// Records `event` if the state machine allows it from the job's current state. Nothing is
    /// written for an illegal event.
    pub fn append(
        &self,
        job: JobId,
        event: LifecycleEvent,
        recorded_at: &str,
        note: &str,
        attempt_id: Option<AttemptId>,
    ) -> PersistenceResult<LifecycleRecord> {
        if recorded_at.trim().is_empty() {
            return Err(PersistenceError::LifecycleViolation {
                detail: "a lifecycle event requires a recorded_at timestamp".into(),
            });
        }
        let dir = self.job_dir(job);
        std::fs::create_dir_all(&dir)?;
        for _ in 0..MAX_RACES {
            let history = self.events(job)?;
            let from = history.last().map_or(JobState::Draft, |r| r.to);
            let to = transition(from, event).map_err(|e| PersistenceError::LifecycleViolation {
                detail: e.to_string(),
            })?;
            let record = LifecycleRecord {
                schema_version: SCHEMA_VERSION,
                job_id: job,
                sequence: history.len() as u32 + 1,
                event,
                from,
                to,
                recorded_at: recorded_at.to_string(),
                note: note.to_string(),
                attempt_id,
            };
            if write_exclusive(&dir, &record)? {
                return Ok(record);
            }
        }
        Err(PersistenceError::LifecycleViolation {
            detail: "the job kept changing while the event was being recorded".into(),
        })
    }
}

/// Returns false when another writer already took this sequence number.
fn write_exclusive(dir: &Path, record: &LifecycleRecord) -> PersistenceResult<bool> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let name = format!("{:08}.json", record.sequence);
    let tmp = dir.join(format!(
        "{name}.{}-{}.tmp",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(PersistenceError::from_io)?;
        file.write_all(&serde_json::to_vec(record)?)
            .map_err(PersistenceError::from_io)?;
        file.sync_all().map_err(PersistenceError::from_io)?;
    }
    let linked = std::fs::hard_link(&tmp, dir.join(&name));
    let _ = std::fs::remove_file(&tmp);
    match linked {
        Ok(()) => {
            sync_dir(dir)?;
            Ok(true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(PersistenceError::from_io(e)),
    }
}

/// Several processes may open a fresh store at once; losing the race to write `VERSION` is fine
/// as long as the winner wrote the version this build expects.
fn write_version(root: &Path) -> PersistenceResult<()> {
    match crate::jobs::atomic_write(&root.join("VERSION"), SCHEMA_VERSION.to_string().as_bytes()) {
        Ok(()) => Ok(()),
        Err(e) => match read_schema_version(root) {
            Ok(SCHEMA_VERSION) => Ok(()),
            _ => Err(e),
        },
    }
}

fn corrupt(job: JobId, n: u32, why: &str) -> PersistenceError {
    PersistenceError::Corrupt {
        detail: format!("lifecycle {job} event {n}: {why}"),
    }
}
