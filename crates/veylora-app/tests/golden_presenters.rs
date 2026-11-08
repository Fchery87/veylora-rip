//! Golden agreement between the Rust presenters and the TypeScript fallback
//! presenters (`app/src/lib/services/presentation.ts`).
//!
//! This test builds a fixed set of inputs, runs the Rust presenters, and
//! compares the result to `tests/golden/presenters.json`. The Vitest suite
//! (`app/tests/golden.test.ts`) runs the same inputs through the TypeScript
//! fallback and asserts the same expected output, so the two cannot drift.
//!
//! Regenerate after an intended presenter change with:
//! `UPDATE_GOLDEN=1 cargo test -p veylora-app --test golden_presenters`

use std::path::PathBuf;

use serde_json::{json, Value};
use veylora_app_lib::jobs::JobStore;
use veylora_app_lib::screens::inspect::{self, InspectSelection};
use veylora_app_lib::screens::{job_actions, preflight, review};
use veylora_app_lib::state::{CapabilityState, DataSource};
use veylora_app_lib::view_models::{Acknowledgement, JobDetail, Plane, StageId};

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/presenters.json")
}

fn selection(stage: StageId, planes: &[Plane], composite: bool) -> InspectSelection {
    InspectSelection {
        stage,
        planes: planes.to_vec(),
        composite,
    }
}

fn case(name: &str, caps: &CapabilityState, job: &JobDetail, sel: &InspectSelection) -> Value {
    let inspect_result = match inspect::present(job, caps, sel) {
        Ok(view) => json!({ "ok": view }),
        Err(message) => json!({ "error": message }),
    };
    json!({
        "name": name,
        "input": { "caps": caps, "job": job, "selection": sel },
        "expected": {
            "preflight": preflight::present(job, caps),
            "actions": job_actions::present(caps, job),
            "inspect": inspect_result,
            "review": review::present(caps, job),
        }
    })
}

fn toggle(name: &str, sel: &InspectSelection, op: &str, plane: Plane, on: bool) -> Value {
    let next = match op {
        "plane" => inspect::toggle_plane(sel, plane, on),
        _ => inspect::set_composite(sel, on),
    };
    json!({
        "name": name,
        "input": { "selection": sel, "op": op, "plane": plane, "on": on },
        "expected": next,
    })
}

fn build() -> Value {
    let store = JobStore::fixture();
    let caps = store.capabilities();
    let default = inspect::default_selection();
    let mut cases = Vec::new();

    for id in ["J-0142", "J-0143", "J-0144", "J-0145"] {
        let job = store.get(id).unwrap();
        cases.push(case(
            &format!("{id} default selection"),
            &caps,
            &job,
            &default,
        ));
    }

    let held = store.get("J-0143").unwrap();
    cases.push(case(
        "slots with logical white is dropped",
        &caps,
        &held,
        &selection(StageId::Slots, &[Plane::W], false),
    ));
    cases.push(case(
        "slots single slot",
        &caps,
        &held,
        &selection(StageId::Slots, &[Plane::W1], false),
    ));
    cases.push(case(
        "screened composite",
        &caps,
        &held,
        &selection(StageId::Screened, &[], true),
    ));
    cases.push(case(
        "garment stage is refused",
        &caps,
        &held,
        &selection(StageId::Garment, &[], false),
    ));

    let mut live = caps.clone();
    live.data_source = DataSource::Live;
    cases.push(case(
        "live data source",
        &live,
        &store.get("J-0142").unwrap(),
        &default,
    ));

    let mut no_screened = caps.clone();
    no_screened.mode.screened_inspect = false;
    cases.push(case(
        "screened unsupported by the mode",
        &no_screened,
        &held,
        &selection(StageId::Screened, &[Plane::K], false),
    ));
    cases.push(case(
        "screened option disabled",
        &no_screened,
        &held,
        &default,
    ));

    let mut acknowledged = store.get("J-0145").unwrap();
    acknowledged.findings[0].acknowledged = true;
    acknowledged.findings[0].acknowledgement = Some(Acknowledgement {
        operator: "Fixture operator".to_owned(),
        at: "10:00:00 UTC".to_owned(),
    });
    cases.push(case(
        "acknowledged confirm finding",
        &caps,
        &acknowledged,
        &default,
    ));

    let mut unrecorded = store.get("J-0142").unwrap();
    unrecorded.attempts[0].spool_job_id = None;
    unrecorded.attempts[0].bytes_written = None;
    unrecorded.attempts[0].artifact_hash = None;
    cases.push(case(
        "review with unrecorded facts",
        &caps,
        &unrecorded,
        &default,
    ));

    let mut gate_open = caps.clone();
    gate_open.sinks.physical_printer = true;
    cases.push(case("physical gate open", &gate_open, &held, &default));

    let start = selection(StageId::Slots, &[Plane::K], false);
    let composite = selection(StageId::Logical, &[], true);
    let toggles = vec![
        toggle("select replaces", &start, "plane", Plane::C, true),
        toggle("deselect leaves none", &start, "plane", Plane::K, false),
        toggle(
            "plane leaves composite",
            &composite,
            "plane",
            Plane::M,
            true,
        ),
        toggle("composite on", &start, "composite", Plane::K, true),
        toggle("composite off", &composite, "composite", Plane::K, false),
    ];

    json!({ "cases": cases, "toggles": toggles })
}

#[test]
fn rust_presenters_match_the_golden_fixture() {
    let actual = build();
    let path = golden_path();
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let text = serde_json::to_string_pretty(&actual).unwrap() + "\n";
        std::fs::write(&path, text).unwrap();
    }
    let stored: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(
        stored, actual,
        "golden fixture is stale; regenerate with UPDATE_GOLDEN=1"
    );
}
