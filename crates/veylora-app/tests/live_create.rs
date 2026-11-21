//! Creating a job from the app: choices come from what is stored, and the job is built by the workflow.
#[path = "../../veylora-workflow/tests/common.rs"]
mod common;

use common::*;
use veylora_app_lib::live::{self, NewJobRequest};
use veylora_app_lib::view_models::JobState;
use veylora_workflow::Workspace;

fn stocked() -> (Workspace, live::CreationOptions) {
    let ws = Workspace::open(dir("live-create")).unwrap();
    let s = setup();
    let m = mode(s.id());
    ws.import_setup(&s.to_record()).unwrap();
    ws.import_mode(&m.to_record()).unwrap();
    ws.import_recipe(&recipe()).unwrap();
    let options = live::creation_options(&ws).unwrap();
    (ws, options)
}

fn art_path() -> String {
    format!(
        "{}/../../fixtures/images/alpha-ramp.png",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn request(o: &live::CreationOptions) -> NewJobRequest {
    NewJobRequest {
        artwork_path: art_path(),
        setup: o.setups[0].hash.clone(),
        mode: o.modes[0].hash.clone(),
        recipe: o.recipes[0].hash.clone(),
        page_width_mm: 1.0,
        page_height_mm: 0.25,
        x_mm: 0.0,
        y_mm: 0.0,
        width_mm: 1.0,
        height_mm: 0.25,
        rotation_deg: 0,
        mirror: false,
    }
}

#[test]
fn an_empty_workspace_offers_nothing_to_choose() {
    let ws = Workspace::open(dir("live-create-empty")).unwrap();
    let o = live::creation_options(&ws).unwrap();
    assert!(o.setups.is_empty() && o.modes.is_empty() && o.recipes.is_empty());
}

#[test]
fn options_describe_what_is_stored_and_link_modes_to_their_setup() {
    let (_ws, o) = stocked();
    assert_eq!((o.setups.len(), o.modes.len(), o.recipes.len()), (1, 1, 1));
    assert!(!o.setups[0].verified);
    assert_eq!(o.modes[0].setup_id, o.setups[0].id);
    assert!(!o.recipes[0].retired);
    assert_eq!(o.setups[0].hash.len(), 64);
}

#[test]
fn a_valid_request_creates_a_draft_job_that_then_lists_and_opens() {
    let (ws, o) = stocked();
    let d = live::create_job(&ws, &request(&o)).unwrap();
    assert_eq!(d.summary.state, JobState::Draft);
    assert_eq!(d.placements.len(), 1);
    assert_eq!(
        (d.placements[0].width_mm, d.placements[0].height_mm),
        (1.0, 0.25)
    );
    assert_eq!(d.summary.sheet, "1 x 0.250 mm");
    let listed = live::job_summaries(&ws).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, d.summary.id);
    // The artwork is retained by content, so the same file can be used again without a second copy.
    let again = live::create_job(&ws, &request(&o)).unwrap();
    assert_ne!(again.summary.id, d.summary.id);
}

#[test]
fn bad_files_are_refused_with_a_reason() {
    let (ws, o) = stocked();
    let mut r = request(&o);
    r.artwork_path = format!("{}.missing", art_path());
    assert!(live::create_job(&ws, &r)
        .unwrap_err()
        .contains("Cannot read"));

    let dir = dir("live-create-notpng");
    std::fs::create_dir_all(&dir).unwrap();
    let txt = dir.join("notes.txt");
    std::fs::write(&txt, b"not an image").unwrap();
    r.artwork_path = txt.to_string_lossy().into_owned();
    assert!(live::create_job(&ws, &r).is_err());
    assert!(
        live::job_summaries(&ws).unwrap().is_empty(),
        "nothing is half-created"
    );
}

#[test]
fn a_directory_or_an_oversized_file_is_refused() {
    let (ws, o) = stocked();
    let mut r = request(&o);
    let folder = dir("live-create-dir");
    std::fs::create_dir_all(&folder).unwrap();
    r.artwork_path = folder.to_string_lossy().into_owned();
    assert!(live::create_job(&ws, &r).is_err());
}

#[test]
fn nonsense_sizes_are_refused_before_anything_is_created() {
    let (ws, o) = stocked();
    for mutate in [
        (|r: &mut NewJobRequest| r.page_width_mm = 0.0) as fn(&mut NewJobRequest),
        |r| r.page_height_mm = -1.0,
        |r| r.width_mm = f64::NAN,
        |r| r.height_mm = f64::INFINITY,
        |r| r.x_mm = f64::NAN,
        |r| r.page_width_mm = 5_000_000.0,
        |r| r.rotation_deg = 45,
    ] {
        let mut r = request(&o);
        mutate(&mut r);
        assert!(live::create_job(&ws, &r).is_err());
    }
    assert!(live::job_summaries(&ws).unwrap().is_empty());
}

#[test]
fn unknown_or_mismatched_records_are_refused() {
    let (ws, o) = stocked();
    let mut r = request(&o);
    r.setup = "0".repeat(64);
    assert!(live::create_job(&ws, &r).is_err());
    let mut r = request(&o);
    r.recipe = "not-a-hash".into();
    assert!(live::create_job(&ws, &r).unwrap_err().contains("hash"));
    // A mode that belongs to another setup cannot be combined with this one.
    let other = setup();
    ws.import_setup(&other.to_record()).unwrap();
    let o2 = live::creation_options(&ws).unwrap();
    let mut r = request(&o2);
    r.setup = o2
        .setups
        .iter()
        .find(|s| s.id != o2.modes[0].setup_id)
        .unwrap()
        .hash
        .clone();
    assert!(live::create_job(&ws, &r).is_err());
}

#[test]
fn rotation_and_mirror_are_recorded() {
    let (ws, o) = stocked();
    let mut r = request(&o);
    r.rotation_deg = 90;
    r.width_mm = 0.25;
    r.height_mm = 1.0;
    r.page_width_mm = 1.0;
    r.page_height_mm = 1.0;
    r.mirror = true;
    let d = live::create_job(&ws, &r).unwrap();
    let id: veylora_domain::ids::JobId = d.summary.id.parse().unwrap();
    let layout = ws.job_snapshot(id).unwrap();
    let l = layout.inputs().layout.inputs();
    assert!(l.mirror);
    assert_eq!(
        l.placements[0].rotation,
        veylora_domain::layout::Rotation::Deg90
    );
}
