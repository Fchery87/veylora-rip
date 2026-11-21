//! Durable job lifecycle and the submission service. Every step is recorded before the next one
//! is taken, nothing is ever retried automatically, and an outcome that cannot be recorded leaves
//! the job open for operator review instead of guessing.
use crate::{
    error::{WorkflowError, WorkflowResult},
    workspace::Workspace,
};
use serde::Serialize;
use veylora_device::mode::{check_compatibility, ModeStatus};
use veylora_domain::{
    errors::ErrorCode,
    ids::{AttemptId, JobId},
    job::{AttemptState, SubmissionAttempt},
    lifecycle::{JobState, LifecycleEvent},
    preflight::{Finding, Severity},
    provenance::ContentHash,
    recipe::RecipeStatus,
};
use veylora_persistence::{JobStore, LifecycleRecord};
use veylora_transport::{
    PrinterArtifact, PrinterTransport, SubmissionObserver, TransportError, TransportOutcome,
    TransportResult, TransportStatus,
};

#[derive(Debug, Clone, Serialize)]
pub struct SubmitOutcome {
    pub attempt_id: AttemptId,
    pub job_state: JobState,
    pub attempt_state: AttemptState,
    pub bytes_written: u64,
    pub artifact_hash: ContentHash,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecoveryReport {
    pub before: JobState,
    pub after: JobState,
}

/// What the operator found out about a submission the system could not settle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewResolution {
    /// The transfer was printed. Recorded on the attempt and the job.
    ConfirmedFinished,
    /// Nothing usable was printed. The job is cancelled; a new print is a new job.
    NotPrinted,
}

/// UTC time as `YYYY-MM-DDThh:mm:ssZ`.
pub(crate) fn utc_now() -> String {
    format_utc(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() as i64),
    )
}

fn format_utc(secs: i64) -> String {
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3_600,
        rem % 3_600 / 60,
        rem % 60
    )
}

struct ByteCount(u64);
impl SubmissionObserver for ByteCount {
    fn on_bytes(&mut self, count: u64) {
        self.0 = self.0.max(count);
    }
    fn on_status(&mut self, _status: TransportStatus) {}
}

/// What a transport result means for the record. Anything short of proof that nothing was sent,
/// or proof that everything was, is ambiguous.
fn classify(
    result: &Result<TransportResult, TransportError>,
    intended: u64,
    observed: u64,
) -> (LifecycleEvent, AttemptState, u64) {
    use AttemptState as A;
    use LifecycleEvent as E;
    let nothing_sent = (E::SubmissionRejectedBeforeWrite, A::Failed, 0);
    match result {
        Ok(r) => match r.outcome {
            TransportOutcome::Submitted if r.bytes_written == intended => {
                (E::SubmissionAccepted, A::Submitted, r.bytes_written)
            }
            TransportOutcome::Failed if r.bytes_written == 0 => nothing_sent,
            TransportOutcome::Cancelled if r.bytes_written == 0 => {
                (E::SubmissionRejectedBeforeWrite, A::Cancelled, 0)
            }
            _ => (E::SubmissionUncertain, A::Ambiguous, r.bytes_written),
        },
        Err(TransportError::BeforeWrite) => nothing_sent,
        Err(TransportError::Cancelled {
            bytes_written: 0, ..
        }) => (E::SubmissionRejectedBeforeWrite, A::Cancelled, 0),
        Err(TransportError::DuringWrite { bytes_written, .. })
        | Err(TransportError::Ambiguous { bytes_written, .. })
        | Err(TransportError::Cancelled { bytes_written, .. }) => {
            (E::SubmissionUncertain, A::Ambiguous, *bytes_written)
        }
        Err(TransportError::Io(_)) => (E::SubmissionUncertain, A::Ambiguous, observed),
    }
}

impl Workspace {
    pub fn state(&self, id: JobId) -> WorkflowResult<JobState> {
        self.jobs.get(id)?;
        Ok(self.lifecycle.state(id)?)
    }

