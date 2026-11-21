//! The Inspect tab reading a real retained render, not fixture values.
#[path = "../../veylora-workflow/tests/common.rs"]
mod common;

use common::*;
use std::collections::BTreeMap;
use veylora_app_lib::artifacts::retained_for_job;
use veylora_app_lib::screens::inspect;
use veylora_app_lib::view_models::{InspectSelection, Plane, PreviewSource, StageId};
use veylora_workflow::{JobSpec, LayoutSpec, PlacementSpec, RenderOptions, Workspace};

fn created_job() -> (Workspace, veylora_domain::ids::JobId) {
    let ws = Workspace::open(dir("app-ws")).unwrap();
    let s = setup();
    let m = mode(s.id());
    let setup_hash = ws.import_setup(&s.to_record()).unwrap();
    let mode_hash = ws.import_mode(&m.to_record()).unwrap();
    let recipe_hash = ws.import_recipe(&recipe()).unwrap();
    let art = ws.import_asset(&fixture("alpha-ramp.png")).unwrap();
    let spec = JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1735000000000,
        setup: setup_hash,
        mode: mode_hash,
        recipe: recipe_hash,
        assets: BTreeMap::from([("art".to_string(), art)]),
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
                rotation: veylora_domain::layout::Rotation::Deg0,
                crop: None,
            }],
        },
    };
    let id = ws.create_job(&spec).unwrap().job_id;
    (ws, id)
}

fn rendered_job() -> (Workspace, String, std::path::PathBuf) {
    let (ws, id) = created_job();
    let out = ws.render(id, &RenderOptions::default()).unwrap();
    (ws, id.to_string(), out.dir)
}

fn select(stage: StageId, planes: &[Plane]) -> InspectSelection {
    InspectSelection {
        stage,
        planes: planes.to_vec(),
        composite: false,
    }
}

fn value<'a>(view: &'a veylora_app_lib::view_models::InspectView, label: &str) -> &'a str {
    &view
        .metadata
        .iter()
        .find(|r| r.label == label)
        .unwrap_or_else(|| panic!("no row {label}: {:?}", view.metadata))
        .value
}

#[test]
fn a_created_but_unrendered_job_has_no_retained_artifact() {
    let (ws, id) = created_job();
    assert_eq!(retained_for_job(&ws, &id.to_string()).unwrap(), None);
}

#[test]
fn unknown_or_malformed_job_ids_are_errors_not_empty_results() {
    let ws = Workspace::open(dir("app-empty")).unwrap();
    assert!(retained_for_job(&ws, "not-a-uuid").is_err());
    let missing = "00000000-0000-4000-8000-000000000000";
    assert!(retained_for_job(&ws, missing).is_err());
}

#[test]
fn the_view_is_labeled_retained_and_shows_values_read_from_the_bytes() {
    let (ws, id, _dir) = rendered_job();
    let retained = retained_for_job(&ws, &id).unwrap().expect("rendered");
    let view =
        inspect::present_retained(&retained, &select(StageId::Logical, &[Plane::W])).unwrap();
    assert_eq!(view.source, PreviewSource::Retained);
    assert!(view.chip.contains("retained artifact"));
    assert!(view.chip.ends_with("Not physical output"));
    assert!(!view.chip.contains("Fixture"));
    assert!(!view.raster_shown, "statistics only; no raster is drawn");
    // Alpha ramp: three of four pixels carry white; this is read from logical-w.f32.
    assert!(
        value(&view, "Coverage W").contains("3 of 4"),
        "{:?}",
        view.metadata
    );
    assert_eq!(value(&view, "Pixels"), "4 x 1");
    assert_eq!(
        value(&view, "Verification"),
        "Verified against the manifest"
    );
    assert_eq!(value(&view, "Evidence"), "software_only");
    assert_eq!(value(&view, "Job"), id);
    assert!(value(&view, "Recipe hash").len() >= 64);
    assert!(view.metadata.iter().all(|r| !r.value.contains("fixture")));
    assert!(!view.metadata_note.to_lowercase().contains("fixture"));
}

#[test]
fn another_plane_reads_its_own_file() {
    let (ws, id, _dir) = rendered_job();
    let retained = retained_for_job(&ws, &id).unwrap().unwrap();
    let view =
        inspect::present_retained(&retained, &select(StageId::Logical, &[Plane::C])).unwrap();
    assert!(
        value(&view, "Coverage C").starts_with("0 of 4"),
        "{:?}",
        view.metadata
    );
}

