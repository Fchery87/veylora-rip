//! One claim at a time on a job while something is being done to it. `render`, `submit` and
//! `recover` each hold the claim for their whole run, so `recover` can tell a job that is being
//! worked on from one that a crash left open.
//!
//! The claim is an operating-system lock on a per-job file. The system releases it when the
//! holder exits for any reason, including a crash, so a crashed process never leaves a claim that
//! blocks the recovery meant to clean up after it. A refused claim changes nothing.
use crate::{
    error::{WorkflowError, WorkflowResult},
    workspace::Workspace,
};
use std::fs::{File, OpenOptions, TryLockError};
use veylora_domain::ids::JobId;

/// A claim on one job. Dropping it releases the claim.
#[derive(Debug)]
pub struct JobLock {
    _file: File,
}

impl Workspace {
    /// Claims the job for the caller, or refuses at once if something else holds it. Never waits.
    pub fn try_lock_job(&self, id: JobId) -> WorkflowResult<JobLock> {
        let dir = self.root.join("state").join("locks");
        std::fs::create_dir_all(&dir)?;
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(dir.join(format!("{id}.lock")))?;
        match file.try_lock() {
            Ok(()) => Ok(JobLock { _file: file }),
            Err(TryLockError::WouldBlock) => Err(WorkflowError::Invalid(
                "Another process is working on this job (rendering, submitting or recovering it). \
                 Wait for it to finish and try again; if it has stopped, its claim is already released."
                    .into(),
            )),
            Err(TryLockError::Error(e)) => Err(WorkflowError::Io(e)),
        }
    }
}
