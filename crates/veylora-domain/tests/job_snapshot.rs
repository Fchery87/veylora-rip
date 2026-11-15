use veylora_domain::{ids::*, job::*, layout::*, provenance::*, recipe::*};
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
fn populated_clear(eligible_mask_id: Option<AssetId>) -> ClearPolicy {
    ClearPolicy {
        knockouts: vec![
            KnockoutPolicy {
                target: KnockoutTarget::Black,
                tolerance_permille: 80,
                transition_permille: 40,
            },
            KnockoutPolicy {
                target: KnockoutTarget::Color { rgb: [10, 20, 30] },
                tolerance_permille: 20,
                transition_permille: 0,
            },
        ],
        perforation: Some(PerforationPolicy {
            diameter_um: 400,
            spacing_um: 1000,
            phase_x_px: 3,
            phase_y_px: -2,
            eligible_mask_id,
            edge_protection_um: 200,
        }),
    }
}
fn input() -> JobSnapshotInput {
    let asset = AssetId::new();
    let mask = AssetId::new();
    JobSnapshotInput {
        schema_version: 1,
        id: JobId::new(),
        created_at_ms: 1735000000000,
        assets: vec![AssetReference {
            id: asset,
            content_hash: ContentHash::digest(b"source"),
        }],
        masks: vec![AssetReference {
            id: mask,
            content_hash: ContentHash::digest(b"mask"),
        }],
        layout: LayoutSnapshot::new(LayoutInput {
            schema_version: 1,
            page_width_um: 300000,
            page_height_um: 400000,
            mirror: true,
            placements: vec![Placement {
                id: PlacementId::new(),
                asset_id: asset,
                mask_id: Some(mask),
                x_um: 0,
                y_um: 0,
                width_um: 10000,
                height_um: 20000,
                rotation: Rotation::Deg0,
                crop_x_px: 0,
                crop_y_px: 0,
                crop_width_px: 100,
                crop_height_px: 200,
            }],
        })
        .unwrap(),
        device_setup_id: DeviceSetupId::new(),
        device_setup: reference(),
        print_mode_id: PrintModeId::new(),
        print_mode: reference(),
        recipe: PrintRecipe::new({
            let mut r = recipe();
            r.clear = populated_clear(Some(mask));
            r
        })
        .unwrap(),
        provenance: Provenance {
            software_version: "test".into(),
            backend_version: "simulation".into(),
            content_hashes: vec![],
        },
    }
}
#[test]
fn roundtrip_and_frozen_inputs_preserve_identity() {
    let job = JobSnapshot::new(input()).unwrap();
    let restored: JobSnapshot =
        serde_json::from_value(serde_json::to_value(&job).unwrap()).unwrap();
    assert_eq!(job, restored);
    assert_eq!(job.content_hash(), restored.content_hash());
    let render = job.begin_render().unwrap();
    let hash = render.content_hash();
    let mut detached = render.snapshot().inputs().clone();
    detached.created_at_ms += 1;
    assert_ne!(hash, JobSnapshot::new(detached).unwrap().content_hash());
    assert_eq!(hash, render.content_hash());
    let restored: RenderingSnapshot =
        serde_json::from_str(&serde_json::to_string(&render).unwrap()).unwrap();
    assert_eq!(render, restored);
}
#[test]
fn dangling_references_and_duplicate_metadata_fail_on_all_construction_paths() {
    for kind in 0..8 {
        let mut data = input();
        match kind {
            0 => data.assets.clear(),
            1 => data.masks.clear(),
            2 => data.created_at_ms = 0,
            3 => data.assets.push(data.assets[0].clone()),
            4 => {
                let mut r = recipe();
                r.white.source = WhiteSource::Supplied {
                    mask_id: AssetId::new(),
                };
                data.recipe = PrintRecipe::new(r).unwrap();
            }
            5 => {
                data.provenance.content_hashes = vec![
                    ("same".into(), ContentHash::digest(b"a")),
                    ("same".into(), ContentHash::digest(b"b")),
                ]
            }
            6 => {
                let mut r = recipe();
                r.clear = populated_clear(Some(AssetId::new()));
                data.recipe = PrintRecipe::new(r).unwrap();
            }
            _ => data.schema_version = 99,
        }
        assert!(JobSnapshot::new(data.clone()).is_err(), "case {kind}");
        assert!(
            serde_json::from_value::<JobSnapshot>(serde_json::to_value(data).unwrap()).is_err()
        );
    }
}
#[test]
fn every_retained_scalar_affects_hash_or_is_explicitly_rejected() {
    fn variants(value: &serde_json::Value) -> Vec<serde_json::Value> {
        use serde_json::Value;
        match value {
            Value::Object(fields) => fields
                .iter()
                .flat_map(|(key, v)| {
                    variants(v)
                        .into_iter()
                        .map(|new| {
                            let mut out = fields.clone();
                            out.insert(key.clone(), new);
                            Value::Object(out)
                        })
                        .collect::<Vec<_>>()
                })
                .collect(),
            Value::Array(items) => items
                .iter()
                .enumerate()
                .flat_map(|(i, v)| {
                    variants(v)
                        .into_iter()
                        .map(|new| {
                            let mut out = items.clone();
                            out[i] = new;
                            Value::Array(out)
                        })
                        .collect::<Vec<_>>()
                })
                .collect(),
            Value::Number(n) => vec![Value::from(n.as_i64().unwrap() + 1)],
            Value::Bool(b) => vec![Value::Bool(!b)],
            Value::String(s) => vec![Value::String(if s.len() == 64 {
                ContentHash::digest(s.as_bytes()).to_string()
            } else if s.len() == 36 {
                AssetId::new().to_string()
            } else {
                format!("{s}-changed")
            })],
            _ => vec![],
        }
    }
    let job = JobSnapshot::new(input()).unwrap();
    for changed in variants(&serde_json::to_value(&job).unwrap()) {
        if let Ok(changed) = serde_json::from_value::<JobSnapshot>(changed) {
            assert_ne!(job.content_hash(), changed.content_hash());
        }
    }
}
#[test]
fn invalid_policy_values_are_rejected_and_profile_is_explicitly_optional() {
    for kind in 0..6 {
        let mut data = recipe();
        match kind {
            0 => data.white.response.clear(),
            1 => data.screen.thresholds.clear(),
            2 => data.finish.press_seconds = 0,
            3 => data.calibration.channel_limits_percent[0] = 101,
            4 => data.revision = 0,
            _ => data.created_at_ms = 0,
        }
        assert!(PrintRecipe::new(data.clone()).is_err());
        assert!(
            serde_json::from_value::<PrintRecipe>(serde_json::to_value(data).unwrap()).is_err()
        );
    }
    let mut data = recipe();
    data.color.profile = None;
    let recipe = PrintRecipe::new(data).unwrap();
    let restored: PrintRecipe =
        serde_json::from_value(serde_json::to_value(&recipe).unwrap()).unwrap();
    assert_eq!(recipe.content_hash(), restored.content_hash());
}

