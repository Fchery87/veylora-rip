use veylora_diagnostics::charts::{export_chart_json, export_chart_metadata};

#[test]
fn chart_export_is_stable_and_explicitly_synthetic() {
    let a = export_chart_metadata(
        "chart-1",
        vec!["p1".into(), "p2".into()],
        "synthetic diagnostic chart; not printer output",
    );
    let b = export_chart_metadata(
        "chart-1",
        vec!["p1".into(), "p2".into()],
        "synthetic diagnostic chart; not printer output",
    );
    assert_eq!(a, b);
    assert!(export_chart_json(&a).contains("not printer output"));
}
