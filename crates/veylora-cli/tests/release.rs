use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use veylora_domain::{ids::JobId, lifecycle::LifecycleEvent as E};
use veylora_persistence::FileLifecycleStore;
use veylora_workflow::Workspace;

const CLI: &str = env!("CARGO_BIN_EXE_veylora-cli");

fn dir(label: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let p = std::env::temp_dir().join(format!(
        "veylora-cli-release-{label}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(CLI)
        .arg("release")
        .args(args)
        .output()
        .unwrap()
}
fn json(o: &std::process::Output) -> serde_json::Value {
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    serde_json::from_slice(&o.stdout).unwrap()
}
fn err(o: &std::process::Output) -> String {
    assert!(!o.status.success(), "unexpected success");
    String::from_utf8_lossy(&o.stderr).into_owned()
}
fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

/// A release directory: the real CLI as the program, so the smoke test runs something real.
fn release(version: &str, smoke_arg: &str, max_schema: &str) -> PathBuf {
    let d = dir("rel");
    fs::copy(CLI, d.join("veylora-cli.exe")).unwrap();
    fs::write(d.join("LICENSE.txt"), format!("license {version}")).unwrap();
    let o = run(&[
        "manifest",
        "--dir",
        s(&d),
        "--version",
        version,
        "--max-data-schema",
        max_schema,
        "--smoke-test",
        "veylora-cli.exe",
        "--smoke-arg",
        smoke_arg,
    ]);
    json(&o);
    d
}

fn status(root: &Path) -> serde_json::Value {
    json(&run(&["status", "--install-root", s(root)]))
}

#[test]
fn a_packaged_release_verifies_and_tampering_is_caught() {
    let d = release("1.0.0", "--version", "1");
    let v = json(&run(&["verify", "--dir", s(&d)]));
    assert_eq!(v["version"], "1.0.0");
    assert_eq!(
        v["smoke_test"],
        serde_json::json!(["veylora-cli.exe", "--version"])
    );
    fs::write(d.join("LICENSE.txt"), b"changed").unwrap();
    assert!(err(&run(&["verify", "--dir", s(&d)])).contains("LICENSE.txt"));
}

#[test]
fn development_tooling_cannot_be_packaged() {
    let d = dir("dev");
    fs::write(d.join("a.txt"), b"x").unwrap();
    fs::create_dir(d.join("node_modules")).unwrap();
    fs::write(d.join("node_modules").join("x.js"), b"x").unwrap();
    let e = err(&run(&[
        "manifest",
        "--dir",
        s(&d),
        "--version",
        "1.0.0",
        "--max-data-schema",
        "1",
    ]));
    assert!(e.contains("development tool"), "{e}");
    assert!(!d.join("release-manifest.json").exists());
}

#[test]
fn install_update_rollback_and_recover_work_end_to_end() {
    let root = dir("root");
    let first = release("1.0.0", "--version", "1");
    let out = json(&run(&[
        "install",
        "--install-root",
        s(&root),
        "--from",
        s(&first),
        "--no-workspace",
    ]));
    assert_eq!(out["installed"], "1.0.0");
    let st = status(&root);
    assert_eq!(st["current_version"], "1.0.0");
    assert_eq!(st["current_verified"], true);

    let second = release("1.1.0", "--version", "1");
    json(&run(&[
        "install",
        "--install-root",
        s(&root),
        "--from",
        s(&second),
        "--no-workspace",
    ]));
    assert_eq!(status(&root)["state"]["previous"], "1.0.0");
    // Not an update.
    let e = err(&run(&[
        "install",
        "--install-root",
        s(&root),
        "--from",
        s(&first),
        "--no-workspace",
    ]));
    assert!(e.contains("not newer"), "{e}");

    assert_eq!(
        json(&run(&[
            "rollback",
            "--install-root",
            s(&root),
            "--no-workspace"
        ]))["installed"],
        "1.0.0"
    );
    assert_eq!(status(&root)["current_version"], "1.0.0");
    assert!(err(&run(&[
        "rollback",
        "--install-root",
        s(&root),
        "--no-workspace"
    ]))
    .contains("no previous"));

    fs::write(root.join("install.lock"), b"stale").unwrap();
    let rec = json(&run(&["recover", "--install-root", s(&root)]));
    assert_eq!(rec["abandoned_update"], false);
    assert!(!root.join("install.lock").exists());
}

#[test]
fn an_update_needs_a_workspace_or_an_explicit_statement_that_there_is_none() {
    let root = dir("root");
    let rel = release("1.0.0", "--version", "1");
    let e = err(&run(&[
        "install",
        "--install-root",
        s(&root),
        "--from",
        s(&rel),
    ]));
    assert!(e.contains("--workspace"), "{e}");
    let ws = dir("ws");
    let e = err(&run(&[
        "install",
        "--install-root",
        s(&root),
        "--from",
        s(&rel),
        "--workspace",
        s(&ws),
        "--no-workspace",
    ]));
    assert!(e.contains("not both"), "{e}");
    let missing = ws.join("nope");
    let e = err(&run(&[
        "install",
        "--install-root",
        s(&root),
        "--from",
        s(&rel),
        "--workspace",
        s(&missing),
    ]));
    assert!(e.contains("does not exist"), "{e}");
    assert_eq!(status(&root)["current_version"], serde_json::Value::Null);
}

fn workspace_with_submission_in_flight() -> PathBuf {
    let ws = dir("ws");
    Workspace::open(&ws).unwrap();
    let store = FileLifecycleStore::open(ws.join("state")).unwrap();
    let job = JobId::new();
    for (i, ev) in [
        E::PreflightPassed,
        E::BeginRender,
        E::RenderCompleted,
        E::BeginSubmission,
    ]
    .into_iter()
    .enumerate()
    {
        store
            .append(job, ev, &format!("2026-11-20T00:00:0{i}Z"), "", None)
            .unwrap();
    }
    ws
}

#[test]
fn an_update_is_refused_while_a_submission_is_in_flight() {
    let ws = workspace_with_submission_in_flight();
    let b = json(&run(&["blockers", "--workspace", s(&ws)]));
    assert_eq!(b["safe_to_update"], false);
    assert_eq!(b["blockers"][0]["state"], "submitting");

    let root = dir("root");
    let rel = release("1.0.0", "--version", "1");
    let e = err(&run(&[
        "install",
        "--install-root",
        s(&root),
        "--from",
        s(&rel),
        "--workspace",
        s(&ws),
    ]));
    assert!(e.contains("in flight"), "{e}");
    assert_eq!(status(&root)["current_version"], serde_json::Value::Null);
    assert!(err(&run(&[
        "rollback",
        "--install-root",
        s(&root),
        "--workspace",
        s(&ws)
    ]))
    .contains("in flight"));
}

#[test]
fn an_idle_workspace_allows_the_update() {
    let ws = dir("ws");
    Workspace::open(&ws).unwrap();
    assert_eq!(
        json(&run(&["blockers", "--workspace", s(&ws)]))["safe_to_update"],
        true
    );
    let root = dir("root");
    let rel = release("1.0.0", "--version", "1");
    json(&run(&[
        "install",
        "--install-root",
        s(&root),
        "--from",
        s(&rel),
        "--workspace",
        s(&ws),
    ]));
    assert_eq!(status(&root)["current_version"], "1.0.0");
}

#[test]
fn a_release_that_cannot_read_the_workspace_data_is_refused() {
    let ws = dir("ws");
    Workspace::open(&ws).unwrap();
    let root = dir("root");
    let rel = release("1.0.0", "--version", "0");
    let e = err(&run(&[
        "install",
        "--install-root",
        s(&root),
        "--from",
        s(&rel),
        "--workspace",
        s(&ws),
    ]));
    assert!(e.contains("schema"), "{e}");
    assert_eq!(status(&root)["current_version"], serde_json::Value::Null);
}

#[test]
fn a_release_that_fails_its_smoke_test_is_rolled_back_automatically() {
    let root = dir("root");
    let good = release("1.0.0", "--version", "1");
    json(&run(&[
        "install",
        "--install-root",
        s(&root),
        "--from",
        s(&good),
        "--no-workspace",
    ]));
    let bad = release("1.1.0", "--no-such-option", "1");
    let e = err(&run(&[
        "install",
        "--install-root",
        s(&root),
        "--from",
        s(&bad),
        "--no-workspace",
    ]));
    assert!(e.contains("previous release was restored"), "{e}");
    let st = status(&root);
    assert_eq!(st["current_version"], "1.0.0");
    assert_eq!(st["state"]["previous"], serde_json::Value::Null);
    let names: Vec<_> = fs::read_dir(root.join("releases"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names.len(), 1);
}

#[test]
fn bad_usage_is_refused() {
    assert!(!run(&[]).status.success());
    assert!(!run(&["bogus"]).status.success());
    assert!(!run(&["verify"]).status.success());
    assert!(!run(&["verify", "--dir"]).status.success());
    assert!(!run(&["verify", "--dir", "a", "--extra", "b"])
        .status
        .success());
    let d = dir("x");
    fs::write(d.join("a.txt"), b"x").unwrap();
    // --smoke-arg without a program.
    assert!(err(&run(&[
        "manifest",
        "--dir",
        s(&d),
        "--version",
        "1.0.0",
        "--max-data-schema",
        "1",
        "--smoke-arg",
        "x"
    ]))
    .contains("--smoke-test"));
    assert!(err(&run(&[
        "manifest",
        "--dir",
        s(&d),
        "--version",
        "1.0.0",
        "--max-data-schema",
        "x"
    ]))
    .contains("number"));
    assert!(err(&run(&[
        "manifest",
        "--dir",
        s(&d),
        "--version",
        "1.0",
        "--max-data-schema",
        "1"
    ]))
    .contains("MAJOR.MINOR.PATCH"));
}
