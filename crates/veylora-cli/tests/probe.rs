use std::{fs, process::Command};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_veylora-cli"))
}

#[test]
fn svg_visualization_has_geometry_labels_and_exact_manifest_hash() {
    let dir = tempfile_dir("svg");
    for name in ["first.svg", "second.svg"] {
        let result = cli()
            .args(["probe", "render-bounded-target", "--output"])
            .arg(dir.join(name))
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
    }
    let bytes = fs::read(dir.join("first.svg")).unwrap();
    assert_eq!(bytes, fs::read(dir.join("second.svg")).unwrap());
    let svg = String::from_utf8(bytes.clone()).unwrap();
    for text in [
        "width=\"100mm\"",
        "viewBox=\"0 0 100 100\"",
        "DIAGNOSTIC ONLY",
        "NOT PRINTER-READY",
        "TOP LEFT R 123",
        "Isolated W",
        "K + W overlap",
        "<rect",
        "<path",
    ] {
        assert!(svg.contains(text), "missing {text}");
    }
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("first.svg.manifest.json")).unwrap()).unwrap();
    assert_eq!(
        manifest["artifact_sha256"],
        veylora_domain::provenance::ContentHash::digest(&bytes).to_string()
    );
    assert_eq!(manifest["artifact_byte_count"], bytes.len());
    assert_eq!(manifest["printer_ready"], false);
    assert_eq!(manifest["target"]["kind"], "channel_identity");
}

#[test]
fn existing_svg_manifest_blocks_export_without_replacing_either_file() {
    let dir = tempfile_dir("svg-existing");
    fs::write(dir.join("target.svg.manifest.json"), b"retain").unwrap();
    let result = cli()
        .args(["probe", "render-bounded-target", "--output"])
        .arg(dir.join("target.svg"))
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!dir.join("target.svg").exists());
    assert_eq!(
        fs::read(dir.join("target.svg.manifest.json")).unwrap(),
        b"retain"
    );
}

#[test]
fn probe_help_describes_safe_actions() {
    let output = cli().args(["probe", "--help"]).output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("inspect-reference"));
    assert!(text.contains("record-observation"));
    assert!(text.contains("diagnostic"));
}

