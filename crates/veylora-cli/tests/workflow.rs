#[path = "../../veylora-workflow/tests/common.rs"]
mod common;
use common::*;
use std::{collections::BTreeMap, fs, path::Path, process::Command};
use veylora_domain::layout::Rotation;
use veylora_workflow::{JobSpec, LayoutSpec, PlacementSpec};

fn cli(ws: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_veylora-cli"))
        .arg("--workspace")
        .arg(ws)
        .args(args)
        .output()
        .unwrap()
}
fn json(o: &std::process::Output) -> serde_json::Value {
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    serde_json::from_slice(&o.stdout).unwrap()
}
fn hash_of(v: &serde_json::Value) -> String {
    v["content_hash"].as_str().unwrap().to_string()
}

#[test]
fn headless_workflow_runs_from_files_to_a_verified_report() {
    let ws = dir("cli");
    let files = dir("cli-files");
    fs::create_dir_all(&files).unwrap();
    let s = setup();
    let m = mode(s.id());
    let put = |name: &str, bytes: Vec<u8>| {
        let p = files.join(name);
        fs::write(&p, bytes).unwrap();
        p
    };
    let setup_f = put("setup.json", serde_json::to_vec(&s.to_record()).unwrap());
    let mode_f = put("mode.json", serde_json::to_vec(&m.to_record()).unwrap());
    let recipe_f = put("recipe.json", serde_json::to_vec(&recipe()).unwrap());
    let png = put("art.png", fixture("alpha-ramp.png"));

    let sh = hash_of(&json(&cli(
        &ws,
        &["setup", "import", "--input", setup_f.to_str().unwrap()],
    )));
    let mh = hash_of(&json(&cli(
        &ws,
        &["mode", "import", "--input", mode_f.to_str().unwrap()],
    )));
    let rh = hash_of(&json(&cli(
        &ws,
        &["recipe", "import", "--input", recipe_f.to_str().unwrap()],
    )));
    let ah = hash_of(&json(&cli(
        &ws,
        &["asset", "import", "--input", png.to_str().unwrap()],
    )));
    let inspected = json(&cli(
        &ws,
        &["asset", "inspect", "--input", png.to_str().unwrap()],
    ));
    assert_eq!(inspected["width"], 4);

    let spec = JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1735000000000,
        setup: sh.parse().unwrap(),
        mode: mh.parse().unwrap(),
        recipe: rh.parse().unwrap(),
        assets: BTreeMap::from([("art".to_string(), ah.parse().unwrap())]),
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
    let spec_f = put("job.json", serde_json::to_vec(&spec).unwrap());
    let created = json(&cli(
        &ws,
        &["job", "create", "--spec", spec_f.to_str().unwrap()],
    ));
    let job = created["job_id"].as_str().unwrap().to_string();

    let pre = json(&cli(&ws, &["job", "preflight", "--job", &job]));
    assert_eq!(pre["can_render"], true);
    let rendered = json(&cli(&ws, &["job", "render", "--job", &job]));
    let dir = rendered["dir"].as_str().unwrap().to_string();

    let art = json(&cli(&ws, &["artifact", "inspect", "--dir", &dir]));
    assert_eq!(art["verified"], true);

    let out = files.join("report.json");
    let o = cli(
        &ws,
        &[
            "report",
            "export",
            "--job",
            &job,
            "--output",
            out.to_str().unwrap(),
        ],
    );
    assert!(o.status.success());
    let report: serde_json::Value = serde_json::from_slice(&fs::read(&out).unwrap()).unwrap();
    assert_eq!(report["renders"].as_array().unwrap().len(), 1);
    assert_eq!(report["submission_attempts"].as_array().unwrap().len(), 0);
    // Exports never overwrite.
    assert!(!cli(
        &ws,
        &[
            "report",
            "export",
            "--job",
            &job,
            "--output",
            out.to_str().unwrap()
        ]
    )
    .status
    .success());
}

#[test]
fn bad_usage_and_blocked_renders_fail_without_output() {
    let ws = dir("cli-bad");
    assert!(!cli(&ws, &["job", "submit"]).status.success());
    assert!(!cli(&ws, &["job", "render"]).status.success());
    assert!(
        !cli(&ws, &["asset", "import", "--input", "/nonexistent.png"])
            .status
            .success()
    );
}

