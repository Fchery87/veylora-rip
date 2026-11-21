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
    assert!(view.raster_shown, "a verified run is drawn from its bytes");
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
fn a_slot_letter_selects_the_file_its_setup_binds_to_it() {
    let (ws, id, _dir) = rendered_job();
    let retained = retained_for_job(&ws, &id).unwrap().unwrap();
    for (stage, ext, prefix) in [
        (StageId::Slots, "f32", "physical"),
        (StageId::Screened, "u16", "screened"),
    ] {
        for (plane, protocol, slot) in [
            (Plane::C, 10, 0),
            (Plane::K, 13, 3),
            (Plane::W1, 14, 4),
            (Plane::W2, 15, 5),
        ] {
            let view = inspect::present_retained(&retained, &select(stage, &[plane])).unwrap();
            assert_eq!(
                view.raster_files,
                vec![format!("{prefix}-p{protocol}.{ext}")],
                "{plane:?} on {stage:?}"
            );
            let row = value(&view, &format!("Slot {}", plane.name()));
            for part in [
                format!("protocol channel {protocol}"),
                format!("slot {slot}"),
                format!("tank t{slot}"),
                format!("head h{slot}"),
            ] {
                assert!(row.contains(&part), "{row}");
            }
        }
    }
}

#[test]
fn with_no_slot_chosen_files_are_listed_by_name_and_no_letter_is_invented() {
    let (ws, id, _dir) = rendered_job();
    let retained = retained_for_job(&ws, &id).unwrap().unwrap();
    for (stage, prefix) in [
        (StageId::Slots, "physical-"),
        (StageId::Screened, "screened-"),
    ] {
        let view = inspect::present_retained(&retained, &select(stage, &[])).unwrap();
        let rows: Vec<&str> = view.metadata.iter().map(|r| r.label.as_str()).collect();
        assert!(
            rows.iter().filter(|l| l.starts_with(prefix)).count() == 6,
            "{rows:?}"
        );
        assert!(rows.iter().all(|l| !l.starts_with("Slot ")), "{rows:?}");
        assert_eq!(view.raster_files.len(), 6);
    }
}

#[test]
fn the_note_says_where_slot_letters_come_from_and_that_they_are_not_observed() {
    let (ws, id, _dir) = rendered_job();
    let retained = retained_for_job(&ws, &id).unwrap().unwrap();
    let view = inspect::present_retained(&retained, &select(StageId::Slots, &[Plane::K])).unwrap();
    let note = &view.metadata_note;
    assert!(note.contains("channel map version 1"), "{note}");
    assert!(note.contains("not physically verified"), "{note}");
    assert!(note.contains("as recorded"), "{note}");
    // The logical stage needs no map and does not talk about one.
    let logical =
        inspect::present_retained(&retained, &select(StageId::Logical, &[Plane::K])).unwrap();
    assert!(
        !logical.metadata_note.contains("channel map"),
        "{}",
        logical.metadata_note
    );
}

fn composite_selection(stage: StageId) -> InspectSelection {
    InspectSelection {
        stage,
        planes: Vec::new(),
        composite: true,
    }
}

#[test]
fn a_verified_run_offers_a_labeled_composite_of_its_logical_planes() {
    let (ws, id, _dir) = rendered_job();
    let retained = retained_for_job(&ws, &id).unwrap().unwrap();
    let view =
        inspect::present_retained(&retained, &composite_selection(StageId::Logical)).unwrap();
    assert!(view.composite.pressed && view.selection.composite);
    assert_eq!(view.raster_files, vec!["composite".to_owned()]);
    assert!(view.raster_shown && view.panels.is_empty());
    assert!(
        view.chip
            .contains("Composite simulation of retained planes"),
        "{}",
        view.chip
    );
    assert!(view.chip.contains("retained artifact"), "{}", view.chip);
    // It says what the picture is not.
    let note = view.placeholder.to_lowercase();
    assert!(
        note.contains("not color managed") && note.contains("not what the print"),
        "{note}"
    );
}

#[test]
fn the_composite_covers_only_the_logical_stage() {
    let (ws, id, _dir) = rendered_job();
    let retained = retained_for_job(&ws, &id).unwrap().unwrap();
    for stage in [StageId::Slots, StageId::Screened] {
        let err = inspect::present_retained(&retained, &composite_selection(stage)).unwrap_err();
        assert!(err.contains("Logical"), "{err}");
    }
}

#[test]
fn a_damaged_run_is_not_composited() {
    let (ws, id, dir) = rendered_job();
    let run = std::fs::read_dir(ws.root().join("artifacts").join(&id))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let file = run.join("logical-w.f32");
    let mut bytes = std::fs::read(&file).unwrap();
    bytes[0] ^= 0xff;
    std::fs::write(&file, bytes).unwrap();
    let _ = dir;
    let retained = retained_for_job(&ws, &id).unwrap().unwrap();
    let view =
        inspect::present_retained(&retained, &composite_selection(StageId::Logical)).unwrap();
    assert!(!view.raster_shown && view.raster_files.is_empty());
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
