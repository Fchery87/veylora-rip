mod common;
use common::*;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
};
use veylora_domain::{
    errors::ErrorCode,
    ids::JobId,
    job::AttemptState,
    layout::Rotation,
    lifecycle::{JobState, LifecycleEvent as E},
};
use veylora_persistence::{FileJobStore, FileLifecycleStore, JobStore};
use veylora_transport::{
    fake::{FailurePoint, FakeTransport},
    Cancellation, PrinterArtifact, PrinterTransport, SubmissionObserver, TransportError,
    TransportResult,
};
use veylora_workflow::{
    JobSpec, LayoutSpec, PlacementSpec, RenderOptions, ReviewResolution, WorkflowError, Workspace,
};

struct Env {
    ws: Workspace,
    job: JobId,
}

/// A job that has been rendered and is ready to submit. `production` verifies the setup and
/// validates the mode, as physical evidence would.
fn env(production: bool) -> Env {
    env_with(production, production)
}

/// `verified_setup` and `validated_mode` are independent, so each gate can be tested alone.
fn env_with(verified_setup: bool, validated_mode: bool) -> Env {
    let ws = Workspace::open(dir("lc")).unwrap();
    let s = setup();
    let mut m = mode(s.id());
    if validated_mode {
        m.mark_validated().unwrap();
    }
    let setup_hash = ws.import_setup(&s.to_record()).unwrap();
    if verified_setup {
        ws.verify_setup(&setup_hash, evidence_for_record(&s.to_record()))
            .unwrap();
    }
    let spec = JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1735000000000,
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
    let job = ws.create_job(&spec).unwrap().job_id;
    Env { ws, job }
}

fn ready(production: bool) -> Env {
    let e = env(production);
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
    e
}

fn artifact() -> PrinterArtifact {
    PrinterArtifact::from_encoded_bytes(vec![7u8; 4096])
}

fn events(e: &Env) -> Vec<E> {
    e.ws.lifecycle_events(e.job)
        .unwrap()
        .into_iter()
        .map(|r| r.event)
        .collect()
}

/// Counts how often the printer would have been touched.
struct Counting<T: PrinterTransport>(T, Arc<AtomicUsize>);
impl<T: PrinterTransport> PrinterTransport for Counting<T> {
    fn submit(
        &self,
        a: &PrinterArtifact,
        o: &mut dyn SubmissionObserver,
    ) -> Result<TransportResult, TransportError> {
        self.1.fetch_add(1, Ordering::SeqCst);
        self.0.submit(a, o)
    }
}
fn counted(f: FailurePoint) -> (Counting<FakeTransport>, Arc<AtomicUsize>) {
    let n = Arc::new(AtomicUsize::new(0));
    (Counting(FakeTransport::new(f), n.clone()), n)
}

#[test]
fn a_new_job_is_a_draft_and_preflight_moves_it() {
    let e = env(false);
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Draft);
    assert!(events(&e).is_empty());
    e.ws.preflight(e.job, &Default::default()).unwrap();
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Held);
    // Asking again changes nothing.
    e.ws.preflight(e.job, &Default::default()).unwrap();
    assert_eq!(events(&e), vec![E::PreflightPassed]);
}

#[test]
fn a_blocked_preflight_is_recorded_and_stops_the_render() {
    let e = env(false);
    let tight = veylora_workflow::PreflightOptions {
        supported_page_sizes_um: vec![(100, 100)],
        ..Default::default()
    };
    let r = e.ws.preflight(e.job, &tight).unwrap();
    assert!(!r.can_render);
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::PreflightFailed);
    let opts = RenderOptions {
        layout: tight,
        ..Default::default()
    };
    assert!(matches!(
        e.ws.render(e.job, &opts),
        Err(WorkflowError::PreflightBlocked(_))
    ));
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::PreflightFailed);
    // Repeating the same rejection is not new history.
    assert_eq!(events(&e), vec![E::PreflightRejected]);
    // Once the cause is gone the job can render.
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Ready);
}

#[test]
fn rendering_walks_the_lifecycle_and_a_second_render_goes_through_hold() {
    let e = env(false);
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Ready);
    assert_eq!(
        events(&e),
        vec![E::PreflightPassed, E::BeginRender, E::RenderCompleted]
    );
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Ready);
    assert_eq!(
        &events(&e)[3..],
        [E::Hold, E::BeginRender, E::RenderCompleted]
    );
}