#[test]
fn acknowledgements_are_parsed_and_required_for_confirmation_findings() {
    let ws = dir("cli-ack");
    let files = dir("cli-ack-files");
    fs::create_dir_all(&files).unwrap();
    let s = setup();
    let m = mode(s.id());
    let put = |name: &str, bytes: Vec<u8>| {
        let p = files.join(name);
        fs::write(&p, bytes).unwrap();
        p
    };
    let imp = |kind: &str, flag: &str, f: &std::path::Path| {
        hash_of(&json(&cli(
            &ws,
            &[kind, "import", flag, f.to_str().unwrap()],
        )))
    };
    let sh = imp(
        "setup",
        "--input",
        &put("s.json", serde_json::to_vec(&s.to_record()).unwrap()),
    );
    let mh = imp(
        "mode",
        "--input",
        &put("m.json", serde_json::to_vec(&m.to_record()).unwrap()),
    );
    let rh = imp(
        "recipe",
        "--input",
        &put("r.json", serde_json::to_vec(&recipe()).unwrap()),
    );
    let ah = imp("asset", "--input", &put("a.png", fixture("alpha-ramp.png")));
    let spec = JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1,
        setup: sh.parse().unwrap(),
        mode: mh.parse().unwrap(),
        recipe: rh.parse().unwrap(),
        assets: BTreeMap::from([("art".to_string(), ah.parse().unwrap())]),
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
    let spec_f = put("job.json", serde_json::to_vec(&spec).unwrap());
    let job = json(&cli(
        &ws,
        &["job", "create", "--spec", spec_f.to_str().unwrap()],
    ))["job_id"]
        .as_str()
        .unwrap()
        .to_string();
    // Without acknowledgement a strict policy refuses to render.
    let refused = cli(&ws, &["job", "render", "--job", &job, "--min-ppi", "5000"]);
    assert!(!refused.status.success());
    // A malformed code is rejected, not ignored.
    assert!(!cli(
        &ws,
        &[
            "job",
            "render",
            "--job",
            &job,
            "--min-ppi",
            "5000",
            "--acknowledge",
            "nonsense"
        ]
    )
    .status
    .success());
    // Acknowledgement names the finding's code and subject, as preflight printed them.
    let pre = json(&cli(
        &ws,
        &["job", "preflight", "--job", &job, "--min-ppi", "5000"],
    ));
    let finding = pre["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["code"] == "low_effective_resolution")
        .unwrap();
    let ack = format!(
        "low_effective_resolution:{}",
        finding["subject"].as_str().unwrap()
    );
    // A code alone does not cover a finding about a specific placement.
    assert!(!cli(
        &ws,
        &[
            "job",
            "render",
            "--job",
            &job,
            "--min-ppi",
            "5000",
            "--acknowledge",
            "low_effective_resolution"
        ]
    )
    .status
    .success());
    let ok = cli(
        &ws,
        &[
            "job",
            "render",
            "--job",
            &job,
            "--min-ppi",
            "5000",
            "--acknowledge",
            &ack,
        ],
    );
    assert!(
        ok.status.success(),
        "{}",
        String::from_utf8_lossy(&ok.stderr)
    );
}

#[test]
fn submission_and_reprint_are_refused_with_a_reason_and_no_side_effects() {
    let ws = dir("cli-refuse");
    for args in [
        &[
            "job",
            "submit",
            "--job",
            "00000000-0000-0000-0000-000000000000",
        ][..],
        &[
            "job",
            "submit",
            "--job",
            "00000000-0000-0000-0000-000000000000",
            "--sink",
            "out.bin",
        ][..],
        &[
            "job",
            "reprint",
            "--job",
            "00000000-0000-0000-0000-000000000000",
        ][..],
    ] {
        let o = cli(&ws, args);
        assert_eq!(o.status.code(), Some(2));
        let err = String::from_utf8_lossy(&o.stderr);
        assert!(
            err.contains("is refused") && err.contains("no encoder"),
            "{err}"
        );
        assert!(o.stdout.is_empty());
    }
    // Nothing was created, not even the workspace.
    assert!(!ws.exists());
}

#[test]
fn setups_are_created_unverified_and_verified_only_with_matching_evidence() {
    use veylora_device::setup::{ChannelEvidence, EvidenceRecord, SetupOrigin};
    let ws = dir("cli-setup");
    let files = dir("cli-setup-files");
    fs::create_dir_all(&files).unwrap();
    let draft = serde_json::json!({
        "id": null,
        "origin": SetupOrigin::Recorded,
        "details": setup().details(),
    });
    let draft_f = files.join("draft.json");
    fs::write(&draft_f, serde_json::to_vec(&draft).unwrap()).unwrap();
    let created = json(&cli(
        &ws,
        &["setup", "create", "--input", draft_f.to_str().unwrap()],
    ));
    assert_eq!(created["verification"], "unverified");
    let hash = created["content_hash"].as_str().unwrap().to_string();

    let record = setup();
    let evidence = |observed_ok: bool| {
        let channels = record
            .details()
            .channel_map
            .bindings()
            .iter()
            .enumerate()
            .map(|(i, b)| ChannelEvidence {
                target_label: format!("l{i}"),
                slot: b.slot,
                protocol: b.protocol,
                expected_logical: b.logical,
                observed_logical: if observed_ok || i > 0 {
                    b.logical
                } else {
                    veylora_device::channels::LogicalChannel::Black
                },
            })
            .collect();
        EvidenceRecord {
            operator: "op".into(),
            setup_id: serde_json::from_value(created["id"].clone()).unwrap(),
            revision: 1,
            setup_hash: hash.parse().unwrap(),
            map_version: record.details().channel_map.version(),
            channels,
            target_id: "t".into(),
            observed_at: "2026-11-15T10:00:00Z".into(),
            artifact_hash: hash.parse().unwrap(),
            evidence_hash: hash.parse().unwrap(),
        }
    };
    let bad_f = files.join("bad.json");
    fs::write(&bad_f, serde_json::to_vec(&evidence(false)).unwrap()).unwrap();
    let bad = cli(
        &ws,
        &[
            "setup",
            "verify",
            "--setup",
            &hash,
            "--evidence",
            bad_f.to_str().unwrap(),
        ],
    );
    assert_eq!(bad.status.code(), Some(2));
    let good_f = files.join("good.json");
    fs::write(&good_f, serde_json::to_vec(&evidence(true)).unwrap()).unwrap();
    let ok = json(&cli(
        &ws,
        &[
            "setup",
            "verify",
            "--setup",
            &hash,
            "--evidence",
            good_f.to_str().unwrap(),
        ],
    ));
    assert!(ok["verification"]["verified"].is_object());
}

#[test]
fn targets_are_generated_as_diagnostic_files_without_a_workspace() {
    let ws = dir("cli-target");
    let out = dir("cli-target-out");
    fs::create_dir_all(&out).unwrap();
    let file = out.join("t.svg");
    let o = cli(
        &ws,
        &[
            "target",
            "generate",
            "--kind",
            "channel_identity",
            "--width-mm",
            "50",
            "--height-mm",
            "60",
            "--output",
            file.to_str().unwrap(),
        ],
    );
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    // Sizes outside the generator's bounds are refused.
    assert!(!cli(
        &ws,
        &[
            "target",
            "generate",
            "--kind",
            "channel_identity",
            "--width-mm",
            "50",
            "--height-mm",
            "40",
            "--output",
            out.join("small.json").to_str().unwrap()
        ]
    )
    .status
    .success());
    assert!(!out.join("small.json").exists());
    let svg = fs::read_to_string(&file).unwrap();
    assert!(svg.contains("width=\"50mm\"") && svg.contains("NOT PRINTER-READY"));
    // Never overwrites, and an unknown kind is refused.
    assert!(!cli(
        &ws,
        &[
            "target",
            "generate",
            "--kind",
            "channel_identity",
            "--width-mm",
            "50",
            "--height-mm",
            "60",
            "--output",
            file.to_str().unwrap()
        ]
    )
    .status
    .success());
    assert!(!cli(
        &ws,
        &[
            "target",
            "generate",
            "--kind",
            "bogus",
            "--width-mm",
            "50",
            "--height-mm",
            "60",
            "--output",
            out.join("x.json").to_str().unwrap()
        ]
    )
    .status
    .success());
    // Other kinds export as JSON; the SVG preview only supports channel identity.
    let geometry = out.join("g.json");
    let g = cli(
        &ws,
        &[
            "target",
            "generate",
            "--kind",
            "geometry",
            "--width-mm",
            "60",
            "--height-mm",
            "60",
            "--output",
            geometry.to_str().unwrap(),
        ],
    );
    assert!(g.status.success(), "{}", String::from_utf8_lossy(&g.stderr));
    let g_svg = out.join("g.svg");
    assert!(!cli(
        &ws,
        &[
            "target",
            "generate",
            "--kind",
            "geometry",
            "--width-mm",
            "60",
            "--height-mm",
            "60",
            "--output",
            g_svg.to_str().unwrap()
        ],
    )
    .status
    .success());
    assert!(!g_svg.exists());
    assert!(!ws.exists());
}
