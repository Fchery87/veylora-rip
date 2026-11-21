//! Preflight, acknowledgement and render over a real workspace, as the app presents them.
#[path = "../../veylora-workflow/tests/common.rs"]
mod common;

use common::*;
use std::collections::BTreeMap;
use veylora_app_lib::live;
use veylora_app_lib::screens::{job_actions, preflight};
use veylora_app_lib::state::{DataSource, ModeMaturity};
use veylora_app_lib::view_models::{JobState, Severity};
use veylora_domain::layout::Rotation;
use veylora_workflow::{JobSpec, LayoutSpec, PlacementSpec, Workspace};

fn job(mutate: impl FnOnce(&mut LayoutSpec)) -> (Workspace, String) {
    let ws = Workspace::open(dir("live-act")).unwrap();
    let s = setup();
    let m = mode(s.id());
    let setup_hash = ws.import_setup(&s.to_record()).unwrap();
    let mode_hash = ws.import_mode(&m.to_record()).unwrap();
    let recipe_hash = ws.import_recipe(&recipe()).unwrap();
    let art = ws.import_asset(&fixture("alpha-ramp.png")).unwrap();
    // 4 x 1 px over 1 x 0.25 mm is about 100 ppi: below the default 150, so it needs confirmation.
    let mut layout = LayoutSpec {
        page_width_um: 1_000,
        page_height_um: 250,
        mirror: false,
        placements: vec![PlacementSpec {
            asset: "art".into(),
            mask: None,
            x_um: 0,
            y_um: 0,
            width_um: 1_000,
            height_um: 250,
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
    let id = ws.create_job(&spec).unwrap().job_id.to_string();
    (ws, id)
}

#[test]
fn capabilities_come_from_the_jobs_real_setup_and_mode_and_never_claim_a_printer() {
    let (ws, id) = job(|_| {});
    let caps = live::capabilities(&ws, &id).unwrap();
    assert_eq!(caps.data_source, DataSource::Live);
    assert!(caps.setup.exists);
    assert!(
        !caps.setup.verified,
        "the fixture setup has no physical evidence"
    );
    assert_eq!(caps.mode.maturity, ModeMaturity::Experimental);
    assert!(!caps.sinks.physical_printer);
    assert!(caps.sinks.file_sink);
    assert!(caps.open_review.is_none());
    assert!(!caps.setup.name.contains("Home bench"));
    assert!(!caps.mode.id.contains("PM-exp-01"));
}

#[test]
fn preflight_fills_findings_with_a_correction_for_every_blocking_one() {
    let (ws, id) = job(|_| {});
    let d = live::run_preflight(&ws, &id, &[]).unwrap();
    assert!(!d.findings.is_empty());
    assert!(d
        .findings
        .iter()
        .any(|f| f.severity == Severity::Confirm && f.title == "Low effective resolution"));
    assert!(d.findings.iter().any(|f| f.severity == Severity::Info));
    assert!(d
        .findings
        .iter()
        .all(|f| !f.fix.is_empty() && !f.cause.is_empty()));
    // Causes are the workflow's own finding messages, not text made up by the app.
    let id: veylora_domain::ids::JobId = id.parse().unwrap();
    let real = ws.preflight(id, &Default::default()).unwrap().findings;
    for f in &d.findings {
        assert!(real.iter().any(|r| r.message == f.cause), "{}", f.cause);
    }
    assert_eq!(d.findings.len(), real.len());
    let ids: std::collections::BTreeSet<_> = d.findings.iter().map(|f| &f.id).collect();
    assert_eq!(ids.len(), d.findings.len(), "finding ids are unique");
    assert!(d.findings.iter().all(|f| !f.acknowledged));
}

#[test]
fn blocking_findings_cannot_be_acknowledged_and_unknown_ids_are_errors() {
    let (ws, id) = job(|l| l.page_width_um = 80_000_000);
    let d = live::run_preflight(&ws, &id, &[]).unwrap();
    let blocking = d
        .findings
        .iter()
        .find(|f| f.severity == Severity::Blocking)
        .unwrap();
    let err = live::run_preflight(&ws, &id, std::slice::from_ref(&blocking.id)).unwrap_err();
    assert!(err.contains("cannot be acknowledged"), "{err}");
    let err = live::run_preflight(&ws, &id, &["no-such-finding".into()]).unwrap_err();
    assert!(err.contains("no-such-finding"), "{err}");
}

#[test]
fn acknowledging_a_confirm_finding_marks_it_with_who_and_when() {
    let (ws, id) = job(|_| {});
    let d = live::run_preflight(&ws, &id, &[]).unwrap();
    let confirm = d
        .findings
        .iter()
        .find(|f| f.severity == Severity::Confirm)
        .unwrap();
    let d = live::run_preflight(&ws, &id, std::slice::from_ref(&confirm.id)).unwrap();
    let f = d.findings.iter().find(|f| f.id == confirm.id).unwrap();
    assert!(f.acknowledged);
    let a = f.acknowledgement.as_ref().unwrap();
    assert!(!a.operator.is_empty() && a.at.ends_with("UTC"));
}

#[test]
fn the_preflight_view_counts_open_findings_from_real_data() {
    let (ws, id) = job(|_| {});
    let d = live::run_preflight(&ws, &id, &[]).unwrap();
    let caps = live::capabilities(&ws, &id).unwrap();
    let v = preflight::present(&d, &caps);
    assert_eq!(v.open_count, 1, "{:?}", v.summary);
}

#[test]
fn render_is_refused_until_confirmations_are_acknowledged_then_retains_a_run() {
    let (ws, id) = job(|_| {});
    let err = live::render_job(&ws, &id, &[]).unwrap_err();
    assert!(err.contains("Low effective resolution"), "{err}");
    assert!(!live::job_detail(&ws, &id).unwrap().rendered);

    let findings = live::run_preflight(&ws, &id, &[]).unwrap().findings;
    let confirm = findings
        .iter()
        .find(|f| f.severity == Severity::Confirm)
        .unwrap();
    let d = live::render_job(&ws, &id, std::slice::from_ref(&confirm.id)).unwrap();
    assert!(d.rendered);
    assert_eq!(d.summary.state, JobState::Ready);
    // The acknowledgement is recorded in the retained manifest, not only shown.
    let run = veylora_app_lib::artifacts::retained_for_job(&ws, &id)
        .unwrap()
        .unwrap();
    assert!(run.verified);
    assert_eq!(run.acknowledged_findings, vec![confirm.id.clone()]);
    let caps = live::capabilities(&ws, &id).unwrap();
    assert!(job_actions::present(&caps, &d).inspect.enabled);
}

#[test]
fn a_blocking_job_cannot_render_even_with_everything_acknowledged() {
    let (ws, id) = job(|l| l.page_width_um = 80_000_000);
    let all: Vec<String> = live::run_preflight(&ws, &id, &[])
        .unwrap()
        .findings
        .into_iter()
        .filter(|f| f.severity != Severity::Blocking)
        .map(|f| f.id)
        .collect();
    assert!(live::render_job(&ws, &id, &all).is_err());
    assert!(!live::job_detail(&ws, &id).unwrap().rendered);
}

#[test]
fn an_empty_workspace_reports_no_setup_instead_of_inventing_one() {
    let ws = Workspace::open(dir("live-nocaps")).unwrap();
    let caps = live::workspace_capabilities(&ws).unwrap();
    assert_eq!(caps.data_source, DataSource::Live);
    assert!(!caps.setup.exists);
    assert!(caps.mode.id.is_empty() && caps.setup.name.is_empty());
    assert!(!caps.sinks.physical_printer);
}

#[test]
fn workspace_capabilities_follow_the_newest_job() {
    let (ws, id) = job(|_| {});
    let caps = live::workspace_capabilities(&ws).unwrap();
    assert!(caps.setup.exists);
    assert_eq!(
        caps.setup.name,
        live::capabilities(&ws, &id).unwrap().setup.name
    );
}

#[test]
fn session_acknowledgements_are_per_job_deduplicated_and_clearable() {
    let acks = live::Acknowledgements::default();
    assert!(acks.for_job("a").is_empty());
    acks.add("a", "x").unwrap();
    acks.add("a", "x").unwrap();
    acks.add("a", "y").unwrap();
    acks.add("b", "z").unwrap();
    assert_eq!(acks.for_job("a"), vec!["x".to_string(), "y".to_string()]);
    assert_eq!(acks.for_job("b"), vec!["z".to_string()]);
    acks.clear("a");
    assert!(acks.for_job("a").is_empty());
    assert_eq!(acks.for_job("b").len(), 1);
}

#[test]
fn mode_capabilities_come_from_the_mode_record() {
    let (ws, id) = job(|_| {});
    let caps = live::capabilities(&ws, &id).unwrap();
    let device = ws.job_device(id.parse().unwrap()).unwrap();
    let (w, h) = device.printable_um;
    assert_eq!(caps.mode.sheet_sizes.len(), 1);
    assert!(
        caps.mode.sheet_sizes[0].contains("mm"),
        "{:?}",
        caps.mode.sheet_sizes
    );
    assert!(device.white_capable, "the fixture mode caps white at 1.0");
    assert_eq!(
        caps.mode.white_policies,
        vec![
            veylora_app_lib::view_models::WhitePolicyKind::Underbase,
            veylora_app_lib::view_models::WhitePolicyKind::NoWhite
        ]
    );
    assert!(
        caps.mode.white_choke,
        "the renderer applies a recipe's white choke"
    );
    assert!(
        !caps.mode.white_mask_invert,
        "mask inversion does not exist in recipes"
    );
    assert!(caps.mode.screened_inspect);
    assert!(w > 0 && h > 0);
}

#[test]
fn a_mode_without_white_offers_only_no_white() {
    let ws = Workspace::open(dir("live-nowhite")).unwrap();
    let s = setup();
    let base = mode(s.id());
    let m = veylora_device::mode::PrintMode::new(
        veylora_domain::ids::PrintModeId::new(),
        s.id(),
        base.raster().clone(),
        base.protocol().clone(),
        base.media().clone(),
        veylora_device::mode::DepositionSpec {
            passes: 4,
            overlap_percent: 50,
            coverage_caps: vec![("cyan".into(), 1.0)],
        },
        base.quality().clone(),
    )
    .unwrap();
    let setup_hash = ws.import_setup(&s.to_record()).unwrap();
    let mode_hash = ws.import_mode(&m.to_record()).unwrap();
    let recipe_hash = ws.import_recipe(&recipe()).unwrap();
    let art = ws.import_asset(&fixture("alpha-ramp.png")).unwrap();
    let spec = JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1,
        setup: setup_hash,
        mode: mode_hash,
        recipe: recipe_hash,
        assets: BTreeMap::from([("art".to_string(), art)]),
        masks: BTreeMap::new(),
        layout: LayoutSpec {
            page_width_um: 1_000,
            page_height_um: 250,
            mirror: false,
            placements: vec![PlacementSpec {
                asset: "art".into(),
                mask: None,
                x_um: 0,
                y_um: 0,
                width_um: 1_000,
                height_um: 250,
                rotation: Rotation::Deg0,
                crop: None,
            }],
        },
    };
    let id = ws.create_job(&spec).unwrap().job_id.to_string();
    let caps = live::capabilities(&ws, &id).unwrap();
    assert_eq!(
        caps.mode.white_policies,
        vec![veylora_app_lib::view_models::WhitePolicyKind::NoWhite]
    );
}

#[test]
fn render_is_offered_after_preflight_moves_the_job_to_held() {
    let (ws, id) = job(|_| {});
    // Preflight records the job as Held; Render must still be available, not "already rendered".
    let d = live::run_preflight(&ws, &id, &[]).unwrap();
    assert_eq!(d.summary.state, JobState::Held);
    let caps = live::capabilities(&ws, &id).unwrap();
    let open = job_actions::present(&caps, &d).render_and_hold;
    assert!(!open.enabled);
    assert!(
        open.reason
            .as_deref()
            .unwrap()
            .contains("Confirm 1 finding"),
        "{open:?}"
    );
    let confirm = d
        .findings
        .iter()
        .find(|f| f.severity == Severity::Confirm)
        .unwrap();
    let d = live::run_preflight(&ws, &id, std::slice::from_ref(&confirm.id)).unwrap();
    assert!(job_actions::present(&caps, &d).render_and_hold.enabled);
}

#[test]
fn the_file_sink_export_writes_a_verified_folder_and_is_listed_in_provenance() {
    let (ws, id) = job(|_| {});
    let confirm = live::run_preflight(&ws, &id, &[])
        .unwrap()
        .findings
        .into_iter()
        .find(|f| f.severity == Severity::Confirm)
        .unwrap();
    let d = live::render_job(&ws, &id, std::slice::from_ref(&confirm.id)).unwrap();
    let caps = live::capabilities(&ws, &id).unwrap();
    assert!(job_actions::present(&caps, &d).write_to_file_sink.enabled);
    let d = live::write_to_file_sink(&ws, &id).unwrap();
    let row = d
        .provenance
        .iter()
        .find(|r| r.label == "Exported to")
        .expect("export row");
    assert!(row.value.contains("exports"), "{}", row.value);
    assert!(row.value.contains("diagnostic"), "{}", row.value);
    assert!(
        d.attempts.is_empty(),
        "an export sends nothing, so it is not an attempt"
    );
    assert!(live::write_to_file_sink(&ws, &id)
        .unwrap_err()
        .contains("already exported"));
}

#[test]
fn the_file_sink_refuses_a_job_that_was_never_rendered() {
    let (ws, id) = job(|_| {});
    assert!(live::write_to_file_sink(&ws, &id).is_err());
}

#[test]
fn a_finding_acknowledged_for_a_render_stays_acknowledged_afterwards() {
    let (ws, id) = job(|_| {});
    let findings = live::run_preflight(&ws, &id, &[]).unwrap().findings;
    let confirm = findings
        .iter()
        .find(|f| f.severity == Severity::Confirm)
        .unwrap();
    live::render_job(&ws, &id, std::slice::from_ref(&confirm.id)).unwrap();
    // The session list was cleared by the render; the manifest still remembers the decision.
    let after = live::run_preflight(&ws, &id, &[]).unwrap();
    let again = after.findings.iter().find(|f| f.id == confirm.id).unwrap();
    assert!(again.acknowledged);
    let caps = live::capabilities(&ws, &id).unwrap();
    let view = preflight::present(&after, &caps);
    assert_eq!(view.open_count, 0);
}
