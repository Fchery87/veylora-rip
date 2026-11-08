//! Presentation behavior for preflight, inspect, hold/release and review.
//! Tested through the public view-model seam; no Tauri runtime involved.

use veylora_app_lib::jobs::JobStore;
use veylora_app_lib::screens::preflight;
use veylora_app_lib::view_models::{Finding, Severity, Tone};

fn blocking(id: &str, fix: &str) -> Finding {
    Finding {
        id: id.to_owned(),
        severity: Severity::Blocking,
        title: "Placement is outside the printable area".to_owned(),
        cause: "sleeve.png extends past the left edge.".to_owned(),
        fix: fix.to_owned(),
        acknowledged: false,
    }
}

#[test]
fn every_blocking_fixture_finding_has_a_visible_correction() {
    let store = JobStore::fixture();
    let mut seen_blocking = 0;
    for summary in store.list() {
        let job = store.get(&summary.id).unwrap();
        let view = preflight::present(&job, "Home bench rev 3", "PM-exp-01");
        for group in &view.groups {
            for finding in &group.findings {
                if finding.severity == Severity::Blocking {
                    seen_blocking += 1;
                    assert!(
                        !finding.correction.trim().is_empty(),
                        "finding {} has no correction",
                        finding.id
                    );
                }
            }
        }
    }
    assert!(
        seen_blocking > 0,
        "fixtures must include a blocking finding"
    );
}

#[test]
fn a_blocking_finding_without_a_fix_still_shows_a_correction() {
    let mut job = JobStore::fixture().get("J-0144").unwrap();
    job.findings = vec![blocking("f9", "   ")];
    let view = preflight::present(&job, "Home bench rev 3", "PM-exp-01");
    let finding = &view.groups[0].findings[0];
    assert!(!finding.correction.trim().is_empty());
    assert!(finding.correction.contains("no automatic fix"));
}

#[test]
fn blocking_findings_use_the_authored_fix_and_group_first_with_a_bad_summary() {
    let job = JobStore::fixture().get("J-0144").unwrap();
    let view = preflight::present(&job, "Home bench rev 3", "PM-exp-01");
    assert_eq!(view.groups[0].title, "Blocking");
    assert_eq!(
        view.groups[0].findings[0].correction,
        "Move it left by at least 4.2 mm"
    );
    assert_eq!(view.summary.text, "1 blocking");
    assert_eq!(view.summary.tone, Tone::Bad);
    assert_eq!(
        view.checked_against,
        "Checked just now against Home bench rev 3, mode PM-exp-01"
    );
}

#[test]
fn only_confirm_findings_can_be_acknowledged() {
    let job = JobStore::fixture().get("J-0145").unwrap();
    let view = preflight::present(&job, "Home bench rev 3", "PM-exp-01");
    assert!(view.groups[0].findings[0].can_acknowledge);
    assert_eq!(view.summary.text, "1 to confirm");
    let blocked = JobStore::fixture().get("J-0144").unwrap();
    let view = preflight::present(&blocked, "Home bench rev 3", "PM-exp-01");
    assert!(!view.groups[0].findings[0].can_acknowledge);
}

// ---- Step 2: plane toggles and metadata ----

use veylora_app_lib::screens::inspect::{self, InspectSelection};
use veylora_app_lib::state::{CapabilityState, DataSource};
use veylora_app_lib::view_models::{Plane, StageId};

fn selection(stage: StageId, planes: &[Plane]) -> InspectSelection {
    InspectSelection {
        stage,
        planes: planes.to_vec(),
    }
}

#[test]
fn toggling_a_plane_returns_to_logical_stage_in_canonical_order() {
    let start = selection(StageId::Slots, &[Plane::K]);
    let next = inspect::toggle_plane(&start, Plane::C, true);
    assert_eq!(next.stage, StageId::Logical);
    assert_eq!(next.planes, vec![Plane::C, Plane::K]);
    let off = inspect::toggle_plane(&next, Plane::K, false);
    assert_eq!(off.planes, vec![Plane::C]);
    let again = inspect::toggle_plane(&off, Plane::C, true);
    assert_eq!(again.planes, vec![Plane::C], "no duplicates");
}

#[test]
fn inspect_view_lists_every_plane_and_the_stage_options() {
    let caps = CapabilityState::fixture();
    let job = JobStore::fixture().get("J-0143").unwrap();
    let view = inspect::present(&job, &caps, &selection(StageId::Logical, &[Plane::K])).unwrap();
    let pressed: Vec<_> = view.planes.iter().map(|p| (p.plane, p.pressed)).collect();
    assert_eq!(
        pressed,
        vec![
            (Plane::C, false),
            (Plane::M, false),
            (Plane::Y, false),
            (Plane::K, true),
            (Plane::W, false)
        ]
    );
    let stages: Vec<_> = view.stages.iter().map(|s| s.id).collect();
    assert_eq!(
        stages,
        vec![StageId::Logical, StageId::Slots, StageId::Screened]
    );
    assert!(!view.raster_shown);
}

