//! The desktop executable's headless self-check, used by release verification and the clean-machine
//! test. It must prove the program loads and runs without opening a window or needing WebView2.
use std::process::Command;
use std::time::{Duration, Instant};

#[test]
fn version_prints_and_exits_without_opening_a_window() {
    let start = Instant::now();
    let out = Command::new(env!("CARGO_BIN_EXE_veylora-app"))
        .arg("--version")
        .output()
        .expect("the app binary runs");
    assert!(out.status.success(), "{out:?}");
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        text.trim(),
        format!("veylora-app {}", env!("CARGO_PKG_VERSION"))
    );
    assert!(
        start.elapsed() < Duration::from_secs(20),
        "a self-check must not start the UI"
    );
}

#[test]
fn arguments_after_the_flag_do_not_start_the_ui() {
    let out = Command::new(env!("CARGO_BIN_EXE_veylora-app"))
        .arg("--version")
        .arg("extra")
        .output()
        .expect("runs");
    assert!(out.status.success());
}
