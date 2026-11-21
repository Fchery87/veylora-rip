mod common;
use common::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use veylora_device::setup::VerificationState;
use veylora_diagnostics::bundle::{read_tar, write_tar, BundleEntry};
use veylora_domain::{ids::JobId, layout::Rotation, lifecycle::JobState};
use veylora_transport::{
    fake::{FailurePoint, FakeTransport},
    PrinterArtifact,
};
use veylora_workflow::{
    JobSpec, LayoutSpec, PlacementSpec, RenderOptions, ReviewResolution, WorkflowError, Workspace,
};

struct Env {
    ws: Workspace,
    job: JobId,
    source: Vec<u8>,
}

fn env(production: bool) -> Env {
    env_with_mask(production, false)
}

fn env_with_mask(production: bool, with_mask: bool) -> Env {
    let ws = Workspace::open(dir("ar")).unwrap();
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
    let source = fixture("alpha-ramp.png");
    let mask = gray_png(2, 2, &[0, 255, 255, 0]);
    let spec = JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1735000000000,
        setup: setup_hash,
        mode: ws.import_mode(&m.to_record()).unwrap(),
        recipe: ws.import_recipe(&recipe()).unwrap(),
        assets: BTreeMap::from([("art".to_string(), ws.import_asset(&source).unwrap())]),
        masks: if with_mask {
            BTreeMap::from([("m".to_string(), ws.import_asset(&mask).unwrap())])
        } else {
            BTreeMap::new()
        },
        layout: LayoutSpec {
            page_width_um: 80,
            page_height_um: 20,
            mirror: false,
            placements: vec![PlacementSpec {
                asset: "art".into(),
                mask: with_mask.then(|| "m".to_string()),
                x_um: 0,
                y_um: 0,
                width_um: 80,
                height_um: 20,
                rotation: Rotation::Deg0,
                crop: None,
            }],
        },
    };
    let job = ws.create_job(&spec).unwrap().job_id;
    Env { ws, job, source }
}

fn fresh() -> Workspace {
    Workspace::open(dir("ar-dest")).unwrap()
}

fn entries(archive: &[u8]) -> Vec<BundleEntry> {
    read_tar(archive).unwrap()
}

/// Rewrites the file list in `archive.json` so only the edit under test is wrong.
fn with_fresh_manifest(mut entries: Vec<BundleEntry>) -> Vec<u8> {
    let files: Vec<Value> = entries
        .iter()
        .filter(|e| e.path != "archive.json")
        .map(|e| {
            json!({"path": e.path, "bytes": e.bytes.len(), "sha256": hash(&e.bytes).to_string()})
        })
        .collect();
    let m = entries
        .iter_mut()
        .find(|e| e.path == "archive.json")
        .unwrap();
    let mut v: Value = serde_json::from_slice(&m.bytes).unwrap();
    v["files"] = Value::Array(files);
    m.bytes = serde_json::to_vec_pretty(&v).unwrap();
    write_tar(&entries).unwrap()
}

fn edit_manifest(archive: &[u8], f: impl FnOnce(&mut Value)) -> Vec<u8> {
    let mut es = entries(archive);
    let m = es.iter_mut().find(|e| e.path == "archive.json").unwrap();
    let mut v: Value = serde_json::from_slice(&m.bytes).unwrap();
    f(&mut v);
    m.bytes = serde_json::to_vec_pretty(&v).unwrap();
    write_tar(&es).unwrap()
}

fn refuses(dest: &Workspace, archive: &[u8], job: JobId) -> WorkflowError {
    let err = dest.import_job_archive(archive).unwrap_err();
    assert!(
        dest.state(job).is_err(),
        "a refused import left a job behind"
    );
    err
}

fn artifact() -> PrinterArtifact {
    PrinterArtifact::from_encoded_bytes(vec![7u8; 4096])
}

