use veylora_color::{
    check_recipe_calibration, CalibrationRecord, ChannelCurve, InkLimits, MeasurementProvenance,
    RecipeStatus, WhiteCalibration,
};
use veylora_domain::{
    errors::ErrorCode,
    ids::RecipeId,
    preflight::Severity,
    provenance::{ContentHash, Provenance},
    recipe::{
        CalibrationPolicy, ClearPolicy, ColorPolicy, FinishPolicy, MaskCombination, PeelCondition,
        PrintRecipe, PrintRecipeInput, RenderingIntent, ScreenAlgorithm, ScreenPolicy,
        UntaggedRgbPolicy, WhitePolicy, WhiteSource,
    },
};

fn recipe(limits: [u8; 4], total: u16) -> PrintRecipe {
    PrintRecipe::new(PrintRecipeInput {
        schema_version: 2,
        id: RecipeId::new(),
        revision: 1,
        created_at_ms: 1735000000000,
        status: veylora_domain::recipe::RecipeStatus::Experimental,
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
            channel_limits_percent: limits,
            total_coverage_percent: total,
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
            garment: "g".into(),
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

fn record(channel_max: Vec<(&str, f64)>, wet: Option<f64>) -> CalibrationRecord {
    CalibrationRecord::new(
        vec![ChannelCurve {
            channel: "C".into(),
            points: vec![(0.0, 0.0), (1.0, 1.0)],
        }],
        InkLimits {
            channel_max: channel_max
                .into_iter()
                .map(|(n, v)| (n.into(), v))
                .collect(),
            combined_wet_load_max: wet,
        },
        WhiteCalibration { slots: vec![] },
        MeasurementProvenance {
            chart_id: "chart".into(),
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
fn a_recipe_inside_the_calibration_limits_has_no_findings() {
    let r = record(vec![("C", 1.0), ("M", 0.8)], Some(3.0));
    assert!(check_recipe_calibration(&recipe([100, 80, 100, 100], 300), &r).is_empty());
}

#[test]
fn exceeding_a_channel_limit_is_blocking_and_names_the_channel() {
    let r = record(vec![("C", 1.0), ("M", 0.8)], None);
    let f = check_recipe_calibration(&recipe([100, 81, 100, 100], 300), &r);
    assert_eq!(f.len(), 1, "{f:?}");
    assert_eq!(f[0].code, ErrorCode::CoverageLimitExceeded);
    assert_eq!(f[0].severity, Severity::Blocking);
    assert_eq!(
        f[0].subject.as_deref(),
        Some("recipe.calibration.channel_limits.M")
    );
}

#[test]
fn channel_names_match_case_insensitively_and_missing_limits_are_not_invented() {
    let r = record(vec![("y", 0.5)], None);
    let f = check_recipe_calibration(&recipe([100, 100, 51, 100], 300), &r);
    assert_eq!(f.len(), 1);
    assert_eq!(
        f[0].subject.as_deref(),
        Some("recipe.calibration.channel_limits.Y")
    );
}

#[test]
fn exceeding_the_combined_wet_load_is_blocking() {
    let r = record(vec![], Some(2.5));
    let f = check_recipe_calibration(&recipe([100; 4], 251), &r);
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].code, ErrorCode::CoverageLimitExceeded);
    assert_eq!(
        f[0].subject.as_deref(),
        Some("recipe.calibration.total_coverage")
    );
    assert!(check_recipe_calibration(&recipe([100; 4], 250), &r).is_empty());
}

#[test]
fn a_tampered_or_superseded_record_is_refused() {
    let mut r = record(vec![("C", 1.0)], None);
    r.limits.channel_max[0].1 = 2.0; // changed without rehashing
    let f = check_recipe_calibration(&recipe([100; 4], 300), &r);
    assert!(f
        .iter()
        .any(|x| x.code == ErrorCode::IncompatibleRecipe && x.severity == Severity::Blocking));

    let mut r = record(vec![("C", 1.0)], None);
    r.status = RecipeStatus::Superseded;
    let f = check_recipe_calibration(&recipe([100; 4], 300), &r);
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].code, ErrorCode::IncompatibleRecipe);
}
