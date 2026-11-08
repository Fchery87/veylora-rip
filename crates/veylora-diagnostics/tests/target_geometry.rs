use veylora_diagnostics::targets::{generate_target, PhysicalSize, TargetKind};

#[test]
fn stable_hash_covers_kind_size_and_all_serialized_geometry() {
    use veylora_domain::provenance::ContentHash;
    let target = generate_target(
        TargetKind::ChannelIdentity,
        PhysicalSize::new(110.0, 150.0).unwrap(),
    );
    let fixture: veylora_diagnostics::targets::DiagnosticTarget = serde_json::from_str(
        include_str!("../../../fixtures/targets/channel-identity.json"),
    )
    .unwrap();
    assert_eq!(target, fixture);
    assert_eq!(target.stable_hash(), fixture.stable_hash());
    assert_eq!(
        target.stable_hash().to_string(),
        "b665e1899d93e1290bc076320186b7667c53b72848ebb1c9937b669e90dd6437"
    );
    assert_eq!(
        target.stable_hash(),
        ContentHash::digest(&serde_json::to_vec(&target).unwrap())
    );
    assert_ne!(
        target.stable_hash(),
        generate_target(
            TargetKind::ChannelIdentity,
            PhysicalSize::new(111.0, 150.0).unwrap()
        )
        .stable_hash()
    );
    assert_ne!(
        target.stable_hash(),
        generate_target(
            TargetKind::ChannelIdentity,
            PhysicalSize::new(110.0, 151.0).unwrap()
        )
        .stable_hash()
    );
    let hashes: std::collections::HashSet<_> = KINDS
        .into_iter()
        .map(|kind| generate_target(kind, target.size()).stable_hash())
        .collect();
    assert_eq!(hashes.len(), 7);
}

const KINDS: [TargetKind; 7] = [
    TargetKind::ChannelIdentity,
    TargetKind::Geometry,
    TargetKind::Registration,
    TargetKind::WhiteRamp,
    TargetKind::WetLoad,
    TargetKind::AlphaEdge,
    TargetKind::Screening,
];

#[test]
fn standalone_targets_round_trip_and_reject_corruption() {
    use veylora_diagnostics::targets::DiagnosticTarget;
    for kind in KINDS {
        let target = generate_target(kind, PhysicalSize::new(100.0, 150.0).unwrap());
        let bytes = serde_json::to_vec(&target).unwrap();
        let restored: DiagnosticTarget = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(target, restored);
        assert_eq!(bytes, serde_json::to_vec(&restored).unwrap());
        let value = serde_json::to_value(target).unwrap();
        assert_eq!(value["schema"], "veylora.diagnostic-target");
        assert_eq!(value["version"], 1);
        assert!(value["provenance"].as_str().unwrap().contains("synthetic"));
        for (pointer, bad) in [
            ("/version", serde_json::json!(2)),
            ("/size/width_mm", serde_json::json!(-1)),
            ("/objects/0/bounds/x_mm", serde_json::json!(-1)),
            ("/objects/0/alpha", serde_json::json!(2)),
        ] {
            let mut malformed = value.clone();
            *malformed.pointer_mut(pointer).unwrap() = bad;
            assert!(serde_json::from_value::<DiagnosticTarget>(malformed).is_err());
        }
    }
}

