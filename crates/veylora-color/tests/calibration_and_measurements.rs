use veylora_color::{
    import_csv, import_json, CalibrationRecord, ChannelCurve, InkLimits, MeasurementError,
    MeasurementProvenance, WhiteCalibration,
};
use veylora_domain::provenance::{ContentHash, Provenance};

#[test]
fn calibration_hash_and_status_are_deterministic_and_unvalidated() {
    let p = Provenance {
        software_version: "test".into(),
        backend_version: "test".into(),
        content_hashes: vec![],
    };
    let m = MeasurementProvenance {
        chart_id: "c".into(),
        instrument: "i".into(),
        units: "u".into(),
        source_hash: ContentHash::digest(b"raw"),
        provenance: p,
    };
    let a = CalibrationRecord::new(
        vec![ChannelCurve {
            channel: "C".into(),
            points: vec![(0.0, 0.1), (1.0, 0.9)],
        }],
        InkLimits {
            channel_max: vec![("C".into(), 1.0)],
            combined_wet_load_max: Some(1.5),
        },
        WhiteCalibration {
            slots: vec![("W1".into(), 1.0)],
        },
        m,
    )
    .unwrap();
    assert!(a.is_unvalidated());
    assert_eq!(a.content_hash, a.compute_hash());
}

#[test]
fn imports_preserve_raw_values_and_reject_duplicate_or_nan() {
    let json = br#"{"chart_id":"chart","units":"L*","instrument":"meter","provenance":"operator","patches":[{"patch_id":"p1","values":[12.5,0.25]}]}"#;
    let record = import_json(json).unwrap();
    assert_eq!(record.patches[0].values[0], 12.5);
    assert_eq!(record.source_hash, ContentHash::digest(json));
    let duplicate = br#"{"chart_id":"c","units":"u","instrument":"i","provenance":"p","patches":[{"patch_id":"p1","values":[1]},{"patch_id":"p1","values":[2]}]}"#;
    assert!(matches!(
        import_json(duplicate),
        Err(MeasurementError::DuplicatePatch(_))
    ));
    let nan = br#"{"chart_id":"c","units":"u","instrument":"i","provenance":"p","patches":[{"patch_id":"p1","values":[NaN]}]}"#;
    assert!(import_json(nan).is_err());
}

#[test]
fn csv_import_is_bounded_and_deterministic() {
    let csv = b"patch_id,v1,v2\np1,1.5,2.5\np2,3,4";
    let a = import_csv(csv).unwrap();
    let b = import_csv(csv).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.patches.len(), 2);
}