#[test]
fn a_job_survives_the_move_and_renders_identically_on_the_other_side() {
    let e = env(false);
    let archive = e.ws.export_job_archive(e.job).unwrap();
    let first = e.ws.render(e.job, &RenderOptions::default()).unwrap();
    let original_root = e.ws.root().to_path_buf();
    let dest = fresh();
    let got = dest.import_job_archive(&archive).unwrap();
    assert_eq!(got.job_id, e.job);
    assert_eq!(got.state, JobState::Draft);
    assert!(!got.setup_verification_carried);
    // The source workspace can disappear: the imported job needs nothing from it.
    std::fs::remove_dir_all(original_root).unwrap();
    let again = dest.render(e.job, &RenderOptions::default()).unwrap();
    assert_eq!(again.manifest.snapshot_hash, first.manifest.snapshot_hash);
    assert_eq!(
        again.manifest.composed_page_hash,
        first.manifest.composed_page_hash
    );
    assert_eq!(
        serde_json::to_value(&again.manifest.physical_planes).unwrap(),
        serde_json::to_value(&first.manifest.physical_planes).unwrap()
    );
}

#[test]
fn re_exporting_an_imported_job_gives_the_same_archive() {
    let e = env(false);
    e.ws.preflight(e.job, &Default::default()).unwrap();
    let archive = e.ws.export_job_archive(e.job).unwrap();
    assert_eq!(archive, e.ws.export_job_archive(e.job).unwrap());
    let dest = fresh();
    dest.import_job_archive(&archive).unwrap();
    assert_eq!(dest.export_job_archive(e.job).unwrap(), archive);
}

#[test]
fn an_ambiguous_submission_stays_ambiguous_after_the_move() {
    let e = env(true);
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
    let out =
        e.ws.submit(
            e.job,
            &artifact(),
            &FakeTransport::new(FailurePoint::DuringWrite { bytes: 100 }),
        )
        .unwrap();
    assert_eq!(out.job_state, JobState::Ambiguous);
    let dest = fresh();
    let got = dest
        .import_job_archive(&e.ws.export_job_archive(e.job).unwrap())
        .unwrap();
    assert_eq!(got.state, JobState::Ambiguous);
    assert_eq!(
        dest.lifecycle_events(e.job).unwrap(),
        e.ws.lifecycle_events(e.job).unwrap()
    );
    assert_eq!(
        serde_json::to_value(dest.report(e.job).unwrap().submission_attempts).unwrap(),
        serde_json::to_value(e.ws.report(e.job).unwrap().submission_attempts).unwrap()
    );
    // Moving the job did not make it printable again.
    assert!(dest.render(e.job, &RenderOptions::default()).is_err());
    assert_eq!(dest.state(e.job).unwrap(), JobState::Ambiguous);
    // The operator's own decision still works on the other side.
    assert_eq!(
        dest.resolve_review(e.job, ReviewResolution::NotPrinted, "op", "checked")
            .unwrap(),
        JobState::Cancelled
    );
}

#[test]
fn a_submitted_job_cannot_be_printed_again_by_moving_it() {
    let e = env(true);
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
    e.ws.submit(e.job, &artifact(), &FakeTransport::new(FailurePoint::None))
        .unwrap();
    let dest = fresh();
    dest.import_job_archive(&e.ws.export_job_archive(e.job).unwrap())
        .unwrap();
    assert_eq!(dest.state(e.job).unwrap(), JobState::Submitted);
    assert!(dest
        .submit(e.job, &artifact(), &FakeTransport::new(FailurePoint::None))
        .is_err());
}

#[test]
fn device_verification_is_not_carried() {
    let e = env(true);
    let hash = e.ws.report(e.job).unwrap().setup;
    assert!(e
        .ws
        .setup_history(&hash)
        .unwrap()
        .iter()
        .any(|s| matches!(s, VerificationState::Verified { .. })));
    let dest = fresh();
    dest.import_job_archive(&e.ws.export_job_archive(e.job).unwrap())
        .unwrap();
    assert!(dest
        .setup_history(&hash)
        .unwrap()
        .iter()
        .all(|s| !matches!(s, VerificationState::Verified { .. })));
}

