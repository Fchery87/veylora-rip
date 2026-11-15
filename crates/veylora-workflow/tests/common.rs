#![allow(dead_code)]
use std::path::PathBuf;
use veylora_device::channels::{
    ChannelBinding, LogicalChannel, LogicalChannelMap, PhysicalSlotId, ProtocolChannel,
};
use veylora_device::mode::{
    DepositionSpec, MediaSpec, PrintMode, ProtocolSpec, QualitySpec, RasterSpec,
};
use veylora_device::setup::{DeviceIdentity, DeviceSetup, PhysicalSlot, SetupDetails, SetupOrigin};
use veylora_domain::{
    ids::{DeviceSetupId, PrintModeId, RecipeId},
    provenance::ContentHash,
    recipe::{
        CalibrationPolicy, ClearPolicy, ColorPolicy, FinishPolicy, MaskCombination, PeelCondition,
        PrintRecipe, PrintRecipeInput, RecipeStatus, RenderingIntent, ScreenAlgorithm,
        ScreenPolicy, UntaggedRgbPolicy, WhitePolicy, WhiteSource,
    },
};

pub fn dir(label: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let p = std::env::temp_dir().join(format!(
        "veylora-workflow-{label}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&p);
    p
}

pub fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/../../fixtures/images/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

pub fn setup() -> DeviceSetup {
    use LogicalChannel::*;
    let bindings = [Cyan, Magenta, Yellow, Black, White, White]
        .into_iter()
        .enumerate()
        .map(|(i, logical)| ChannelBinding {
            logical,
            slot: PhysicalSlotId(i as u8),
            protocol: ProtocolChannel(10 + i as u8),
            fraction: if logical == White { 0.5 } else { 1.0 },
        })
        .collect();
    DeviceSetup::new(
        DeviceSetupId::new(),
        SetupDetails {
            identity: DeviceIdentity {
                model: "synthetic".into(),
                serial_or_local_id: None,
            },
            firmware: Some("fw".into()),
            connection: "fixture".into(),
            operating_system: "fixture".into(),
            conversion_supplier: "fixture".into(),
            ink: "fixture".into(),
            film: "fixture".into(),
            feed_path: "feed".into(),
            installation_date: None,
            installation_notes: String::new(),
            reference_settings: "fixture".into(),
            physical_slots: (0..6)
                .map(|id| PhysicalSlot {
                    id: PhysicalSlotId(id),
                    tank_label: format!("t{id}"),
                    head_label: format!("h{id}"),
                    ink: "fixture".into(),
                })
                .collect(),
            channel_map: LogicalChannelMap::new(1, bindings).unwrap(),
        },
        SetupOrigin::Recorded,
    )
    .unwrap()
}

/// 1270 dpi is a 20 um pixel, so a 4 px image is exactly 80 um wide.
pub fn mode(setup: DeviceSetupId) -> PrintMode {
    PrintMode::new(
        PrintModeId::new(),
        setup,
        RasterSpec {
            dpi_x: 1270,
            dpi_y: 1270,
            printable_width_px: 2400,
            printable_height_px: 3600,
            band_height_px: 128,
        },
        ProtocolSpec {
            label: "p".into(),
            channel_sequence: vec![0, 1],
            firmware_fingerprint: "fw".into(),
        },
        MediaSpec {
            media_label: "m".into(),
            feed_path: "feed".into(),
            film_fingerprint: "f".into(),
        },
        DepositionSpec {
            passes: 4,
            overlap_percent: 50,
            coverage_caps: vec![("cyan".into(), 1.0), ("white".into(), 1.0)],
        },
        QualitySpec {
            target_label: "t".into(),
            notes: String::new(),
            tolerance_percent: 2.5,
        },
    )
    .unwrap()
}

pub fn recipe() -> PrintRecipe {
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
            untagged_rgb_policy: UntaggedRgbPolicy::AssumeSrgb,
        },
        screen: ScreenPolicy {
            algorithm: ScreenAlgorithm::OrderedThreshold,
            revision: 1,
            seed: 0,
            phase_x: 0,
            phase_y: 0,
            cell_width: 2,
            cell_height: 2,
            thresholds: vec![8192, 40960, 57344, 24576],
        },
        clear: ClearPolicy::default(),
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

pub fn hash(b: &[u8]) -> ContentHash {
    ContentHash::digest(b)
}
