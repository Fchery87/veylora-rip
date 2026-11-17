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

/// Evidence that matches every channel of the setup record, as a physical check would produce.
pub fn evidence_for_record(
    record: &veylora_device::setup::SetupRecord,
) -> veylora_device::setup::EvidenceRecord {
    use veylora_device::setup::{ChannelEvidence, EvidenceRecord};
    EvidenceRecord {
        operator: "operator".into(),
        setup_id: record.id,
        revision: record.revision,
        setup_hash: record.content_hash,
        map_version: record.details.channel_map.version(),
        channels: record
            .details
            .channel_map
            .bindings()
            .iter()
            .enumerate()
            .map(|(i, b)| ChannelEvidence {
                target_label: format!("label-{i}"),
                slot: b.slot,
                protocol: b.protocol,
                expected_logical: b.logical,
                observed_logical: b.logical,
            })
            .collect(),
        target_id: "target-1".into(),
        observed_at: "2026-11-15T10:00:00Z".into(),
        artifact_hash: hash(b"printed target"),
        evidence_hash: hash(b"evidence"),
    }
}

/// A Gray8 PNG built by hand (stored zlib blocks), so tests can supply masks without fixtures.
pub fn gray_png(width: u32, height: u32, samples: &[u8]) -> Vec<u8> {
    assert_eq!(samples.len(), (width * height) as usize);
    fn crc(data: &[u8]) -> u32 {
        let mut c = 0xffff_ffffu32;
        for &b in data {
            c ^= u32::from(b);
            for _ in 0..8 {
                c = if c & 1 == 1 {
                    0xedb8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
            }
        }
        !c
    }
    fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        out.extend((data.len() as u32).to_be_bytes());
        let mut body = kind.to_vec();
        body.extend(data);
        out.extend(&body);
        out.extend(crc(&body).to_be_bytes());
    }
    let mut raw = Vec::new();
    for row in samples.chunks(width as usize) {
        raw.push(0);
        raw.extend(row);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &x in &raw {
        a = (a + u32::from(x)) % 65521;
        b = (b + a) % 65521;
    }
    let mut z = vec![0x78, 0x01, 0x01];
    z.extend((raw.len() as u16).to_le_bytes());
    z.extend((!(raw.len() as u16)).to_le_bytes());
    z.extend(&raw);
    z.extend(((b << 16) | a).to_be_bytes());
    let mut png = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut ihdr = Vec::new();
    ihdr.extend(width.to_be_bytes());
    ihdr.extend(height.to_be_bytes());
    ihdr.extend([8, 0, 0, 0, 0]);
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &z);
    chunk(&mut png, b"IEND", &[]);
    png
}