#[test]
fn an_existing_job_is_never_overwritten_or_merged() {
    let e = env(false);
    let archive = e.ws.export_job_archive(e.job).unwrap();
    let err = e.ws.import_job_archive(&archive).unwrap_err();
    assert!(err.to_string().contains("already exists"), "{err}");
    let dest = fresh();
    dest.import_job_archive(&archive).unwrap();
    assert!(dest.import_job_archive(&archive).is_err());
}

#[test]
fn a_source_that_has_gone_missing_blocks_the_export() {
    let e = env(false);
    let h = hash(&e.source).to_string();
    std::fs::remove_file(e.ws.root().join("assets").join(h)).unwrap();
    let err = e.ws.export_job_archive(e.job).unwrap_err();
    assert!(err.to_string().contains("missing"), "{err}");
}

#[test]
fn damage_is_refused_before_anything_is_written() {
    let e = env(false);
    let good = e.ws.export_job_archive(e.job).unwrap();
    let source_path = format!("sources/{}", hash(&e.source));
    let dest = fresh();

    // A flipped byte in a source: the checksum no longer matches.
    let mut es = entries(&good);
    es.iter_mut().find(|x| x.path == source_path).unwrap().bytes[0] ^= 1;
    let err = refuses(&dest, &write_tar(&es).unwrap(), e.job);
    assert!(err.to_string().contains("checksum"), "{err}");

    // A source that is not the image the job names, even with a consistent file list.
    let mut es = entries(&good);
    es.iter_mut().find(|x| x.path == source_path).unwrap().bytes[0] ^= 1;
    let err = refuses(&dest, &with_fresh_manifest(es), e.job);
    assert!(err.to_string().contains("not the image"), "{err}");

    // A missing source, with a consistent file list.
    let mut es = entries(&good);
    es.retain(|x| x.path != source_path);
    let err = refuses(&dest, &with_fresh_manifest(es), e.job);
    assert!(err.to_string().contains("missing"), "{err}");

    // A missing file the list still names.
    let mut es = entries(&good);
    es.retain(|x| x.path != source_path);
    assert!(refuses(&dest, &write_tar(&es).unwrap(), e.job)
        .to_string()
        .contains("file list"));

    // A file the list does not name.
    let mut es = entries(&good);
    es.push(BundleEntry {
        path: "extra.txt".into(),
        bytes: b"x".to_vec(),
    });
    assert!(refuses(&dest, &write_tar(&es).unwrap(), e.job)
        .to_string()
        .contains("file list"));

    // A listed file this version does not understand.
    let mut es = entries(&good);
    es.push(BundleEntry {
        path: "extra.txt".into(),
        bytes: b"x".to_vec(),
    });
    assert!(refuses(&dest, &with_fresh_manifest(es), e.job)
        .to_string()
        .contains("does not understand"));

    // A snapshot that is not the one the archive describes.
    let other = edit_manifest(&good, |v| v["job_id"] = json!(JobId::new().to_string()));
    assert!(refuses(&dest, &other, e.job)
        .to_string()
        .contains("snapshot"));
    let other = edit_manifest(&good, |v| {
        v["snapshot_hash"] = json!(hash(b"x").to_string())
    });
    assert!(refuses(&dest, &other, e.job)
        .to_string()
        .contains("snapshot"));

    // A record that is not what the snapshot names.
    let mut es = entries(&good);
    let r = es
        .iter_mut()
        .find(|x| x.path == "records/recipe.json")
        .unwrap();
    let mut v: Value = serde_json::from_slice(&r.bytes).unwrap();
    v["notes"] = json!("changed");
    r.bytes = serde_json::to_vec(&v).unwrap();
    assert!(refuses(&dest, &with_fresh_manifest(es), e.job)
        .to_string()
        .contains("usable job archive"));

    // Not an archive at all, or a damaged one.
    assert!(refuses(&dest, b"hello", e.job)
        .to_string()
        .contains("usable"));
    assert!(refuses(&dest, &good[..good.len() - 1], e.job)
        .to_string()
        .contains("usable"));
    // Nothing was ever written for any of the above.
    assert!(std::fs::read_dir(dest.root().join("assets"))
        .map(|mut d| d.next().is_none())
        .unwrap_or(true));
}