#[test]
fn an_unacknowledged_confirmation_does_not_fail_the_job() {
    let e = env(false);
    let strict = veylora_workflow::PreflightOptions {
        min_effective_ppi: 5_000,
        ..Default::default()
    };
    let opts = RenderOptions {
        layout: strict,
        ..Default::default()
    };
    assert!(e.ws.render(e.job, &opts).is_err());
    // Refused before rendering began, so the job is not marked failed.
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Held);
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
}

#[test]
fn a_failed_render_can_be_released_and_retried() {
    let e = env(false);
    // A file where the job's artifact folder should be, as a damaged disk might leave.
    let blocker = e.ws.root().join("artifacts").join(e.job.to_string());
    std::fs::create_dir_all(blocker.parent().unwrap()).unwrap();
    std::fs::write(&blocker, b"in the way").unwrap();
    assert!(e.ws.render(e.job, &RenderOptions::default()).is_err());
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Failed);
    std::fs::remove_file(&blocker).unwrap();
    // Nothing from the failed run is reported as a render.
    assert!(e.ws.report(e.job).unwrap().renders.is_empty());
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Ready);
    assert_eq!(e.ws.report(e.job).unwrap().renders.len(), 1);
}

#[test]
fn a_successful_submission_is_recorded_before_and_after_the_write() {
    let e = ready(true);
    let (t, calls) = counted(FailurePoint::None);
    let out = e.ws.submit(e.job, &artifact(), &t).unwrap();
    assert_eq!(out.job_state, JobState::Submitted);
    assert_eq!(out.attempt_state, AttemptState::Submitted);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let attempts = e.ws.review(e.job).unwrap().attempts;
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].bytes_written, 4096);
    assert_eq!(events(&e)[3..], [E::BeginSubmission, E::SubmissionAccepted]);
    // A submitted job is not submitted again.
    assert!(e.ws.submit(e.job, &artifact(), &t).is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn submission_needs_verified_evidence_and_touches_nothing_otherwise() {
    let e = ready(false);
    let (t, calls) = counted(FailurePoint::None);
    match e.ws.submit(e.job, &artifact(), &t) {
        Err(WorkflowError::PreflightBlocked(f)) => {
            assert!(f.iter().any(|x| x.code == ErrorCode::UnverifiedDeviceSetup));
            let subjects: Vec<_> = f.iter().filter_map(|x| x.subject.as_deref()).collect();
            assert!(subjects.contains(&"device_setup") && subjects.contains(&"print_mode"));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Ready);
    assert!(e.ws.review(e.job).unwrap().attempts.is_empty());
}

#[test]
fn a_job_that_is_not_ready_cannot_be_submitted() {
    let e = env(true);
    let (t, calls) = counted(FailurePoint::None);
    assert!(e.ws.submit(e.job, &artifact(), &t).is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn faults_are_recorded_conservatively_and_never_retried() {
    // (fault, expected job state, expected attempt state)
    let cases = [
        (
            FailurePoint::BeforeWrite,
            JobState::Failed,
            AttemptState::Failed,
        ),
        (
            FailurePoint::DuringWrite { bytes: 100 },
            JobState::Ambiguous,
            AttemptState::Ambiguous,
        ),
        (
            FailurePoint::AfterWrite,
            JobState::Ambiguous,
            AttemptState::Ambiguous,
        ),
    ];
    for (fault, job_state, attempt_state) in cases {
        let e = ready(true);
        let (t, calls) = counted(fault);
        let out = e.ws.submit(e.job, &artifact(), &t).unwrap();
        assert_eq!(
            (out.job_state, out.attempt_state),
            (job_state, attempt_state),
            "{fault:?}"
        );
        assert_eq!(e.ws.state(e.job).unwrap(), job_state);
        // The same call again does not reach the printer.
        assert!(e.ws.submit(e.job, &artifact(), &t).is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 1, "{fault:?}");
    }
}

#[test]
fn an_ambiguous_job_cannot_be_rendered_or_cancelled_around_review() {
    let e = ready(true);
    let (t, _) = counted(FailurePoint::AfterWrite);
    e.ws.submit(e.job, &artifact(), &t).unwrap();
    assert!(e.ws.render(e.job, &RenderOptions::default()).is_err());
    assert!(e.ws.cancel(e.job).is_err());
    assert!(e.ws.hold(e.job).is_err());
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Ambiguous);
    assert!(e.ws.review(e.job).unwrap().requires_review);
}

struct Flag(AtomicBool);
impl Cancellation for Flag {
    fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

#[test]
fn a_cancellation_before_any_byte_is_a_clean_failure() {
    let e = ready(true);
    let t = FakeTransport::new(FailurePoint::None)
        .with_cancellation(Arc::new(Flag(AtomicBool::new(true))));
    let out = e.ws.submit(e.job, &artifact(), &t).unwrap();
    assert_eq!(out.attempt_state, AttemptState::Cancelled);
    assert_eq!(out.job_state, JobState::Failed);
}

#[test]
fn a_failed_submission_can_be_followed_by_a_new_attempt_that_keeps_history() {
    let e = ready(true);
    let (bad, _) = counted(FailurePoint::BeforeWrite);
    e.ws.submit(e.job, &artifact(), &bad).unwrap();
    // The operator re-renders (release) and submits again as a new attempt.
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
    let (good, _) = counted(FailurePoint::None);
    let out = e.ws.submit(e.job, &artifact(), &good).unwrap();
    assert_eq!(out.job_state, JobState::Submitted);
    let attempts = e.ws.review(e.job).unwrap().attempts;
    assert_eq!(attempts.len(), 2);
    assert_ne!(attempts[0].attempt_id, attempts[1].attempt_id);
    assert_eq!(
        attempts
            .iter()
            .map(|a| a.state.as_str())
            .collect::<Vec<_>>(),
        ["Failed", "Submitted"]
    );
}

#[test]
fn a_restart_reconstructs_state_and_attempts() {
    let e = ready(true);
    let (t, _) = counted(FailurePoint::AfterWrite);
    e.ws.submit(e.job, &artifact(), &t).unwrap();
    let root = e.ws.root().to_path_buf();
    drop(e.ws);
    let ws = Workspace::open(root).unwrap();
    assert_eq!(ws.state(e.job).unwrap(), JobState::Ambiguous);
    let r = ws.review(e.job).unwrap();
    assert!(r.requires_review);
    assert_eq!(r.attempts.len(), 1);
}

/// Makes the attempt log unwritable, as a full disk would.
fn lock_attempts(e: &Env, locked: bool) {
    let path = e.ws.root().join("jobs").join("attempts.log");
    if !path.exists() {
        std::fs::write(&path, b"").unwrap();
    }
    let mut p = std::fs::metadata(&path).unwrap().permissions();
    p.set_readonly(locked);
    std::fs::set_permissions(&path, p).unwrap();
}

#[test]
fn an_attempt_that_cannot_be_recorded_never_reaches_the_printer() {
    let e = ready(true);
    lock_attempts(&e, true);
    let (t, calls) = counted(FailurePoint::None);
    let r = e.ws.submit(e.job, &artifact(), &t);
    lock_attempts(&e, false);
    assert!(r.is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    // Nothing was sent, so the job is a clean failure rather than a mystery.
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Failed);
}

/// A transport that succeeds but takes the attempt log away first, so the outcome cannot be saved.
struct Sabotage<'a>(&'a Env);
impl PrinterTransport for Sabotage<'_> {
    fn submit(
        &self,
        a: &PrinterArtifact,
        o: &mut dyn SubmissionObserver,
    ) -> Result<TransportResult, TransportError> {
        lock_attempts(self.0, true);
        FakeTransport::new(FailurePoint::None).submit(a, o)
    }
}

#[test]
fn an_unsaved_outcome_leaves_the_job_open_and_recovery_makes_it_reviewable() {
    let e = ready(true);
    let r = e.ws.submit(e.job, &artifact(), &Sabotage(&e));
    lock_attempts(&e, false);
    assert!(
        matches!(r, Err(WorkflowError::OutcomeNotRecorded(_))),
        "{r:?}"
    );
    // The printer may have the data; the record says only that submission began.
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Submitting);
    let (t, calls) = counted(FailurePoint::None);
    assert!(e.ws.submit(e.job, &artifact(), &t).is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    let report = e.ws.recover(e.job).unwrap();
    assert_eq!(
        (report.before, report.after),
        (JobState::Submitting, JobState::Review)
    );
    let review = e.ws.review(e.job).unwrap();
    assert!(review.requires_review);
    assert_eq!(review.attempts[0].state, "Ambiguous");
    // Recovery is idempotent on a job that needs nothing.
    assert_eq!(e.ws.recover(e.job).unwrap().after, JobState::Review);
}

#[test]
fn recovery_fails_an_interrupted_render_so_it_can_be_retried() {
    let e = env(false);
    let store = FileLifecycleStore::open(e.ws.root().join("state")).unwrap();
    for ev in [E::PreflightPassed, E::BeginRender] {
        store
            .append(e.job, ev, "2026-11-15T10:00:00Z", "", None)
            .unwrap();
    }
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Rendering);
    assert_eq!(e.ws.recover(e.job).unwrap().after, JobState::Failed);
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Ready);
}