#[test]
fn metadata_names_job_mode_and_orientation_per_stage() {
    let caps = CapabilityState::fixture();
    let job = JobStore::fixture().get("J-0143").unwrap();
    let logical = inspect::present(&job, &caps, &selection(StageId::Logical, &[])).unwrap();
    let slots = inspect::present(&job, &caps, &selection(StageId::Slots, &[])).unwrap();
    let value = |v: &veylora_app_lib::view_models::InspectView, label: &str| {
        v.metadata
            .iter()
            .find(|m| m.label == label)
            .unwrap()
            .value
            .clone()
    };
    assert_eq!(value(&logical, "Job"), "J-0143");
    assert_eq!(value(&logical, "Mode"), "PM-exp-01");
    assert_eq!(value(&logical, "Orientation"), "Artwork orientation");
    assert_eq!(
        value(&slots, "Orientation"),
        "Film orientation, mirrored by the mode"
    );
}

#[test]
fn screened_stage_is_disabled_with_a_reason_when_the_mode_cannot_inspect_it() {
    let mut caps = CapabilityState::fixture();
    caps.mode.screened_inspect = false;
    let job = JobStore::fixture().get("J-0143").unwrap();
    let view = inspect::present(&job, &caps, &selection(StageId::Logical, &[])).unwrap();
    let screened = view
        .stages
        .iter()
        .find(|s| s.id == StageId::Screened)
        .unwrap();
    assert!(screened.disabled);
    assert!(screened.reason.is_some());
    assert!(inspect::present(&job, &caps, &selection(StageId::Screened, &[])).is_err());
}

#[test]
fn inspect_requires_a_rendered_job() {
    let caps = CapabilityState::fixture();
    let draft = JobStore::fixture().get("J-0145").unwrap();
    let err = inspect::present(&draft, &caps, &selection(StageId::Logical, &[])).unwrap_err();
    assert_eq!(err, "Render the job to inspect its retained output.");
    let _ = DataSource::Fixture;
}

// ---- Step 3: hold, inspect, release, and ambiguous review ----

use veylora_app_lib::screens::{job_actions, review};
use veylora_app_lib::view_models::{AttemptKind, AttemptOutcome, JobState, ReviewOutcome};

#[test]
fn render_and_hold_is_gated_by_blocking_then_unconfirmed_findings() {
    let mut store = JobStore::fixture();
    let caps = store.capabilities();
    let hoodie = store.get("J-0144").unwrap();
    let actions = job_actions::present(&caps, &hoodie);
    assert!(!actions.render_and_hold.enabled);
    assert_eq!(
        actions.render_and_hold.reason.as_deref(),
        Some("Fix 1 blocking finding to render")
    );
    assert!(store.render_and_hold("J-0144").is_err());

    let pocket = store.get("J-0145").unwrap();
    assert_eq!(
        job_actions::present(&caps, &pocket)
            .render_and_hold
            .reason
            .as_deref(),
        Some("Confirm 1 finding to render")
    );
    store.acknowledge_finding("J-0145", "f2").unwrap();
    let pocket = store.get("J-0145").unwrap();
    assert!(job_actions::present(&caps, &pocket).render_and_hold.enabled);
    let held = store.render_and_hold("J-0145").unwrap();
    assert_eq!(held.summary.state, JobState::Held);
    assert!(held.rendered);
}

#[test]
fn only_confirm_findings_can_be_acknowledged_in_the_store() {
    let mut store = JobStore::fixture();
    assert!(store.acknowledge_finding("J-0144", "f1").is_err());
}

#[test]
fn inspect_is_available_only_after_render() {
    let store = JobStore::fixture();
    let caps = store.capabilities();
    let draft = job_actions::present(&caps, &store.get("J-0145").unwrap());
    assert!(!draft.inspect.enabled);
    assert_eq!(
        draft.inspect.reason.as_deref(),
        Some("Render the job to inspect its retained output.")
    );
    let held = job_actions::present(&caps, &store.get("J-0143").unwrap());
    assert!(held.inspect.enabled);
}

