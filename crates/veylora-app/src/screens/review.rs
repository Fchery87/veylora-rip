//! Ambiguous-submission review presentation (design 3.x, 7.2).
//!
//! An ambiguous attempt is never retried automatically: the operator records
//! what happened, and a reprint is a new held job state, not a send.

use crate::state::CapabilityState;
use crate::view_models::{
    Attempt, JobDetail, ReviewAttemptView, ReviewChoiceView, ReviewFact, ReviewOutcome, ReviewView,
    WhatWasSent,
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

const NOT_RECORDED: &str = "Not recorded";

fn recorded<T: ToString>(value: Option<&T>) -> String {
    value.map_or_else(|| NOT_RECORDED.to_owned(), ToString::to_string)
}

fn fact(label: &str, value: String) -> ReviewFact {
    ReviewFact {
        label: label.to_owned(),
        value,
    }
}

fn facts(caps: &CapabilityState, attempt: &Attempt) -> Vec<ReviewFact> {
    let bytes = match (attempt.bytes_written, attempt.bytes_intended) {
        (Some(written), Some(intended)) => format!("{written} / {intended}"),
        _ => NOT_RECORDED.to_owned(),
    };
    vec![
        fact("Bytes written / intended", bytes),
        fact("Spool job ID", recorded(attempt.spool_job_id.as_ref())),
        fact("Last signal", attempt.at.clone()),
        fact("Source", caps.data_source.label().to_owned()),
    ]
}

fn what_was_sent(job: &JobDetail, attempt: &Attempt) -> WhatWasSent {
    let snapshot_hash = job
        .provenance
        .iter()
        .find(|row| row.label == "Snapshot hash")
        .map_or_else(|| NOT_RECORDED.to_owned(), |row| row.value.clone());
    WhatWasSent {
        artifact_hash: recorded(attempt.artifact_hash.as_ref()),
        artifact_size: attempt
            .bytes_intended
            .map_or_else(|| NOT_RECORDED.to_owned(), |bytes| format!("{bytes} bytes")),
        snapshot_hash,
        placements: job
            .placements
            .iter()
            .map(|p| {
                format!(
                    "{} \u{b7} {}, {} mm \u{b7} {} x {} mm",
                    p.name, p.x_mm, p.y_mm, p.width_mm, p.height_mm
                )
            })
            .collect(),
    }
}

pub fn present(caps: &CapabilityState, job: &JobDetail) -> Option<ReviewView> {
    let attempt = open_attempt(job)?;
    Some(ReviewView {
        headline: "Check the printer before printing anything else".to_owned(),
        body: format!(
            "Attempt {} ended without a confirmed result. It may or may not have printed. Nothing can be sent to this printer until you record what happened.",
            attempt.id
        ),
        attempt: ReviewAttemptView {
            id: attempt.id.clone(),
            source: caps.data_source,
            source_label: caps.data_source.label().to_owned(),
        },
        facts: facts(caps, attempt),
        what_was_sent: what_was_sent(job, attempt),
        choices: choices(),
        printer_blocked: true,
    })
}