#[test]
fn an_operator_resolves_an_ambiguous_submission_explicitly() {
    // Printed: the attempt and the job both record the operator's confirmation.
    let e = ready(true);
    let (t, _) = counted(FailurePoint::AfterWrite);
    e.ws.submit(e.job, &artifact(), &t).unwrap();
    assert!(e
        .ws
        .resolve_review(
            e.job,
            ReviewResolution::ConfirmedFinished,
            " ",
            "checked the film"
        )
        .is_err());
    assert!(e
        .ws
        .resolve_review(e.job, ReviewResolution::ConfirmedFinished, "op", "")
        .is_err());
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Ambiguous);
    let s =
        e.ws.resolve_review(
            e.job,
            ReviewResolution::ConfirmedFinished,
            "op",
            "checked the film",
        )
        .unwrap();
    assert_eq!(s, JobState::OperatorConfirmedFinished);
    let store = FileJobStore::open(e.ws.root().join("jobs")).unwrap();
    assert_eq!(
        store.list_attempts(e.job).unwrap()[0].state,
        AttemptState::OperatorConfirmedFinished
    );
    assert!(!e.ws.review(e.job).unwrap().requires_review);

    // Not printed: the job is cancelled, history is kept, and nothing is resubmitted.
    let e = ready(true);
    let (t, calls) = counted(FailurePoint::AfterWrite);
    e.ws.submit(e.job, &artifact(), &t).unwrap();
    let s =
        e.ws.resolve_review(e.job, ReviewResolution::NotPrinted, "op", "film was blank")
            .unwrap();
    assert_eq!(s, JobState::Cancelled);
    assert!(e.ws.submit(e.job, &artifact(), &t).is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(e.ws.review(e.job).unwrap().attempts.len(), 1);
}