#[test]
fn release_to_printer_is_refused_until_the_physical_gate_passes() {
    let mut store = JobStore::fixture();
    let caps = store.capabilities();
    assert!(!caps.sinks.physical_printer);
    let held = store.get("J-0143").unwrap();
    let actions = job_actions::present(&caps, &held);
    assert!(!actions.release_to_printer.enabled);
    assert!(actions
        .release_to_printer
        .reason
        .as_deref()
        .unwrap()
        .contains("physical gate passes"));
    assert!(store.release_to_printer("J-0143").is_err());
    assert_eq!(
        store.get("J-0143").unwrap().attempts.len(),
        0,
        "no attempt is created"
    );
}

#[test]
fn file_sink_writes_a_labeled_attempt_that_is_not_physical_output() {
    let mut store = JobStore::fixture();
    let job = store.write_to_file_sink("J-0143").unwrap();
    let attempt = job.attempts.last().unwrap();
    assert_eq!(attempt.kind, AttemptKind::FileSink);
    assert_eq!(attempt.outcome, AttemptOutcome::Finished);
    assert!(attempt.note.contains("Not physical output"));
    assert!(attempt.sink_path.as_deref().unwrap().ends_with(".vrip"));
    assert_eq!(job.summary.state, JobState::Held);
    assert!(
        store.write_to_file_sink("J-0145").is_err(),
        "draft jobs cannot be written"
    );
}

#[test]
fn an_open_review_blocks_the_printer_and_surfaces_in_capabilities() {
    let store = JobStore::fixture();
    let caps = store.capabilities();
    let open = caps.open_review.as_ref().unwrap();
    assert_eq!(
        (open.job_id.as_str(), open.attempt_id.as_str()),
        ("J-0142", "A-0142-1")
    );
    let view = review::present(&store.get("J-0142").unwrap()).unwrap();
    assert!(view.printer_blocked);
    assert_eq!(view.attempt.id, "A-0142-1");
    let outcomes: Vec<_> = view.choices.iter().map(|c| c.outcome).collect();
    assert_eq!(
        outcomes,
        vec![
            ReviewOutcome::Finished,
            ReviewOutcome::Reprint,
            ReviewOutcome::PrintedBadly,
            ReviewOutcome::DecideLater
        ]
    );
    assert!(view
        .choices
        .iter()
        .all(|c| !c.effect.is_empty() && !c.confirm_label.is_empty()));
    assert!(review::present(&store.get("J-0143").unwrap()).is_none());
}

#[test]
fn resolving_a_review_never_retries_or_sends_anything() {
    for outcome in [
        ReviewOutcome::Finished,
        ReviewOutcome::Reprint,
        ReviewOutcome::PrintedBadly,
        ReviewOutcome::DecideLater,
    ] {
        let mut store = JobStore::fixture();
        store.resolve_review("A-0142-1", outcome).unwrap();
        let job = store.get("J-0142").unwrap();
        assert_eq!(job.attempts.len(), 1, "{outcome:?} must not add an attempt");
        assert!(
            job.attempts
                .iter()
                .all(|a| a.outcome != AttemptOutcome::Pending),
            "{outcome:?} must not leave a pending attempt"
        );
        assert!(!matches!(
            job.summary.state,
            JobState::Submitting | JobState::Submitted | JobState::Rendering
        ));
    }
}

#[test]
fn review_outcomes_map_to_operator_recorded_states() {
    let resolve = |outcome| {
        let mut store = JobStore::fixture();
        store.resolve_review("A-0142-1", outcome).unwrap();
        (store.get("J-0142").unwrap(), store.capabilities())
    };
    let (job, caps) = resolve(ReviewOutcome::Finished);
    assert_eq!(job.summary.state, JobState::OperatorConfirmedFinished);
    assert_eq!(job.attempts[0].outcome, AttemptOutcome::Finished);
    assert!(caps.open_review.is_none());

    let (job, caps) = resolve(ReviewOutcome::Reprint);
    assert_eq!(job.summary.state, JobState::Held);
    assert_eq!(job.attempts[0].outcome, AttemptOutcome::Reprint);
    assert!(caps.open_review.is_none());

    let (job, caps) = resolve(ReviewOutcome::DecideLater);
    assert_eq!(job.summary.state, JobState::NeedsReview);
    assert_eq!(job.attempts[0].outcome, AttemptOutcome::KeptInReview);
    assert!(caps.open_review.is_some(), "the printer stays blocked");
}

#[test]
fn only_an_attempt_in_review_can_be_resolved() {
    let mut store = JobStore::fixture();
    assert!(store
        .resolve_review("A-9999-1", ReviewOutcome::Finished)
        .is_err());
    store
        .resolve_review("A-0142-1", ReviewOutcome::Finished)
        .unwrap();
    assert!(
        store
            .resolve_review("A-0142-1", ReviewOutcome::Reprint)
            .is_err(),
        "a recorded outcome is never overwritten"
    );
}

