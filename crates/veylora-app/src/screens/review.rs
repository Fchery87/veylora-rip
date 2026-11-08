//! Ambiguous-submission review presentation (design 3.x, 7.2).
//!
//! An ambiguous attempt is never retried automatically: the operator records
//! what happened, and a reprint is a new held job state, not a send.

use crate::view_models::{
    Attempt, JobDetail, ReviewAttemptView, ReviewChoiceView, ReviewOutcome, ReviewView,
};

fn choice(outcome: ReviewOutcome, label: &str, confirm: &str, effect: &str) -> ReviewChoiceView {
    ReviewChoiceView {
        outcome,
        label: label.to_owned(),
        confirm_label: confirm.to_owned(),
        effect: effect.to_owned(),
    }
}

fn choices() -> Vec<ReviewChoiceView> {
    vec![
        choice(
            ReviewOutcome::Finished,
            "It printed correctly",
            "Mark finished",
            "The attempt will be recorded as finished by the operator. No new attempt is created.",
        ),
        choice(
            ReviewOutcome::Reprint,
            "Nothing printed, reprint",
            "Create reprint attempt",
            "The attempt will be recorded as not printed and a reprint attempt will be created held.",
        ),
        choice(
            ReviewOutcome::PrintedBadly,
            "Printed badly, reprint",
            "Create reprint attempt",
            "The attempt will be recorded as printed badly and a reprint attempt will be created held.",
        ),
        choice(
            ReviewOutcome::DecideLater,
            "Decide later",
            "Keep in review",
            "The attempt stays in review. The printer remains blocked until it is resolved.",
        ),
    ]
}

/// The attempt waiting for an operator outcome, if any.
pub fn open_attempt(job: &JobDetail) -> Option<&Attempt> {
    job.attempts.iter().find(|a| a.outcome.is_open_review())
}

pub fn present(job: &JobDetail) -> Option<ReviewView> {
    let attempt = open_attempt(job)?;
    Some(ReviewView {
        headline: "Check the printer before printing anything else".to_owned(),
        body: format!(
            "Attempt {} ended without a confirmed result. It may or may not have printed. Nothing can be sent to this printer until you record what happened.",
            attempt.id
        ),
        attempt: ReviewAttemptView {
            id: attempt.id.clone(),
            last_signal: attempt.at.clone(),
            source: "Fixture data".to_owned(),
        },
        choices: choices(),
        printer_blocked: true,
    })
}
