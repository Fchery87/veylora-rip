mod common;
use common::*;
use std::collections::BTreeMap;
use veylora_domain::{errors::ErrorCode, layout::Rotation, preflight::Severity};
use veylora_image::UntaggedPolicy;
use veylora_workflow::{
    inspect_artifacts, inspect_asset, JobSpec, LayoutSpec, PlacementSpec, PreflightOptions,
    RenderOptions, WorkflowError, Workspace,
};

struct Fixture {
    ws: Workspace,
    spec: JobSpec,
}

fn fixture_job(mutate: impl FnOnce(&mut LayoutSpec)) -> Fixture {
    let ws = Workspace::open(dir("ws")).unwrap();
    let s = setup();
    let m = mode(s.id());
    let setup_hash = ws.import_setup(&s.to_record()).unwrap();
    let mode_hash = ws.import_mode(&m.to_record()).unwrap();
    let recipe_hash = ws.import_recipe(&recipe()).unwrap();
    let art = ws.import_asset(&fixture("alpha-ramp.png")).unwrap();
    let mut layout = LayoutSpec {
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
    };
    mutate(&mut layout);
    let spec = JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1735000000000,
        setup: setup_hash,
        mode: mode_hash,
        recipe: recipe_hash,
        assets: BTreeMap::from([("art".to_string(), art)]),
        masks: BTreeMap::new(),
        layout,
    };
    Fixture { ws, spec }
}

#[test]
fn asset_inspection_reports_the_import_without_changing_it() {
    let bytes = fixture("alpha-ramp.png");
    let r = inspect_asset(&bytes, UntaggedPolicy::AssumeSrgb).unwrap();
    assert_eq!((r.width, r.height), (4, 1));
    assert_eq!(r.sha256, hash(&bytes));
    assert_eq!(r.layout, "rgba8");
    assert!(inspect_asset(b"not a png", UntaggedPolicy::Reject).is_err());
}

#[test]
fn png_to_retained_artifacts_end_to_end_without_a_printer() {
    let f = fixture_job(|_| {});
    let created = f.ws.create_job(&f.spec).unwrap();

    let report =
        f.ws.preflight(created.job_id, &PreflightOptions::default())
            .unwrap();
    assert!(report.can_render, "{:?}", report.findings);
    // The setup is not physically verified, which is said out loud.
    assert!(report.findings.iter().any(
        |x| x.code == ErrorCode::UnverifiedDeviceSetup && x.severity == Severity::Informational
    ));

    let out =
        f.ws.render(created.job_id, &RenderOptions::default())
            .unwrap();
    assert_eq!(out.manifest.evidence, "software_only");
    assert_eq!(
        (out.manifest.page_width_px, out.manifest.page_height_px),
        (4, 1)
    );
    assert_eq!(out.manifest.snapshot_hash, created.snapshot_hash);

    // Inspection re-reads the retained bytes and agrees with the manifest.
    let inspected = inspect_artifacts(&out.dir).unwrap();
    assert!(inspected.verified, "{inspected:?}");
    assert_eq!(inspected.manifest, out.manifest);
    // Alpha ramp: alpha 0, 64, 128, 255 becomes white coverage; process ink stays zero (white only).
    let white = inspected.plane("logical-w.f32").unwrap();
    assert_eq!(white.nonzero, 3);
    let cyan = inspected.plane("logical-c.f32").unwrap();
    assert_eq!(cyan.nonzero, 0);

    // Reports name the job, its inputs and the render.
    let job_report = f.ws.report(created.job_id).unwrap();
    assert_eq!(job_report.snapshot_hash, created.snapshot_hash);
    assert_eq!(job_report.renders.len(), 1);
    assert!(job_report.submission_attempts.is_empty());
}

#[test]
fn rendering_twice_creates_separate_identical_runs() {
    let f = fixture_job(|_| {});
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    let a = f.ws.render(id, &RenderOptions::default()).unwrap();
    let b = f.ws.render(id, &RenderOptions::default()).unwrap();
    assert_ne!(a.dir, b.dir);
    assert_eq!(
        a.manifest.screened_planes.hash,
        b.manifest.screened_planes.hash
    );
    assert_eq!(f.ws.report(id).unwrap().renders.len(), 2);
}

