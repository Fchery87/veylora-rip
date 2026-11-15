//! Setup creation and verification, and read-only job review. Nothing here talks to a printer.
use crate::{
    error::{WorkflowError, WorkflowResult},
    workspace::Workspace,
};
use serde::{Deserialize, Serialize};
use veylora_device::setup::{
    DeviceSetup, EvidenceRecord, SetupDetails, SetupOrigin, SetupRecord, VerificationState,
};
use veylora_domain::{
    ids::{DeviceSetupId, JobId},
    job::AttemptState,
    provenance::ContentHash,
};
use veylora_persistence::JobStore;

/// An operator-authored setup. Every field is stated by the operator; nothing is inferred.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetupDraft {
    /// Generated when absent.
    pub id: Option<DeviceSetupId>,
    pub origin: SetupOrigin,
    pub details: SetupDetails,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SetupStatus {
    pub id: DeviceSetupId,
    pub revision: u32,
    pub content_hash: ContentHash,
    pub verification: VerificationState,
}

impl SetupStatus {
    fn of(setup: &DeviceSetup) -> Self {
        Self {
            id: setup.id(),
            revision: setup.revision(),
            content_hash: setup.content_hash(),
            verification: setup.verification().clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AttemptReview {
    pub attempt_id: String,
    pub sequence: u32,
    pub state: String,
    pub bytes_written: u64,
    pub note: String,
    pub recorded_at: String,
    pub requires_review: bool,
    pub reason: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewReport {
    pub job_id: JobId,
    pub requires_review: bool,
    pub attempts: Vec<AttemptReview>,
    /// What this command does not do.
    pub note: &'static str,
}

fn invalid(f: veylora_domain::preflight::Finding) -> WorkflowError {
    WorkflowError::Invalid(f.message)
}

impl Workspace {
    /// A new setup is always unverified. Verification needs physical evidence.
    pub fn create_setup(&self, draft: &SetupDraft) -> WorkflowResult<SetupStatus> {
        let setup = DeviceSetup::new(
            draft.id.unwrap_or_default(),
            draft.details.clone(),
            draft.origin,
        )
        .map_err(invalid)?;
        self.devices.put_setup(&setup)?;
        Ok(SetupStatus::of(&setup))
    }

    pub fn setup_record(&self, hash: &ContentHash) -> WorkflowResult<SetupRecord> {
        Ok(self.devices.get_setup(hash)?.to_record())
    }

    pub fn setup_history(&self, hash: &ContentHash) -> WorkflowResult<Vec<VerificationState>> {
        Ok(self.devices.setup_states(hash)?)
    }

    /// Applies operator-supplied evidence. The setup's own checks decide; a refusal stores
    /// nothing, and an accepted result is appended as a new state, never a rewrite.
    pub fn verify_setup(
        &self,
        hash: &ContentHash,
        evidence: EvidenceRecord,
    ) -> WorkflowResult<SetupStatus> {
        let mut setup = self.devices.get_setup(hash)?;
        setup.verify(evidence).map_err(invalid)?;
        self.devices.put_setup(&setup)?;
        Ok(SetupStatus::of(&setup))
    }

    /// Read-only. An attempt still open or ambiguous needs an operator decision before anything
    /// else may happen to the job; recording that decision belongs to the durable lifecycle.
    pub fn review(&self, id: JobId) -> WorkflowResult<ReviewReport> {
        self.jobs.get(id)?;
        let attempts: Vec<AttemptReview> = self
            .jobs
            .list_attempts(id)?
            .into_iter()
            .map(|a| {
                let reason = match a.state {
                    AttemptState::Ambiguous => {
                        Some("The outcome is unknown; it must not be retried automatically.")
                    }
                    AttemptState::Submitting => {
                        Some("The attempt never recorded an outcome; it may have been interrupted.")
                    }
                    _ => None,
                };
                AttemptReview {
                    attempt_id: a.attempt_id.to_string(),
                    sequence: a.sequence,
                    state: format!("{:?}", a.state),
                    bytes_written: a.bytes_written,
                    note: a.note,
                    recorded_at: a.recorded_at,
                    requires_review: reason.is_some(),
                    reason,
                }
            })
            .collect();
        Ok(ReviewReport {
            job_id: id,
            requires_review: attempts.iter().any(|a| a.requires_review),
            attempts,
            note: "Read only. Recording an operator decision is not available yet.",
        })
    }
}
