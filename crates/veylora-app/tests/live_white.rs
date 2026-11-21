//! White edits on a real job: a new job on a new recipe revision, never a change to the stored one.
#[path = "../../veylora-workflow/tests/common.rs"]
mod common;

use common::*;
use std::collections::BTreeMap;
use veylora_app_lib::live::{self, WhiteChange};
use veylora_app_lib::view_models::{MaskSource, WhitePolicyKind};
use veylora_workflow::{JobSpec, LayoutSpec, PlacementSpec, Workspace};

fn created_job() -> (Workspace, String) {
    let ws = Workspace::open(dir("live-white")).unwrap();
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
    let id = ws.create_job(&spec).unwrap().job_id.to_string();
    (ws, id)
}

fn change() -> WhiteChange {
    WhiteChange {
        policy: None,
        source: None,
        invert_mask: None,
        choke_mm: None,
    }
}

#[test]
fn turning_white_off_returns_a_new_job_and_leaves_the_first_alone() {
    let (ws, id) = created_job();
    let made = live::edit_white(
        &ws,
        &id,
        &WhiteChange {
            policy: Some(WhitePolicyKind::NoWhite),
            ..change()
        },
    )
    .unwrap();
    assert_ne!(made.summary.id, id);
    assert_eq!(made.white_policy.policy, WhitePolicyKind::NoWhite);
    let first = live::job_detail(&ws, &id).unwrap();
    assert_eq!(first.white_policy.policy, WhitePolicyKind::Underbase);
    assert_eq!(live::job_summaries(&ws).unwrap().len(), 2);
}

#[test]
fn a_choke_is_kept_in_millimetres_and_other_settings_carry_over() {
    let (ws, id) = created_job();
    let made = live::edit_white(
        &ws,
        &id,
        &WhiteChange {
            choke_mm: Some(0.3),
            ..change()
        },
    )
    .unwrap();
    assert_eq!(made.white_policy.choke_mm, 0.3);
    assert_eq!(made.white_policy.policy, WhitePolicyKind::Underbase);
    // Changing the policy later keeps that choke.
    let later = live::edit_white(
        &ws,
        &made.summary.id,
        &WhiteChange {
            policy: Some(WhitePolicyKind::NoWhite),
            ..change()
        },
    )
    .unwrap();
    assert_eq!(later.white_policy.choke_mm, 0.3);
    assert_eq!(later.white_policy.policy, WhitePolicyKind::NoWhite);
}

#[test]
fn settings_a_recipe_cannot_state_are_refused_not_approximated() {
    let (ws, id) = created_job();
    let cases = [
        (
            WhiteChange {
                invert_mask: Some(true),
                ..change()
            },
            "invert",
        ),
        (
            WhiteChange {
                source: Some(MaskSource::WholePlacement),
                ..change()
            },
            "whole placement",
        ),
        (
            WhiteChange {
                choke_mm: Some(-0.1),
                ..change()
            },
            "choke",
        ),
        (
            WhiteChange {
                choke_mm: Some(f64::NAN),
                ..change()
            },
            "choke",
        ),
    ];
    for (c, word) in cases {
        let err = live::edit_white(&ws, &id, &c).unwrap_err().to_lowercase();
        assert!(err.contains(word), "{err}");
    }
    assert_eq!(
        live::job_summaries(&ws).unwrap().len(),
        1,
        "nothing was created"
    );
}

#[test]
fn an_edit_that_changes_nothing_creates_nothing() {
    let (ws, id) = created_job();
    for c in [
        change(),
        WhiteChange {
            policy: Some(WhitePolicyKind::Underbase),
            ..change()
        },
        WhiteChange {
            source: Some(MaskSource::AutoFromAlpha),
            invert_mask: Some(false),
            ..change()
        },
    ] {
        let err = live::edit_white(&ws, &id, &c).unwrap_err();
        assert!(err.contains("already"), "{err}");
    }
    assert_eq!(live::job_summaries(&ws).unwrap().len(), 1);
}

#[test]
fn the_changes_the_interface_sends_read_back_as_one_change_each() {
    let read = |json: &str| serde_json::from_str::<WhiteChange>(json).unwrap();
    assert_eq!(
        read(r#"{"policy":"No white"}"#),
        WhiteChange {
            policy: Some(WhitePolicyKind::NoWhite),
            ..change()
        }
    );
    assert_eq!(
        read(r#"{"source":"Auto from alpha","invertMask":false}"#),
        WhiteChange {
            source: Some(MaskSource::AutoFromAlpha),
            invert_mask: Some(false),
            ..change()
        }
    );
    assert_eq!(
        read(r#"{"chokeMm":0.3}"#),
        WhiteChange {
            choke_mm: Some(0.3),
            ..change()
        }
    );
}