#[test]
fn a_tampered_artifact_is_detected_by_inspection() {
    let f = fixture_job(|_| {});
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    let out = f.ws.render(id, &RenderOptions::default()).unwrap();
    let file = out.dir.join("logical-w.f32");
    let mut bytes = std::fs::read(&file).unwrap();
    bytes[0] ^= 0xff;
    std::fs::write(&file, bytes).unwrap();
    let r = inspect_artifacts(&out.dir).unwrap();
    assert!(!r.verified);
    assert!(r.problems.iter().any(|p| p.contains("logical-w.f32")));
}

#[test]
fn preflight_reports_layout_and_device_problems_without_rendering() {
    // Placement past the page edge.
    let f = fixture_job(|l| l.placements[0].x_um = 10);
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    let r = f.ws.preflight(id, &PreflightOptions::default()).unwrap();
    assert!(!r.can_render);
    assert!(r
        .findings
        .iter()
        .any(|x| x.code == ErrorCode::Clipping && x.severity == Severity::Blocking));
    assert!(matches!(
        f.ws.render(id, &RenderOptions::default()),
        Err(WorkflowError::PreflightBlocked(_))
    ));
    // Nothing was written for the refused render.
    assert!(f.ws.report(id).unwrap().renders.is_empty());

    // A page bigger than the mode's printable area is blocked: 2400 px * 20 um = 48 000 um.
    let f = fixture_job(|l| l.page_width_um = 48_100);
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    let r = f.ws.preflight(id, &PreflightOptions::default()).unwrap();
    assert!(r
        .findings
        .iter()
        .any(|x| x.code == ErrorCode::UnsupportedMediaSize && x.severity == Severity::Blocking));
}

#[test]
fn confirmation_findings_must_be_acknowledged_to_render() {
    let f = fixture_job(|_| {});
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    let strict = PreflightOptions {
        min_effective_ppi: 5_000,
        ..PreflightOptions::default()
    };
    let r = f.ws.preflight(id, &strict).unwrap();
    let needs: Vec<_> = r
        .findings
        .iter()
        .filter(|x| x.severity == Severity::ConfirmationRequired)
        .collect();
    assert_eq!(needs.len(), 1);
    assert_eq!(needs[0].code, ErrorCode::LowEffectiveResolution);

    let unack = RenderOptions {
        layout: strict.clone(),
        ..RenderOptions::default()
    };
    assert!(f.ws.render(id, &unack).is_err());
    let ack = RenderOptions {
        layout: strict,
        acknowledged: vec![(needs[0].code, needs[0].subject.clone())],
        ..RenderOptions::default()
    };
    let out = f.ws.render(id, &ack).unwrap();
    assert_eq!(out.manifest.acknowledged_layout_findings.len(), 1);
}

#[test]
fn job_creation_rejects_inconsistent_inputs() {
    // Unknown asset name in a placement.
    let f = fixture_job(|l| l.placements[0].asset = "missing".into());
    assert!(matches!(
        f.ws.create_job(&f.spec),
        Err(WorkflowError::Invalid(_))
    ));
    // A setup and mode that do not belong together.
    let f = fixture_job(|_| {});
    let other = setup();
    let mut spec = f.spec.clone();
    spec.setup = f.ws.import_setup(&other.to_record()).unwrap();
    assert!(matches!(
        f.ws.create_job(&spec),
        Err(WorkflowError::Invalid(_))
    ));
    // An asset that is not in the store.
    let mut spec = f.spec.clone();
    spec.assets.insert("art".into(), hash(b"never imported"));
    assert!(f.ws.create_job(&spec).is_err());
    // An asset that is not a PNG.
    let mut spec = f.spec.clone();
    spec.assets
        .insert("art".into(), f.ws.import_asset(b"plain text").unwrap());
    assert!(matches!(
        f.ws.create_job(&spec),
        Err(WorkflowError::Image(_))
    ));
}

