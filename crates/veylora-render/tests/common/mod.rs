#![allow(dead_code)]
//! Shared fixtures. The separator here is SYNTHETIC: a naive subtractive RGB->CMY used only to
//! exercise the pipeline. It is not a color separation and not evidence of printer behavior.
use std::{
    ops::Range,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU32, Ordering},
};
use veylora_device::channels::{
    ChannelBinding, LogicalChannel, LogicalChannelMap, PhysicalSlotId, ProtocolChannel,
};
use veylora_domain::{
    ids::{AssetId, DeviceSetupId, JobId, PlacementId, PrintModeId, RecipeId},
    job::{AssetReference, JobSnapshot, JobSnapshotInput, RenderingSnapshot},
    layout::{LayoutInput, LayoutSnapshot, Placement, Rotation},
    provenance::{ContentHash, Provenance},
    recipe::*,
};
use veylora_image::{
    AlphaDescription, ColorEncoding, DecodedImage, ImageMetadata, SampleLayout, SourceAsset,
    UntaggedPolicy,
};
use veylora_render::{
    ArtifactError, ArtifactSink, ArtifactStore, CancellationToken, DirectoryStore, ProcessBand,
    ProcessSeparator, RenderLimits, RenderRequest, SeparatorError,
};
use veylora_separation::{PixelPitch, Plane};

pub const PITCH: PixelPitch = PixelPitch {
    x_nm: 100_000,
    y_nm: 100_000,
};

