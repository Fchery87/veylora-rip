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
        placement_id: None,
        acknowledged: false,
        acknowledgement: None,
    }
}

#[test]
fn every_blocking_fixture_finding_has_a_visible_correction() {
    let store = JobStore::fixture();
    let mut seen_blocking = 0;
    for summary in store.list() {
        let job = store.get(&summary.id).unwrap();
        let view = preflight::present(&job, &CapabilityState::fixture());
        for group in &view.groups {
            for finding in &group.findings {
                if finding.severity == Severity::Blocking {
                    seen_blocking += 1;
                    assert!(
                        !finding.correction.trim().is_empty(),
                        "finding {} has no correction",
                        finding.id
                    );
                    assert_ne!(
                        finding.correction,
                        preflight::NO_AUTOMATIC_FIX,
                        "fixture finding {} must carry a concrete fix, not the generic fallback",
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
    let view = preflight::present(&job, &CapabilityState::fixture());
    let finding = &view.groups[0].findings[0];
    assert!(!finding.correction.trim().is_empty());
    assert!(finding.correction.contains("no automatic fix"));
}

#[test]
fn blocking_findings_use_the_authored_fix_and_group_first_with_a_bad_summary() {
    let job = JobStore::fixture().get("J-0144").unwrap();
    let view = preflight::present(&job, &CapabilityState::fixture());
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
    let view = preflight::present(&job, &CapabilityState::fixture());
    assert!(view.groups[0].findings[0].can_acknowledge);
    assert_eq!(view.summary.text, "1 to confirm");
    let blocked = JobStore::fixture().get("J-0144").unwrap();
    let view = preflight::present(&blocked, &CapabilityState::fixture());
    assert!(!view.groups[0].findings[0].can_acknowledge);
}

#[test]
fn findings_that_point_at_a_placement_carry_its_id() {
    let job = JobStore::fixture().get("J-0144").unwrap();
    let view = preflight::present(&job, &CapabilityState::fixture());
    let id = view.groups[0].findings[0].placement_id.as_deref().unwrap();
    assert!(job.placements.iter().any(|p| p.id == id));
}

#[test]
fn acknowledging_a_confirm_finding_records_who_and_when() {
    let mut store = JobStore::fixture();
    let before = preflight::present(&store.get("J-0145").unwrap(), &CapabilityState::fixture());
    assert_eq!(before.groups[0].findings[0].acknowledgement, None);
    let job = store.acknowledge_finding("J-0145", "f2").unwrap();
    let ack = job.findings[0].acknowledgement.as_ref().unwrap();
    assert_eq!(ack.operator, "Fixture operator");
    assert!(!ack.at.is_empty());
    let view = preflight::present(&job, &CapabilityState::fixture());
    let text = view.groups[0].findings[0]
        .acknowledgement
        .as_deref()
        .unwrap();
    assert_eq!(
        text,
        format!("Acknowledged by Fixture operator at {}", ack.at)
    );
}

#[test]
fn white_policy_wire_names_are_unchanged_and_unknown_values_are_rejected() {
    use veylora_app_lib::view_models::{MaskSource, WhitePolicyKind};
    let job = JobStore::fixture().get("J-0145").unwrap();
    let json = serde_json::to_value(&job.white_policy).unwrap();
    assert_eq!(json["policy"], "Underbase");
    assert_eq!(json["source"], "Auto from alpha");
    assert_eq!(
        serde_json::from_value::<WhitePolicyKind>("No white".into()).unwrap(),
        WhitePolicyKind::NoWhite
    );
    assert_eq!(
        serde_json::from_value::<MaskSource>("Whole placement".into()).unwrap(),
        MaskSource::WholePlacement
    );
    assert!(serde_json::from_value::<WhitePolicyKind>("Highlight".into()).is_err());
}

// ---- Step 2: plane toggles and metadata ----

use veylora_app_lib::screens::inspect::{self, InspectSelection};
use veylora_app_lib::state::{CapabilityState, DataSource};
use veylora_app_lib::view_models::{Plane, StageId};

fn selection(stage: StageId, planes: &[Plane]) -> InspectSelection {
    InspectSelection {
        stage,
        planes: planes.to_vec(),
        composite: false,
    }
}

#[test]
fn inspect_starts_on_one_logical_plane() {
    let default = inspect::default_selection();
    assert_eq!(default, selection(StageId::Logical, &[Plane::K]));
}

#[test]
fn toggling_a_plane_keeps_the_stage_and_shows_one_plane_at_a_time() {
    let start = selection(StageId::Slots, &[Plane::K]);
    let next = inspect::toggle_plane(&start, Plane::C, true);
    assert_eq!(next.stage, StageId::Slots, "the stage does not jump");
    assert_eq!(
        next.planes,
        vec![Plane::C],
        "selecting replaces, never adds"
    );
    let again = inspect::toggle_plane(&next, Plane::C, true);
    assert_eq!(again.planes, vec![Plane::C]);
    let off = inspect::toggle_plane(&next, Plane::C, false);
    assert_eq!(off.planes, Vec::<Plane>::new());
    assert_eq!(off.stage, StageId::Slots);
}

#[test]
fn composite_is_a_labeled_simulation_that_replaces_the_single_plane() {
    let caps = CapabilityState::fixture();
    let job = JobStore::fixture().get("J-0143").unwrap();
    let start = selection(StageId::Slots, &[Plane::K]);
    let composite = inspect::set_composite(&start, true);
    assert!(composite.composite && composite.planes.is_empty());
    assert_eq!(composite.stage, StageId::Slots);
    let view = inspect::present(&job, &caps, &composite).unwrap();
    assert!(view.composite.pressed);
    assert_eq!(
        view.composite.label,
        "Composite simulation of retained planes"
    );
    assert!(view.planes.iter().all(|p| !p.pressed));
    assert!(view
        .chip
        .contains("Composite simulation of retained planes"));
    assert!(view.chip.ends_with("Not physical output"));
    let back = inspect::toggle_plane(&composite, Plane::M, true);
    assert!(!back.composite);
    assert_eq!(back.planes, vec![Plane::M]);
    let plain = inspect::present(&job, &caps, &start).unwrap();
    assert!(!plain.composite.pressed);
    assert!(!plain.chip.contains("Composite"));
}

#[test]
fn slots_and_screened_offer_both_white_slots_and_logical_offers_one_white_plane() {
    let caps = CapabilityState::fixture();
    let job = JobStore::fixture().get("J-0143").unwrap();
    let names = |stage| {
        inspect::present(&job, &caps, &selection(stage, &[]))
            .unwrap()
            .planes
            .iter()
            .map(|p| p.plane)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        names(StageId::Logical),
        vec![Plane::C, Plane::M, Plane::Y, Plane::K, Plane::W]
    );
    let slots = vec![Plane::C, Plane::M, Plane::Y, Plane::K, Plane::W1, Plane::W2];
    assert_eq!(names(StageId::Slots), slots);
    assert_eq!(names(StageId::Screened), slots);
    let view = inspect::present(&job, &caps, &selection(StageId::Slots, &[Plane::W])).unwrap();
    assert!(
        view.selection.planes.is_empty(),
        "a logical W is not a physical slot"
    );
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
fn metadata_covers_the_design_fields_and_is_labeled_fixture() {
    use veylora_app_lib::view_models::MetaKind;
    let caps = CapabilityState::fixture();
    let job = JobStore::fixture().get("J-0143").unwrap();
    let view = inspect::present(&job, &caps, &selection(StageId::Slots, &[Plane::K])).unwrap();
    let labels: Vec<_> = view.metadata.iter().map(|m| m.label.as_str()).collect();
    assert_eq!(
        labels,
        vec![
            "Job",
            "Dimensions",
            "Plane",
            "Mode ID",
            "Setup ID",
            "Recipe ID",
            "Profile hash",
            "Screen ID",
            "Transform ownership",
            "Artifact hash",
            "Orientation"
        ]
    );
    let row = |label: &str| view.metadata.iter().find(|m| m.label == label).unwrap();
    assert_eq!(row("Job").kind, MetaKind::Identifier);
    assert_eq!(row("Orientation").kind, MetaKind::Text);
    assert_eq!(row("Plane").value, "Physical slot K");
    assert!(row("Artifact hash").value.starts_with("sha256:fixture"));
    assert!(row("Profile hash").value.starts_with("sha256:fixture"));
    assert!(row("Dimensions").value.contains("297 x 420 mm"));
    assert!(view.metadata_note.contains("Fixture data"));
    assert!(row("Transform ownership").value.contains("PM-exp-01"));
}

#[test]
fn coverage_and_loupe_are_stubs_without_numbers_until_a_raster_exists() {
    let caps = CapabilityState::fixture();
    let job = JobStore::fixture().get("J-0143").unwrap();
    let view = inspect::present(&job, &caps, &selection(StageId::Logical, &[Plane::K])).unwrap();
    let titles: Vec<_> = view.panels.iter().map(|p| p.title.as_str()).collect();
    assert_eq!(titles, vec!["Coverage", "Loupe"]);
    for panel in &view.panels {
        assert_eq!(panel.message, "Available when a raster exists");
    }
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
    assert_eq!(value(&logical, "Mode ID"), "PM-exp-01");
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
    let draft = store.get("J-0145").unwrap();
    assert_eq!(
        store.release_to_printer("J-0145").unwrap_err(),
        job_actions::release_to_printer(&caps, &draft)
            .reason
            .unwrap(),
        "a draft job is refused for the presenter's reason too"
    );
    assert_eq!(
        store.release_to_printer("J-0143").unwrap_err(),
        actions.release_to_printer.reason.unwrap(),
        "the command refuses with exactly the reason the presenter shows"
    );
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
    let view = review::present(&caps, &store.get("J-0142").unwrap()).unwrap();
    assert!(view.printer_blocked);
    assert_eq!(view.attempt.id, "A-0142-1");
    assert_eq!(view.attempt.source, DataSource::Fixture);
    assert_eq!(view.attempt.source_label, "Fixture data");
    let mut live = caps.clone();
    live.data_source = DataSource::Live;
    let live_view = review::present(&live, &store.get("J-0142").unwrap()).unwrap();
    assert_eq!(live_view.attempt.source, DataSource::Live);
    assert_ne!(live_view.attempt.source_label, "Fixture data");
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
    assert!(review::present(&caps, &store.get("J-0143").unwrap()).is_none());
}

#[test]
fn review_shows_the_facts_and_what_was_sent() {
    let store = JobStore::fixture();
    let caps = store.capabilities();
    let view = review::present(&caps, &store.get("J-0142").unwrap()).unwrap();
    let facts: Vec<_> = view
        .facts
        .iter()
        .map(|f| (f.label.as_str(), f.value.as_str()))
        .collect();
    assert_eq!(
        facts,
        vec![
            ("Bytes written / intended", "1048576 / 4194304"),
            ("Spool job ID", "SPOOL-0142"),
            ("Last signal", "09:14:02"),
            ("Source", "Fixture data"),
        ]
    );
    let sent = &view.what_was_sent;
    assert_eq!(sent.artifact_hash, "sha256:fixture-artifact-0142");
    assert_eq!(sent.artifact_size, "4194304 bytes");
    assert_eq!(sent.snapshot_hash, "sha256:fixture-0000");
    assert_eq!(
        sent.placements,
        vec!["artwork.png \u{b7} 20, 30 mm \u{b7} 180 x 240 mm"]
    );
}

#[test]
fn unrecorded_review_facts_say_so_instead_of_inventing_values() {
    let store = JobStore::fixture();
    let caps = store.capabilities();
    let mut job = store.get("J-0142").unwrap();
    let attempt = &mut job.attempts[0];
    attempt.spool_job_id = None;
    attempt.bytes_written = None;
    attempt.bytes_intended = None;
    attempt.artifact_hash = None;
    let view = review::present(&caps, &job).unwrap();
    assert_eq!(view.facts[0].value, "Not recorded");
    assert_eq!(view.facts[1].value, "Not recorded");
    assert_eq!(view.what_was_sent.artifact_hash, "Not recorded");
    assert_eq!(view.what_was_sent.artifact_size, "Not recorded");
}

#[test]
fn a_reprint_creates_one_held_attempt_that_is_never_sent_or_retried() {
    for (outcome, recorded) in [
        (ReviewOutcome::Reprint, AttemptOutcome::Reprint),
        (ReviewOutcome::PrintedBadly, AttemptOutcome::PrintedBadly),
    ] {
        let mut store = JobStore::fixture();
        store.resolve_review("A-0142-1", outcome).unwrap();
        let job = store.get("J-0142").unwrap();
        assert_eq!(job.attempts.len(), 2, "{outcome:?} appends one attempt");
        assert_eq!(job.attempts[0].outcome, recorded, "the original is kept");
        let reprint = &job.attempts[1];
        assert_eq!(reprint.id, "A-0142-2");
        assert_eq!(reprint.kind, AttemptKind::Printer);
        assert_eq!(reprint.outcome, AttemptOutcome::Held);
        assert!(reprint.note.contains("not been sent"));
        assert_eq!(job.summary.state, JobState::Held);
        // Nothing after the resolution sends or retries it.
        assert!(store.release_to_printer("J-0142").is_err());
        assert!(store.resolve_review("A-0142-2", outcome).is_err());
        let after = store.get("J-0142").unwrap();
        assert_eq!(after.attempts, job.attempts, "attempts are append-only");
        assert!(store.capabilities().open_review.is_none());
        assert!(after.attempts.iter().all(|a| !matches!(
            a.outcome,
            AttemptOutcome::Pending | AttemptOutcome::Ambiguous
        )));
    }
}

#[test]
fn finished_and_decide_later_create_no_reprint_attempt() {
    for outcome in [ReviewOutcome::Finished, ReviewOutcome::DecideLater] {
        let mut store = JobStore::fixture();
        store.resolve_review("A-0142-1", outcome).unwrap();
        assert_eq!(store.get("J-0142").unwrap().attempts.len(), 1);
    }
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
        let expected = if matches!(
            outcome,
            ReviewOutcome::Reprint | ReviewOutcome::PrintedBadly
        ) {
            2
        } else {
            1
        };
        assert_eq!(job.attempts.len(), expected, "{outcome:?} attempt count");
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
            let chip = inspect::stage_chip_text(stage, source, false);
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
        let sim = inspect::stage_chip_text(stage, PreviewSource::Simulation, false);
        assert!(
            sim.contains("simulation") && !sim.contains("retained"),
            "{sim}"
        );
        let fixture = inspect::stage_chip_text(stage, PreviewSource::Fixture, false);
        assert!(
            fixture.contains("simulation") && fixture.contains("Fixture data"),
            "{fixture}"
        );
    }
    assert_eq!(
        inspect::stage_chip_text(StageId::Logical, PreviewSource::Fixture, false),
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
    let json = serde_json::to_value(preflight::present(&job, &caps)).unwrap();
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
    use veylora_app_lib::view_models::{MaskSource, WhitePolicyKind};
    let err = store
        .set_white_policy("J-0143", WhitePolicyKind::NoWhite)
        .unwrap_err();
    assert_eq!(err, "Snapshot frozen. Duplicate the job to edit.");
    let job = store
        .set_white_policy("J-0145", WhitePolicyKind::NoWhite)
        .unwrap();
    assert_eq!(job.white_policy.policy, WhitePolicyKind::NoWhite);
    assert!(store
        .set_white_mask("J-0145", MaskSource::WholePlacement, true)
        .is_err());
    let job = store
        .set_white_mask("J-0145", MaskSource::WholePlacement, false)
        .unwrap();
    assert_eq!(job.white_policy.source, MaskSource::WholePlacement);
    assert!(store.set_white_choke("J-0145", 0.5).is_err());
}