#[test]
fn history_that_the_state_machine_would_not_allow_is_refused() {
    let e = env(false);
    e.ws.preflight(e.job, &Default::default()).unwrap();
    let good = e.ws.export_job_archive(e.job).unwrap();
    let dest = fresh();

    let edit = |f: &dyn Fn(&mut Value)| {
        let mut es = entries(&good);
        let h = es
            .iter_mut()
            .find(|x| x.path == "history/lifecycle.json")
            .unwrap();
        let mut v: Value = serde_json::from_slice(&h.bytes).unwrap();
        f(&mut v);
        h.bytes = serde_json::to_vec(&v).unwrap();
        with_fresh_manifest(es)
    };
    // The same event twice cannot be legal.
    let dup = edit(&|v| {
        let first = v[0].clone();
        v.as_array_mut().unwrap().push(first);
    });
    assert!(refuses(&dest, &dup, e.job)
        .to_string()
        .contains("lifecycle event 2"));
    // A claimed result the machine would not produce.
    let wrong = edit(&|v| v[0]["to"] = v[0]["from"].clone());
    assert!(refuses(&dest, &wrong, e.job)
        .to_string()
        .contains("lifecycle event 1"));
    // A sequence that does not start at one.
    let skip = edit(&|v| v[0]["sequence"] = json!(2));
    assert!(refuses(&dest, &skip, e.job)
        .to_string()
        .contains("lifecycle event 1"));
    // An event for another job.
    let foreign = edit(&|v| v[0]["job_id"] = json!(JobId::new().to_string()));
    assert!(refuses(&dest, &foreign, e.job)
        .to_string()
        .contains("lifecycle event 1"));
}

#[test]
fn attempts_for_another_job_are_refused() {
    let e = env(true);
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
    e.ws.submit(e.job, &artifact(), &FakeTransport::new(FailurePoint::None))
        .unwrap();
    let good = e.ws.export_job_archive(e.job).unwrap();
    let mut es = entries(&good);
    let h = es
        .iter_mut()
        .find(|x| x.path == "history/attempts.json")
        .unwrap();
    let mut v: Value = serde_json::from_slice(&h.bytes).unwrap();
    v[0]["job_id"] = json!(JobId::new().to_string());
    h.bytes = serde_json::to_vec(&v).unwrap();
    let dest = fresh();
    assert!(refuses(&dest, &with_fresh_manifest(es), e.job)
        .to_string()
        .contains("different job"));
    let mut es = entries(&good);
    let h = es
        .iter_mut()
        .find(|x| x.path == "history/attempts.json")
        .unwrap();
    let mut v: Value = serde_json::from_slice(&h.bytes).unwrap();
    v[0]["snapshot_hash"] = json!(hash(b"x").to_string());
    h.bytes = serde_json::to_vec(&v).unwrap();
    assert!(refuses(&dest, &with_fresh_manifest(es), e.job)
        .to_string()
        .contains("different job"));
}

#[test]
fn another_layout_version_or_format_is_named_and_refused() {
    let e = env(false);
    let good = e.ws.export_job_archive(e.job).unwrap();
    let dest = fresh();
    let newer = edit_manifest(&good, |v| v["schema_version"] = json!(2));
    match refuses(&dest, &newer, e.job) {
        WorkflowError::ArchiveVersion { found, supported } => {
            assert_eq!((found, supported), (2, 1));
        }
        other => panic!("{other}"),
    }
    let zero = edit_manifest(&good, |v| v["schema_version"] = json!(0));
    assert!(matches!(
        refuses(&dest, &zero, e.job),
        WorkflowError::ArchiveVersion { found: 0, .. }
    ));
    let none = edit_manifest(&good, |v| {
        v.as_object_mut().unwrap().remove("schema_version");
    });
    assert!(refuses(&dest, &none, e.job).to_string().contains("version"));
    let other = edit_manifest(&good, |v| v["format"] = json!("something-else"));
    assert!(refuses(&dest, &other, e.job)
        .to_string()
        .contains("not a Veylora"));
}

