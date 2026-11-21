//! Attempt history and operator review over a real workspace, with a fake transport. Nothing here
//! touches a printer; the fake only stands in for one.
#[path = "../../veylora-workflow/tests/common.rs"]
mod common;

use common::*;
use std::collections::BTreeMap;
use veylora_app_lib::live;
use veylora_app_lib::view_models::{AttemptKind, AttemptOutcome, JobState, ReviewOutcome};
use veylora_domain::layout::Rotation;
use veylora_transport::{
    fake::{FailurePoint, FakeTransport},
    PrinterArtifact,
};
use veylora_workflow::{JobSpec, LayoutSpec, PlacementSpec, RenderOptions, Workspace};

fn ready_job() -> (Workspace, veylora_domain::ids::JobId) {
    let ws = Workspace::open(dir("live-rev")).unwrap();
    let s = setup();
    let mut m = mode(s.id());
    m.mark_validated().unwrap();
    let setup_hash = ws.import_setup(&s.to_record()).unwrap();
    ws.verify_setup(&setup_hash, evidence_for_record(&s.to_record()))
        .unwrap();
    let spec = JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1_735_000_000_000,
        setup: setup_hash,
        mode: ws.import_mode(&m.to_record()).unwrap(),
        recipe: ws.import_recipe(&recipe()).unwrap(),
        assets: BTreeMap::from([(
            "art".to_string(),
            ws.import_asset(&fixture("alpha-ramp.png")).unwrap(),
        )]),
        masks: BTreeMap::new(),
        layout: LayoutSpec {
            page_width_um: 80,
            page_height_um: 20,
            mirror: false,
            placements: vec![PlacementSpec {
                asset: "art".into(),
                mask: None,
                x_um: 0,
                y_um: 0,
                width_um: 80,
                height_um: 20,
                rotation: Rotation::Deg0,
                crop: None,
            }],
        },
    };
    let id = ws.create_job(&spec).unwrap().job_id;
    ws.render(id, &RenderOptions::default()).unwrap();
    (ws, id)
}

fn submit(ws: &Workspace, id: veylora_domain::ids::JobId, f: FailurePoint) {
    let artifact = PrinterArtifact::from_encoded_bytes(vec![7u8; 4096]);
    let _ = ws.submit(id, &artifact, &FakeTransport::new(f));
}

#[test]
fn a_job_with_no_attempts_shows_none_and_offers_no_review() {
    let (ws, id) = ready_job();
    let d = live::job_detail(&ws, &id.to_string()).unwrap();
    assert!(d.attempts.is_empty());
    assert!(live::review_view(&ws, &id.to_string()).unwrap().is_none());
}

#[test]
fn an_accepted_attempt_is_not_called_finished() {
    let (ws, id) = ready_job();
    submit(&ws, id, FailurePoint::None);
    let d = live::job_detail(&ws, &id.to_string()).unwrap();
    assert_eq!(d.attempts.len(), 1);
    let a = &d.attempts[0];
    assert_eq!(a.kind, AttemptKind::Transport);
    assert_eq!(a.outcome, AttemptOutcome::Accepted);
    assert_ne!(a.outcome, AttemptOutcome::Finished);
    assert_eq!(a.bytes_written, Some(4096));
    assert!(a.note.contains("artifact"), "{}", a.note);
    assert_eq!(d.summary.state, JobState::Submitted);
    assert!(live::review_view(&ws, &id.to_string()).unwrap().is_none());
}

#[test]
fn a_rejected_before_write_attempt_is_failed_with_nothing_written() {
    let (ws, id) = ready_job();
    submit(&ws, id, FailurePoint::BeforeWrite);
    let a = &live::job_detail(&ws, &id.to_string()).unwrap().attempts[0];
    assert_eq!(a.outcome, AttemptOutcome::Failed);
    assert_eq!(a.bytes_written, Some(0));
}

