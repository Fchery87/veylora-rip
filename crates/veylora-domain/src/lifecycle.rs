//! Pure lifecycle transitions. Callers must durably record events and attempts.
//! This state machine neither verifies physical evidence nor authorizes transport.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Draft,
    PreflightFailed,
    Held,
    Rendering,
    /// Artifacts are ready for inspection, not necessarily printer-ready.
    Ready,
    Submitting,
    /// Transport accepted output. Physical completion remains unconfirmed.
    Submitted,
    Ambiguous,
    Review,
    Cancelled,
    Failed,
    OperatorConfirmedFinished,
}

/// Facts/decisions supplied by application services, not raw UI commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleEvent {
    PreflightRejected,
    PreflightPassed,
    BeginRender,
    RenderCompleted,
    RenderFailed,
    Hold,
    Release,
    BeginSubmission,
    SubmissionAccepted,
    SubmissionUncertain,
    /// Use only when the transport proves that no bytes were submitted.
    SubmissionRejectedBeforeWrite,
    RequestReview,
    OperatorConfirmFinished,
    /// Operator resolves a reviewed attempt without claiming successful output.
    OperatorCancel,
    /// Acknowledged cancellation, not merely a request to a running worker.
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LifecycleError {
    pub current: JobState,
    pub event: LifecycleEvent,
}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot apply {:?} while job is {:?}.",
            self.event, self.current
        )
    }
}
impl std::error::Error for LifecycleError {}

/// No retry edge exists after submission. An operator-requested reprint must
/// create a separate append-only attempt through application services. Releasing
/// a hold re-renders because Held does not assert retained-artifact validity. A failed job can be
/// released back to Held by an operator; that never applies to an ambiguous or reviewed submission.
pub fn transition(current: JobState, event: LifecycleEvent) -> Result<JobState, LifecycleError> {
    use JobState::*;
    use LifecycleEvent::*;
    let next = match (current, event) {
        (Draft | PreflightFailed, PreflightRejected) => PreflightFailed,
        (Draft | PreflightFailed, PreflightPassed) => Held,
        (Held, BeginRender | Release) => Rendering,
        // An operator may release a failed job to try again. Earlier attempts stay on record.
        (Failed, Release) => Held,
        (Rendering, RenderCompleted) => Ready,
        (Rendering, RenderFailed) => Failed,
        (Ready, Hold) => Held,
        (Ready, BeginSubmission) => Submitting,
        (Submitting, SubmissionAccepted) => Submitted,
        (Submitting, SubmissionUncertain | Cancel) => Ambiguous,
        (Submitting, SubmissionRejectedBeforeWrite) => Failed,
        (Ambiguous | Submitted, RequestReview) => Review,
        (Submitted | Review, OperatorConfirmFinished) => OperatorConfirmedFinished,
        (Review, OperatorCancel) => Cancelled,
        (Draft | PreflightFailed | Held | Rendering | Ready, Cancel) => Cancelled,
        _ => return Err(LifecycleError { current, event }),
    };
    Ok(next)
}
