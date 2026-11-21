use veylora_domain::{ids::*, provenance::*, recipe::*};
fn reference() -> VersionedReference {
    VersionedReference {
        version: 1,
        content_hash: ContentHash::digest(b"synthetic"),
    }
}
fn recipe() -> PrintRecipeInput {
    PrintRecipeInput {
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
            profile: Some(reference()),
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
            ink_id: "synthetic ink".into(),
            film_id: "synthetic film".into(),
            powder_id: "synthetic powder".into(),
            garment: "synthetic garment".into(),
            cure_temperature_c: 100,
            cure_seconds: 10,
            press_temperature_c: 100,
            press_seconds: 10,
            press_pressure_pa: 1000,
            peel: PeelCondition::Cold,
        },
    }
}

fn sample(session: &str, label: &str, ok: bool) -> PressedSample {
    PressedSample {
        session: session.into(),
        label: label.into(),
        within_tolerance: ok,
        note: "pressed, peeled cold, measured".into(),
    }
}
fn evidence() -> RecipeEvidence {
    RecipeEvidence {
        validated_revision: 1,
        validated_hash: ContentHash::digest(b"the experimental revision"),
        setup_hash: ContentHash::digest(b"setup"),
        mode_hash: ContentHash::digest(b"mode"),
        operator: "F. Chery".into(),
        tolerances: "dE2000 <= 3 on the 6 patch strip; white registration within 0.3 mm".into(),
        samples: vec![
            sample("2026-11-18 morning", "A1", true),
            sample("2026-11-18 morning", "A2", true),
            sample("2026-11-19 afternoon", "B1", true),
        ],
    }
}
fn validation() -> RecipeValidation {
    RecipeValidation {
        evidence: evidence(),
        signed_off_by: "F. Chery".into(),
        signed_off_at: "2026-11-20T09:00:00Z".into(),
        sign_off_note: "Reviewed the pressed samples against the strip.".into(),
    }
}
fn validated(v: RecipeValidation) -> Result<PrintRecipe, veylora_domain::errors::ValidationError> {
    let mut input = recipe();
    input.revision = 2;
    input.status = RecipeStatus::Validated;
    input.validation = Some(v);
    PrintRecipe::new(input)
}

#[test]
fn a_recipe_with_complete_evidence_and_a_sign_off_is_validated() {
    let r = validated(validation()).unwrap();
    assert_eq!(r.inputs().status, RecipeStatus::Validated);
    assert_eq!(
        r.inputs()
            .validation
            .as_ref()
            .unwrap()
            .evidence
            .samples
            .len(),
        3
    );
}

#[test]
fn validated_without_evidence_is_refused_and_so_is_evidence_without_validated() {
    let mut input = recipe();
    input.status = RecipeStatus::Validated;
    assert!(
        PrintRecipe::new(input).is_err(),
        "validated needs its evidence"
    );
    let mut input = recipe();
    input.validation = Some(validation());
    assert!(
        PrintRecipe::new(input).is_err(),
        "an experimental recipe carries none"
    );
}

#[test]
fn repeated_samples_from_more_than_one_session_are_required() {
    let mut v = validation();
    v.evidence.samples.truncate(2);
    assert!(validated(v).is_err(), "two samples are not repeated enough");
    let mut v = validation();
    for s in &mut v.evidence.samples {
        s.session = "2026-11-18 morning".into();
    }
    assert!(
        validated(v).is_err(),
        "one session cannot show it holds across sessions"
    );
}

#[test]
fn a_sample_outside_tolerance_means_the_recipe_is_not_validated() {
    let mut v = validation();
    v.evidence.samples[1].within_tolerance = false;
    assert!(validated(v).is_err());
}

#[test]
fn the_people_the_tolerances_and_the_dates_must_be_stated() {
    for edit in [
        (|v: &mut RecipeValidation| v.evidence.operator = " ".into()) as fn(&mut RecipeValidation),
        |v| v.evidence.tolerances = String::new(),
        |v| v.signed_off_by = String::new(),
        |v| v.signed_off_at = String::new(),
        |v| v.sign_off_note = " ".into(),
        |v| v.evidence.samples[0].label = String::new(),
        |v| v.evidence.samples[0].session = String::new(),
    ] {
        let mut v = validation();
        edit(&mut v);
        assert!(validated(v).is_err());
    }
}

#[test]
fn the_evidence_is_part_of_the_recipe_so_changing_it_changes_the_hash() {
    let a = validated(validation()).unwrap();
    let mut other = validation();
    other.sign_off_note = "A different note.".into();
    let b = validated(other).unwrap();
    assert_ne!(a.content_hash(), b.content_hash());
}

#[test]
fn a_recipe_without_validation_serializes_exactly_as_before() {
    let r = PrintRecipe::new(recipe()).unwrap();
    let json = serde_json::to_string(&r).unwrap();
    assert!(!json.contains("validation"), "{json}");
    let back: PrintRecipe = serde_json::from_str(&json).unwrap();
    assert_eq!(back.content_hash(), r.content_hash());
}