#[test]
fn an_ambiguous_attempt_opens_a_review_with_truthful_choices() {
    let (ws, id) = ready_job();
    submit(&ws, id, FailurePoint::DuringWrite { bytes: 100 });
    let job = id.to_string();
    let d = live::job_detail(&ws, &job).unwrap();
    assert_eq!(d.attempts[0].outcome, AttemptOutcome::Ambiguous);
    assert_eq!(d.attempts[0].bytes_written, Some(100));
    let v = live::review_view(&ws, &job).unwrap().expect("review");
    assert!(v.printer_blocked);
    let outcomes: Vec<_> = v.choices.iter().map(|c| c.outcome).collect();
    assert_eq!(
        outcomes,
        vec![
            ReviewOutcome::Finished,
            ReviewOutcome::Reprint,
            ReviewOutcome::DecideLater
        ]
    );
    // "Printed badly" is not offered: the workflow cannot record it, and nothing is promised that it cannot do.
    assert!(!outcomes.contains(&ReviewOutcome::PrintedBadly));
    let not_printed = v
        .choices
        .iter()
        .find(|c| c.outcome == ReviewOutcome::Reprint)
        .unwrap();
    assert!(not_printed.effect.contains("cancelled") && not_printed.effect.contains("new job"));
    assert!(!not_printed.effect.contains("held"));
    let caps = live::workspace_capabilities(&ws).unwrap();
    assert_eq!(
        caps.open_review.as_ref().unwrap().attempt_id,
        d.attempts[0].id
    );
}

#[test]
fn confirming_finished_is_recorded_and_closes_the_review() {
    let (ws, id) = ready_job();
    submit(&ws, id, FailurePoint::DuringWrite { bytes: 100 });
    let job = id.to_string();
    let attempt = live::job_detail(&ws, &job).unwrap().attempts[0].id.clone();
    let caps = live::resolve_review(&ws, &attempt, ReviewOutcome::Finished).unwrap();
    assert!(caps.open_review.is_none());
    let d = live::job_detail(&ws, &job).unwrap();
    assert_eq!(d.summary.state, JobState::OperatorConfirmedFinished);
    assert!(d
        .attempts
        .iter()
        .any(|a| a.outcome == AttemptOutcome::Finished));
    assert!(live::review_view(&ws, &job).unwrap().is_none());
}

#[test]
fn nothing_printed_cancels_the_job_and_creates_no_attempt() {
    let (ws, id) = ready_job();
    submit(&ws, id, FailurePoint::DuringWrite { bytes: 100 });
    let job = id.to_string();
    let before = live::job_detail(&ws, &job).unwrap().attempts.len();
    let attempt = live::job_detail(&ws, &job).unwrap().attempts[0].id.clone();
    live::resolve_review(&ws, &attempt, ReviewOutcome::Reprint).unwrap();
    let d = live::job_detail(&ws, &job).unwrap();
    assert_eq!(d.summary.state, JobState::Cancelled);
    assert_eq!(
        d.attempts.len(),
        before,
        "a reprint would be a new job, never a new attempt"
    );
}

#[test]
fn printed_badly_is_refused_and_decide_later_changes_nothing() {
    let (ws, id) = ready_job();
    submit(&ws, id, FailurePoint::DuringWrite { bytes: 100 });
    let job = id.to_string();
    let attempt = live::job_detail(&ws, &job).unwrap().attempts[0].id.clone();
    let err = live::resolve_review(&ws, &attempt, ReviewOutcome::PrintedBadly).unwrap_err();
    assert!(err.contains("not available"), "{err}");
    let state = ws.state(id).unwrap();
    live::resolve_review(&ws, &attempt, ReviewOutcome::DecideLater).unwrap();
    assert_eq!(ws.state(id).unwrap(), state);
    assert!(live::review_view(&ws, &job).unwrap().is_some());
}

#[test]
fn resolving_an_unknown_attempt_is_an_error() {
    let (ws, _id) = ready_job();
    let err = live::resolve_review(
        &ws,
        "00000000-0000-4000-8000-000000000000",
        ReviewOutcome::Finished,
    )
    .unwrap_err();
    assert!(err.contains("No attempt"), "{err}");
}