#[test]
fn specialized_targets_have_measurement_controls() {
    let size = PhysicalSize::new(100.0, 150.0).unwrap();
    let registration =
        serde_json::to_value(generate_target(TargetKind::Registration, size)).unwrap();
    let crosses: Vec<_> = registration["objects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["shape"]["type"] == "registration_cross")
        .collect();
    assert_eq!(crosses.len(), 5);
    assert_eq!(crosses[4]["bounds"]["x_mm"], 47.0);
    assert_eq!(crosses[4]["bounds"]["y_mm"], 72.0);
    for cross in crosses {
        assert_eq!(cross["coverage"]["k"], 1.0);
        assert_eq!(cross["coverage"]["w"], 1.0);
    }
    let white = generate_target(TargetKind::WhiteRamp, size);
    let patches: Vec<_> = white
        .objects()
        .iter()
        .filter(|o| o.label.starts_with("White "))
        .collect();
    assert_eq!(
        patches.iter().map(|o| o.coverage.w).collect::<Vec<_>>(),
        vec![0.0, 0.25, 0.5, 0.75, 1.0]
    );
    assert!(patches
        .iter()
        .all(|o| o.coverage.k == 0.0 && o.alpha == 1.0));
    let alpha = generate_target(TargetKind::AlphaEdge, size);
    assert_eq!(
        alpha
            .objects()
            .iter()
            .filter(|o| o.label.starts_with("Alpha "))
            .map(|o| o.alpha)
            .collect::<Vec<_>>(),
        vec![0.0, 0.25, 0.5, 0.75, 1.0]
    );
    let wet = generate_target(TargetKind::WetLoad, size);
    assert!(wet.objects().iter().any(|o| o.coverage.c == 1.0
        && o.coverage.m == 1.0
        && o.coverage.y == 1.0
        && o.coverage.k == 1.0
        && o.coverage.w == 1.0));
    let screening = serde_json::to_value(generate_target(TargetKind::Screening, size)).unwrap();
    assert!(screening["objects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["shape"]["type"] == "line_pairs"));
    let geometry = generate_target(TargetKind::Geometry, size);
    assert!(geometry.objects().iter().any(|o| o.label == "10 mm square"
        && o.bounds.width_mm == 10.0
        && o.bounds.height_mm == 10.0));
}

#[test]
fn every_target_is_bounded_and_deterministic_at_extreme_sizes() {
    for kind in KINDS {
        for (w, h) in [
            (50.0, 50.0),
            (50.0, 1000.0),
            (1000.0, 50.0),
            (1000.0, 1000.0),
            (123.456, 78.91),
        ] {
            let size = PhysicalSize::new(w, h).unwrap();
            let target = generate_target(kind, size);
            assert_eq!(target, generate_target(kind, size));
            let bytes = serde_json::to_vec(&target).unwrap();
            let restored: veylora_diagnostics::targets::DiagnosticTarget =
                serde_json::from_slice(&bytes).unwrap();
            assert_eq!(restored.stable_hash(), target.stable_hash());
            assert!(target.objects().len() <= 64);
            for object in target.objects() {
                let b = object.bounds;
                assert!(b.x_mm >= 0.0 && b.y_mm >= 0.0 && b.width_mm > 0.0 && b.height_mm > 0.0);
                assert!(b.x_mm + b.width_mm <= w && b.y_mm + b.height_mm <= h);
                for value in [
                    object.coverage.c,
                    object.coverage.m,
                    object.coverage.y,
                    object.coverage.k,
                    object.coverage.w,
                    object.alpha,
                ] {
                    assert!((0.0..=1.0).contains(&value));
                }
            }
        }
    }
}

#[test]
fn dimensions_are_validated_at_construction_and_json_boundary() {
    for value in [
        0.0,
        -1.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        0.001,
        1001.0,
    ] {
        assert!(PhysicalSize::new(value, 100.0).is_err());
        assert!(PhysicalSize::new(100.0, value).is_err());
    }
    assert!(serde_json::from_str::<PhysicalSize>(r#"{"width_mm":0,"height_mm":100}"#).is_err());
    let size = PhysicalSize::new(100.0, 150.0).unwrap();
    let target = generate_target(TargetKind::ChannelIdentity, size);
    assert_eq!(target.size(), size);
}

#[test]
fn channel_identity_has_isolated_labeled_inks_overlap_and_orientation_controls() {
    let target = generate_target(
        TargetKind::ChannelIdentity,
        PhysicalSize::new(100.0, 150.0).unwrap(),
    );
    let json = serde_json::to_value(target).unwrap();
    let objects = json["objects"].as_array().unwrap();
    for channel in ["c", "m", "y", "k", "w"] {
        let patch = objects
            .iter()
            .find(|o| o["label"] == format!("Isolated {}", channel.to_uppercase()))
            .unwrap();
        for ink in ["c", "m", "y", "k", "w"] {
            assert_eq!(
                patch["coverage"][ink],
                if ink == channel { 1.0 } else { 0.0 }
            );
        }
    }
    let overlap = objects
        .iter()
        .find(|o| o["label"] == "K + W overlap")
        .unwrap();
    assert_eq!(overlap["coverage"]["k"], 1.0);
    assert_eq!(overlap["coverage"]["w"], 1.0);
    assert!(objects
        .iter()
        .any(|o| o["shape"]["type"] == "text" && o["shape"]["text"] == "TOP LEFT R 123"));
    assert!(objects
        .iter()
        .any(|o| o["shape"]["type"] == "asymmetric_marker"));
    assert_eq!(
        objects
            .iter()
            .filter(|o| o["shape"]["type"] == "ruler")
            .count(),
        2
    );
}
