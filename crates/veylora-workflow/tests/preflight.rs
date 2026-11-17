mod common;
use common::*;
use std::collections::BTreeMap;
use veylora_domain::{
    errors::ErrorCode,
    layout::Rotation,
    lifecycle::JobState,
    preflight::{Finding, Severity},
    provenance::ContentHash,
    recipe::{PrintRecipe, VersionedReference},
};
use veylora_workflow::{
    JobSpec, LayoutSpec, PlacementSpec, PreflightOptions, RenderOptions, WorkflowError, Workspace,
};

struct Env {
    ws: Workspace,
    spec: JobSpec,
    asset: ContentHash,
}

fn env_with_recipe(recipe: PrintRecipe) -> Env {
    let ws = Workspace::open(dir("pf")).unwrap();
    let s = setup();
    let m = mode(s.id());
    let asset = ws.import_asset(&fixture("alpha-ramp.png")).unwrap();
    let spec = JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1735000000000,
        setup: ws.import_setup(&s.to_record()).unwrap(),
        mode: ws.import_mode(&m.to_record()).unwrap(),
        recipe: ws.import_recipe(&recipe).unwrap(),
        assets: BTreeMap::from([("art".to_string(), asset)]),
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
    Env { ws, spec, asset }
}

fn env() -> Env {
    env_with_recipe(recipe())
}

fn blocking(findings: &[Finding], code: ErrorCode) -> Vec<&Finding> {
    findings
        .iter()
        .filter(|f| f.code == code && f.severity == Severity::Blocking)
        .collect()
}

fn asset_file(e: &Env) -> std::path::PathBuf {
    e.ws.root().join("assets").join(e.asset.to_string())
}

#[test]
fn a_clean_job_has_no_blocking_findings() {
    let e = env();
    let id = e.ws.create_job(&e.spec).unwrap().job_id;
    let r = e.ws.preflight(id, &PreflightOptions::default()).unwrap();
    assert!(r.can_render, "{:?}", r.findings);
}

#[test]
fn a_source_that_has_gone_missing_blocks_with_the_asset_named() {
    let e = env();
    let id = e.ws.create_job(&e.spec).unwrap().job_id;
    std::fs::remove_file(asset_file(&e)).unwrap();
    let r = e.ws.preflight(id, &PreflightOptions::default()).unwrap();
    let found = blocking(&r.findings, ErrorCode::MissingAsset);
    assert_eq!(found.len(), 1, "{:?}", r.findings);
    assert!(found[0].subject.is_some());
    assert!(!r.can_render);
    // The refusal is a finding, so the job is rejected at preflight rather than failing a render.
    assert_eq!(e.ws.state(id).unwrap(), JobState::PreflightFailed);
    assert!(matches!(
        e.ws.render(id, &RenderOptions::default()),
        Err(WorkflowError::PreflightBlocked(_))
    ));
}

#[test]
fn a_source_that_was_altered_is_not_trusted() {
    let e = env();
    let id = e.ws.create_job(&e.spec).unwrap().job_id;
    let mut bytes = std::fs::read(asset_file(&e)).unwrap();
    bytes[10] ^= 0xff;
    std::fs::write(asset_file(&e), bytes).unwrap();
    let r = e.ws.preflight(id, &PreflightOptions::default()).unwrap();
    assert_eq!(
        blocking(&r.findings, ErrorCode::UnsupportedInput).len(),
        1,
        "{:?}",
        r.findings
    );
    assert!(blocking(&r.findings, ErrorCode::MissingAsset).is_empty());
}

#[test]
fn a_recipe_that_names_a_profile_is_refused_instead_of_ignored() {
    let mut inputs = recipe().inputs().clone();
    inputs.color.profile = Some(VersionedReference {
        version: 1,
        content_hash: hash(b"some profile"),
    });
    let e = env_with_recipe(PrintRecipe::new(inputs).unwrap());
    let id = e.ws.create_job(&e.spec).unwrap().job_id;
    let r = e.ws.preflight(id, &PreflightOptions::default()).unwrap();
    assert_eq!(
        blocking(&r.findings, ErrorCode::UnsupportedProfile).len(),
        1,
        "{:?}",
        r.findings
    );
    assert!(!r.can_render);
}

#[test]
fn the_page_pixel_limit_is_checked_before_rendering() {
    let e = env();
    let id = e.ws.create_job(&e.spec).unwrap().job_id;
    // 80 x 20 um at 20 um per pixel is 4 x 1 = 4 pixels.
    let at_limit = PreflightOptions {
        max_page_pixels: 4,
        ..Default::default()
    };
    assert!(e.ws.preflight(id, &at_limit).unwrap().can_render);
    let over = PreflightOptions {
        max_page_pixels: 3,
        ..Default::default()
    };
    let r = e.ws.preflight(id, &over).unwrap();
    let found = blocking(&r.findings, ErrorCode::UnsupportedMediaSize);
    assert!(
        found
            .iter()
            .any(|f| f.subject.as_deref() == Some("layout.page_pixels")),
        "{:?}",
        r.findings
    );
}

#[test]
fn the_disk_budget_must_hold_the_estimated_artifacts() {
    let e = env();
    let id = e.ws.create_job(&e.spec).unwrap().job_id;
    // The test setup has six channels bound.
    let need = veylora_render::estimate_artifact_bytes(4, 6);
    let exact = PreflightOptions {
        disk_budget_bytes: Some(need),
        ..Default::default()
    };
    assert!(e.ws.preflight(id, &exact).unwrap().can_render);
    let short = PreflightOptions {
        disk_budget_bytes: Some(need - 1),
        ..Default::default()
    };
    let r = e.ws.preflight(id, &short).unwrap();
    assert_eq!(
        blocking(&r.findings, ErrorCode::InsufficientDiskSpace).len(),
        1,
        "{:?}",
        r.findings
    );
    // The budget is also enforced while writing, not only estimated.
    let render = RenderOptions {
        layout: short,
        ..Default::default()
    };
    assert!(e.ws.render(id, &render).is_err());
    assert!(e.ws.report(id).unwrap().renders.is_empty());
}

#[test]
fn the_limits_in_force_are_echoed_in_the_report() {
    let e = env();
    let id = e.ws.create_job(&e.spec).unwrap().job_id;
    let opts = PreflightOptions {
        max_page_pixels: 1234,
        disk_budget_bytes: Some(1 << 40),
        ..Default::default()
    };
    let r = e.ws.preflight(id, &opts).unwrap();
    assert_eq!(r.policy, opts);
}

#[test]
fn a_missing_mask_blocks_like_a_missing_source() {
    let mut e = env();
    let mask = e.ws.import_asset(&gray_png(2, 1, &[0, 255])).unwrap();
    e.spec.masks.insert("m".into(), mask);
    let id = e.ws.create_job(&e.spec).unwrap().job_id;
    assert!(
        e.ws.preflight(id, &PreflightOptions::default())
            .unwrap()
            .can_render
    );
    std::fs::remove_file(e.ws.root().join("assets").join(mask.to_string())).unwrap();
    let r = e.ws.preflight(id, &PreflightOptions::default()).unwrap();
    let found = blocking(&r.findings, ErrorCode::MissingAsset);
    assert_eq!(found.len(), 1, "{:?}", r.findings);
    assert!(found[0].subject.as_deref().unwrap().starts_with("mask:"));
}
