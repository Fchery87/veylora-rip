mod common;
use common::*;
use serde_json::Value;
use std::collections::BTreeMap;
use veylora_diagnostics::bundle::{read_tar, BundleEntry};
use veylora_domain::{ids::JobId, layout::Rotation};
use veylora_transport::{
    PrinterArtifact, PrinterTransport, SubmissionObserver, TransportError, TransportResult,
};
use veylora_workflow::{
    JobSpec, LayoutSpec, PlacementSpec, RenderOptions, SupportBundleOptions, Workspace,
};

struct Env {
    ws: Workspace,
    job: JobId,
    source: Vec<u8>,
}

fn env(production: bool) -> Env {
    let ws = Workspace::open(dir("sb")).unwrap();
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
    let spec = JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1735000000000,
        setup: setup_hash,
        mode: ws.import_mode(&m.to_record()).unwrap(),
        recipe: ws.import_recipe(&recipe()).unwrap(),
        assets: BTreeMap::from([("art".to_string(), ws.import_asset(&source).unwrap())]),
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
    let job = ws.create_job(&spec).unwrap().job_id;
    Env { ws, job, source }
}

fn bundle(e: &Env, include_artwork: bool) -> Vec<BundleEntry> {
    let bytes =
        e.ws.support_bundle(e.job, &SupportBundleOptions { include_artwork })
            .unwrap();
    read_tar(&bytes).unwrap()
}

fn find<'a>(entries: &'a [BundleEntry], path: &str) -> &'a BundleEntry {
    entries
        .iter()
        .find(|e| e.path == path)
        .unwrap_or_else(|| panic!("missing {path}: {:?}", paths(entries)))
}

fn paths(entries: &[BundleEntry]) -> Vec<&str> {
    entries.iter().map(|e| e.path.as_str()).collect()
}