#[test]
fn crop_defaults_to_the_whole_image_and_can_select_a_region() {
    let f = fixture_job(|l| {
        l.placements[0].crop = Some(veylora_workflow::CropSpec {
            x: 2,
            y: 0,
            width: 2,
            height: 1,
        });
        l.page_width_um = 40;
        l.placements[0].width_um = 40;
    });
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    let out = f.ws.render(id, &RenderOptions::default()).unwrap();
    assert_eq!(out.manifest.page_width_px, 2);
    let white = inspect_artifacts(&out.dir)
        .unwrap()
        .plane("logical-w.f32")
        .unwrap()
        .clone();
    assert_eq!(white.nonzero, 2); // alpha 128 and 255
}

#[test]
fn job_creation_rejects_unsupported_spec_versions_and_non_gray_masks() {
    let f = fixture_job(|_| {});
    let mut spec = f.spec.clone();
    spec.schema_version = 2;
    assert!(matches!(
        f.ws.create_job(&spec),
        Err(WorkflowError::Invalid(_))
    ));
    // A color image is not a mask.
    let mut spec = f.spec.clone();
    let rgba = f.ws.import_asset(&fixture("alpha-ramp.png")).unwrap();
    spec.masks.insert("m".into(), rgba);
    assert!(matches!(
        f.ws.create_job(&spec),
        Err(WorkflowError::Invalid(m)) if m.contains("Gray8")
    ));
}

#[test]
fn a_page_taller_than_the_printable_area_is_blocked() {
    // 3600 px * 20 um = 72 000 um.
    let f = fixture_job(|l| l.page_height_um = 72_100);
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    let r = f.ws.preflight(id, &PreflightOptions::default()).unwrap();
    assert!(r
        .findings
        .iter()
        .any(|x| x.code == ErrorCode::UnsupportedMediaSize
            && x.severity == Severity::Blocking
            && x.subject.as_deref() == Some("layout.page")));
}

#[test]
fn reports_ignore_runs_that_never_completed() {
    let f = fixture_job(|_| {});
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    f.ws.render(id, &RenderOptions::default()).unwrap();
    // A crashed run leaves a directory without a manifest.
    std::fs::create_dir_all(
        f.ws.root()
            .join("artifacts")
            .join(id.to_string())
            .join("run-0002"),
    )
    .unwrap();
    assert_eq!(f.ws.report(id).unwrap().renders.len(), 1);
    // The next render does not reuse the incomplete run's name.
    let next = f.ws.render(id, &RenderOptions::default()).unwrap();
    assert!(next.dir.ends_with("run-0003"));
}

#[test]
fn a_truncated_artifact_is_reported_by_size() {
    let f = fixture_job(|_| {});
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    let out = f.ws.render(id, &RenderOptions::default()).unwrap();
    let file = out.dir.join("logical-w.f32");
    let bytes = std::fs::read(&file).unwrap();
    std::fs::write(&file, &bytes[..bytes.len() - 4]).unwrap();
    let r = inspect_artifacts(&out.dir).unwrap();
    assert!(!r.verified);
    assert!(r.problems.iter().any(|p| p.contains("bytes on disk")));
}

fn draft(ws: &Workspace) -> veylora_workflow::SetupStatus {
    let s = setup();
    ws.create_setup(&veylora_workflow::SetupDraft {
        id: None,
        origin: veylora_device::setup::SetupOrigin::Recorded,
        details: s.details().clone(),
    })
    .unwrap()
}

fn evidence_for(
    ws: &Workspace,
    setup_hash: &veylora_domain::provenance::ContentHash,
) -> veylora_device::setup::EvidenceRecord {
    evidence_for_record(&ws.setup_record(setup_hash).unwrap())
}

#[test]
fn created_setups_start_unverified_and_are_stored_durably() {
    let ws = Workspace::open(dir("setup")).unwrap();
    let created = draft(&ws);
    assert_eq!(
        created.verification,
        veylora_device::setup::VerificationState::Unverified
    );
    assert_eq!(created.revision, 1);
    // The stored record loads back identically.
    assert_eq!(
        ws.setup_record(&created.content_hash).unwrap().content_hash,
        created.content_hash
    );
    // A draft with invalid details is refused, not repaired.
    let mut bad = setup().details().clone();
    bad.physical_slots.clear();
    let r = ws.create_setup(&veylora_workflow::SetupDraft {
        id: None,
        origin: veylora_device::setup::SetupOrigin::Recorded,
        details: bad,
    });
    assert!(matches!(r, Err(WorkflowError::Invalid(_))), "{r:?}");
}