#[test]
fn clear_policy_is_validated_hashed_and_part_of_the_recipe_schema() {
    let solid = PrintRecipe::new(recipe()).unwrap();
    let mut with_knockout = recipe();
    with_knockout.clear = populated_clear(None);
    with_knockout.clear.perforation = None;
    let knockout = PrintRecipe::new(with_knockout).unwrap();
    let mut with_holes = recipe();
    with_holes.clear.perforation = populated_clear(None).perforation;
    with_holes
        .clear
        .perforation
        .as_mut()
        .unwrap()
        .edge_protection_um = 0;
    let perforated = PrintRecipe::new(with_holes).unwrap();
    // Solid and perforated variants are distinct recipe contents.
    let hashes = [
        solid.content_hash(),
        knockout.content_hash(),
        perforated.content_hash(),
    ];
    assert_ne!(hashes[0], hashes[1]);
    assert_ne!(hashes[0], hashes[2]);
    assert_ne!(hashes[1], hashes[2]);
    for recipe in [&solid, &knockout, &perforated] {
        let restored: PrintRecipe =
            serde_json::from_value(serde_json::to_value(recipe).unwrap()).unwrap();
        assert_eq!(recipe.content_hash(), restored.content_hash());
    }
}
#[test]
fn unsafe_clear_policies_and_old_schemas_are_rejected() {
    let mut cases: Vec<(&str, ClearPolicy)> = Vec::new();
    let mut c = populated_clear(Some(AssetId::new()));
    c.knockouts[0].tolerance_permille = 1001;
    cases.push(("tolerance", c));
    let mut c = populated_clear(Some(AssetId::new()));
    c.knockouts[0].transition_permille = 1001;
    cases.push(("transition", c));
    let mut c = populated_clear(Some(AssetId::new()));
    c.knockouts = vec![c.knockouts[0]; MAX_KNOCKOUT_RULES + 1];
    cases.push(("too many rules", c));
    let mut c = populated_clear(Some(AssetId::new()));
    c.perforation.as_mut().unwrap().diameter_um = 0;
    cases.push(("zero diameter", c));
    let mut c = populated_clear(Some(AssetId::new()));
    c.perforation.as_mut().unwrap().spacing_um = 400;
    cases.push(("overlapping holes", c));
    let c = populated_clear(None); // edge protection without an eligible mask
    cases.push(("edge protection without mask", c));
    for (name, clear) in cases {
        let mut data = recipe();
        data.clear = clear;
        assert!(PrintRecipe::new(data.clone()).is_err(), "{name}");
        assert!(
            serde_json::from_value::<PrintRecipe>(serde_json::to_value(data).unwrap()).is_err(),
            "{name} via serde"
        );
    }
    // Schema 1 recipes predate `clear` and are not readable.
    let mut old = recipe();
    old.schema_version = 1;
    assert!(PrintRecipe::new(old).is_err());
    let mut json = serde_json::to_value(PrintRecipe::new(recipe()).unwrap()).unwrap();
    json.as_object_mut().unwrap().remove("clear");
    assert!(serde_json::from_value::<PrintRecipe>(json).is_err());
}