#[test]
fn inspect_reference_reports_exact_hash_and_bounded_metadata() {
    let dir = tempfile_dir("inspect");
    let input = dir.join("reference.bin");
    fs::write(&input, b"abc").unwrap();
    let output = cli()
        .args(["probe", "inspect-reference", "--input"])
        .arg(&input)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["decoded_protocol"], false);
    assert_eq!(json["byte_count"], 3);
    assert_eq!(
        json["sha256"],
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn decode_candidate_is_refused_even_with_experimental() {
    let output = cli()
        .args(["probe", "decode-candidate", "--experimental"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(text.contains("no established protocol"));
    assert!(text.contains("submission refused"));
}

#[test]
fn target_output_refuses_overwrite_and_is_labeled_diagnostic() {
    let dir = tempfile_dir("target");
    let output_path = dir.join("target.svg");
    fs::write(&output_path, b"keep").unwrap();
    let output = cli()
        .args(["probe", "render-bounded-target", "--output"])
        .arg(&output_path)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("refusing to overwrite"));
    assert_eq!(fs::read(&output_path).unwrap(), b"keep");
}

fn tempfile_dir(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("veylora-probe-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn exports_deterministic_shared_diagnostic_target() {
    let dir = tempfile_dir("generated");
    for name in ["one.json", "two.json"] {
        let result = cli()
            .args(["probe", "render-bounded-target", "--output"])
            .arg(dir.join(name))
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
    }
    let bytes = fs::read(dir.join("one.json")).unwrap();
    assert_eq!(bytes, fs::read(dir.join("two.json")).unwrap());
    let artifact: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(artifact["printer_ready"], false);
    assert!(artifact["target"]["objects"].as_array().unwrap().len() > 5);
    assert_eq!(artifact["target"]["kind"], "channel_identity");
}

#[test]
fn concurrent_exports_do_not_replace_each_other() {
    let dir = tempfile_dir("concurrent");
    let path = dir.join("target.json");
    let mut first = cli()
        .args(["probe", "render-bounded-target", "--output"])
        .arg(&path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let mut second = cli()
        .args(["probe", "render-bounded-target", "--output"])
        .arg(&path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    assert_ne!(
        first.wait().unwrap().success(),
        second.wait().unwrap().success()
    );
    let artifact: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(artifact["printer_ready"], false);
}

#[test]
fn rejects_unknown_duplicate_missing_flags_and_submission() {
    for args in [
        vec!["inspect-reference", "--input", "missing", "--unsafe"],
        vec![
            "inspect-reference",
            "--input",
            "missing",
            "--input",
            "other",
        ],
        vec!["inspect-reference", "--input"],
        vec!["--help", "unexpected"],
        vec![
            "render-bounded-target",
            "--output",
            "unused",
            "--experimental",
        ],
    ] {
        let result = cli().arg("probe").args(args).output().unwrap();
        assert!(!result.status.success(), "{result:?}");
    }
    for args in [
        vec!["submit"],
        vec!["submit", "--experimental"],
        vec!["decode-candidate"],
    ] {
        let result = cli().arg("probe").args(args).output().unwrap();
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("submission refused"));
    }
}

#[test]
fn records_verified_evidence_and_one_variable_comparison() {
    let dir = tempfile_dir("observation");
    fs::write(dir.join("baseline.bin"), b"abc").unwrap();
    fs::write(dir.join("candidate.bin"), b"abcd").unwrap();
    let request = serde_json::json!({
        "operator": "bench operator", "source": "owned reference export", "permission": "operator authorized",
        "command_family": "unknown", "framing": null,
        "notes": "bytes differ, meaning unknown",
        "baseline": {"evidence": "baseline.bin", "settings": {"channel": "K", "media": "same"}},
        "candidate": {"evidence": "candidate.bin", "settings": {"channel": "W", "media": "same"}},
        "changed_variable": "channel"
    });
    fs::write(
        dir.join("request.json"),
        serde_json::to_vec(&request).unwrap(),
    )
    .unwrap();
    let result = cli()
        .current_dir(&dir)
        .args([
            "probe",
            "record-observation",
            "--input",
            "request.json",
            "--output",
            "observation.json",
        ])
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    let recorded: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("observation.json")).unwrap()).unwrap();
    assert_eq!(
        recorded["evidence_files"][0]["sha256"],
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(recorded["observation"]["changed_variable"], "channel");
    assert_eq!(recorded["protocol_established"], false);
    let retained = fs::read(dir.join("observation.json")).unwrap();
    let again = cli()
        .current_dir(&dir)
        .args([
            "probe",
            "record-observation",
            "--input",
            "request.json",
            "--output",
            "observation.json",
        ])
        .output()
        .unwrap();
    assert!(!again.status.success());
    assert_eq!(fs::read(dir.join("observation.json")).unwrap(), retained);
    fs::write(dir.join("baseline.bin"), b"changed bytes").unwrap();
    let changed = cli()
        .current_dir(&dir)
        .args([
            "probe",
            "record-observation",
            "--input",
            "request.json",
            "--output",
            "changed.json",
        ])
        .output()
        .unwrap();
    assert!(changed.status.success());
    let changed: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("changed.json")).unwrap()).unwrap();
    assert_ne!(
        recorded["evidence_files"][0]["sha256"],
        changed["evidence_files"][0]["sha256"]
    );
    fs::remove_file(dir.join("candidate.bin")).unwrap();
    let missing = cli()
        .current_dir(&dir)
        .args([
            "probe",
            "record-observation",
            "--input",
            "request.json",
            "--output",
            "missing.json",
        ])
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(!dir.join("missing.json").exists());
    fs::write(dir.join("candidate.bin"), b"abcd").unwrap();
    let mut invalid = request;
    invalid["candidate"]["settings"]["media"] = serde_json::json!("changed too");
    fs::write(
        dir.join("invalid.json"),
        serde_json::to_vec(&invalid).unwrap(),
    )
    .unwrap();
    let result = cli()
        .current_dir(&dir)
        .args([
            "probe",
            "record-observation",
            "--input",
            "invalid.json",
            "--output",
            "invalid-out.json",
        ])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!dir.join("invalid-out.json").exists());
}

#[test]
fn rejects_malformed_and_oversized_inputs_without_creating_output() {
    let dir = tempfile_dir("invalid-input");
    for (name, bytes) in [
        ("broken.json", b"{".as_slice()),
        ("array.json", b"[]"),
        ("unknown.json", b"{\"invented\":true}"),
    ] {
        fs::write(dir.join(name), bytes).unwrap();
        let output = dir.join(format!("{name}.out"));
        let result = cli()
            .args(["probe", "record-observation", "--input"])
            .arg(dir.join(name))
            .arg("--output")
            .arg(&output)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(!output.exists());
    }
    let huge = dir.join("huge.bin");
    fs::File::create(&huge)
        .unwrap()
        .set_len(8 * 1024 * 1024 + 1)
        .unwrap();
    for path in [&huge, &dir] {
        let result = cli()
            .args(["probe", "inspect-reference", "--input"])
            .arg(path)
            .output()
            .unwrap();
        assert!(!result.status.success());
    }
}

#[test]
fn input_limit_is_inclusive_and_empty_reference_is_hashable() {
    let dir = tempfile_dir("limits");
    let path = dir.join("evidence.bin");
    for length in [0, 8 * 1024 * 1024] {
        fs::File::create(&path).unwrap().set_len(length).unwrap();
        let result = cli()
            .args(["probe", "inspect-reference", "--input"])
            .arg(&path)
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
        let record: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(record["byte_count"], length);
        if length == 0 {
            assert_eq!(
                record["sha256"],
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            );
        }
    }
}