#[test]
fn setup_verification_requires_matching_evidence_and_never_changes_history() {
    let ws = Workspace::open(dir("verify")).unwrap();
    let created = draft(&ws);
    let evidence = evidence_for(&ws, &created.content_hash);

    // Evidence that observed a different channel is refused and nothing is stored.
    let mut wrong = evidence.clone();
    wrong.channels[0].observed_logical = veylora_device::channels::LogicalChannel::Black;
    assert!(matches!(
        ws.verify_setup(&created.content_hash, wrong),
        Err(WorkflowError::Invalid(_))
    ));
    assert_eq!(
        ws.setup_record(&created.content_hash).unwrap().verification,
        veylora_device::setup::VerificationState::Unverified
    );

    // Evidence for another setup revision is refused.
    let mut stale = evidence.clone();
    stale.revision = 9;
    assert!(ws.verify_setup(&created.content_hash, stale).is_err());

    let verified = ws
        .verify_setup(&created.content_hash, evidence.clone())
        .unwrap();
    assert!(matches!(
        verified.verification,
        veylora_device::setup::VerificationState::Verified { .. }
    ));
    assert_eq!(verified.content_hash, created.content_hash);
    // The earlier states remain on record.
    assert_eq!(ws.setup_history(&created.content_hash).unwrap().len(), 2);
}

#[test]
fn verified_setups_no_longer_report_the_unverified_finding() {
    let f = fixture_job(|_| {});
    let created = f.ws.create_job(&f.spec).unwrap();
    let has = || {
        f.ws.preflight(created.job_id, &PreflightOptions::default())
            .unwrap()
            .findings
            .iter()
            .any(|x| x.code == ErrorCode::UnverifiedDeviceSetup)
    };
    assert!(has());
    f.ws.verify_setup(&f.spec.setup, evidence_for(&f.ws, &f.spec.setup))
        .unwrap();
    assert!(!has());
}

#[test]
fn review_flags_ambiguous_and_interrupted_attempts_without_changing_them() {
    use veylora_domain::{
        ids::AttemptId,
        job::{AttemptState, SubmissionAttempt},
    };
    use veylora_persistence::{FileJobStore, JobStore};
    let f = fixture_job(|_| {});
    let created = f.ws.create_job(&f.spec).unwrap();
    let id = created.job_id;
    assert!(!f.ws.review(id).unwrap().requires_review);

    let store = FileJobStore::open(f.ws.root().join("jobs")).unwrap();
    let attempt = |attempt_id, sequence, state| SubmissionAttempt {
        attempt_id,
        job_id: id,
        snapshot_hash: created.snapshot_hash,
        sequence,
        state,
        bytes_written: 0,
        note: "fixture".into(),
        recorded_at: "2026-11-15T10:00:00Z".into(),
    };
    let a = AttemptId::new();
    store
        .append_attempt(attempt(a, 1, AttemptState::Submitting))
        .unwrap();
    // A crash can leave an attempt open; nothing here may assume it finished.
    let open = f.ws.review(id).unwrap();
    assert!(open.requires_review);
    store
        .append_attempt(attempt(a, 2, AttemptState::Ambiguous))
        .unwrap();
    let r = f.ws.review(id).unwrap();
    assert!(r.requires_review);
    assert_eq!(r.attempts.len(), 1);
    assert!(r.attempts[0].requires_review);
    // Reviewing is read only.
    assert_eq!(f.ws.review(id).unwrap().attempts.len(), 1);
    assert_eq!(f.ws.report(id).unwrap().submission_attempts.len(), 1);
}

#[test]
fn reviewing_an_unknown_job_is_an_error_not_an_empty_report() {
    let f = fixture_job(|_| {});
    assert!(f.ws.review(veylora_domain::ids::JobId::new()).is_err());
}

#[test]
fn a_job_with_no_render_has_nothing_to_inspect() {
    let f = fixture_job(|_| {});
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    assert!(f.ws.inspect_latest_run(id).unwrap().is_none());
}

