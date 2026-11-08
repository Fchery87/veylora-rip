use std::process::Command;

#[test]
fn version_reports_product_and_version_successfully() {
    let output = Command::new(env!("CARGO_BIN_EXE_veylora-cli"))
        .arg("--version")
        .output()
        .expect("CLI should launch");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"Veylora RIP 0.0.0\n");
    assert!(output.stderr.is_empty(), "{output:?}");
}