// ---- Step 4: a simulated preview is never final output ----

use veylora_app_lib::view_models::PreviewSource;

const ALL_STAGES: [StageId; 5] = [
    StageId::Garment,
    StageId::Film,
    StageId::Logical,
    StageId::Slots,
    StageId::Screened,
];

#[test]
fn every_preview_label_says_it_is_not_physical_output() {
    for stage in ALL_STAGES {
        for source in [
            PreviewSource::Simulation,
            PreviewSource::Fixture,
            PreviewSource::Retained,
        ] {
            let chip = inspect::stage_chip_text(stage, source);
            assert!(chip.ends_with("Not physical output"), "{chip}");
            let lower = chip.to_lowercase();
            assert!(!lower.contains("final"), "{chip}");
            assert!(!lower.contains("submitted"), "{chip}");
            assert!(!lower.contains("printed"), "{chip}");
        }
    }
}

#[test]
fn simulations_and_fixtures_are_labeled_and_never_called_retained() {
    for stage in ALL_STAGES {
        let sim = inspect::stage_chip_text(stage, PreviewSource::Simulation);
        assert!(
            sim.contains("simulation") && !sim.contains("retained"),
            "{sim}"
        );
        let fixture = inspect::stage_chip_text(stage, PreviewSource::Fixture);
        assert!(
            fixture.contains("simulation") && fixture.contains("Fixture data"),
            "{fixture}"
        );
    }
    assert_eq!(
        inspect::stage_chip_text(StageId::Logical, PreviewSource::Fixture),
        "Logical planes: C M Y K W \u{b7} continuous tone \u{b7} simulation \u{b7} Fixture data \u{b7} Not physical output"
    );
}

#[test]
fn inspect_view_never_claims_a_retained_raster_while_none_exists() {
    let job = JobStore::fixture().get("J-0143").unwrap();
    for data_source in [DataSource::Fixture, DataSource::Live] {
        let mut caps = CapabilityState::fixture();
        caps.data_source = data_source;
        let view = inspect::present(&job, &caps, &selection(StageId::Slots, &[Plane::W])).unwrap();
        assert_ne!(view.source, PreviewSource::Retained);
        assert!(!view.raster_shown);
        assert!(view.chip.contains("Not physical output"));
    }
}

#[test]
fn view_models_serialize_with_the_frontend_field_names() {
    let store = JobStore::fixture();
    let caps = store.capabilities();
    let job = store.get("J-0144").unwrap();
    let json =
        serde_json::to_value(preflight::present(&job, "Home bench rev 3", "PM-exp-01")).unwrap();
    assert_eq!(json["groups"][0]["findings"][0]["canAcknowledge"], false);
    assert_eq!(json["groups"][0]["tone"], "bad");
    assert_eq!(json["openCount"], 1);

    let held = store.get("J-0143").unwrap();
    let json = serde_json::to_value(job_actions::present(&caps, &held)).unwrap();
    assert_eq!(json["releaseToPrinter"]["enabled"], false);
    assert!(json["writeToFileSink"].get("reason").is_none());

    let view = inspect::present(&held, &caps, &selection(StageId::Logical, &[Plane::K])).unwrap();
    let json = serde_json::to_value(view).unwrap();
    assert_eq!(json["selection"]["planes"][0], "K");
    assert_eq!(json["source"], "fixture");
    assert_eq!(json["rasterShown"], false);

    let detail = serde_json::to_value(&job).unwrap();
    assert_eq!(detail["placementCount"], 1, "summary fields are flattened");
    assert_eq!(detail["whitePolicy"]["invertMask"], false);
    assert_eq!(detail["state"], "draft");
}

// ---- White policy edits keep the frozen-snapshot and mode rules ----

#[test]
fn white_policy_edits_respect_frozen_snapshots_and_mode_support() {
    let mut store = JobStore::fixture();
    let err = store.set_white_policy("J-0143", "No white").unwrap_err();
    assert_eq!(err, "Snapshot frozen. Duplicate the job to edit.");
    let job = store.set_white_policy("J-0145", "No white").unwrap();
    assert_eq!(job.white_policy.policy, "No white");
    assert!(store.set_white_policy("J-0145", "Highlight").is_err());
    assert!(store
        .set_white_mask("J-0145", "Whole placement", true)
        .is_err());
    let job = store
        .set_white_mask("J-0145", "Whole placement", false)
        .unwrap();
    assert_eq!(job.white_policy.source, "Whole placement");
    assert!(store.set_white_choke("J-0145", 0.5).is_err());
}
