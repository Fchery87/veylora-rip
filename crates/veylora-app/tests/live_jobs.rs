//! The job list and job detail read from a real workspace, never from fixture values.
#[path = "../../veylora-workflow/tests/common.rs"]
mod common;

use common::*;
use std::collections::BTreeMap;
use veylora_app_lib::live;
use veylora_app_lib::view_models::{JobState, MaskSource, WhitePolicyKind};
use veylora_domain::layout::Rotation;
use veylora_workflow::{JobSpec, LayoutSpec, PlacementSpec, RenderOptions, Workspace};

struct Fx {
    ws: Workspace,
    spec: JobSpec,
}

fn fx(mutate: impl FnOnce(&mut LayoutSpec)) -> Fx {
    let ws = Workspace::open(dir("live-ws")).unwrap();
    let s = setup();
    let m = mode(s.id());
    let setup_hash = ws.import_setup(&s.to_record()).unwrap();
    let mode_hash = ws.import_mode(&m.to_record()).unwrap();
    let recipe_hash = ws.import_recipe(&recipe()).unwrap();
    let art = ws.import_asset(&fixture("alpha-ramp.png")).unwrap();
    let mut layout = LayoutSpec {
        page_width_um: 80_000,
        page_height_um: 20_000,
        mirror: false,
        placements: vec![PlacementSpec {
            asset: "art".into(),
            mask: None,
            x_um: 1_000,
            y_um: 2_000,
            width_um: 40_000,
            height_um: 10_000,
            rotation: Rotation::Deg0,
            crop: None,
        }],
    };
    mutate(&mut layout);
    let spec = JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1_735_000_000_000,
        setup: setup_hash,
        mode: mode_hash,
        recipe: recipe_hash,
        assets: BTreeMap::from([("art".to_string(), art)]),
        masks: BTreeMap::new(),
        layout,
    };
    Fx { ws, spec }
}

#[test]
fn an_empty_workspace_has_no_jobs_and_no_fixture_jobs_appear() {
    let ws = Workspace::open(dir("live-empty")).unwrap();
    assert!(live::job_summaries(&ws).unwrap().is_empty());
}

#[test]
fn the_list_shows_real_jobs_newest_first_with_real_sheet_and_count() {
    let f = fx(|_| {});
    let mut older = f.spec.clone();
    older.created_at_ms = 1_000;
    let a = f.ws.create_job(&older).unwrap().job_id;
    let b = f.ws.create_job(&f.spec).unwrap().job_id;
    let list = live::job_summaries(&f.ws).unwrap();
    assert_eq!(
        list.iter().map(|j| j.id.clone()).collect::<Vec<_>>(),
        vec![b.to_string(), a.to_string()]
    );
    assert_eq!(list[0].sheet, "80 x 20 mm");
    assert_eq!(list[0].placement_count, 1);
    assert_eq!(list[0].state, JobState::Draft);
    assert!(list[0].progress_percent.is_none());
}

#[test]
fn detail_shows_the_snapshot_not_fixture_values() {
    let f = fx(|_| {});
    let id = f.ws.create_job(&f.spec).unwrap();
    let d = live::job_detail(&f.ws, &id.job_id.to_string()).unwrap();
    assert_eq!(d.summary.id, id.job_id.to_string());
    assert_eq!(d.placements.len(), 1);
    let p = &d.placements[0];
    assert_eq!(
        (p.x_mm, p.y_mm, p.width_mm, p.height_mm),
        (1.0, 2.0, 40.0, 10.0)
    );
    assert!(!p.clipped);
    assert!(!d.rendered);
    assert!(d.attempts.is_empty());
    let row = |label: &str| {
        d.provenance
            .iter()
            .find(|r| r.label == label)
            .unwrap_or_else(|| panic!("no {label}: {:?}", d.provenance))
            .value
            .clone()
    };
    assert_eq!(row("Snapshot hash"), id.snapshot_hash.to_string());
    assert!(d.provenance.iter().all(|r| !r.value.contains("fixture")));
    assert!(!d.provenance.iter().any(|r| r.value.contains("Home bench")));
    let stored = f.ws.job_snapshot(id.job_id).unwrap();
    let recipe_id = stored.inputs().recipe.inputs().id.to_string();
    assert!(
        row("Recipe").starts_with(&format!("{recipe_id} rev 1")),
        "{}",
        row("Recipe")
    );
}

#[test]
fn a_placement_past_the_page_edge_is_marked_clipped_including_when_rotated() {
    let f = fx(|l| l.placements[0].x_um = 50_000);
    let id = f.ws.create_job(&f.spec).unwrap().job_id.to_string();
    assert!(live::job_detail(&f.ws, &id).unwrap().placements[0].clipped);
    // 40 x 10 mm rotated 90 degrees occupies 10 x 40 mm, which overflows a 20 mm tall page.
    let g = fx(|l| {
        l.placements[0].x_um = 0;
        l.placements[0].y_um = 0;
        l.placements[0].rotation = Rotation::Deg90;
    });
    let id = g.ws.create_job(&g.spec).unwrap().job_id.to_string();
    assert!(live::job_detail(&g.ws, &id).unwrap().placements[0].clipped);
}

