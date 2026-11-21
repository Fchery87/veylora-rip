use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
};
use veylora_release::{
    build_manifest, verify_release, write_manifest, Blocker, InstallState, InstallStore,
    PendingUpdate, ReleaseError, UpdateGuard, UpdateRequest, MANIFEST_NAME,
};

fn dir(label: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let p = std::env::temp_dir().join(format!(
        "veylora-release-{label}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, bytes: &[u8]) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, bytes).unwrap();
}

/// A staged release directory with a manifest, as the packaging script would produce.
fn release(version: &str, max_data_schema: u32) -> PathBuf {
    let d = dir("rel");
    write(
        &d,
        "veylora-cli.exe",
        format!("binary {version}").as_bytes(),
    );
    write(&d, "resources/profile.txt", b"data");
    write(&d, "LICENSE.txt", b"license");
    let m = build_manifest(&d, version, max_data_schema, None).unwrap();
    write_manifest(&d, &m).unwrap();
    d
}

struct Quiet;
impl UpdateGuard for Quiet {
    fn blockers(&self) -> Result<Vec<Blocker>, String> {
        Ok(vec![])
    }
}
struct Busy;
impl UpdateGuard for Busy {
    fn blockers(&self) -> Result<Vec<Blocker>, String> {
        Ok(vec![Blocker {
            subject: "job-1".into(),
            reason: "a submission is in progress".into(),
        }])
    }
}
struct Unknown;
impl UpdateGuard for Unknown {
    fn blockers(&self) -> Result<Vec<Blocker>, String> {
        Err("cannot read the job state".into())
    }
}

fn ok(_: &Path, _: &veylora_release::ReleaseManifest) -> Result<(), String> {
    Ok(())
}

fn install(
    store: &InstallStore,
    src: &Path,
) -> Result<veylora_release::InstallOutcome, ReleaseError> {
    store.install(&UpdateRequest {
        source: src,
        data_schema: None,
        guard: &Quiet,
        smoke_test: &ok,
    })
}

fn names(root: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(root.join("releases"))
        .map(|d| {
            d.map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

// ---- manifest ----

#[test]
fn a_built_manifest_lists_every_file_with_its_hash_and_verifies() {
    let d = release("1.0.0", 1);
    let m = verify_release(&d).unwrap();
    assert_eq!(m.version, "1.0.0");
    let paths: Vec<&str> = m.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(
        paths,
        ["LICENSE.txt", "resources/profile.txt", "veylora-cli.exe"]
    );
    assert!(m.files.iter().all(|f| f.bytes > 0));
}

#[test]
fn tampering_is_refused() {
    let d = release("1.0.0", 1);
    write(&d, "veylora-cli.exe", b"changed!!!");
    assert!(matches!(
        verify_release(&d),
        Err(ReleaseError::Mismatch { .. })
    ));

    let d = release("1.0.0", 1);
    fs::remove_file(d.join("LICENSE.txt")).unwrap();
    assert!(matches!(
        verify_release(&d),
        Err(ReleaseError::Mismatch { .. })
    ));

    let d = release("1.0.0", 1);
    write(&d, "extra.dll", b"x");
    assert!(matches!(
        verify_release(&d),
        Err(ReleaseError::Mismatch { .. })
    ));

    let d = release("1.0.0", 1);
    fs::remove_file(d.join(MANIFEST_NAME)).unwrap();
    assert!(matches!(
        verify_release(&d),
        Err(ReleaseError::Malformed(_))
    ));
}

#[test]
fn a_hand_edited_manifest_is_refused() {
    let edit = |f: &dyn Fn(&mut serde_json::Value)| {
        let d = release("1.0.0", 1);
        let p = d.join(MANIFEST_NAME);
        let mut v: serde_json::Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
        f(&mut v);
        fs::write(&p, serde_json::to_vec(&v).unwrap()).unwrap();
        verify_release(&d)
    };
    assert!(matches!(
        edit(&|v| v["product"] = "other".into()),
        Err(ReleaseError::Malformed(_))
    ));
    assert!(matches!(
        edit(&|v| v["schema_version"] = 9.into()),
        Err(ReleaseError::Malformed(_))
    ));
    assert!(matches!(
        edit(&|v| v["version"] = "1.0".into()),
        Err(ReleaseError::Version(_))
    ));
    assert!(matches!(
        edit(&|v| v["files"][0]["path"] = "../evil".into()),
        Err(ReleaseError::UnsafePath(_))
    ));
    assert!(matches!(
        edit(&|v| {
            let f = v["files"][0].clone();
            v["files"].as_array_mut().unwrap().push(f);
        }),
        Err(ReleaseError::Malformed(_))
    ));
    assert!(matches!(
        edit(&|v| v["smoke_test"] = serde_json::json!(["../outside.exe"])),
        Err(ReleaseError::UnsafePath(_))
    ));
}

#[test]
fn development_tooling_is_kept_out_of_a_release() {
    for bad in [
        "node_modules/x/index.js",
        ".git/config",
        "target/debug/a.exe",
        "Cargo.toml",
        "Cargo.lock",
        "package.json",
        "pnpm-lock.yaml",
        "src/main.rs",
        "app/main.ts",
        "veylora.pdb",
        "tsconfig.json",
        "vite.config.ts",
        "rust-toolchain.toml",
        "NODE_MODULES/x.js",
        "lib/a.rlib",
    ] {
        let d = dir("dev");
        write(&d, "veylora-cli.exe", b"x");
        write(&d, bad, b"x");
        assert!(
            matches!(
                build_manifest(&d, "1.0.0", 1, None),
                Err(ReleaseError::DevTool(_))
            ),
            "{bad}"
        );
    }
    // The same files are caught at verification even if a manifest names them.
    let d = release("1.0.0", 1);
    write(&d, "node_modules/x.js", b"x");
    assert!(verify_release(&d).is_err());
}

#[test]
fn versions_must_be_plain_numeric_triples() {
    for good in ["0.0.1", "1.2.3", "10.20.30"] {
        let d = dir("ver");
        write(&d, "a.txt", b"x");
        build_manifest(&d, good, 1, None).unwrap_or_else(|e| panic!("{good}: {e}"));
    }
    for bad in [
        "",
        "1",
        "1.2",
        "v1.2.3",
        "1.2.3-beta",
        "1.2.3.4",
        "1..3",
        "a.b.c",
        "01.2.3",
        "1.2.-3",
    ] {
        let d = dir("ver");
        write(&d, "a.txt", b"x");
        assert!(
            matches!(
                build_manifest(&d, bad, 1, None),
                Err(ReleaseError::Version(_))
            ),
            "{bad}"
        );
    }
}

#[test]
fn a_release_must_not_be_empty() {
    let d = dir("empty");
    assert!(build_manifest(&d, "1.0.0", 1, None).is_err());
}

// ---- install ----

#[test]
fn a_first_install_activates_the_release_and_copies_it_intact() {
    let root = dir("inst");
    let store = InstallStore::open(&root).unwrap();
    assert_eq!(store.state().unwrap(), InstallState::default());
    let out = install(&store, &release("1.0.0", 1)).unwrap();
    assert_eq!(out.installed, "1.0.0");
    assert_eq!(out.previous, None);
    assert_eq!(store.state().unwrap().current.as_deref(), Some("1.0.0"));
    // The installed copy is a verified release of its own.
    verify_release(&root.join("releases").join("1.0.0")).unwrap();
    assert_eq!(
        store.current_dir().unwrap(),
        Some(root.join("releases").join("1.0.0"))
    );
    assert!(!root.join("install-state.json.tmp").exists());
}

#[test]
fn an_update_keeps_exactly_one_previous_release_for_rollback() {
    let root = dir("upd");
    let store = InstallStore::open(&root).unwrap();
    install(&store, &release("1.0.0", 1)).unwrap();
    install(&store, &release("1.1.0", 1)).unwrap();
    let s = store.state().unwrap();
    assert_eq!(
        (s.current.as_deref(), s.previous.as_deref()),
        (Some("1.1.0"), Some("1.0.0"))
    );
    install(&store, &release("1.2.0", 1)).unwrap();
    let s = store.state().unwrap();
    assert_eq!(
        (s.current.as_deref(), s.previous.as_deref()),
        (Some("1.2.0"), Some("1.1.0"))
    );
    assert_eq!(names(&root), ["1.1.0", "1.2.0"]);
}

#[test]
fn the_same_or_an_older_version_is_not_an_update() {
    let root = dir("ord");
    let store = InstallStore::open(&root).unwrap();
    install(&store, &release("1.5.0", 1)).unwrap();
    for v in ["1.5.0", "1.4.9", "0.9.0"] {
        assert!(
            matches!(
                install(&store, &release(v, 1)),
                Err(ReleaseError::NotNewer { .. })
            ),
            "{v}"
        );
    }
    // Numeric, not textual, ordering.
    install(&store, &release("1.10.0", 1)).unwrap();
}

#[test]
fn an_update_is_refused_while_work_is_in_flight_and_changes_nothing() {
    let root = dir("busy");
    let store = InstallStore::open(&root).unwrap();
    install(&store, &release("1.0.0", 1)).unwrap();
    let before = store.state().unwrap();
    let src = release("1.1.0", 1);
    let err = store
        .install(&UpdateRequest {
            source: &src,
            data_schema: None,
            guard: &Busy,
            smoke_test: &ok,
        })
        .unwrap_err();
    match err {
        ReleaseError::Blocked(b) => assert_eq!(b[0].subject, "job-1"),
        other => panic!("{other}"),
    }
    assert_eq!(store.state().unwrap(), before);
    assert_eq!(names(&root), ["1.0.0"]);
}

#[test]
fn an_update_is_refused_when_the_job_state_cannot_be_read() {
    let root = dir("unk");
    let store = InstallStore::open(&root).unwrap();
    let src = release("1.0.0", 1);
    let err = store
        .install(&UpdateRequest {
            source: &src,
            data_schema: None,
            guard: &Unknown,
            smoke_test: &ok,
        })
        .unwrap_err();
    assert!(matches!(err, ReleaseError::GuardUnavailable(_)), "{err}");
    assert!(names(&root).is_empty());
}

#[test]
fn a_tampered_source_is_refused_before_anything_is_written() {
    let root = dir("tamper");
    let store = InstallStore::open(&root).unwrap();
    let src = release("1.0.0", 1);
    write(&src, "veylora-cli.exe", b"evil");
    assert!(install(&store, &src).is_err());
    assert!(names(&root).is_empty());
    assert_eq!(store.state().unwrap(), InstallState::default());
}

#[test]
fn a_release_that_cannot_read_the_existing_data_is_refused() {
    let root = dir("schema");
    let store = InstallStore::open(&root).unwrap();
    let src = release("2.0.0", 1);
    let req = |data_schema| UpdateRequest {
        source: &src,
        data_schema,
        guard: &Quiet,
        smoke_test: &ok,
    };
    assert!(matches!(
        store.install(&req(Some(2))),
        Err(ReleaseError::DataTooNew {
            data: 2,
            supported: 1
        })
    ));
    assert!(names(&root).is_empty());
    // Data at or below what the release reads is fine.
    store.install(&req(Some(1))).unwrap();
}

#[test]
fn a_failed_smoke_test_puts_the_old_release_back() {
    let root = dir("smoke");
    let store = InstallStore::open(&root).unwrap();
    install(&store, &release("1.0.0", 1)).unwrap();
    let before = store.state().unwrap();
    let src = release("1.1.0", 1);
    let seen = RefCell::new(None);
    let fail = |p: &Path, m: &veylora_release::ReleaseManifest| -> Result<(), String> {
        *seen.borrow_mut() = Some((p.to_path_buf(), m.version.clone()));
        Err("does not start".into())
    };
    let err = store
        .install(&UpdateRequest {
            source: &src,
            data_schema: None,
            guard: &Quiet,
            smoke_test: &fail,
        })
        .unwrap_err();
    match err {
        ReleaseError::RolledBack { version, reason } => {
            assert_eq!(version, "1.1.0");
            assert!(reason.contains("does not start"));
        }
        other => panic!("{other}"),
    }
    // The smoke test ran against the installed copy, then everything went back.
    let (path, version) = seen.borrow().clone().unwrap();
    assert_eq!(version, "1.1.0");
    assert_eq!(path, root.join("releases").join("1.1.0"));
    assert_eq!(store.state().unwrap(), before);
    assert_eq!(names(&root), ["1.0.0"]);
}

#[test]
fn a_failed_first_install_leaves_nothing_active() {
    let root = dir("smoke1");
    let store = InstallStore::open(&root).unwrap();
    let src = release("1.0.0", 1);
    let fail = |_: &Path, _: &veylora_release::ReleaseManifest| Err("no".to_string());
    assert!(store
        .install(&UpdateRequest {
            source: &src,
            data_schema: None,
            guard: &Quiet,
            smoke_test: &fail,
        })
        .is_err());
    assert_eq!(store.state().unwrap(), InstallState::default());
    assert!(names(&root).is_empty());
}

// ---- rollback ----

#[test]
fn rollback_returns_to_the_previous_release_once() {
    let root = dir("rb");
    let store = InstallStore::open(&root).unwrap();
    install(&store, &release("1.0.0", 1)).unwrap();
    install(&store, &release("1.1.0", 1)).unwrap();
    let out = store.rollback(&Quiet).unwrap();
    assert_eq!(out.installed, "1.0.0");
    let s = store.state().unwrap();
    assert_eq!(
        (s.current.as_deref(), s.previous.as_deref()),
        (Some("1.0.0"), None)
    );
    assert_eq!(names(&root), ["1.0.0"]);
    assert!(matches!(
        store.rollback(&Quiet),
        Err(ReleaseError::NothingToRollBackTo)
    ));
}

#[test]
fn rollback_obeys_the_same_guard_and_checks_what_it_returns_to() {
    let root = dir("rb2");
    let store = InstallStore::open(&root).unwrap();
    install(&store, &release("1.0.0", 1)).unwrap();
    install(&store, &release("1.1.0", 1)).unwrap();
    assert!(matches!(
        store.rollback(&Busy),
        Err(ReleaseError::Blocked(_))
    ));
    assert_eq!(store.state().unwrap().current.as_deref(), Some("1.1.0"));
    // A damaged previous release is not switched to.
    write(
        &root.join("releases").join("1.0.0"),
        "veylora-cli.exe",
        b"damaged",
    );
    assert!(store.rollback(&Quiet).is_err());
    assert_eq!(store.state().unwrap().current.as_deref(), Some("1.1.0"));
}

// ---- recovery ----

#[test]
fn recovery_abandons_an_update_that_never_committed() {
    let root = dir("rec");
    let store = InstallStore::open(&root).unwrap();
    install(&store, &release("1.0.0", 1)).unwrap();
    // A crash after the journal entry and a partial copy, before the commit.
    let mut s = store.state().unwrap();
    s.pending = Some(PendingUpdate {
        from: Some("1.0.0".into()),
        to: "1.1.0".into(),
    });
    fs::write(
        root.join("install-state.json"),
        serde_json::to_vec(&s).unwrap(),
    )
    .unwrap();
    write(
        &root.join("releases"),
        "1.1.0.staging/veylora-cli.exe",
        b"par",
    );
    write(
        &root.join("releases"),
        "1.1.0/veylora-cli.exe",
        b"renamed but never committed",
    );
    let report = store.recover().unwrap();
    assert!(report.abandoned_update);
    let s = store.state().unwrap();
    assert_eq!(
        (s.current.as_deref(), s.pending.as_ref()),
        (Some("1.0.0"), None)
    );
    assert_eq!(names(&root), ["1.0.0"]);
    // Nothing further to do the second time.
    let again = store.recover().unwrap();
    assert!(!again.abandoned_update && again.removed.is_empty());
}

#[test]
fn recovery_removes_leftover_staging_and_the_lock() {
    let root = dir("rec2");
    let store = InstallStore::open(&root).unwrap();
    install(&store, &release("1.0.0", 1)).unwrap();
    write(&root.join("releases"), "1.1.0.staging/a", b"x");
    fs::write(root.join("install.lock"), b"stale").unwrap();
    let report = store.recover().unwrap();
    assert_eq!(report.removed, ["1.1.0.staging"]);
    assert!(!root.join("install.lock").exists());
    install(&store, &release("1.1.0", 1)).unwrap();
}

#[test]
fn recovery_falls_back_when_the_current_release_is_damaged() {
    let root = dir("rec3");
    let store = InstallStore::open(&root).unwrap();
    install(&store, &release("1.0.0", 1)).unwrap();
    install(&store, &release("1.1.0", 1)).unwrap();
    write(
        &root.join("releases").join("1.1.0"),
        "veylora-cli.exe",
        b"bit rot",
    );
    let report = store.recover().unwrap();
    assert!(report.fell_back_from_damaged_current);
    assert_eq!(store.state().unwrap().current.as_deref(), Some("1.0.0"));
    // With nothing good left, it says so rather than guessing.
    write(
        &root.join("releases").join("1.0.0"),
        "veylora-cli.exe",
        b"also rot",
    );
    assert!(matches!(
        store.recover(),
        Err(ReleaseError::CurrentDamaged(_))
    ));
}

#[test]
fn an_unreadable_state_file_is_an_error_not_an_empty_install() {
    let root = dir("corrupt");
    let store = InstallStore::open(&root).unwrap();
    install(&store, &release("1.0.0", 1)).unwrap();
    fs::write(root.join("install-state.json"), b"{ not json").unwrap();
    assert!(matches!(store.state(), Err(ReleaseError::Malformed(_))));
    assert!(install(&store, &release("1.1.0", 1)).is_err());
    assert!(store.recover().is_err());
}

#[test]
fn an_update_in_progress_blocks_another() {
    let root = dir("lock");
    let store = InstallStore::open(&root).unwrap();
    fs::write(root.join("install.lock"), b"other process").unwrap();
    assert!(matches!(
        install(&store, &release("1.0.0", 1)),
        Err(ReleaseError::Locked)
    ));
    assert!(names(&root).is_empty());
}

#[test]
fn the_current_release_can_be_verified_in_place() {
    let root = dir("cur");
    let store = InstallStore::open(&root).unwrap();
    assert!(store.verify_current().unwrap().is_none());
    install(&store, &release("1.0.0", 1)).unwrap();
    assert_eq!(store.verify_current().unwrap().unwrap().version, "1.0.0");
    write(
        &root.join("releases").join("1.0.0"),
        "LICENSE.txt",
        b"altered",
    );
    assert!(store.verify_current().is_err());
}

#[test]
fn recovery_does_not_fall_back_to_a_previous_release_that_is_also_damaged() {
    let root = dir("rec4");
    let store = InstallStore::open(&root).unwrap();
    install(&store, &release("1.0.0", 1)).unwrap();
    install(&store, &release("1.1.0", 1)).unwrap();
    write(
        &root.join("releases").join("1.1.0"),
        "veylora-cli.exe",
        b"rot",
    );
    write(
        &root.join("releases").join("1.0.0"),
        "veylora-cli.exe",
        b"rot too",
    );
    let before = store.state().unwrap();
    assert!(matches!(
        store.recover(),
        Err(ReleaseError::CurrentDamaged(_))
    ));
    assert_eq!(store.state().unwrap(), before);
}

#[test]
fn a_smoke_test_must_name_a_file_in_the_release() {
    let d = dir("smoke-name");
    write(&d, "a.txt", b"x");
    assert!(matches!(
        build_manifest(&d, "1.0.0", 1, Some(vec!["missing.exe".into()])),
        Err(ReleaseError::Malformed(_))
    ));
    assert!(matches!(
        build_manifest(&d, "1.0.0", 1, Some(vec![])),
        Err(ReleaseError::Malformed(_))
    ));
    build_manifest(&d, "1.0.0", 1, Some(vec!["a.txt".into(), "--flag".into()])).unwrap();
    // A manifest edited afterwards to name a missing program is refused too.
    let d = release("1.0.0", 1);
    let p = d.join(MANIFEST_NAME);
    let mut v: serde_json::Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
    v["smoke_test"] = serde_json::json!(["ghost.exe"]);
    fs::write(&p, serde_json::to_vec(&v).unwrap()).unwrap();
    assert!(matches!(
        verify_release(&d),
        Err(ReleaseError::Malformed(_))
    ));
}