#[test]
fn a_submitted_job_can_be_confirmed_finished_by_the_operator() {
    let e = ready(true);
    let (t, _) = counted(FailurePoint::None);
    e.ws.submit(e.job, &artifact(), &t).unwrap();
    let s =
        e.ws.resolve_review(
            e.job,
            ReviewResolution::ConfirmedFinished,
            "op",
            "pressed a good transfer",
        )
        .unwrap();
    assert_eq!(s, JobState::OperatorConfirmedFinished);
    // Nothing needs resolving on a job that is not awaiting review.
    assert!(e
        .ws
        .resolve_review(e.job, ReviewResolution::NotPrinted, "op", "x")
        .is_err());
}

#[test]
fn hold_and_cancel_follow_the_state_machine() {
    let e = ready(false);
    assert_eq!(e.ws.hold(e.job).unwrap(), JobState::Held);
    assert_eq!(e.ws.cancel(e.job).unwrap(), JobState::Cancelled);
    assert!(e.ws.render(e.job, &RenderOptions::default()).is_err());
    assert!(e.ws.hold(e.job).is_err());
    // Review still reads a cancelled job.
    assert!(e.ws.review(e.job).is_ok());
}

struct Scripted(std::sync::Mutex<Option<Result<TransportResult, TransportError>>>);
impl PrinterTransport for Scripted {
    fn submit(
        &self,
        _: &PrinterArtifact,
        _: &mut dyn SubmissionObserver,
    ) -> Result<TransportResult, TransportError> {
        self.0.lock().unwrap().take().unwrap()
    }
}
fn scripted(r: Result<TransportResult, TransportError>) -> Scripted {
    Scripted(std::sync::Mutex::new(Some(r)))
}
fn reported(outcome: veylora_transport::TransportOutcome, written: u64) -> TransportResult {
    TransportResult {
        attempt_id: uuid::Uuid::nil(),
        outcome,
        bytes_intended: 4096,
        bytes_written: written,
    }
}

