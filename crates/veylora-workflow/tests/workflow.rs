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
