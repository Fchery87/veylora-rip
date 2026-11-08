use std::{fs, process::Command};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_veylora-cli"))
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