#[test]
fn a_tampered_artifact_is_shown_as_unverified_with_the_problem_named() {
    let (ws, id, dir) = rendered_job();
    let file = dir.join("logical-w.f32");
    let mut bytes = std::fs::read(&file).unwrap();
    bytes[0] ^= 0xff;
    std::fs::write(&file, bytes).unwrap();
    let retained = retained_for_job(&ws, &id).unwrap().unwrap();
    let view =
        inspect::present_retained(&retained, &select(StageId::Logical, &[Plane::W])).unwrap();
    assert!(
        value(&view, "Verification").starts_with("FAILED"),
        "{:?}",
        view.metadata
    );
    assert!(value(&view, "Verification").contains("logical-w.f32"));
    assert!(view.metadata_note.contains("failed verification"));
}

#[test]
fn physical_stages_list_files_without_guessing_slot_letters() {
    let (ws, id, _dir) = rendered_job();
    let retained = retained_for_job(&ws, &id).unwrap().unwrap();
    for stage in [StageId::Slots, StageId::Screened] {
        let view = inspect::present_retained(&retained, &select(stage, &[Plane::K])).unwrap();
        let rows: Vec<&str> = view.metadata.iter().map(|r| r.label.as_str()).collect();
        assert!(
            rows.iter()
                .any(|l| l.starts_with("physical-") || l.starts_with("screened-")),
            "{rows:?}"
        );
        assert!(view.metadata_note.contains("channel map"));
        assert!(rows.iter().all(|l| *l != "Coverage K"));
    }
}

#[test]
fn composite_is_refused_because_no_composite_of_retained_planes_exists() {
    let (ws, id, _dir) = rendered_job();
    let retained = retained_for_job(&ws, &id).unwrap().unwrap();
    let composite = InspectSelection {
        stage: StageId::Logical,
        planes: Vec::new(),
        composite: true,
    };
    let err = inspect::present_retained(&retained, &composite).unwrap_err();
    assert!(err.contains("Composite"), "{err}");
}

#[test]
fn garment_and_film_are_not_inspect_stages() {
    let (ws, id, _dir) = rendered_job();
    let retained = retained_for_job(&ws, &id).unwrap().unwrap();
    assert!(inspect::present_retained(&retained, &select(StageId::Film, &[])).is_err());
}

#[test]
fn each_stage_shows_only_its_own_files() {
    let (ws, id, _dir) = rendered_job();
    let retained = retained_for_job(&ws, &id).unwrap().unwrap();
    for (stage, own, others) in [
        (StageId::Slots, "physical-", ["screened-", "logical-"]),
        (StageId::Screened, "screened-", ["physical-", "logical-"]),
    ] {
        let view = inspect::present_retained(&retained, &select(stage, &[])).unwrap();
        let files: Vec<&str> = view
            .metadata
            .iter()
            .map(|r| r.label.as_str())
            .filter(|l| l.ends_with(".f32") || l.ends_with(".u16"))
            .collect();
        assert!(!files.is_empty());
        assert!(files.iter().all(|f| f.starts_with(own)), "{files:?}");
        assert!(files
            .iter()
            .all(|f| !others.iter().any(|o| f.starts_with(o))));
    }
}

#[test]
fn the_snapshot_hash_shown_is_the_jobs_own() {
    let (ws, id, _dir) = rendered_job();
    let retained = retained_for_job(&ws, &id).unwrap().unwrap();
    let view =
        inspect::present_retained(&retained, &select(StageId::Logical, &[Plane::W])).unwrap();
    let job: veylora_domain::ids::JobId = id.parse().unwrap();
    assert_eq!(
        value(&view, "Snapshot hash"),
        ws.report(job).unwrap().snapshot_hash.to_string()
    );
}

#[test]
fn a_stage_the_run_did_not_retain_is_disabled_and_refused() {
    let (ws, id, _dir) = rendered_job();
    let mut retained = retained_for_job(&ws, &id).unwrap().unwrap();
    retained.planes.retain(|p| p.file.starts_with("logical-"));
    let err = inspect::present_retained(&retained, &select(StageId::Slots, &[])).unwrap_err();
    assert!(err.contains("retained no planes"), "{err}");
    let view =
        inspect::present_retained(&retained, &select(StageId::Logical, &[Plane::K])).unwrap();
    let slots = view.stages.iter().find(|s| s.id == StageId::Slots).unwrap();
    assert!(slots.disabled && slots.reason.is_some());
    let logical = view
        .stages
        .iter()
        .find(|s| s.id == StageId::Logical)
        .unwrap();
    assert!(!logical.disabled);
}