#[test]
fn the_latest_completed_run_is_inspected_from_its_retained_bytes() {
    let f = fixture_job(|_| {});
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    f.ws.render(id, &RenderOptions::default()).unwrap();
    let second = f.ws.render(id, &RenderOptions::default()).unwrap();
    // A run that never wrote a manifest did not complete and is not a render.
    std::fs::create_dir_all(
        f.ws.root()
            .join("artifacts")
            .join(id.to_string())
            .join("run-9999"),
    )
    .unwrap();

    let latest =
        f.ws.inspect_latest_run(id)
            .unwrap()
            .expect("a completed run");
    assert_eq!(
        latest.run,
        second.dir.file_name().unwrap().to_str().unwrap()
    );
    assert!(latest.report.verified, "{:?}", latest.report.problems);
    assert_eq!(latest.report.manifest, second.manifest);
    assert_eq!(latest.report.plane("logical-w.f32").unwrap().nonzero, 3);
}

#[test]
fn inspecting_the_latest_run_reports_tampering_instead_of_hiding_it() {
    let f = fixture_job(|_| {});
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    let out = f.ws.render(id, &RenderOptions::default()).unwrap();
    let file = out.dir.join("logical-w.f32");
    let mut bytes = std::fs::read(&file).unwrap();
    bytes[0] ^= 0xff;
    std::fs::write(&file, bytes).unwrap();
    let latest = f.ws.inspect_latest_run(id).unwrap().unwrap();
    assert!(!latest.report.verified);
    assert!(latest
        .report
        .problems
        .iter()
        .any(|p| p.contains("logical-w.f32")));
}

#[test]
fn an_empty_workspace_lists_no_jobs() {
    let ws = Workspace::open(dir("ws-empty")).unwrap();
    assert!(ws.list_jobs().unwrap().is_empty());
}

#[test]
fn jobs_are_listed_newest_first_and_their_snapshots_can_be_read() {
    let f = fixture_job(|_| {});
    let mut older = f.spec.clone();
    older.created_at_ms = 1_000;
    let mut newer = f.spec.clone();
    newer.created_at_ms = 2_000;
    let a = f.ws.create_job(&older).unwrap().job_id;
    let b = f.ws.create_job(&newer).unwrap().job_id;
    assert_eq!(f.ws.list_jobs().unwrap(), vec![b, a]);
    let snap = f.ws.job_snapshot(a).unwrap();
    assert_eq!(snap.inputs().created_at_ms, 1_000);
    assert_eq!(snap.inputs().layout.inputs().placements.len(), 1);
}

#[test]
fn listing_ignores_stray_files_and_reading_an_unknown_job_is_an_error() {
    let f = fixture_job(|_| {});
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    std::fs::write(f.ws.root().join("jobs").join("jobs").join("notes.txt"), "x").unwrap();
    std::fs::write(
        f.ws.root()
            .join("jobs")
            .join("jobs")
            .join("not-a-uuid.json"),
        "{}",
    )
    .unwrap();
    std::fs::write(
        f.ws.root()
            .join("jobs")
            .join("jobs")
            .join("00000000-0000-4000-8000-000000000001.txt"),
        "x",
    )
    .unwrap();
    assert_eq!(f.ws.list_jobs().unwrap(), vec![id]);
    let unknown: veylora_domain::ids::JobId =
        "00000000-0000-4000-8000-000000000000".parse().unwrap();
    assert!(f.ws.job_snapshot(unknown).is_err());
}

#[test]
fn an_export_copies_the_verified_run_to_a_new_folder_and_re_verifies_it() {
    let f = fixture_job(|_| {});
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    assert!(f.ws.exports(id).unwrap().is_empty());
    assert!(f.ws.export_latest_run(id).is_err(), "nothing rendered yet");
    let out = f.ws.render(id, &RenderOptions::default()).unwrap();
    let exported = f.ws.export_latest_run(id).unwrap();
    assert!(exported.dir.starts_with(f.ws.root().join("exports")));
    assert!(exported.dir.is_dir());
    let again = inspect_artifacts(&exported.dir).unwrap();
    assert!(again.verified, "{:?}", again.problems);
    assert_eq!(again.manifest, out.manifest);
    assert_eq!(
        exported.files,
        1 + out.manifest.logical_planes.files.len()
            + out.manifest.physical_planes.files.len()
            + out.manifest.screened_planes.files.len()
    );
    assert_eq!(f.ws.exports(id).unwrap(), vec![exported.dir.clone()]);
    // Exports never touch the job's history: nothing was sent anywhere.
    assert!(f.ws.report(id).unwrap().submission_attempts.is_empty());
}

