//! Changing a job's white settings makes a new job on a new recipe revision. A stored job and its
//! recipe are never altered.
mod common;

use common::*;
use std::collections::BTreeMap;
use veylora_domain::ids::JobId;
use veylora_domain::recipe::WhiteSource;
use veylora_workflow::{
    JobSpec, LayoutSpec, PlacementSpec, RenderOptions, WhiteEdit, WorkflowError, Workspace,
};

fn job() -> (Workspace, JobId) {
    let ws = Workspace::open(dir("white-edit")).unwrap();
    let s = setup();
    let m = mode(s.id());
    let spec = JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1735000000000,
        setup: ws.import_setup(&s.to_record()).unwrap(),
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
                rotation: veylora_domain::layout::Rotation::Deg0,
                crop: None,
            }],
        },
    };
    let id = ws.create_job(&spec).unwrap().job_id;
    (ws, id)
}

const LATER: u64 = 1_736_000_000_000;

#[test]
fn turning_white_off_makes_a_new_job_on_the_next_recipe_revision() {
    let (ws, id) = job();
    let before = ws.job_snapshot(id).unwrap();
    let edit = WhiteEdit {
        enabled: false,
        spread_um: 0,
    };
    let made = ws.derive_job_with_white(id, &edit, LATER).unwrap();
    assert_ne!(made.job_id, id);

    let (old, new) = (before.inputs(), ws.job_snapshot(made.job_id).unwrap());
    let new = new.inputs();
    assert_eq!(new.recipe.inputs().white.source, WhiteSource::Disabled);
    assert_eq!(new.recipe.inputs().id, old.recipe.inputs().id);
    assert_eq!(
        new.recipe.inputs().revision,
        old.recipe.inputs().revision + 1
    );
    assert_eq!(new.recipe.inputs().created_at_ms, LATER);
    // Everything else the job and recipe say is carried over unchanged.
    assert_eq!(new.layout.content_hash(), old.layout.content_hash());
    assert_eq!((&new.assets, &new.masks), (&old.assets, &old.masks));
    assert_eq!(new.device_setup, old.device_setup);
    assert_eq!(new.print_mode, old.print_mode);
    let mut rest = new.recipe.inputs().clone();
    rest.white = old.recipe.inputs().white.clone();
    rest.revision = old.recipe.inputs().revision;
    rest.created_at_ms = old.recipe.inputs().created_at_ms;
    assert_eq!(
        &rest,
        old.recipe.inputs(),
        "only the white policy, revision and date differ"
    );
    // The provenance names the new recipe, not the old one.
    let recipe_entry = |s: &veylora_domain::job::JobSnapshotInput| {
        s.provenance
            .content_hashes
            .iter()
            .find(|(label, _)| label == "recipe")
            .map(|(_, h)| *h)
    };
    assert_eq!(recipe_entry(new), Some(new.recipe.content_hash()));
    assert_ne!(recipe_entry(new), recipe_entry(old));

    // The first job and its recipe are exactly as they were.
    let again = ws.job_snapshot(id).unwrap();
    assert_eq!(again.content_hash(), before.content_hash());
    assert_eq!(ws.list_jobs().unwrap().len(), 2);
}

#[test]
fn a_choke_is_a_negative_spread_and_each_edit_takes_the_next_revision() {
    let (ws, id) = job();
    let first = ws
        .derive_job_with_white(
            id,
            &WhiteEdit {
                enabled: true,
                spread_um: -300,
            },
            LATER,
        )
        .unwrap();
    let first = ws.job_snapshot(first.job_id).unwrap();
    assert_eq!(first.inputs().recipe.inputs().white.spread_um, -300);
    assert_eq!(
        first.inputs().recipe.inputs().white.source,
        WhiteSource::Alpha
    );
    assert_eq!(first.inputs().recipe.inputs().revision, 2);

    // A second edit of the same job does not reuse revision 2 for different content.
    let second = ws
        .derive_job_with_white(
            id,
            &WhiteEdit {
                enabled: true,
                spread_um: -600,
            },
            LATER + 1,
        )
        .unwrap();
    let second = ws.job_snapshot(second.job_id).unwrap();
    assert_eq!(second.inputs().recipe.inputs().revision, 3);
}

#[test]
fn an_edit_that_changes_nothing_is_refused() {
    let (ws, id) = job();
    let err = ws
        .derive_job_with_white(
            id,
            &WhiteEdit {
                enabled: true,
                spread_um: 0,
            },
            LATER,
        )
        .unwrap_err();
    assert!(matches!(err, WorkflowError::Invalid(m) if m.contains("already")));
    assert_eq!(ws.list_jobs().unwrap().len(), 1);
}

#[test]
fn growing_white_beyond_the_artwork_is_not_offered() {
    let (ws, id) = job();
    let err = ws
        .derive_job_with_white(
            id,
            &WhiteEdit {
                enabled: true,
                spread_um: 200,
            },
            LATER,
        )
        .unwrap_err();
    assert!(matches!(err, WorkflowError::Invalid(m) if m.contains("choke")));
}

#[test]
fn the_new_job_renders_with_its_own_recipe_and_leaves_the_first_render_alone() {
    let (ws, id) = job();
    ws.render(id, &RenderOptions::default()).unwrap();
    let made = ws
        .derive_job_with_white(
            id,
            &WhiteEdit {
                enabled: false,
                spread_um: 0,
            },
            LATER,
        )
        .unwrap();
    ws.render(made.job_id, &RenderOptions::default()).unwrap();
    let old = ws.inspect_latest_run(id).unwrap().unwrap().report;
    let new = ws.inspect_latest_run(made.job_id).unwrap().unwrap().report;
    assert_ne!(old.manifest.recipe_hash, new.manifest.recipe_hash);
    let white = |r: &veylora_workflow::ArtifactReport| {
        r.planes
            .iter()
            .find(|p| p.name == "logical-w.f32")
            .unwrap()
            .max
    };
    assert!(white(&old) > 0.0, "the first job still prints white");
    assert_eq!(white(&new), 0.0, "the edited job prints none");
}
