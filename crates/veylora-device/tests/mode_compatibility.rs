use std::collections::BTreeSet;
use veylora_device::capability::{Capability, ControlId, DeviceCapability, MirrorOwner};
use veylora_device::channels::{
    ChannelBinding, LogicalChannel, LogicalChannelMap, PhysicalSlotId, ProtocolChannel,
};
use veylora_device::mode::{
    check_compatibility, control_offered, DepositionSpec, MediaSpec, ModeStatus, PrintMode,
    ProtocolSpec, QualitySpec, RasterSpec,
};
use veylora_device::setup::{DeviceIdentity, DeviceSetup, PhysicalSlot, SetupDetails, SetupOrigin};
use veylora_domain::ids::{DeviceSetupId, PrintModeId, RecipeId};
use veylora_domain::provenance::ContentHash;
use veylora_domain::recipe::{
    CalibrationPolicy, ClearPolicy, ColorPolicy, FinishPolicy, MaskCombination, PeelCondition,
    PrintRecipe, PrintRecipeInput, RecipeStatus, RenderingIntent, ScreenAlgorithm, ScreenPolicy,
    UntaggedRgbPolicy, VersionedReference, WhitePolicy, WhiteSource,
};

fn bindings() -> Vec<ChannelBinding> {
    [
        LogicalChannel::Cyan,
        LogicalChannel::Magenta,
        LogicalChannel::Yellow,
        LogicalChannel::Black,
        LogicalChannel::White,
        LogicalChannel::White,
    ]
    .into_iter()
    .enumerate()
    .map(|(i, logical)| ChannelBinding {
        logical,
        slot: PhysicalSlotId(i as u8),
        protocol: ProtocolChannel(i as u8),
        fraction: if logical == LogicalChannel::White {
            0.5
        } else {
            1.0
        },
    })
    .collect()
}

fn details() -> SetupDetails {
    SetupDetails {
        identity: DeviceIdentity {
            model: "synthetic test device".into(),
            serial_or_local_id: Some("fixture-only".into()),
        },
        firmware: Some("fixture-fw-recorded".into()),
        connection: "fixture".into(),
        operating_system: "fixture".into(),
        conversion_supplier: "fixture".into(),
        ink: "fixture".into(),
        film: "fixture".into(),
        feed_path: "fixture-feed".into(),
        installation_date: None,
        installation_notes: "synthetic".into(),
        reference_settings: "fixture".into(),
        physical_slots: (0..6)
            .map(|id| PhysicalSlot {
                id: PhysicalSlotId(id),
                tank_label: format!("t{id}"),
                head_label: format!("h{id}"),
                ink: "fixture".into(),
            })
            .collect(),
        channel_map: LogicalChannelMap::new(1, bindings()).unwrap(),
    }
}

fn setup() -> DeviceSetup {
    DeviceSetup::new(DeviceSetupId::new(), details(), SetupOrigin::Recorded).unwrap()
}

fn raster() -> RasterSpec {
    RasterSpec {
        dpi_x: 720,
        dpi_y: 720,
        printable_width_px: 2400,
        printable_height_px: 3600,
        band_height_px: 128,
    }
}

fn protocol() -> ProtocolSpec {
    ProtocolSpec {
        label: "fixture-protocol".into(),
        channel_sequence: vec![0, 1, 2, 3, 4, 5],
        firmware_fingerprint: "fixture-fw-recorded".into(),
    }
}

fn media() -> MediaSpec {
    MediaSpec {
        media_label: "fixture-film".into(),
        feed_path: "fixture-feed".into(),
        film_fingerprint: "fixture-film-fp".into(),
    }
}

fn deposition() -> DepositionSpec {
    DepositionSpec {
        passes: 4,
        overlap_percent: 50,
        coverage_caps: vec![
            ("cyan".into(), 1.0),
            ("magenta".into(), 1.0),
            ("yellow".into(), 1.0),
            ("black".into(), 1.0),
            ("white".into(), 1.0),
        ],
    }
}

fn quality() -> QualitySpec {
    QualitySpec {
        target_label: "fixture target".into(),
        notes: String::new(),
        tolerance_percent: 2.0,
    }
}

fn mode(setup_id: DeviceSetupId) -> PrintMode {
    PrintMode::new(
        PrintModeId::new(),
        setup_id,
        raster(),
        protocol(),
        media(),
        deposition(),
        quality(),
    )
    .unwrap()
}

fn capability() -> DeviceCapability {
    DeviceCapability {
        independent_white: Capability::Experimental,
        process_channels: ["cyan", "magenta", "yellow", "black"]
            .iter()
            .map(|c| (*c).to_string())
            .collect(),
        mirror_owner: MirrorOwner::Operator,
        supported_controls: BTreeSet::from([ControlId::new("mirror")]),
    }
}

fn recipe() -> PrintRecipe {
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
            profile: Some(VersionedReference {
                version: 1,
                content_hash: ContentHash::digest(b"synthetic"),
            }),
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
    })
    .unwrap()
}

