use veylora_color::{
    CalibrationRecord, ChannelCurve, InkLimits, MeasurementProvenance, WhiteCalibration,
};
use veylora_domain::provenance::{ContentHash, Provenance};
use veylora_persistence::CalibrationStore;
fn record() -> CalibrationRecord {
    CalibrationRecord::new(
        vec![ChannelCurve {
            channel: "C".into(),
            points: vec![(0.0, 0.0), (1.0, 1.0)],
        }],
        InkLimits {
            channel_max: vec![("C".into(), 1.0)],
            combined_wet_load_max: None,
        },
        WhiteCalibration { slots: vec![] },
        MeasurementProvenance {
            chart_id: "chart".into(),
            instrument: "unknown".into(),
            units: "raw".into(),
            source_hash: ContentHash::digest(b"raw"),
            provenance: Provenance {
                software_version: "test".into(),
                backend_version: "test".into(),
                content_hashes: vec![],
            },
        },
    )
    .unwrap()
}
#[test]
fn stores_by_verified_content_hash_and_replays_idempotently() {
    let r = record();
    let hash = r.content_hash.to_string();
    let mut s = CalibrationStore::new();
    s.put(r.clone()).unwrap();
    s.put(r).unwrap();
    assert_eq!(s.len(), 1);
    assert!(s.get(&hash).is_some());
    assert!(s.export_json(&hash).unwrap().is_some());
}
#[test]
fn empty_store_is_explicit() {
    assert!(CalibrationStore::new().is_empty());
}