#[test]
fn masks_travel_with_the_job() {
    let e = env_with_mask(false, true);
    let archive = e.ws.export_job_archive(e.job).unwrap();
    let names: Vec<String> = entries(&archive).into_iter().map(|x| x.path).collect();
    assert_eq!(
        names.iter().filter(|n| n.starts_with("sources/")).count(),
        2
    );
    let dest = fresh();
    dest.import_job_archive(&archive).unwrap();
    assert_eq!(dest.export_job_archive(e.job).unwrap(), archive);
}

#[test]
fn a_file_listed_twice_is_refused() {
    let e = env(false);
    let good = e.ws.export_job_archive(e.job).unwrap();
    let twice = edit_manifest(&good, |v| {
        let first = v["files"][0].clone();
        v["files"].as_array_mut().unwrap().push(first);
    });
    let err = refuses(&fresh(), &twice, e.job);
    assert!(err.to_string().contains("listed twice"), "{err}");
}

fn swap_record(good: &[u8], path: &str, with: &[u8]) -> Vec<u8> {
    let mut es = entries(good);
    es.iter_mut().find(|x| x.path == path).unwrap().bytes = with.to_vec();
    with_fresh_manifest(es)
}

#[test]
fn records_that_are_valid_but_not_the_ones_the_job_names_are_refused() {
    let e = env(false);
    let other = env(false);
    let good = e.ws.export_job_archive(e.job).unwrap();
    let theirs = entries(&other.ws.export_job_archive(other.job).unwrap());
    let theirs = |p: &str| theirs.iter().find(|x| x.path == p).unwrap().bytes.clone();
    let dest = fresh();
    for path in ["records/setup.json", "records/mode.json"] {
        let err = refuses(&dest, &swap_record(&good, path, &theirs(path)), e.job);
        assert!(err.to_string().contains("does not match"), "{path}: {err}");
    }
    // Both swapped together are consistent with each other, but not with the snapshot.
    let both = swap_record(&good, "records/setup.json", &theirs("records/setup.json"));
    let both = swap_record(&both, "records/mode.json", &theirs("records/mode.json"));
    assert!(refuses(&dest, &both, e.job)
        .to_string()
        .contains("does not match"));
    // A recipe that is valid but differs from the snapshot's.
    let mut es = entries(&good);
    let r = es
        .iter_mut()
        .find(|x| x.path == "records/recipe.json")
        .unwrap();
    let mut v: Value = serde_json::from_slice(&r.bytes).unwrap();
    v["created_at_ms"] = json!(v["created_at_ms"].as_u64().unwrap() + 1);
    r.bytes = serde_json::to_vec(&v).unwrap();
    let err = refuses(&dest, &with_fresh_manifest(es), e.job);
    assert!(err.to_string().contains("does not match"), "{err}");
}

#[test]
fn the_exported_setup_record_carries_no_verification() {
    let e = env(true);
    let archive = e.ws.export_job_archive(e.job).unwrap();
    let rec: Value = serde_json::from_slice(
        &entries(&archive)
            .iter()
            .find(|x| x.path == "records/setup.json")
            .unwrap()
            .bytes,
    )
    .unwrap();
    assert_eq!(rec["verification"], "unverified");
    assert!(rec["evidence"].is_null());
}

#[test]
fn an_archive_holds_only_its_own_jobs_attempts() {
    let a = env(true);
    a.ws.render(a.job, &RenderOptions::default()).unwrap();
    a.ws.submit(a.job, &artifact(), &FakeTransport::new(FailurePoint::None))
        .unwrap();
    let archive = a.ws.export_job_archive(a.job).unwrap();
    // A workspace that already has another job's attempts in its log.
    let b = env(true);
    b.ws.render(b.job, &RenderOptions::default()).unwrap();
    b.ws.submit(b.job, &artifact(), &FakeTransport::new(FailurePoint::None))
        .unwrap();
    b.ws.import_job_archive(&archive).unwrap();
    assert_eq!(b.ws.export_job_archive(a.job).unwrap(), archive);
}