#[test]
fn new_modes_start_experimental_and_hash_stably() {
    let setup = setup();
    let m1 = mode(setup.id());
    let m2 = PrintMode::new(
        m1.id(),
        setup.id(),
        raster(),
        protocol(),
        media(),
        deposition(),
        quality(),
    )
    .unwrap();
    assert_eq!(m1.status(), ModeStatus::Experimental);
    assert_eq!(m1.content_hash(), m2.content_hash());
    assert_eq!(m1.revision(), 1);
}

#[test]
fn any_protocol_or_media_change_changes_the_hash() {
    let setup = setup();
    let base = mode(setup.id());
    let base_hash = base.content_hash();
    for change in 0..9 {
        let mut r = raster();
        let mut p = protocol();
        let mut md = media();
        let mut dep = deposition();
        let mut q = quality();
        match change {
            0 => r.dpi_x += 1,
            1 => r.band_height_px += 1,
            2 => p.label.push('x'),
            3 => p.firmware_fingerprint.push('x'),
            4 => p.channel_sequence.pop().map(|_| ()).unwrap_or_default(),
            5 => md.media_label.push('x'),
            6 => md.film_fingerprint.push('x'),
            7 => dep.passes += 1,
            _ => q.tolerance_percent += 0.1,
        }
        let revised = base.revise(r, p, md, dep, q).unwrap();
        assert_ne!(revised.content_hash(), base_hash, "change {change}");
        assert_eq!(revised.revision(), 2);
        assert_eq!(revised.status(), ModeStatus::Experimental);
    }
}

#[test]
fn compatibility_rejects_mismatched_setup_firmware_and_feed() {
    let setup = setup();
    let other = DeviceSetup::new(DeviceSetupId::new(), details(), SetupOrigin::Recorded).unwrap();
    let m = mode(other.id());
    let findings = check_compatibility(&m, &setup, &capability(), &recipe());
    assert!(!findings.is_empty());

    // Firmware mismatch is reported when the mode fingerprint differs.
    let mut p = protocol();
    p.firmware_fingerprint = "different".into();
    let m = PrintMode::new(
        PrintModeId::new(),
        setup.id(),
        raster(),
        p,
        media(),
        deposition(),
        quality(),
    )
    .unwrap();
    let findings = check_compatibility(&m, &setup, &capability(), &recipe());
    assert!(findings.iter().any(|f| f.message.contains("firmware")));

    // Feed-path mismatch is reported when the media feed path differs.
    let mut md = media();
    md.feed_path = "other-feed".into();
    let m = PrintMode::new(
        PrintModeId::new(),
        setup.id(),
        raster(),
        protocol(),
        md,
        deposition(),
        quality(),
    )
    .unwrap();
    let findings = check_compatibility(&m, &setup, &capability(), &recipe());
    assert!(findings.iter().any(|f| f.message.contains("feed path")));
}

#[test]
fn compatibility_rejects_missing_backend_process_channels() {
    let setup = setup();
    let m = mode(setup.id());
    let mut cap = capability();
    cap.process_channels.remove("black");
    let findings = check_compatibility(&m, &setup, &cap, &recipe());
    assert!(findings
        .iter()
        .any(|f| f.message.contains("process channel")));
}

#[test]
fn validated_mode_requires_physically_verified_setup() {
    let setup = setup(); // unverified
    let mut m = mode(setup.id());
    m.mark_validated().unwrap();
    assert_eq!(m.status(), ModeStatus::Validated);
    let findings = check_compatibility(&m, &setup, &capability(), &recipe());
    assert!(findings
        .iter()
        .any(|f| f.message.contains("physically verified")));
}

#[test]
fn ui_controls_are_gated_by_capability() {
    let cap = capability();
    assert!(control_offered(&cap, &ControlId::new("mirror")));
    assert!(!control_offered(&cap, &ControlId::new("overprint")));
}

#[test]
fn invalid_specs_are_rejected() {
    let setup = setup();
    let mut r = raster();
    r.band_height_px = r.printable_height_px + 1;
    assert!(PrintMode::new(
        PrintModeId::new(),
        setup.id(),
        r,
        protocol(),
        media(),
        deposition(),
        quality()
    )
    .is_err());

    let mut d = deposition();
    d.coverage_caps[0].1 = 1.1;
    assert!(PrintMode::new(
        PrintModeId::new(),
        setup.id(),
        raster(),
        protocol(),
        media(),
        d,
        quality()
    )
    .is_err());
}

#[test]
fn white_requires_the_backend_to_offer_independent_white() {
    let setup = setup();
    let m = mode(setup.id()); // declares a white coverage cap
    let mut cap = capability();
    cap.independent_white = Capability::Unsupported;
    // The mode's white cap and the recipe's white policy each need independent white.
    let findings = check_compatibility(&m, &setup, &cap, &recipe());
    assert_eq!(findings.len(), 2, "{findings:?}");
    // Offered white raises neither.
    assert!(check_compatibility(&m, &setup, &capability(), &recipe()).is_empty());
    // A recipe with white disabled needs no white capability from the recipe's side.
    let mut input = recipe().inputs().clone();
    input.white.source = WhiteSource::Disabled;
    let no_white = PrintRecipe::new(input).unwrap();
    let findings = check_compatibility(&m, &setup, &cap, &no_white);
    assert_eq!(findings.len(), 1, "{findings:?}");
}