pub struct TempDir(pub PathBuf);
impl TempDir {
    pub fn new() -> Self {
        static N: AtomicU32 = AtomicU32::new(0);
        let path = std::env::temp_dir().join(format!(
            "veylora-render-test-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
    pub fn files(&self) -> Vec<String> {
        let mut v: Vec<_> = std::fs::read_dir(&self.0)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        v.sort();
        v
    }
    pub fn read(&self, name: &str) -> Vec<u8> {
        std::fs::read(self.0.join(name)).unwrap()
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub fn asset(w: u32, h: u32, layout: SampleLayout, samples: Vec<u8>) -> SourceAsset {
    let alpha = match layout {
        SampleLayout::Gray8 | SampleLayout::Rgb8 => AlphaDescription::Opaque,
        _ => AlphaDescription::StraightCoverage,
    };
    SourceAsset {
        id: AssetId::new(),
        original_hash: ContentHash::digest(&samples),
        original_bytes: samples.clone(),
        metadata: ImageMetadata {
            width: w,
            height: h,
            color: ColorEncoding::AssumedSrgb,
            physical: None,
            warnings: vec![],
            untagged_policy: UntaggedPolicy::AssumeSrgb,
            decoder_version: "test",
            file_type: "image/png",
        },
        profile: None,
        alpha,
        decoded: DecodedImage {
            samples,
            layout,
            alpha,
            converted_to_working_space: false,
        },
    }
}

pub const W: usize = 12;
pub const H: usize = 23;

/// Opaque colored art, a black strip, a transparent region with saturated hidden RGB, and a
/// partially transparent edge.
pub fn artwork() -> SourceAsset {
    let mut s = Vec::new();
    for y in 0..H {
        for x in 0..W {
            let px: [u8; 4] = if y % 7 == 3 {
                [0, 0, 0, 255] // black strip -> knocked out
            } else if x < 3 && y > 12 {
                [255, 0, 127, 0] // hidden saturated RGB under alpha 0
            } else if x == 11 {
                [40, 200, 90, 128] // partial alpha edge
            } else {
                [(x * 20) as u8, (y * 10) as u8, 200, 255]
            };
            s.extend_from_slice(&px);
        }
    }
    asset(W as u32, H as u32, SampleLayout::Rgba8, s)
}
/// Gray8 mask where the left two thirds of the page are eligible.
pub fn eligible_mask() -> SourceAsset {
    let mut s = Vec::new();
    for _y in 0..H {
        for x in 0..W {
            s.push(if x < 8 { 255 } else { 0 });
        }
    }
    asset(W as u32, H as u32, SampleLayout::Gray8, s)
}
pub fn white_mask() -> SourceAsset {
    let mut s = Vec::new();
    for y in 0..H {
        for x in 0..W {
            s.push(if (x + y) % 3 == 0 { 255 } else { 90 });
        }
    }
    asset(W as u32, H as u32, SampleLayout::Gray8, s)
}

pub fn recipe(clear: ClearPolicy, white: WhitePolicy) -> PrintRecipe {
    PrintRecipe::new(PrintRecipeInput {
        schema_version: RECIPE_SCHEMA_VERSION,
        id: RecipeId::new(),
        revision: 1,
        created_at_ms: 1735000000000,
        status: RecipeStatus::Experimental,
        white,
        calibration: CalibrationPolicy {
            revision: 1,
            process_curves: std::array::from_fn(|_| vec![0, 40000, 65535]),
            channel_limits_percent: [100, 90, 80, 100],
            total_coverage_percent: 220,
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
            phase_x: 1,
            phase_y: 2,
            cell_width: 3,
            cell_height: 2,
            thresholds: vec![0, 11000, 22000, 33000, 44000, 55000],
        },
        clear,
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
pub fn white_policy(source: WhiteSource, spread_um: i32) -> WhitePolicy {
    WhitePolicy {
        source,
        combine: MaskCombination::Replace,
        density_percent: 90,
        spread_um,
        response: vec![0, 30000, 65535],
    }
}
pub fn clear_policy(eligible: Option<AssetId>) -> ClearPolicy {
    ClearPolicy {
        knockouts: vec![KnockoutPolicy {
            target: KnockoutTarget::Black,
            tolerance_permille: 50,
            transition_permille: 0,
        }],
        perforation: Some(PerforationPolicy {
            diameter_um: 200,
            spacing_um: 400,
            phase_x_px: 1,
            phase_y_px: 1,
            eligible_mask_id: eligible,
            edge_protection_um: if eligible.is_some() { 100 } else { 0 },
        }),
    }
}

pub struct Job {
    pub snapshot: RenderingSnapshot,
    pub source: SourceAsset,
    pub supplied: Option<SourceAsset>,
    pub eligible: Option<SourceAsset>,
    pub map: LogicalChannelMap,
}
pub fn channel_map() -> LogicalChannelMap {
    use LogicalChannel::*;
    let b = |l, slot, protocol, fraction| ChannelBinding {
        logical: l,
        slot: PhysicalSlotId(slot),
        protocol: ProtocolChannel(protocol),
        fraction,
    };
    LogicalChannelMap::new(
        1,
        vec![
            b(Cyan, 0, 10, 1.0),
            b(Magenta, 1, 11, 1.0),
            b(Yellow, 2, 12, 1.0),
            b(Black, 3, 13, 1.0),
            b(White, 4, 14, 0.6),
            b(White, 5, 15, 0.4),
        ],
    )
    .unwrap()
}
/// Layout of one identity placement filling the page at `PITCH`.
pub fn identity_layout(
    source: &SourceAsset,
    mutate: impl FnOnce(&mut LayoutInput),
) -> LayoutSnapshot {
    let (w, h) = (source.metadata.width, source.metadata.height);
    let mut input = LayoutInput {
        schema_version: 1,
        page_width_um: w * 100,
        page_height_um: h * 100,
        mirror: false,
        placements: vec![Placement {
            id: PlacementId::new(),
            asset_id: source.id,
            mask_id: None,
            x_um: 0,
            y_um: 0,
            width_um: w * 100,
            height_um: h * 100,
            rotation: Rotation::Deg0,
            crop_x_px: 0,
            crop_y_px: 0,
            crop_width_px: w,
            crop_height_px: h,
        }],
    };
    mutate(&mut input);
    LayoutSnapshot::new(input).unwrap()
}
pub fn job_with(
    source: SourceAsset,
    supplied: Option<SourceAsset>,
    eligible: Option<SourceAsset>,
    recipe: PrintRecipe,
    layout: impl FnOnce(&mut LayoutInput),
) -> Job {
    let masks: Vec<_> = supplied
        .iter()
        .chain(eligible.iter())
        .map(|m| AssetReference {
            id: m.id,
            content_hash: m.original_hash,
        })
        .collect();
    let reference = || VersionedReference {
        version: 1,
        content_hash: ContentHash::digest(b"synthetic"),
    };
    let input = JobSnapshotInput {
        schema_version: 1,
        id: JobId::new(),
        created_at_ms: 1735000000000,
        assets: vec![AssetReference {
            id: source.id,
            content_hash: source.original_hash,
        }],
        masks,
        layout: identity_layout(&source, layout),
        device_setup_id: DeviceSetupId::new(),
        device_setup: reference(),
        print_mode_id: PrintModeId::new(),
        print_mode: reference(),
        recipe,
        provenance: Provenance {
            software_version: "test".into(),
            backend_version: "simulation".into(),
            content_hashes: vec![],
        },
    };
    Job {
        snapshot: JobSnapshot::new(input).unwrap().begin_render().unwrap(),
        source,
        supplied,
        eligible,
        map: channel_map(),
    }
}
/// The standard scenario: alpha white with 2 px spread, knockout, and perforation with an
/// eligible mask and edge protection, so every stage and a 2-row halo are exercised.
pub fn standard_job() -> Job {
    let eligible = eligible_mask();
    let recipe = recipe(
        clear_policy(Some(eligible.id)),
        white_policy(WhiteSource::Alpha, 200),
    );
    job_with(artwork(), None, Some(eligible), recipe, |_| {})
}

pub struct SyntheticSeparator;
impl ProcessSeparator for SyntheticSeparator {
    fn separate(
        &self,
        source: &SourceAsset,
        rows: Range<usize>,
    ) -> Result<ProcessBand, SeparatorError> {
        let n = 4;
        let w = source.metadata.width as usize;
        let mut planes = [(); 4].map(|_| Vec::new());
        for y in rows.clone() {
            for x in 0..w {
                let p = &source.decoded.samples[(y * w + x) * n..(y * w + x) * n + n];
                planes[0].push(1.0 - f32::from(p[0]) / 255.0);
                planes[1].push(1.0 - f32::from(p[1]) / 255.0);
                planes[2].push(1.0 - f32::from(p[2]) / 255.0);
                planes[3].push(0.0);
            }
        }
        let [c, m, y, k] = planes.map(|v| Plane::from_pixels(w, rows.len(), v).unwrap());
        Ok(ProcessBand { c, m, y, k })
    }
}

pub fn limits(band_height: usize) -> RenderLimits {
    RenderLimits {
        max_page_pixels: 10_000,
        band_height,
        max_working_pixels: 10_000,
    }
}
pub fn request<'a>(
    job: &'a Job,
    separator: &'a dyn ProcessSeparator,
    band_height: usize,
    cancellation: &'a CancellationToken,
) -> RenderRequest<'a> {
    RenderRequest {
        snapshot: &job.snapshot,
        source: &job.source,
        supplied_mask: job.supplied.as_ref(),
        eligible_mask: job.eligible.as_ref(),
        separator,
        channel_map: &job.map,
        pitch: PITCH,
        dot_level: 255,
        limits: limits(band_height),
        cancellation,
    }
}

pub fn f32s(bytes: &[u8]) -> Vec<f32> {
    let (chunks, rest) = bytes.as_chunks::<4>();
    assert!(rest.is_empty());
    chunks.iter().map(|c| f32::from_le_bytes(*c)).collect()
}
pub fn u16s(bytes: &[u8]) -> Vec<u16> {
    let (chunks, rest) = bytes.as_chunks::<2>();
    assert!(rest.is_empty());
    chunks.iter().map(|c| u16::from_le_bytes(*c)).collect()
}

/// Delegates to a directory store but fails every write once `fail_after` bytes have been
/// written, simulating a disk that fills mid-render without telling the renderer in advance.
pub struct FlakyStore {
    pub inner: DirectoryStore,
    pub fail_after: u64,
    pub written: std::sync::Arc<AtomicU32>,
}
struct FlakySink {
    inner: Box<dyn ArtifactSink>,
    fail_after: u64,
    written: std::sync::Arc<AtomicU32>,
}
impl ArtifactSink for FlakySink {
    fn write(&mut self, bytes: &[u8]) -> Result<(), ArtifactError> {
        let total = u64::from(self.written.fetch_add(bytes.len() as u32, Ordering::SeqCst))
            + bytes.len() as u64;
        if total > self.fail_after {
            return Err(ArtifactError::NoSpace);
        }
        self.inner.write(bytes)
    }
    fn finish(self: Box<Self>) -> Result<veylora_render::ArtifactFile, ArtifactError> {
        self.inner.finish()
    }
}
impl ArtifactStore for FlakyStore {
    fn create(&mut self, name: &str) -> Result<Box<dyn ArtifactSink>, ArtifactError> {
        Ok(Box::new(FlakySink {
            inner: self.inner.create(name)?,
            fail_after: self.fail_after,
            written: self.written.clone(),
        }))
    }
    fn remaining_bytes(&self) -> Option<u64> {
        None
    }
    fn rollback(&mut self) {
        self.inner.rollback()
    }
}
