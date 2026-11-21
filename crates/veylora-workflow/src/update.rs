//! What an application update must not interrupt.

use crate::{error::WorkflowResult, workspace::Workspace};
use serde::Serialize;
use veylora_domain::{ids::JobId, lifecycle::JobState};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UpdateBlocker {
    pub job_id: JobId,
    pub state: JobState,
    pub reason: &'static str,
}

impl Workspace {
    /// Jobs whose work an update would cut off: a submission whose outcome is not yet recorded
    /// (printer data may be in flight) and a render in progress. Ambiguous and review jobs do not
    /// block: their state is durable and waits for the operator across a restart.
    ///
    /// Any job whose state cannot be read is an error, not an assumption that it is idle.
    pub fn update_blockers(&self) -> WorkflowResult<Vec<UpdateBlocker>> {
        let dir = self.root.join("state").join("lifecycle");
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(e.into()),
        };
        let mut blockers = Vec::new();
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let id: JobId = name.parse().map_err(|_| {
                crate::error::WorkflowError::Invalid(format!(
                    "unexpected entry {name:?} in the lifecycle store"
                ))
            })?;
            let (state, reason) = match self.lifecycle.state(id)? {
                s @ JobState::Submitting => (
                    s,
                    "a submission is in progress and printer data may be in flight",
                ),
                s @ JobState::Rendering => (s, "a render is in progress"),
                _ => continue,
            };
            blockers.push(UpdateBlocker {
                job_id: id,
                state,
                reason,
            });
        }
        blockers.sort_by_key(|b| b.job_id);
        Ok(blockers)
    }
}

impl Workspace {
    /// The data schema version on disk, which an update must be able to read.
    pub fn data_schema_version(&self) -> WorkflowResult<u32> {
        Ok(self.jobs.schema_version()?)
    }
}
