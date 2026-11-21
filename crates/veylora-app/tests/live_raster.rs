//! Drawing retained planes: pictures and loupe values come from verified bytes only.
#[path = "../../veylora-workflow/tests/common.rs"]
mod common;

use common::*;
use std::collections::BTreeMap;
use veylora_app_lib::artifacts::retained_for_job;
use veylora_app_lib::live;
use veylora_app_lib::screens::inspect;
use veylora_app_lib::view_models::{InspectSelection, Plane, StageId};
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

fn rendered() -> (Workspace, String) {
    let (ws, id) = created_job();
    ws.render(id, &RenderOptions::default()).unwrap();
    (ws, id.to_string())
}

#[test]
fn the_retained_view_lists_the_files_it_can_draw_and_says_so() {
    let (ws, id) = rendered();
    let retained = retained_for_job(&ws, &id).unwrap().unwrap();
    let sel = InspectSelection {
        stage: StageId::Logical,
        planes: vec![Plane::W],
        composite: false,
    };
    let view = inspect::present_retained(&retained, &sel).unwrap();
    assert_eq!(view.raster_files, vec!["logical-w.f32".to_owned()]);
    assert!(view.raster_shown);
    assert!(
        view.panels.is_empty(),
        "no 'not built yet' stubs once drawn"
    );
    assert!(!view.placeholder.contains("not built"));
}

#[test]
fn a_plane_image_is_a_png_data_url_with_its_scale() {
    let (ws, id) = rendered();
    let img = live::plane_image(&ws, &id, "logical-w.f32", 512).unwrap();
    assert!(img.data_url.starts_with("data:image/png;base64,"));
    assert_eq!((img.source_width, img.scale), (img.width, 1));
    let b64 = img.data_url.trim_start_matches("data:image/png;base64,");
    let bytes = live::base64_decode(b64).unwrap();
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn a_composite_image_is_a_png_data_url_and_refuses_what_it_cannot_verify() {
    let (ws, id) = rendered();
    let img = live::composite_image(&ws, &id, 512).unwrap();
    assert!(img.data_url.starts_with("data:image/png;base64,"));
    assert_eq!((img.source_width, img.scale), (img.width, 1));
    assert!(live::composite_image(&ws, "nope", 512).is_err());
    assert!(live::composite_image(&ws, &id, 0).is_err());
}

#[test]
fn loupe_values_are_exact_and_bounded() {
    let (ws, id) = rendered();
    let w = live::plane_window(&ws, &id, "logical-w.f32", 0, 0, 4, 1).unwrap();
    assert_eq!(w.values.len(), w.width * w.height);
    assert!(w.values.iter().all(|v| (0.0..=1.0).contains(v)));
    assert!(live::plane_window(&ws, &id, "logical-w.f32", 0, 0, 1000, 1).is_err());
    assert!(live::plane_window(&ws, &id, "../x", 0, 0, 1, 1).is_err());
}

#[test]
fn unknown_jobs_and_unrendered_jobs_are_refused() {
    let ws = Workspace::open(dir("live-raster-empty")).unwrap();
    assert!(live::plane_image(&ws, "nope", "logical-w.f32", 100).is_err());
    assert!(live::plane_image(
        &ws,
        "00000000-0000-4000-8000-000000000000",
        "logical-w.f32",
        100
    )
    .is_err());
}

#[test]
fn base64_matches_known_vectors() {
    for (raw, enc) in [
        (&b""[..], ""),
        (b"f", "Zg=="),
        (b"fo", "Zm8="),
        (b"foo", "Zm9v"),
        (b"foob", "Zm9vYg=="),
        (b"fooba", "Zm9vYmE="),
        (b"foobar", "Zm9vYmFy"),
    ] {
        assert_eq!(live::base64_encode(raw), enc);
        assert_eq!(live::base64_decode(enc).unwrap(), raw);
    }
    assert!(live::base64_decode("Zg=").is_err());
}

#[test]
fn a_damaged_run_is_not_drawn() {
    let (ws, id) = rendered();
    let job: veylora_domain::ids::JobId = id.parse().unwrap();
    let run = ws.root().join("artifacts").join(id.clone());
    let run = std::fs::read_dir(run)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let file = run.join("logical-w.f32");
    let mut bytes = std::fs::read(&file).unwrap();
    bytes[0] ^= 0xff;
    std::fs::write(&file, bytes).unwrap();
    let _ = job;
    let retained = retained_for_job(&ws, &id).unwrap().unwrap();
    let sel = InspectSelection {
        stage: StageId::Logical,
        planes: vec![Plane::W],
        composite: false,
    };
    let view = inspect::present_retained(&retained, &sel).unwrap();
    assert!(!view.raster_shown && view.raster_files.is_empty());
    assert!(live::plane_image(&ws, &id, "logical-w.f32", 100).is_err());
}