#[test]
fn a_transports_claims_are_believed_only_when_they_prove_the_outcome() {
    use veylora_transport::TransportOutcome as O;
    // (what the transport said, expected job state, expected attempt state)
    let cases: Vec<(
        Result<TransportResult, TransportError>,
        JobState,
        AttemptState,
    )> = vec![
        (
            Ok(reported(O::Submitted, 4096)),
            JobState::Submitted,
            AttemptState::Submitted,
        ),
        // Claimed success with fewer bytes than were sent is not success.
        (
            Ok(reported(O::Submitted, 100)),
            JobState::Ambiguous,
            AttemptState::Ambiguous,
        ),
        (
            Ok(reported(O::Failed, 0)),
            JobState::Failed,
            AttemptState::Failed,
        ),
        (
            Ok(reported(O::Failed, 100)),
            JobState::Ambiguous,
            AttemptState::Ambiguous,
        ),
        (
            Ok(reported(O::Cancelled, 0)),
            JobState::Failed,
            AttemptState::Cancelled,
        ),
        (
            Ok(reported(O::Cancelled, 100)),
            JobState::Ambiguous,
            AttemptState::Ambiguous,
        ),
        (
            Ok(reported(O::Ambiguous, 4096)),
            JobState::Ambiguous,
            AttemptState::Ambiguous,
        ),
        (
            Err(TransportError::BeforeWrite),
            JobState::Failed,
            AttemptState::Failed,
        ),
        (
            Err(TransportError::Cancelled {
                bytes_written: 0,
                bytes_intended: 4096,
            }),
            JobState::Failed,
            AttemptState::Cancelled,
        ),
        (
            Err(TransportError::Cancelled {
                bytes_written: 9,
                bytes_intended: 4096,
            }),
            JobState::Ambiguous,
            AttemptState::Ambiguous,
        ),
        (
            Err(TransportError::DuringWrite {
                bytes_written: 0,
                bytes_intended: 4096,
            }),
            JobState::Ambiguous,
            AttemptState::Ambiguous,
        ),
        (
            Err(TransportError::Ambiguous {
                bytes_written: 4096,
                bytes_intended: 4096,
            }),
            JobState::Ambiguous,
            AttemptState::Ambiguous,
        ),
        (
            Err(TransportError::Io(std::io::Error::other("unplugged"))),
            JobState::Ambiguous,
            AttemptState::Ambiguous,
        ),
    ];
    for (i, (said, job_state, attempt_state)) in cases.into_iter().enumerate() {
        let e = ready(true);
        let out = e.ws.submit(e.job, &artifact(), &scripted(said)).unwrap();
        assert_eq!(
            (out.job_state, out.attempt_state),
            (job_state, attempt_state),
            "case {i}"
        );
    }
}