fn json(entries: &[BundleEntry], path: &str) -> Value {
    serde_json::from_slice(&find(entries, path).bytes).unwrap()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

#[test]
fn a_bundle_without_artwork_carries_no_source_bytes() {
    let e = env(false);
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
    let entries = bundle(&e, false);
    assert!(entries.iter().all(|x| !x.path.starts_with("artwork/")));
    assert!(entries.iter().all(|x| !contains(&x.bytes, &e.source)));
    let manifest = json(&entries, "manifest.json");
    assert_eq!(manifest["include_artwork"], false);
    assert_eq!(manifest["job_id"], e.job.to_string());
    // The summary and the render record are always there.
    let report = json(&entries, "job-report.json");
    assert_eq!(report["job_id"], e.job.to_string());
    assert_eq!(report["renders"].as_array().unwrap().len(), 1);
    let run = report["renders"][0]["run"].as_str().unwrap();
    find(&entries, &format!("render-manifests/{run}.json"));
}

#[test]
fn a_bundle_with_artwork_carries_the_stored_sources_under_their_hash() {
    let e = env(false);
    let entries = bundle(&e, true);
    let hash = hash(&e.source).to_string();
    assert_eq!(find(&entries, &format!("artwork/{hash}")).bytes, e.source);
    assert_eq!(json(&entries, "manifest.json")["include_artwork"], true);
}

#[test]
fn a_job_that_never_rendered_still_produces_a_bundle() {
    let e = env(false);
    let entries = bundle(&e, false);
    assert!(json(&entries, "job-report.json")["renders"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(entries
        .iter()
        .all(|x| !x.path.starts_with("render-manifests/")));
}

#[test]
fn the_manifest_lists_every_file_with_its_hash_and_size() {
    let e = env(false);
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
    let entries = bundle(&e, true);
    let manifest = json(&entries, "manifest.json");
    let listed = manifest["files"].as_array().unwrap();
    // Every file except the manifest itself is listed, and nothing else is.
    let mut expected: Vec<&str> = paths(&entries)
        .into_iter()
        .filter(|p| *p != "manifest.json")
        .collect();
    expected.sort();
    let mut got: Vec<&str> = listed.iter().map(|f| f["path"].as_str().unwrap()).collect();
    got.sort();
    assert_eq!(got, expected);
    for f in listed {
        let entry = find(&entries, f["path"].as_str().unwrap());
        assert_eq!(f["bytes"], entry.bytes.len());
        assert_eq!(f["sha256"], hash(&entry.bytes).to_string());
    }
}

#[test]
fn equal_state_gives_equal_bytes() {
    let e = env(false);
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
    let o = SupportBundleOptions {
        include_artwork: true,
    };
    assert_eq!(
        e.ws.support_bundle(e.job, &o).unwrap(),
        e.ws.support_bundle(e.job, &o).unwrap()
    );
}

struct Leaky(String);
impl PrinterTransport for Leaky {
    fn submit(
        &self,
        _: &PrinterArtifact,
        _: &mut dyn SubmissionObserver,
    ) -> Result<TransportResult, TransportError> {
        Err(TransportError::Io(std::io::Error::other(self.0.clone())))
    }
}

#[test]
fn machine_specific_paths_in_notes_never_reach_the_bundle() {
    let e = env(true);
    e.ws.render(e.job, &RenderOptions::default()).unwrap();
    let root = e.ws.root().display().to_string();
    let message = format!("cannot read {root}\\spool and C:\\Users\\Frantz\\secret.prn");
    e.ws.submit(
        e.job,
        &PrinterArtifact::from_encoded_bytes(vec![1; 64]),
        &Leaky(message),
    )
    .unwrap();
    // The operator's own note is free text too.
    e.ws.resolve_review(
        e.job,
        veylora_workflow::ReviewResolution::NotPrinted,
        "op",
        &format!("looked at {root}/out and /home/frantz/x"),
    )
    .unwrap();
    for include in [false, true] {
        for entry in bundle(&e, include) {
            assert!(!contains(&entry.bytes, root.as_bytes()), "{}", entry.path);
            assert!(
                !contains(&entry.bytes, root.replace('\\', "/").as_bytes()),
                "{}",
                entry.path
            );
            let escaped = root.replace('\\', "\\\\");
            assert!(
                !contains(&entry.bytes, escaped.as_bytes()),
                "{}",
                entry.path
            );
            assert!(!contains(&entry.bytes, b"Frantz"), "{}", entry.path);
            assert!(!contains(&entry.bytes, b"frantz"), "{}", entry.path);
        }
    }
    // The redacted text is still useful: the shape of the failure survives.
    let entries = bundle(&e, false);
    let text = String::from_utf8(find(&entries, "lifecycle.json").bytes.clone()).unwrap();
    assert!(text.contains("<workspace>"), "{text}");
    assert!(text.contains("<user>"), "{text}");
    let report = String::from_utf8(find(&entries, "job-report.json").bytes.clone()).unwrap();
    assert!(report.contains("<workspace>"), "{report}");
}

#[test]
fn a_damaged_section_is_named_not_silently_dropped() {
    let e = env(false);
    e.ws.preflight(e.job, &Default::default()).unwrap();
    let dir =
        e.ws.root()
            .join("state")
            .join("lifecycle")
            .join(e.job.to_string());
    let first = std::fs::read_dir(&dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::write(first, b"not json").unwrap();
    let entries = bundle(&e, false);
    assert!(entries.iter().all(|x| x.path != "lifecycle.json"));
    let manifest = json(&entries, "manifest.json");
    let unavailable = manifest["unavailable"].as_array().unwrap();
    assert_eq!(unavailable.len(), 1);
    assert_eq!(unavailable[0]["section"], "lifecycle");
    let reason = unavailable[0]["reason"].as_str().unwrap();
    assert!(!reason.is_empty());
    let root = e.ws.root().display().to_string();
    assert!(!reason.contains(&root), "{reason}");
    assert!(!reason.contains(&root.replace('\\', "/")), "{reason}");
    // The rest of the diagnosis is intact.
    find(&entries, "job-report.json");
}

#[test]
fn a_missing_job_is_an_error_not_an_empty_bundle() {
    let e = env(false);
    let other = JobId::new();
    assert!(e
        .ws
        .support_bundle(other, &SupportBundleOptions::default())
        .is_err());
}

#[test]
fn a_stored_source_that_has_gone_missing_is_reported_when_artwork_is_requested() {
    let e = env(false);
    let h = hash(&e.source).to_string();
    std::fs::remove_file(e.ws.root().join("assets").join(&h)).unwrap();
    let entries = bundle(&e, true);
    assert!(entries.iter().all(|x| !x.path.starts_with("artwork/")));
    let manifest = json(&entries, "manifest.json");
    let sections: Vec<&str> = manifest["unavailable"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["section"].as_str().unwrap())
        .collect();
    assert_eq!(sections, [format!("artwork/{h}")]);
}

#[test]
fn a_manifest_that_names_a_local_path_is_withheld_not_rewritten() {
    let e = env(false);
    let out = e.ws.render(e.job, &RenderOptions::default()).unwrap();
    let manifest_path = out.dir.join(veylora_render::MANIFEST_NAME);
    let mut value: Value = serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    value["evidence"] = Value::String(format!("{}\\spool", e.ws.root().display()));
    std::fs::write(&manifest_path, serde_json::to_vec(&value).unwrap()).unwrap();
    let entries = bundle(&e, false);
    assert!(entries
        .iter()
        .all(|x| !x.path.starts_with("render-manifests/")));
    let manifest = json(&entries, "manifest.json");
    let run = out.dir.file_name().unwrap().to_str().unwrap().to_string();
    let sections: Vec<&str> = manifest["unavailable"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["section"].as_str().unwrap())
        .collect();
    assert_eq!(sections, [format!("render-manifests/{run}.json")]);
}

#[test]
fn a_run_that_never_finished_is_listed_as_incomplete() {
    let e = env(false);
    let out = e.ws.render(e.job, &RenderOptions::default()).unwrap();
    let finished = out.dir.file_name().unwrap().to_str().unwrap().to_string();
    let unfinished = out.dir.parent().unwrap().join("run-unfinished");
    std::fs::create_dir(&unfinished).unwrap();
    let entries = bundle(&e, false);
    let manifest = json(&entries, "manifest.json");
    assert_eq!(
        manifest["incomplete_runs"],
        serde_json::json!(["run-unfinished"])
    );
    find(&entries, &format!("render-manifests/{finished}.json"));
}

#[test]
fn masks_travel_with_the_artwork_and_only_then() {
    let ws = Workspace::open(dir("sb-mask")).unwrap();
    let s = setup();
    let m = mode(s.id());
    let art = fixture("alpha-ramp.png");
    let mask = gray_png(2, 2, &[0, 255, 255, 0]);
    let spec = JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1735000000000,
        setup: ws.import_setup(&s.to_record()).unwrap(),
        mode: ws.import_mode(&m.to_record()).unwrap(),
        recipe: ws.import_recipe(&recipe()).unwrap(),
        assets: BTreeMap::from([("art".to_string(), ws.import_asset(&art).unwrap())]),
        masks: BTreeMap::from([("m".to_string(), ws.import_asset(&mask).unwrap())]),
        layout: LayoutSpec {
            page_width_um: 80,
            page_height_um: 20,
            mirror: false,
            placements: vec![PlacementSpec {
                asset: "art".into(),
                mask: Some("m".into()),
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
    let read = |include_artwork| {
        read_tar(
            &ws.support_bundle(job, &SupportBundleOptions { include_artwork })
                .unwrap(),
        )
        .unwrap()
    };
    let with = read(true);
    find(&with, &format!("artwork/{}", hash(&art)));
    assert_eq!(find(&with, &format!("artwork/{}", hash(&mask))).bytes, mask);
    assert!(read(false).iter().all(|x| !x.path.starts_with("artwork/")));
}
