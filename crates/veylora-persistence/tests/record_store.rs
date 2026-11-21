use std::path::PathBuf;
use veylora_color::{
    CalibrationRecord, ChannelCurve, InkLimits, MeasurementProvenance, WhiteCalibration,
};
use veylora_domain::{
    ids::RecipeId,
    provenance::{ContentHash, Provenance},
    recipe::{
        CalibrationPolicy, ClearPolicy, ColorPolicy, FinishPolicy, MaskCombination, PeelCondition,
        PrintRecipe, PrintRecipeInput, RecipeStatus, RenderingIntent, ScreenAlgorithm,
        ScreenPolicy, UntaggedRgbPolicy, WhitePolicy, WhiteSource,
    },
};
use veylora_persistence::{FileRecordStore, PersistenceError};

fn dir(label: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let p = std::env::temp_dir().join(format!(
        "veylora-records-{label}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&p);
    p
}

fn recipe(garment: &str) -> PrintRecipe {
    PrintRecipe::new(PrintRecipeInput {
        schema_version: 2,
        id: RecipeId::new(),
        revision: 1,
        created_at_ms: 1735000000000,
        status: RecipeStatus::Experimental,
        white: WhitePolicy {
            source: WhiteSource::Alpha,
            combine: MaskCombination::Replace,
            density_percent: 100,
            spread_um: 0,
            response: vec![0, 65535],
        },
        calibration: CalibrationPolicy {
            revision: 1,
            process_curves: std::array::from_fn(|_| vec![0, 65535]),
            channel_limits_percent: [100; 4],
            total_coverage_percent: 300,
        },
        color: ColorPolicy {
            profile: None,
            rendering_intent: RenderingIntent::Perceptual,
            black_point_compensation: false,
            untagged_rgb_policy: UntaggedRgbPolicy::Reject,
        },
        screen: ScreenPolicy {
            algorithm: ScreenAlgorithm::OrderedThreshold,
            revision: 1,
            seed: 0,
            phase_x: 0,
            phase_y: 0,
            cell_width: 1,
            cell_height: 1,
            thresholds: vec![32768],
        },
        clear: ClearPolicy::default(),
        validation: None,
        finish: FinishPolicy {
            ink_id: "i".into(),
            film_id: "f".into(),
            powder_id: "p".into(),
            garment: garment.into(),
            cure_temperature_c: 1,
            cure_seconds: 1,
            press_temperature_c: 1,
            press_seconds: 1,
            press_pressure_pa: 1,
            peel: PeelCondition::Cold,
        },
    })
    .unwrap()
}

fn calibration(chart: &str) -> CalibrationRecord {
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
            chart_id: chart.into(),
            instrument: "unknown".into(),
            units: "raw".into(),
            source_hash: ContentHash::digest(b"raw"),
            provenance: Provenance {
                software_version: "t".into(),
                backend_version: "t".into(),
                content_hashes: vec![],
            },
        },
    )
    .unwrap()
}

#[test]
fn recipes_and_calibrations_survive_a_restart_by_content_hash() {
    let root = dir("restart");
    let (r, c) = (recipe("shirt"), calibration("chart-1"));
    let (rh, ch) = {
        let s = FileRecordStore::open(&root).unwrap();
        (s.put_recipe(&r).unwrap(), s.put_calibration(&c).unwrap())
    };
    assert_eq!(rh, r.content_hash());
    assert_eq!(ch, c.content_hash);
    let s = FileRecordStore::open(&root).unwrap();
    assert_eq!(s.get_recipe(&rh).unwrap(), r);
    assert_eq!(s.get_calibration(&ch).unwrap(), c);
    assert_eq!(s.list_recipes().unwrap(), vec![rh]);
    assert_eq!(s.list_calibrations().unwrap(), vec![ch]);
}

#[test]
fn identical_rewrites_are_idempotent_and_distinct_records_coexist() {
    let s = FileRecordStore::open(dir("idem")).unwrap();
    let r = recipe("a");
    assert_eq!(s.put_recipe(&r).unwrap(), s.put_recipe(&r).unwrap());
    s.put_recipe(&recipe("b")).unwrap();
    assert_eq!(s.list_recipes().unwrap().len(), 2);
}

