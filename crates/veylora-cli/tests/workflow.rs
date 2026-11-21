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

    // Support bundles: artwork only when asked, never overwritten, bad choice refused.
    let plain = files.join("plain.tar");
    let with_art = files.join("art.tar");
    let run = |out: &Path, extra: &[&str]| {
        let mut a = vec!["report", "support-bundle", "--job", &job, "--output"];
        a.push(out.to_str().unwrap());
        a.extend_from_slice(extra);
        cli(&ws, &a)
    };
    assert!(run(&plain, &[]).status.success());
    assert!(run(&with_art, &["--artwork", "include"]).status.success());
    let names = |p: &Path| -> Vec<String> {
        veylora_diagnostics::bundle::read_tar(&fs::read(p).unwrap())
            .unwrap()
            .into_iter()
            .map(|e| e.path)
            .collect()
    };
    assert!(names(&plain).iter().all(|n| !n.starts_with("artwork/")));
    assert!(names(&with_art).iter().any(|n| n.starts_with("artwork/")));
    assert!(!run(&plain, &[]).status.success());
    assert!(!run(&files.join("x.tar"), &["--artwork", "yes"])
        .status
        .success());
    assert!(!files.join("x.tar").exists());
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

/// A workspace with one rendered job, built through the CLI.
fn rendered_job(ws: &Path) -> String {
    let files = dir("cli-life-files");
    fs::create_dir_all(&files).unwrap();
    let s = setup();
    let m = mode(s.id());
    let put = |name: &str, bytes: Vec<u8>| {
        let p = files.join(name);
        fs::write(&p, bytes).unwrap();
        p
    };
    let imp = |kind: &str, f: &Path| {
        hash_of(&json(&cli(
            ws,
            &[kind, "import", "--input", f.to_str().unwrap()],
        )))
    };
    let sh = imp(
        "setup",
        &put("s.json", serde_json::to_vec(&s.to_record()).unwrap()),
    );
    let mh = imp(
        "mode",
        &put("m.json", serde_json::to_vec(&m.to_record()).unwrap()),
    );
    let rh = imp(
        "recipe",
        &put("r.json", serde_json::to_vec(&recipe()).unwrap()),
    );
    let ah = imp("asset", &put("a.png", fixture("alpha-ramp.png")));
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
    json(&cli(
        ws,
        &["job", "create", "--spec", spec_f.to_str().unwrap()],
    ))["job_id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn the_job_lifecycle_is_visible_and_operator_controlled_from_the_cli() {
    let ws = dir("cli-life");
    let job = rendered_job(&ws);
    let state = |ws: &Path| json(&cli(ws, &["job", "status", "--job", &job]))["state"].clone();
    assert_eq!(state(&ws), "draft");
    json(&cli(&ws, &["job", "render", "--job", &job]));
    assert_eq!(state(&ws), "ready");
    let events = json(&cli(&ws, &["job", "status", "--job", &job]))["events"].clone();
    assert_eq!(events.as_array().unwrap().len(), 3);

    assert_eq!(
        json(&cli(&ws, &["job", "hold", "--job", &job]))["state"],
        "held"
    );
    // Recovery changes nothing on a job that was not interrupted.
    let rec = json(&cli(&ws, &["job", "recover", "--job", &job]));
    assert_eq!(
        (&rec["before"], &rec["after"]),
        (&"held".into(), &"held".into())
    );
    assert_eq!(
        json(&cli(&ws, &["job", "cancel", "--job", &job]))["state"],
        "cancelled"
    );
    // A cancelled job accepts nothing further.
    assert!(!cli(&ws, &["job", "hold", "--job", &job]).status.success());
    assert!(!cli(&ws, &["job", "render", "--job", &job]).status.success());
}

#[test]
fn review_decisions_need_an_operator_a_note_and_something_to_resolve() {
    let ws = dir("cli-resolve");
    let job = rendered_job(&ws);
    json(&cli(&ws, &["job", "render", "--job", &job]));
    let resolve = |args: &[&str]| {
        let mut a = vec!["job", "review", "--job", job.as_str()];
        a.extend_from_slice(args);
        cli(&ws, &a)
    };
    // Missing operator or note.
    assert!(!resolve(&["--resolve", "finished"]).status.success());
    assert!(!resolve(&["--resolve", "finished", "--operator", "op"])
        .status
        .success());
    // Unknown decision.
    assert!(
        !resolve(&["--resolve", "maybe", "--operator", "op", "--note", "n"])
            .status
            .success()
    );
    // A decision is refused for a job that is not awaiting review, and nothing changes.
    assert!(
        !resolve(&["--resolve", "finished", "--operator", "op", "--note", "n"])
            .status
            .success()
    );
    assert_eq!(
        json(&cli(&ws, &["job", "status", "--job", &job]))["state"],
        "ready"
    );
    // Plain review still reads.
    assert_eq!(json(&resolve(&[]))["requires_review"], false);
}

/// Leaves a rendered job the way a submission would: begun, with the attempt log as given.
fn interrupted_job(ws: &Path, attempt_states: &[veylora_domain::job::AttemptState]) -> String {
    use veylora_domain::{
        ids::{AttemptId, JobId},
        job::SubmissionAttempt,
        lifecycle::LifecycleEvent as E,
    };
    use veylora_persistence::{FileJobStore, FileLifecycleStore, JobStore};
    let job = rendered_job(ws);
    json(&cli(ws, &["job", "render", "--job", &job]));
    let id: JobId = job.parse().unwrap();
    let attempt = AttemptId::new();
    let life = FileLifecycleStore::open(ws.join("state")).unwrap();
    life.append(
        id,
        E::BeginSubmission,
        "2026-11-15T10:00:00Z",
        "",
        Some(attempt),
    )
    .unwrap();
    let jobs = FileJobStore::open(ws.join("jobs")).unwrap();
    let snapshot_hash = jobs.get(id).unwrap().content_hash();
    for (i, state) in attempt_states.iter().enumerate() {
        jobs.append_attempt(SubmissionAttempt {
            attempt_id: attempt,
            job_id: id,
            snapshot_hash,
            sequence: i as u32 + 1,
            state: *state,
            bytes_written: 0,
            note: String::new(),
            recorded_at: "2026-11-15T10:00:00Z".into(),
        })
        .unwrap();
    }
    job
}

#[test]
fn an_operator_settles_an_ambiguous_submission_from_the_cli() {
    use veylora_domain::job::AttemptState::*;
    use veylora_domain::lifecycle::LifecycleEvent as E;
    let ws = dir("cli-settle");
    let job = interrupted_job(&ws, &[Submitting, Ambiguous]);
    let id: veylora_domain::ids::JobId = job.parse().unwrap();
    veylora_persistence::FileLifecycleStore::open(ws.join("state"))
        .unwrap()
        .append(id, E::SubmissionUncertain, "2026-11-15T10:00:00Z", "", None)
        .unwrap();
    assert_eq!(
        json(&cli(&ws, &["job", "review", "--job", &job]))["requires_review"],
        true
    );

    let resolve = |args: &[&str]| {
        let mut a = vec!["job", "review", "--job", job.as_str()];
        a.extend_from_slice(args);
        cli(&ws, &a)
    };
    // Each of these is refused and leaves the job awaiting review.
    for bad in [
        &["--resolve", "finished"][..],
        &["--resolve", "finished", "--operator", "op"][..],
        &["--resolve", "finished", "--note", "n"][..],
        &["--resolve", "maybe", "--operator", "op", "--note", "n"][..],
        &["--operator", "op", "--note", "n"][..],
    ] {
        assert!(!resolve(bad).status.success(), "{bad:?}");
    }
    assert_eq!(
        json(&cli(&ws, &["job", "status", "--job", &job]))["state"],
        "ambiguous"
    );

    let done = json(&resolve(&[
        "--resolve",
        "finished",
        "--operator",
        "op",
        "--note",
        "checked the film",
    ]));
    assert_eq!(done["state"], "operator_confirmed_finished");
    assert_eq!(json(&resolve(&[]))["requires_review"], false);

    // The other decision cancels the job instead.
    let other = dir("cli-settle-2");
    let job2 = interrupted_job(&other, &[Submitting, Ambiguous]);
    let id2: veylora_domain::ids::JobId = job2.parse().unwrap();
    veylora_persistence::FileLifecycleStore::open(other.join("state"))
        .unwrap()
        .append(
            id2,
            E::SubmissionUncertain,
            "2026-11-15T10:00:00Z",
            "",
            None,
        )
        .unwrap();
    let cancelled = json(&cli(
        &other,
        &[
            "job",
            "review",
            "--job",
            &job2,
            "--resolve",
            "not-printed",
            "--operator",
            "op",
            "--note",
            "blank film",
        ],
    ));
    assert_eq!(cancelled["state"], "cancelled");
}

#[test]
fn recover_settles_a_crashed_submission_for_review() {
    use veylora_domain::job::AttemptState::*;
    let ws = dir("cli-recover");
    let job = interrupted_job(&ws, &[Submitting]);
    assert_eq!(
        json(&cli(&ws, &["job", "status", "--job", &job]))["state"],
        "submitting"
    );
    let r = json(&cli(&ws, &["job", "recover", "--job", &job]));
    assert_eq!(
        (&r["before"], &r["after"]),
        (&"submitting".into(), &"review".into())
    );
}

#[test]
fn resource_limits_are_options_on_preflight_and_render() {
    let ws = dir("cli-limits");
    let job = rendered_job(&ws);
    let ok = json(&cli(&ws, &["job", "preflight", "--job", &job]));
    assert_eq!(ok["can_render"], true);
    assert_eq!(ok["policy"]["max_page_pixels"], 1u64 << 26);
    assert!(ok["policy"]["disk_budget_bytes"].is_null());

    let small = json(&cli(
        &ws,
        &["job", "preflight", "--job", &job, "--max-page-pixels", "3"],
    ));
    assert_eq!(small["can_render"], false);
    assert_eq!(small["policy"]["max_page_pixels"], 3);
    let broke = json(&cli(
        &ws,
        &[
            "job",
            "preflight",
            "--job",
            &job,
            "--disk-budget-bytes",
            "1",
        ],
    ));
    assert_eq!(broke["can_render"], false);
    assert_eq!(broke["policy"]["disk_budget_bytes"], 1);

    // Render takes the same limits and refuses.
    assert!(!cli(
        &ws,
        &["job", "render", "--job", &job, "--max-page-pixels", "3"]
    )
    .status
    .success());
    assert!(!cli(
        &ws,
        &["job", "render", "--job", &job, "--disk-budget-bytes", "1"]
    )
    .status
    .success());
    assert!(!cli(
        &ws,
        &["job", "render", "--job", &job, "--max-page-pixels", "x"]
    )
    .status
    .success());
    json(&cli(
        &ws,
        &["job", "render", "--job", &job, "--max-page-pixels", "4"],
    ));
}