    pub fn lifecycle_events(&self, id: JobId) -> WorkflowResult<Vec<LifecycleRecord>> {
        self.jobs.get(id)?;
        Ok(self.lifecycle.events(id)?)
    }

    pub(crate) fn record(
        &self,
        id: JobId,
        event: LifecycleEvent,
        note: &str,
        attempt: Option<AttemptId>,
    ) -> WorkflowResult<JobState> {
        Ok(self
            .lifecycle
            .append(id, event, &utc_now(), note, attempt)?
            .to)
    }

    /// Records the result of a preflight run, only when it changes what is known.
    pub(crate) fn note_preflight(&self, id: JobId, blocked: bool) -> WorkflowResult<()> {
        match (self.lifecycle.state(id)?, blocked) {
            (JobState::Draft, true) => {
                self.record(id, LifecycleEvent::PreflightRejected, "", None)?;
            }
            (JobState::Draft | JobState::PreflightFailed, false) => {
                self.record(id, LifecycleEvent::PreflightPassed, "", None)?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn hold(&self, id: JobId) -> WorkflowResult<JobState> {
        self.jobs.get(id)?;
        self.record(id, LifecycleEvent::Hold, "held by the operator", None)
    }

    pub fn cancel(&self, id: JobId) -> WorkflowResult<JobState> {
        self.jobs.get(id)?;
        self.record(
            id,
            LifecycleEvent::Cancel,
            "cancelled by the operator",
            None,
        )
    }

    fn append_attempt(
        &self,
        id: JobId,
        snapshot_hash: ContentHash,
        attempt_id: AttemptId,
        state: AttemptState,
        bytes_written: u64,
        note: String,
    ) -> WorkflowResult<()> {
        // Event sequence numbers are shared by every attempt of the job.
        let sequence = self
            .jobs
            .list_attempts(id)?
            .iter()
            .map(|a| a.sequence)
            .max()
            .unwrap_or(0)
            + 1;
        Ok(self.jobs.append_attempt(SubmissionAttempt {
            attempt_id,
            job_id: id,
            snapshot_hash,
            sequence,
            state,
            bytes_written,
            note,
            recorded_at: utc_now(),
        })?)
    }

    /// Findings that stop a submission. A diagnostic render needs none of these; a printer does.
    fn submission_findings(&self, id: JobId) -> WorkflowResult<Vec<Finding>> {
        let loaded = self.load(id)?;
        let blocking = |code, subject: &str, message: &str| Finding {
            code,
            severity: Severity::Blocking,
            message: message.to_string(),
            subject: Some(subject.to_string()),
        };
        let mut out = Vec::new();
        if loaded.setup.require_production().is_err() {
            out.push(blocking(
                ErrorCode::UnverifiedDeviceSetup,
                "device_setup",
                "Submission needs a device setup verified with physical channel evidence.",
            ));
        }
        if loaded.mode.status() != ModeStatus::Validated {
            out.push(blocking(
                ErrorCode::UnverifiedDeviceSetup,
                "print_mode",
                "Submission needs a print mode validated with physical evidence.",
            ));
        }
        let recipe = &loaded.snapshot.inputs().recipe;
        match (&recipe.inputs().status, &recipe.inputs().validation) {
            (RecipeStatus::Retired, _) => out.push(blocking(
                ErrorCode::IncompatibleRecipe,
                "recipe",
                "The recipe is retired.",
            )),
            (RecipeStatus::Validated, Some(v)) => {
                // The evidence belongs to the conditions it was measured under, not to any other.
                if v.evidence.setup_hash != loaded.setup.content_hash()
                    || v.evidence.mode_hash != loaded.mode.content_hash()
                {
                    out.push(blocking(
                        ErrorCode::IncompatibleRecipe,
                        "recipe",
                        "The recipe was validated with a different setup or print mode than \
                         this job uses.",
                    ));
                }
            }
            _ => out.push(blocking(
                ErrorCode::IncompatibleRecipe,
                "recipe",
                "Submission needs a recipe validated with physical evidence.",
            )),
        }
        out.extend(
            check_compatibility(
                &loaded.mode,
                &loaded.setup,
                &crate::workspace::diagnostic_capability(&loaded.setup),
                recipe,
            )
            .into_iter()
            .filter(|f| f.severity.is_blocking()),
        );
        Ok(out)
    }

    /// Sends exact encoder output through `transport` as one new attempt.
    ///
    /// The job must be `Ready` and its setup, mode and recipe must pass the submission gate. The
    /// attempt is recorded before any byte is written, and the outcome is recorded after. Only
    /// proof that nothing was sent produces a clean failure; every other uncertainty becomes
    /// `Ambiguous`, which no later call retries.
    pub fn submit(
        &self,
        id: JobId,
        artifact: &PrinterArtifact,
        transport: &dyn PrinterTransport,
    ) -> WorkflowResult<SubmitOutcome> {
        let _claim = self.try_lock_job(id)?;
        let state = self.state(id)?;
        if state != JobState::Ready {
            return Err(WorkflowError::Invalid(format!(
                "A job that is {state:?} cannot be submitted; only a Ready job can."
            )));
        }
        let findings = self.submission_findings(id)?;
        if !findings.is_empty() {
            return Err(WorkflowError::PreflightBlocked(findings));
        }
        let snapshot_hash = self.jobs.get(id)?.content_hash();
        let attempt_id = AttemptId::new();
        let hash = artifact.hash();
        let note = format!("artifact {hash}, {} bytes", artifact.len());

        // The job moves first, so a crash anywhere after this leaves a state that recovery treats
        // as uncertain rather than a Ready job that may already have been sent.
        self.record(id, LifecycleEvent::BeginSubmission, &note, Some(attempt_id))?;
        if let Err(e) = self.append_attempt(
            id,
            snapshot_hash,
            attempt_id,
            AttemptState::Submitting,
            0,
            note.clone(),
        ) {
            // Nothing was sent, so this is a clean failure.
            let _ = self.record(
                id,
                LifecycleEvent::SubmissionRejectedBeforeWrite,
                "the attempt could not be recorded; nothing was sent",
                Some(attempt_id),
            );
            return Err(e);
        }

        let mut observed = ByteCount(0);
        let result = transport.submit(artifact, &mut observed);
        let (event, attempt_state, bytes) = classify(&result, artifact.len(), observed.0);
        let detail = match &result {
            Ok(r) => format!("{:?}", r.outcome),
            Err(e) => e.to_string(),
        };
        let saved = self
            .append_attempt(
                id,
                snapshot_hash,
                attempt_id,
                attempt_state,
                bytes,
                format!("{note}; {detail}"),
            )
            .and_then(|()| self.record(id, event, &detail, Some(attempt_id)));
        let job_state = saved.map_err(|e| {
            WorkflowError::OutcomeNotRecorded(format!(
                "the transport reported {detail}, but recording it failed ({e}); run recover"
            ))
        })?;
        Ok(SubmitOutcome {
            attempt_id,
            job_state,
            attempt_state,
            bytes_written: bytes,
            artifact_hash: hash,
        })
    }

    /// Settles a job a crash left in the middle of an operation. An interrupted render becomes a
    /// failure that can be retried. An interrupted submission is completed from whatever the
    /// attempt log proves, and otherwise becomes `Review`: the printer may have the data. It refuses a job
    /// another process is working on.
    pub fn recover(&self, id: JobId) -> WorkflowResult<RecoveryReport> {
        // Refused while another process holds the job; a crashed one has already let go.
        let _claim = self.try_lock_job(id)?;
        let before = self.state(id)?;
        match before {
            JobState::Rendering => {
                self.record(
                    id,
                    LifecycleEvent::RenderFailed,
                    "interrupted; recovered",
                    None,
                )?;
            }
            JobState::Submitting => self.recover_submission(id)?,
            _ => {}
        }
        Ok(RecoveryReport {
            before,
            after: self.state(id)?,
        })
    }

    fn recover_submission(&self, id: JobId) -> WorkflowResult<()> {
        use AttemptState as A;
        use LifecycleEvent as E;
        let attempt_id = self
            .lifecycle
            .events(id)?
            .last()
            .and_then(|r| r.attempt_id)
            .ok_or_else(|| {
                WorkflowError::Invalid("The open submission names no attempt.".into())
            })?;
        let latest = self
            .jobs
            .list_attempts(id)?
            .into_iter()
            .find(|a| a.attempt_id == attempt_id);
        let note = "recovered after an interruption";
        match latest.as_ref().map(|a| (a.state, a.bytes_written)) {
            Some((A::Submitted, _)) => {
                self.record(id, E::SubmissionAccepted, note, Some(attempt_id))?;
            }
            Some((A::Failed | A::Cancelled, 0)) => {
                self.record(id, E::SubmissionRejectedBeforeWrite, note, Some(attempt_id))?;
            }
            other => {
                // The attempt is open, missing or already uncertain. Close it as uncertain.
                if matches!(other, Some((A::Submitting, _))) {
                    let a = latest.as_ref().expect("matched above");
                    self.append_attempt(
                        id,
                        a.snapshot_hash,
                        attempt_id,
                        A::Ambiguous,
                        a.bytes_written,
                        note.to_string(),
                    )?;
                }
                self.record(id, E::SubmissionUncertain, note, Some(attempt_id))?;
                self.record(
                    id,
                    E::RequestReview,
                    "operator review required",
                    Some(attempt_id),
                )?;
            }
        }
        Ok(())
    }

    /// Records the operator's decision about a submission that could not be settled
    /// automatically. The operator and a note about what was checked are required.
    pub fn resolve_review(
        &self,
        id: JobId,
        resolution: ReviewResolution,
        operator: &str,
        note: &str,
    ) -> WorkflowResult<JobState> {
        use LifecycleEvent as E;
        if operator.trim().is_empty() || note.trim().is_empty() {
            return Err(WorkflowError::Invalid(
                "A review decision needs the operator's name and a note about what was checked."
                    .into(),
            ));
        }
        let state = self.state(id)?;
        if !matches!(
            state,
            JobState::Ambiguous | JobState::Review | JobState::Submitted
        ) || (state == JobState::Submitted && resolution == ReviewResolution::NotPrinted)
        {
            return Err(WorkflowError::Invalid(format!(
                "There is nothing to resolve that way while the job is {state:?}."
            )));
        }
        let attempt_id = self
            .lifecycle
            .events(id)?
            .iter()
            .rev()
            .find_map(|r| r.attempt_id);
        let text = format!("{operator}: {note}");
        if state == JobState::Ambiguous {
            self.record(id, E::RequestReview, "operator review opened", attempt_id)?;
        }
        match resolution {
            ReviewResolution::ConfirmedFinished => {
                // Only an uncertain attempt can take a follow-up; a submitted one is already
                // settled and the confirmation lives on the job.
                let latest = self
                    .jobs
                    .list_attempts(id)?
                    .into_iter()
                    .find(|a| Some(a.attempt_id) == attempt_id);
                if let Some(a) = latest.filter(|a| a.state.is_ambiguous()) {
                    self.append_attempt(
                        id,
                        a.snapshot_hash,
                        a.attempt_id,
                        AttemptState::OperatorConfirmedFinished,
                        a.bytes_written,
                        text.clone(),
                    )?;
                }
                self.record(id, E::OperatorConfirmFinished, &text, attempt_id)
            }
            ReviewResolution::NotPrinted => self.record(id, E::OperatorCancel, &text, attempt_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::format_utc;

    #[test]
    fn formats_known_instants() {
        assert_eq!(format_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_utc(1_735_000_000), "2024-12-24T00:26:40Z");
        // Leap days and the end of a year.
        assert_eq!(format_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(format_utc(1_709_164_800), "2024-02-29T00:00:00Z");
        assert_eq!(format_utc(1_767_225_599), "2025-12-31T23:59:59Z");
        assert_eq!(format_utc(4_102_444_800), "2100-01-01T00:00:00Z");
    }
}