#[test]
fn a_calibration_that_fails_its_own_hash_is_refused() {
    let s = FileRecordStore::open(dir("badhash")).unwrap();
    let mut c = calibration("x");
    c.limits.channel_max[0].1 = 0.5; // changed after hashing
    assert!(matches!(
        s.put_calibration(&c),
        Err(PersistenceError::Corrupt { .. })
    ));
    assert!(s.list_calibrations().unwrap().is_empty());
}

#[test]
fn missing_records_are_not_found_not_empty() {
    let s = FileRecordStore::open(dir("missing")).unwrap();
    let h = ContentHash::digest(b"nothing");
    assert!(matches!(
        s.get_recipe(&h),
        Err(PersistenceError::NotFound { .. })
    ));
    assert!(matches!(
        s.get_calibration(&h),
        Err(PersistenceError::NotFound { .. })
    ));
}

#[test]
fn a_stored_file_that_no_longer_matches_its_name_is_reported_corrupt() {
    let root = dir("tamper");
    let s = FileRecordStore::open(&root).unwrap();
    let h = s.put_recipe(&recipe("a")).unwrap();
    // Swap in a different, valid recipe under the first one's name.
    let other = serde_json::to_vec(&recipe("b")).unwrap();
    std::fs::write(root.join("recipes").join(format!("{h}.json")), other).unwrap();
    assert!(matches!(
        s.get_recipe(&h),
        Err(PersistenceError::Corrupt { .. })
    ));
    // Garbage is corrupt too, not a panic or an empty result.
    std::fs::write(root.join("recipes").join(format!("{h}.json")), b"{not json").unwrap();
    assert!(matches!(
        s.get_recipe(&h),
        Err(PersistenceError::Corrupt { .. })
    ));
}

#[test]
fn stray_files_do_not_appear_as_records_and_listing_is_sorted() {
    let root = dir("list");
    let s = FileRecordStore::open(&root).unwrap();
    let mut hashes: Vec<_> = (0..4)
        .map(|i| s.put_recipe(&recipe(&format!("g{i}"))).unwrap())
        .collect();
    hashes.sort();
    std::fs::write(root.join("recipes").join("notes.txt"), b"hi").unwrap();
    std::fs::write(root.join("recipes").join("x.json.tmp"), b"hi").unwrap();
    assert_eq!(s.list_recipes().unwrap(), hashes);
}

#[test]
fn newer_or_corrupt_version_files_are_refused_not_downgraded() {
    let root = dir("version");
    FileRecordStore::open(&root).unwrap();
    std::fs::write(root.join("VERSION"), b"99").unwrap();
    assert!(matches!(
        FileRecordStore::open(&root),
        Err(PersistenceError::SchemaOutOfRange { found: 99, .. })
    ));
    std::fs::write(root.join("VERSION"), b"banana").unwrap();
    assert!(matches!(
        FileRecordStore::open(&root),
        Err(PersistenceError::Corrupt { .. })
    ));
}

#[test]
fn rewriting_over_a_damaged_file_reports_corruption_instead_of_pretending_success() {
    let root = dir("damaged");
    let s = FileRecordStore::open(&root).unwrap();
    let r = recipe("a");
    let h = s.put_recipe(&r).unwrap();
    std::fs::write(root.join("recipes").join(format!("{h}.json")), b"damaged").unwrap();
    assert!(matches!(
        s.put_recipe(&r),
        Err(PersistenceError::Corrupt { .. })
    ));
    // The damaged file is left for the operator; nothing was overwritten.
    assert_eq!(
        std::fs::read(root.join("recipes").join(format!("{h}.json"))).unwrap(),
        b"damaged"
    );
}

#[test]
fn a_calibration_file_that_no_longer_matches_its_name_is_reported_corrupt() {
    let root = dir("cal-tamper");
    let s = FileRecordStore::open(&root).unwrap();
    let h = s.put_calibration(&calibration("one")).unwrap();
    let other = serde_json::to_vec(&calibration("two")).unwrap();
    std::fs::write(root.join("calibrations").join(format!("{h}.json")), other).unwrap();
    assert!(matches!(
        s.get_calibration(&h),
        Err(PersistenceError::Corrupt { .. })
    ));
}
