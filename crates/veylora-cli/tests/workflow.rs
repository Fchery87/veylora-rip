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