#[test]
fn an_export_is_never_overwritten_and_a_damaged_run_is_not_exported() {
    let f = fixture_job(|_| {});
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    let out = f.ws.render(id, &RenderOptions::default()).unwrap();
    let first = f.ws.export_latest_run(id).unwrap();
    let err = f.ws.export_latest_run(id).unwrap_err().to_string();
    assert!(err.contains("already exported"), "{err}");
    assert!(first.dir.is_dir());

    // A second, damaged run is refused and leaves no partial folder behind.
    let second = f.ws.render(id, &RenderOptions::default()).unwrap();
    let file = second.dir.join("logical-w.f32");
    let mut bytes = std::fs::read(&file).unwrap();
    bytes[0] ^= 0xff;
    std::fs::write(&file, bytes).unwrap();
    let err = f.ws.export_latest_run(id).unwrap_err().to_string();
    assert!(err.contains("logical-w.f32"), "{err}");
    assert!(
        err.contains("not exported"),
        "refused before any copy: {err}"
    );
    assert_eq!(f.ws.exports(id).unwrap().len(), 1);
    let leftovers: Vec<_> = std::fs::read_dir(f.ws.root().join("exports"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert!(
        leftovers.iter().all(|n| !n.ends_with(".partial")),
        "{leftovers:?}"
    );
    let _ = out;
}

#[test]
fn the_catalog_lists_imported_setups_modes_and_recipes() {
    let ws = Workspace::open(dir("ws-catalog")).unwrap();
    let c = ws.catalog().unwrap();
    assert!(c.setups.is_empty() && c.modes.is_empty() && c.recipes.is_empty());

    let s = setup();
    let m = mode(s.id());
    let r = recipe();
    let setup_hash = ws.import_setup(&s.to_record()).unwrap();
    let mode_hash = ws.import_mode(&m.to_record()).unwrap();
    let recipe_hash = ws.import_recipe(&r).unwrap();
    let c = ws.catalog().unwrap();
    assert_eq!(c.setups.len(), 1);
    assert_eq!(c.setups[0].hash, setup_hash);
    assert_eq!(c.setups[0].id, s.id().to_string());
    assert!(!c.setups[0].physically_verified);
    assert_eq!(c.modes.len(), 1);
    assert_eq!(c.modes[0].hash, mode_hash);
    assert_eq!(c.modes[0].setup_id, s.id().to_string());
    assert_eq!(c.recipes.len(), 1);
    assert_eq!(c.recipes[0].hash, recipe_hash);
    assert_eq!(c.recipes[0].revision, 1);
}

fn wide_job() -> (Fixture, veylora_domain::ids::JobId) {
    // 1 x 0.25 mm at 1270 dpi is about 50 x 13 device pixels, enough to need downsampling.
    let f = fixture_job(|l| {
        l.page_width_um = 1_000;
        l.page_height_um = 250;
        l.placements[0].width_um = 1_000;
        l.placements[0].height_um = 250;
    });
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    f.ws.render(
        id,
        &RenderOptions {
            layout: PreflightOptions {
                min_effective_ppi: 1,
                ..PreflightOptions::default()
            },
            ..RenderOptions::default()
        },
    )
    .unwrap();
    (f, id)
}

fn decode_gray(png: &[u8]) -> (u32, u32, Vec<u8>) {
    let mut r = ::png::Decoder::new(std::io::Cursor::new(png))
        .read_info()
        .unwrap();
    let mut buf = vec![0; r.output_buffer_size().unwrap()];
    let info = r.next_frame(&mut buf).unwrap();
    buf.truncate(info.buffer_size());
    (info.width, info.height, buf)
}

#[test]
fn a_plane_overview_is_a_bounded_gray_picture_of_the_retained_bytes() {
    let (f, id) = wide_job();
    let full = f.ws.plane_overview(id, "logical-w.f32", 2048).unwrap();
    assert_eq!(
        (full.width, full.height, full.scale),
        (full.source_width, full.source_height, 1)
    );
    let (w, h, pixels) = decode_gray(&full.png);
    assert_eq!((w as usize, h as usize), (full.width, full.height));
    assert!(pixels.contains(&255), "opaque alpha is full white coverage");
    assert!(pixels.contains(&0), "the transparent edge has none");

    let small = f.ws.plane_overview(id, "logical-w.f32", 10).unwrap();
    assert!(small.scale > 1);
    assert!(small.width <= 10 && small.height <= 10);
    assert_eq!(small.source_width, full.source_width);
    let (sw, sh, spx) = decode_gray(&small.png);
    assert_eq!((sw as usize, sh as usize), (small.width, small.height));
    assert_eq!(spx.len(), small.width * small.height);

    // A plane with no ink is shown as black, not as noise.
    let cyan = f.ws.plane_overview(id, "logical-c.f32", 2048).unwrap();
    assert!(decode_gray(&cyan.png).2.iter().all(|p| *p == 0));
}

#[test]
fn a_loupe_window_returns_exact_values_clipped_to_the_plane() {
    let (f, id) = wide_job();
    let full = f.ws.plane_overview(id, "logical-w.f32", 2048).unwrap();
    let win = f.ws.plane_window(id, "logical-w.f32", 0, 0, 4, 2).unwrap();
    assert_eq!((win.width, win.height), (4, 2));
    assert_eq!(win.values.len(), 8);
    assert!(win.values.iter().all(|v| (0.0..=1.0).contains(v)));
    // Values agree with the overview at 1:1.
    let (_, _, px) = decode_gray(&full.png);
    for (i, v) in win.values.iter().enumerate() {
        let (x, y) = (i % 4, i / 4);
        assert_eq!(px[y * full.width + x], (v * 255.0).round() as u8);
    }
    // A window hanging off the edge is clipped, never padded with invented values.
    let edge =
        f.ws.plane_window(
            id,
            "logical-w.f32",
            full.source_width - 2,
            full.source_height - 1,
            10,
            10,
        )
        .unwrap();
    assert_eq!((edge.width, edge.height), (2, 1));
    assert_eq!(
        (edge.x, edge.y),
        (full.source_width - 2, full.source_height - 1)
    );
    assert!(f
        .ws
        .plane_window(id, "logical-w.f32", full.source_width, 0, 4, 4)
        .is_err());
}

#[test]
fn plane_reads_are_bounded_and_refuse_unknown_or_damaged_files() {
    let (f, id) = wide_job();
    assert!(f.ws.plane_window(id, "logical-w.f32", 0, 0, 65, 1).is_err());
    assert!(f.ws.plane_window(id, "logical-w.f32", 0, 0, 0, 1).is_err());
    assert!(f.ws.plane_overview(id, "logical-w.f32", 0).is_err());
    assert!(f.ws.plane_overview(id, "logical-w.f32", 100_000).is_err());
    assert!(f.ws.plane_overview(id, "../manifest.json", 100).is_err());
    assert!(f.ws.plane_overview(id, "not-a-plane.f32", 100).is_err());

    let dir = f.ws.root().join("artifacts").join(id.to_string());
    let run = std::fs::read_dir(&dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let file = run.join("logical-w.f32");
    let mut bytes = std::fs::read(&file).unwrap();
    bytes[0] ^= 0xff;
    std::fs::write(&file, bytes).unwrap();
    let err =
        f.ws.plane_overview(id, "logical-w.f32", 100)
            .unwrap_err()
            .to_string();
    assert!(
        err.contains("logical-w.f32") && err.contains("manifest"),
        "{err}"
    );
    assert!(f.ws.plane_window(id, "logical-w.f32", 0, 0, 2, 2).is_err());
}

#[test]
fn a_u16_screened_plane_is_scaled_by_the_dot_level() {
    let (f, id) = wide_job();
    let latest = f.ws.inspect_latest_run(id).unwrap().unwrap();
    let screened = &latest.report.manifest.screened_planes.files[0].name;
    let win = f.ws.plane_window(id, screened, 0, 0, 4, 1).unwrap();
    assert!(win.values.iter().all(|v| (0.0..=1.0).contains(v)));
    assert!(f.ws.plane_overview(id, screened, 100).is_ok());
}