#[test]
fn a_retired_recipe_cannot_be_submitted() {
    let e = ready(true);
    // A second job on the same setup and mode, with a retired recipe.
    let mut r = recipe().inputs().clone();
    r.status = veylora_domain::recipe::RecipeStatus::Retired;
    let retired = veylora_domain::recipe::PrintRecipe::new(r).unwrap();
    let spec = JobSpec {
        recipe: e.ws.import_recipe(&retired).unwrap(),
        ..spec_of(&e)
    };
    let job = e.ws.create_job(&spec).unwrap().job_id;
    e.ws.render(job, &RenderOptions::default()).unwrap();
    let (t, calls) = counted(FailurePoint::None);
    match e.ws.submit(job, &artifact(), &t) {
        Err(WorkflowError::PreflightBlocked(f)) => {
            assert!(f.iter().any(|x| x.code == ErrorCode::IncompatibleRecipe));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

/// The job spec behind an environment's job, rebuilt from its report.
fn spec_of(e: &Env) -> JobSpec {
    let report = e.ws.report(e.job).unwrap();
    JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1735000000001,
        setup: report.setup,
        mode: report.mode,
        recipe: report.recipe,
        assets: BTreeMap::from([("art".to_string(), report.assets[0].content_hash)]),
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
    }
}

#[test]
fn a_mode_that_is_not_validated_blocks_submission_alone() {
    let e = env_with(true, false);
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
    let (t, calls) = counted(FailurePoint::None);
    match e.ws.submit(e.job, &artifact(), &t) {
        Err(WorkflowError::PreflightBlocked(f)) => {
            assert_eq!(f.len(), 1, "{f:?}");
            assert_eq!(f[0].subject.as_deref(), Some("print_mode"));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn a_validated_mode_needs_a_verified_setup_before_anything_renders() {
    // The device rules refuse this combination at preflight, so it never reaches submission.
    let e = env_with(false, true);
    assert!(matches!(
        e.ws.render(e.job, &RenderOptions::default()),
        Err(WorkflowError::PreflightBlocked(_))
    ));
}

/// Builds the state a crash leaves behind: the lifecycle says a submission began, and the attempt
/// log says whatever it managed to write.
fn crashed_submission(attempts: &[AttemptState]) -> (Env, veylora_domain::ids::AttemptId) {
    let e = ready(true);
    let attempt = veylora_domain::ids::AttemptId::new();
    let lifecycle = FileLifecycleStore::open(e.ws.root().join("state")).unwrap();
    lifecycle
        .append(
            e.job,
            E::BeginSubmission,
            "2026-11-15T10:00:00Z",
            "",
            Some(attempt),
        )
        .unwrap();
    let jobs = FileJobStore::open(e.ws.root().join("jobs")).unwrap();
    let hash = e.ws.report(e.job).unwrap().snapshot_hash;
    for (i, state) in attempts.iter().enumerate() {
        jobs.append_attempt(veylora_domain::job::SubmissionAttempt {
            attempt_id: attempt,
            job_id: e.job,
            snapshot_hash: hash,
            sequence: i as u32 + 1,
            state: *state,
            bytes_written: if *state == AttemptState::Submitted {
                4096
            } else {
                0
            },
            note: String::new(),
            recorded_at: "2026-11-15T10:00:00Z".into(),
        })
        .unwrap();
    }
    (e, attempt)
}

#[test]
fn recovery_completes_only_what_the_attempt_log_proves() {
    use AttemptState::*;
    // (attempt log, job state after recovery)
    let cases: [(&[AttemptState], JobState); 5] = [
        (&[Submitting, Submitted], JobState::Submitted),
        (&[Submitting, Failed], JobState::Failed),
        (&[Submitting, Cancelled], JobState::Failed),
        (&[Submitting, Ambiguous], JobState::Review),
        // Nothing recorded, or only the opening record: the printer may have the data.
        (&[Submitting], JobState::Review),
    ];
    for (log, expected) in cases {
        let (e, attempt) = crashed_submission(log);
        assert_eq!(e.ws.recover(e.job).unwrap().after, expected, "{log:?}");
        let jobs = FileJobStore::open(e.ws.root().join("jobs")).unwrap();
        let latest = jobs
            .list_attempts(e.job)
            .unwrap()
            .into_iter()
            .find(|a| a.attempt_id == attempt)
            .unwrap();
        if log == [Submitting] {
            assert_eq!(
                latest.state, Ambiguous,
                "an open attempt is closed as uncertain"
            );
        }
    }
    // No attempt at all.
    let (e, _) = crashed_submission(&[]);
    assert_eq!(e.ws.recover(e.job).unwrap().after, JobState::Review);
}

#[test]
fn a_submitted_job_cannot_be_declared_not_printed() {
    let e = ready(true);
    let (t, _) = counted(FailurePoint::None);
    e.ws.submit(e.job, &artifact(), &t).unwrap();
    assert!(e
        .ws
        .resolve_review(e.job, ReviewResolution::NotPrinted, "op", "x")
        .is_err());
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Submitted);
}

#[test]
fn recorded_times_are_utc_timestamps() {
    let e = ready(false);
    for r in e.ws.lifecycle_events(e.job).unwrap() {
        let t = r.recorded_at.as_bytes();
        assert_eq!(t.len(), 20, "{}", r.recorded_at);
        assert!(r.recorded_at.starts_with("20") && r.recorded_at.ends_with('Z'));
        assert!(t[4] == b'-' && t[10] == b'T' && t[13] == b':');
    }
}

#[test]
fn an_acknowledgement_for_another_subject_does_not_cover_the_finding() {
    let e = env(false);
    let strict = veylora_workflow::PreflightOptions {
        min_effective_ppi: 5_000,
        ..Default::default()
    };
    let opts = RenderOptions {
        layout: strict,
        acknowledged: vec![(
            ErrorCode::LowEffectiveResolution,
            Some("some other placement".into()),
        )],
        ..Default::default()
    };
    assert!(matches!(
        e.ws.render(e.job, &opts),
        Err(WorkflowError::PreflightBlocked(_))
    ));
    assert_eq!(e.ws.state(e.job).unwrap(), JobState::Held);
}

#[test]
fn recovery_refuses_a_job_another_process_is_working_on() {
    let e = ready(true);
    let before = events(&e);
    let claim = e.ws.try_lock_job(e.job).unwrap();
    let err = e.ws.recover(e.job).unwrap_err().to_string().to_lowercase();
    assert!(err.contains("another process"), "{err}");
    assert_eq!(events(&e), before, "a refused recovery records nothing");
    drop(claim);
    assert!(e.ws.recover(e.job).is_ok());
}

#[test]
fn render_and_submit_refuse_a_job_someone_else_is_working_on_and_touch_nothing() {
    let e = ready(true);
    let (t, sent) = counted(FailurePoint::None);
    let before = events(&e);
    let claim = e.ws.try_lock_job(e.job).unwrap();
    for err in [
        e.ws.submit(e.job, &artifact(), &t).unwrap_err().to_string(),
        e.ws.render(e.job, &RenderOptions::default())
            .unwrap_err()
            .to_string(),
    ] {
        assert!(err.to_lowercase().contains("another process"), "{err}");
    }
    assert_eq!(
        sent.load(Ordering::SeqCst),
        0,
        "the printer was never touched"
    );
    assert_eq!(events(&e), before);
    drop(claim);
    e.ws.submit(e.job, &artifact(), &t).unwrap();
    assert_eq!(sent.load(Ordering::SeqCst), 1);
}

#[test]
fn a_claim_is_per_job_and_is_released_when_dropped() {
    let e = ready(true);
    let claim = e.ws.try_lock_job(e.job).unwrap();
    assert!(e.ws.try_lock_job(e.job).is_err(), "one claim at a time");
    assert!(
        e.ws.try_lock_job(veylora_domain::ids::JobId::default())
            .is_ok(),
        "another job is not affected"
    );
    drop(claim);
    assert!(e.ws.try_lock_job(e.job).is_ok());
}

#[test]
fn an_operation_that_fails_still_releases_its_claim() {
    let e = env(true); // never rendered, so submit is refused before it starts
    let (t, _) = counted(FailurePoint::None);
    assert!(e.ws.submit(e.job, &artifact(), &t).is_err());
    assert!(
        e.ws.try_lock_job(e.job).is_ok(),
        "the failed submit did not keep the claim"
    );
}

/// Not a test of its own: the child process of the crash test below takes a claim and holds it.
#[test]
#[ignore = "helper process for a_crashed_process_does_not_keep_its_claim"]
fn hold_the_claim_until_killed() {
    use std::io::Write;
    let (Ok(root), Ok(job)) = (
        std::env::var("VEYLORA_TEST_ROOT"),
        std::env::var("VEYLORA_TEST_JOB"),
    ) else {
        return;
    };
    let ws = Workspace::open(root).unwrap();
    let _claim = ws.try_lock_job(job.parse().unwrap()).unwrap();
    println!("HELD");
    std::io::stdout().flush().unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

#[test]
fn a_crashed_process_does_not_keep_its_claim() {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    let e = ready(true);
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "hold_the_claim_until_killed",
            "--nocapture",
        ])
        .env("VEYLORA_TEST_ROOT", e.ws.root())
        .env("VEYLORA_TEST_JOB", e.job.to_string())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let lines = BufReader::new(child.stdout.take().unwrap()).lines();
    for line in lines {
        if line.unwrap().trim() == "HELD" {
            break;
        }
    }
    // Another process really holds the job: recovery must refuse it.
    let refused = e.ws.recover(e.job);
    // Kill the holder the way a crash would, with no chance to clean up.
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(
        refused
            .unwrap_err()
            .to_string()
            .to_lowercase()
            .contains("another process"),
        "a live holder in another process was not respected"
    );
    // Its claim went with it, so the recovery it would have needed is not blocked.
    assert!(e.ws.recover(e.job).is_ok());
}