#[test]
fn rendering_changes_the_detail_to_rendered_and_the_list_state() {
    let f = fx(|l| {
        l.page_width_um = 80;
        l.page_height_um = 20;
        let p = &mut l.placements[0];
        (p.x_um, p.y_um, p.width_um, p.height_um) = (0, 0, 80, 20);
    });
    let id = f.ws.create_job(&f.spec).unwrap().job_id;
    f.ws.preflight(id, &Default::default()).unwrap();
    f.ws.render(id, &RenderOptions::default()).unwrap();
    let d = live::job_detail(&f.ws, &id.to_string()).unwrap();
    assert!(d.rendered);
    assert_eq!(
        f.ws.state(id).unwrap(),
        veylora_domain::lifecycle::JobState::Ready
    );
    assert_eq!(d.summary.state, JobState::Ready);
}

#[test]
fn the_white_policy_is_read_from_the_recipe() {
    let f = fx(|_| {});
    let id = f.ws.create_job(&f.spec).unwrap().job_id.to_string();
    let w = live::job_detail(&f.ws, &id).unwrap().white_policy;
    let src = recipe().inputs().white.source.clone();
    match src {
        veylora_domain::recipe::WhiteSource::Alpha => {
            assert_eq!(w.source, MaskSource::AutoFromAlpha);
            assert_eq!(w.policy, WhitePolicyKind::Underbase);
        }
        veylora_domain::recipe::WhiteSource::Disabled => {
            assert_eq!(w.policy, WhitePolicyKind::NoWhite)
        }
        other => panic!("fixture recipe uses {other:?}; the test must cover it"),
    }
}

#[test]
fn unknown_and_malformed_ids_are_errors() {
    let ws = Workspace::open(dir("live-bad")).unwrap();
    assert!(live::job_detail(&ws, "nope")
        .unwrap_err()
        .contains("not a job id"));
    assert!(live::job_detail(&ws, "00000000-0000-4000-8000-000000000000").is_err());
}

#[test]
fn a_white_source_the_ui_cannot_represent_is_refused_not_approximated() {
    let f = fx(|_| {});
    let mut input = recipe().inputs().clone();
    input.white.source = veylora_domain::recipe::WhiteSource::Tonal;
    let tonal = veylora_domain::recipe::PrintRecipe::new(input).unwrap();
    let mut spec = f.spec.clone();
    spec.recipe = f.ws.import_recipe(&tonal).unwrap();
    let id = f.ws.create_job(&spec).unwrap().job_id.to_string();
    let err = live::job_detail(&f.ws, &id).unwrap_err();
    assert!(
        err.contains("white source") && err.contains("not represented"),
        "{err}"
    );

    let mut input = recipe().inputs().clone();
    input.white.spread_um = 150;
    let expand = veylora_domain::recipe::PrintRecipe::new(input).unwrap();
    let mut spec = f.spec.clone();
    spec.recipe = f.ws.import_recipe(&expand).unwrap();
    spec.created_at_ms += 1;
    let id = f.ws.create_job(&spec).unwrap().job_id.to_string();
    assert!(live::job_detail(&f.ws, &id)
        .unwrap_err()
        .contains("expands"));
}

#[test]
fn a_choke_is_shown_in_millimetres() {
    let f = fx(|_| {});
    let mut input = recipe().inputs().clone();
    input.white.spread_um = -250;
    let choke = veylora_domain::recipe::PrintRecipe::new(input).unwrap();
    let mut spec = f.spec.clone();
    spec.recipe = f.ws.import_recipe(&choke).unwrap();
    let id = f.ws.create_job(&spec).unwrap().job_id.to_string();
    assert_eq!(
        live::job_detail(&f.ws, &id).unwrap().white_policy.choke_mm,
        0.25
    );
}

#[test]
fn a_placement_off_the_top_or_left_edge_is_clipped() {
    for (x, y) in [(-1_000, 2_000), (1_000, -1_000)] {
        let f = fx(|l| {
            l.placements[0].x_um = x;
            l.placements[0].y_um = y;
        });
        let id = f.ws.create_job(&f.spec).unwrap().job_id.to_string();
        assert!(
            live::job_detail(&f.ws, &id).unwrap().placements[0].clipped,
            "{x},{y}"
        );
    }
}

#[test]
fn every_placement_is_listed_and_counted() {
    let f = fx(|l| {
        let mut second = l.placements[0].clone();
        second.x_um = 45_000;
        second.width_um = 30_000;
        l.placements.push(second);
    });
    let id = f.ws.create_job(&f.spec).unwrap().job_id.to_string();
    let d = live::job_detail(&f.ws, &id).unwrap();
    assert_eq!(d.placements.len(), 2);
    assert_eq!(d.summary.placement_count, 2);
    assert_eq!(live::job_summaries(&f.ws).unwrap()[0].placement_count, 2);
}

#[test]
fn a_recipe_with_white_off_shows_no_white() {
    let f = fx(|_| {});
    let mut input = recipe().inputs().clone();
    input.white.source = veylora_domain::recipe::WhiteSource::Disabled;
    let off = veylora_domain::recipe::PrintRecipe::new(input).unwrap();
    let mut spec = f.spec.clone();
    spec.recipe = f.ws.import_recipe(&off).unwrap();
    let id = f.ws.create_job(&spec).unwrap().job_id.to_string();
    assert_eq!(
        live::job_detail(&f.ws, &id).unwrap().white_policy.policy,
        WhitePolicyKind::NoWhite
    );
}
