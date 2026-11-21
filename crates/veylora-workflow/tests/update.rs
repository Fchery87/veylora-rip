mod common;
use common::*;
use std::{cell::RefCell, collections::BTreeMap};
use veylora_domain::{
    ids::JobId,
    layout::Rotation,
    lifecycle::{JobState, LifecycleEvent},
};
use veylora_persistence::FileLifecycleStore;
use veylora_transport::{
    PrinterArtifact, PrinterTransport, SubmissionObserver, TransportError, TransportOutcome,
    TransportResult,
};
use veylora_workflow::{JobSpec, LayoutSpec, PlacementSpec, RenderOptions, Workspace};

fn job(ws: &Workspace, production: bool) -> JobId {
    let s = setup();
    let mut m = mode(s.id());
    if production {
        m.mark_validated().unwrap();
    }
    let setup_hash = ws.import_setup(&s.to_record()).unwrap();
    if production {
        ws.verify_setup(&setup_hash, evidence_for_record(&s.to_record()))
            .unwrap();
    }
    let spec = JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1735000000000,
        setup: setup_hash,
        mode: ws.import_mode(&m.to_record()).unwrap(),
        recipe: ws
            .import_recipe(&if production {
                validated_recipe(&recipe(), s.content_hash(), m.content_hash())
            } else {
                recipe()
            })
            .unwrap(),
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
                rotation: Rotation::Deg0,
                crop: None,
            }],
        },
    };
    ws.create_job(&spec).unwrap().job_id
}

#[test]
fn an_idle_workspace_has_no_blockers() {
    let ws = Workspace::open(dir("up-idle")).unwrap();
    assert!(ws.update_blockers().unwrap().is_empty());
    let j = job(&ws, false);
    assert!(ws.update_blockers().unwrap().is_empty());
    ws.render(j, &RenderOptions::default()).unwrap();
    assert_eq!(ws.state(j).unwrap(), JobState::Ready);
    assert!(ws.update_blockers().unwrap().is_empty());
}

#[test]
fn a_render_in_progress_blocks() {
    let ws = Workspace::open(dir("up-render")).unwrap();
    let j = job(&ws, false);
    let store = FileLifecycleStore::open(ws.root().join("state")).unwrap();
    store
        .append(
            j,
            LifecycleEvent::PreflightPassed,
            "2026-11-20T00:00:00Z",
            "",
            None,
        )
        .unwrap();
    store
        .append(
            j,
            LifecycleEvent::BeginRender,
            "2026-11-20T00:00:01Z",
            "",
            None,
        )
        .unwrap();
    let b = ws.update_blockers().unwrap();
    assert_eq!(b.len(), 1);
    assert_eq!((b[0].job_id, b[0].state), (j, JobState::Rendering));
}

struct Probe<'a> {
    ws: &'a Workspace,
    seen: RefCell<Vec<(JobId, JobState)>>,
}
impl PrinterTransport for Probe<'_> {
    fn submit(
        &self,
        a: &PrinterArtifact,
        _: &mut dyn SubmissionObserver,
    ) -> Result<TransportResult, TransportError> {
        for b in self.ws.update_blockers().unwrap() {
            self.seen.borrow_mut().push((b.job_id, b.state));
        }
        Ok(TransportResult {
            attempt_id: uuid::Uuid::nil(),
            bytes_intended: a.len(),
            outcome: TransportOutcome::Submitted,
            bytes_written: a.len(),
        })
    }
}

#[test]
fn a_submission_blocks_while_it_is_in_flight_and_not_after() {
    let ws = Workspace::open(dir("up-submit")).unwrap();
    let j = job(&ws, true);
    ws.render(j, &RenderOptions::default()).unwrap();
    let probe = Probe {
        ws: &ws,
        seen: RefCell::new(vec![]),
    };
    ws.submit(j, &PrinterArtifact::from_encoded_bytes(vec![1; 64]), &probe)
        .unwrap();
    assert_eq!(*probe.seen.borrow(), vec![(j, JobState::Submitting)]);
    assert!(ws.update_blockers().unwrap().is_empty());
}

#[test]
fn an_ambiguous_job_waits_for_the_operator_and_does_not_block() {
    let ws = Workspace::open(dir("up-amb")).unwrap();
    let j = job(&ws, true);
    ws.render(j, &RenderOptions::default()).unwrap();
    let t = veylora_transport::fake::FakeTransport::new(
        veylora_transport::fake::FailurePoint::DuringWrite { bytes: 10 },
    );
    ws.submit(j, &PrinterArtifact::from_encoded_bytes(vec![1; 64]), &t)
        .unwrap();
    assert_eq!(ws.state(j).unwrap(), JobState::Ambiguous);
    assert!(ws.update_blockers().unwrap().is_empty());
}

#[test]
fn unreadable_job_state_is_an_error_not_an_idle_workspace() {
    let ws = Workspace::open(dir("up-bad")).unwrap();
    let j = job(&ws, false);
    ws.preflight(j, &Default::default()).unwrap();
    let d = ws
        .root()
        .join("state")
        .join("lifecycle")
        .join(j.to_string());
    let first = std::fs::read_dir(&d)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::write(first, b"garbage").unwrap();
    assert!(ws.update_blockers().is_err());
    std::fs::create_dir(ws.root().join("state").join("lifecycle").join("not-a-job")).unwrap();
    assert!(ws.update_blockers().is_err());
}
