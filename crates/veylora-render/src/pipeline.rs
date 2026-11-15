//! The band pipeline. Each band of page rows runs the full chain
//! separation -> white -> clear mask -> calibration -> slot allocation -> screening and appends
//! its rows to disk, so memory is bounded by the band size, not the page size.
//!
//! Halos: white choke/spread and perforation edge protection read neighboring rows, so each band
//! is computed over its rows plus a halo of that many rows (clipped at the page edge) and only
//! the band's own rows are kept. Everything else is pointwise, and screening uses absolute page
//! coordinates, so the output does not depend on the band height.
//!
//! Layout scope: only an identity layout is rendered (one placement at the page origin, no
//! rotation, mirror or crop, source pixels at the device pitch). Anything else is refused with
//! `UnsupportedLayout` until layout transforms exist.
use crate::{
    artifacts::{ArtifactError, ArtifactFile, ArtifactSink, ArtifactStore},
    process::{ProcessSeparator, SeparatorError},
    SampleFormat, Stage, StageArtifact,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    ops::Range,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use veylora_device::channels::{LogicalChannelMap, ProtocolChannel};
use veylora_domain::{
    ids::{AssetId, JobId},
    job::{AssetReference, RenderingSnapshot},
    layout::Rotation,
    provenance::ContentHash,
    recipe::WhiteSource,
};
use veylora_image::SourceAsset;
use veylora_screen::{
    apply_calibration, screen_band, CalibrationError, PageOrigin, ScreenError, ScreenSpec,
};
use veylora_separation::{
    allocate_logical_to_physical, alpha_rows, build_clear_mask, enforce_clear_mask, generate_white,
    mask_rows, AllocationError, AllocationLimits, ClearMask, LogicalPlanes, MaskInput, PixelPitch,
    Plane, SeparationError, SeparationInput, SuppliedMask,
};

pub const MANIFEST_NAME: &str = "render-manifest.json";

/// Cooperative cancellation shared between the renderer and its owner. Clones observe the same flag.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderLimits {
    /// Cap on page pixels (width x height) of the source.
    pub max_page_pixels: usize,
    /// Rows rendered per band, before halos.
    pub band_height: usize,
    /// Cap on the pixels of one band including its halos.
    pub max_working_pixels: usize,
}

#[derive(Debug)]
pub enum RenderError {
    Cancelled,
    LimitExceeded(&'static str),
    InvalidBandHeight,
    /// A supplied input is not the one the job snapshot retains, or is inconsistent with it.
    InputMismatch(&'static str),
    UnsupportedLayout(&'static str),
    InvalidDotLevel,
    Separation(SeparationError),
    Calibration(CalibrationError),
    Allocation(AllocationError),
    Screen(ScreenError),
    Separator(SeparatorError),
    Artifact(ArtifactError),
    Manifest(String),
}
macro_rules! from_error {
    ($($ty:ty => $variant:ident),*) => {$(
        impl From<$ty> for RenderError {
            fn from(value: $ty) -> Self { Self::$variant(value) }
        }
    )*};
}
from_error!(SeparationError => Separation, CalibrationError => Calibration,
    AllocationError => Allocation, ScreenError => Screen, SeparatorError => Separator,
    ArtifactError => Artifact);

pub struct RenderRequest<'a> {
    pub snapshot: &'a RenderingSnapshot,
    pub source: &'a SourceAsset,
    /// The grayscale mask the recipe's white source names, if it names one.
    pub supplied_mask: Option<&'a SourceAsset>,
    /// The grayscale mask the recipe's perforation names as its eligible area, if any.
    pub eligible_mask: Option<&'a SourceAsset>,
    pub separator: &'a dyn ProcessSeparator,
    pub channel_map: &'a LogicalChannelMap,
    pub pitch: PixelPitch,
    /// Code a printed dot takes in the device raster.
    pub dot_level: u16,
    pub limits: RenderLimits,
    pub cancellation: &'a CancellationToken,
}

/// Deterministic resource counters. Wall-clock time is deliberately not recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderStats {
    pub bands: u32,
    pub band_height: usize,
    pub halo_rows: usize,
    pub page_pixels: u64,
    /// Largest band including halos, in pixels per plane.
    pub peak_working_pixels: u64,
    pub bytes_written: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestAsset {
    pub id: AssetId,
    pub content_hash: ContentHash,
}
/// Everything needed to find and verify the artifacts and the inputs that produced them.
/// Software output only: nothing here is evidence of physical printer behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderManifest {
    pub schema_version: u32,
    pub evidence: String,
    pub job_id: JobId,
    pub snapshot_hash: ContentHash,
    pub recipe_hash: ContentHash,
    pub source: ManifestAsset,
    pub supplied_mask: Option<ManifestAsset>,
    pub eligible_mask: Option<ManifestAsset>,
    pub channel_map_version: u32,
    pub pitch_x_nm: u32,
    pub pitch_y_nm: u32,
    pub page_width_px: usize,
    pub page_height_px: usize,
    pub dot_level: u16,
    pub logical_planes: StageArtifact,
    pub physical_planes: StageArtifact,
    pub screened_planes: StageArtifact,
    pub stats: RenderStats,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderArtifacts {
    pub manifest: RenderManifest,
    pub manifest_file: ArtifactFile,
}

struct Plan {
    width: usize,
    height: usize,
    halo: usize,
    band_height: usize,
    white_supplied: Option<AssetId>,
    eligible: Option<AssetId>,
}

fn crop_rows(plane: &Plane, rows: Range<usize>) -> Plane {
    Plane {
        width: plane.width,
        height: rows.len(),
        pixels: plane.pixels[rows.start * plane.width..rows.end * plane.width].to_vec(),
    }
}
fn bytes_f32(plane: &Plane, buffer: &mut Vec<u8>) {
    buffer.clear();
    for v in &plane.pixels {
        buffer.extend_from_slice(&v.to_le_bytes());
    }
}
fn find(list: &[AssetReference], id: AssetId) -> Option<&AssetReference> {
    list.iter().find(|a| a.id == id)
}

fn check_mask(
    request_mask: Option<&SourceAsset>,
    needed: Option<AssetId>,
    snapshot_masks: &[AssetReference],
    source: &SourceAsset,
    what: &'static str,
) -> Result<Option<ManifestAsset>, RenderError> {
    match (needed, request_mask) {
        (None, None) => Ok(None),
        (None, Some(_)) => Err(RenderError::InputMismatch(
            "a mask was supplied that the recipe does not reference",
        )),
        (Some(_), None) => Err(RenderError::InputMismatch(what)),
        (Some(id), Some(mask)) => {
            let retained = find(snapshot_masks, id).ok_or(RenderError::InputMismatch(what))?;
            if mask.id != id || mask.original_hash != retained.content_hash {
                return Err(RenderError::InputMismatch(
                    "mask is not the mask the job snapshot retains",
                ));
            }
            if mask.metadata.width != source.metadata.width
                || mask.metadata.height != source.metadata.height
            {
                return Err(RenderError::InputMismatch(
                    "mask dimensions differ from the source",
                ));
            }
            Ok(Some(ManifestAsset {
                id,
                content_hash: mask.original_hash,
            }))
        }
    }
}

fn plan(
    request: &RenderRequest<'_>,
) -> Result<
    (
        Plan,
        ManifestAsset,
        Option<ManifestAsset>,
        Option<ManifestAsset>,
    ),
    RenderError,
> {
    let job = request.snapshot.snapshot().inputs();
    let recipe = job.recipe.inputs();
    let source = request.source;
    let (width, height) = (
        source.metadata.width as usize,
        source.metadata.height as usize,
    );

    let retained = find(&job.assets, source.id).ok_or(RenderError::InputMismatch(
        "source is not an asset of the job snapshot",
    ))?;
    if retained.content_hash != source.original_hash {
        return Err(RenderError::InputMismatch(
            "source bytes differ from the job snapshot",
        ));
    }

    // Identity layout only.
    let layout = job.layout.inputs();
    let [placement] = layout.placements.as_slice() else {
        return Err(RenderError::UnsupportedLayout(
            "exactly one placement is required",
        ));
    };
    let pitch = request.pitch;
    let um = |px: usize, nm: u32| (px as u64 * u64::from(nm) + 500) / 1000;
    if layout.mirror
        || placement.rotation != Rotation::Deg0
        || (placement.x_um, placement.y_um) != (0, 0)
        || placement.asset_id != source.id
        || placement.mask_id.is_some()
        || (placement.crop_x_px, placement.crop_y_px) != (0, 0)
        || placement.crop_width_px as usize != width
        || placement.crop_height_px as usize != height
        || u64::from(placement.width_um) != um(width, pitch.x_nm)
        || u64::from(placement.height_um) != um(height, pitch.y_nm)
        || layout.page_width_um != placement.width_um
        || layout.page_height_um != placement.height_um
    {
        return Err(RenderError::UnsupportedLayout(
            "only an unrotated, unmirrored, uncropped placement filling the page at the device pitch",
        ));
    }

    let white_supplied = match &recipe.white.source {
        WhiteSource::Supplied { mask_id } => Some(*mask_id),
        _ => None,
    };
    let eligible = recipe.clear.perforation.and_then(|p| p.eligible_mask_id);
    let supplied = check_mask(
        request.supplied_mask,
        white_supplied,
        &job.masks,
        source,
        "the recipe's supplied white mask is missing",
    )?;
    let eligible_asset = check_mask(
        request.eligible_mask,
        eligible,
        &job.masks,
        source,
        "the recipe's perforation eligible mask is missing",
    )?;

    if request.dot_level == 0 {
        return Err(RenderError::InvalidDotLevel);
    }
    let page_pixels = width
        .checked_mul(height)
        .ok_or(RenderError::LimitExceeded("page pixels overflow"))?;
    if page_pixels > request.limits.max_page_pixels {
        return Err(RenderError::LimitExceeded("page exceeds the pixel limit"));
    }
    if request.limits.band_height == 0 {
        return Err(RenderError::InvalidBandHeight);
    }
    let white_halo = pitch.radius_px(recipe.white.spread_um.unsigned_abs())?;
    let edge_halo = match &recipe.clear.perforation {
        Some(p) => pitch.radius_px(p.edge_protection_um)?,
        None => 0,
    };
    let halo = white_halo.max(edge_halo);
    let band_height = request.limits.band_height.min(height);
    let working = width.saturating_mul(band_height + 2 * halo);
    if working > request.limits.max_working_pixels {
        return Err(RenderError::LimitExceeded(
            "band exceeds the working-pixel limit",
        ));
    }
    Ok((
        Plan {
            width,
            height,
            halo,
            band_height,
            white_supplied,
            eligible,
        },
        ManifestAsset {
            id: source.id,
            content_hash: source.original_hash,
        },
        supplied,
        eligible_asset,
    ))
}

struct Sinks {
    logical: Vec<Box<dyn ArtifactSink>>,
    physical: BTreeMap<ProtocolChannel, Box<dyn ArtifactSink>>,
    screened: BTreeMap<ProtocolChannel, Box<dyn ArtifactSink>>,
}
const LOGICAL_NAMES: [&str; 5] = [
    "logical-c.f32",
    "logical-m.f32",
    "logical-y.f32",
    "logical-k.f32",
    "logical-w.f32",
];

/// Renders the job to `store` and returns the manifest. On any failure, including cancellation,
/// everything created in `store` is removed.
pub fn render(
    request: &RenderRequest<'_>,
    store: &mut dyn ArtifactStore,
) -> Result<RenderArtifacts, RenderError> {
    let result = run(request, store);
    if result.is_err() {
        store.rollback();
    }
    result
}

fn run(
    request: &RenderRequest<'_>,
    store: &mut dyn ArtifactStore,
) -> Result<RenderArtifacts, RenderError> {
    if request.cancellation.is_cancelled() {
        return Err(RenderError::Cancelled);
    }
    let (plan, source_ref, supplied_ref, eligible_ref) = plan(request)?;
    let job = request.snapshot.snapshot().inputs();
    let recipe = job.recipe.inputs();
    let (width, height) = (plan.width, plan.height);
    let page_pixels = (width * height) as u64;
    let bindings = request.channel_map.bindings();

    // Refuse early when the known capacity cannot hold the output.
    let estimate = page_pixels * 4 * 5
        + page_pixels * 4 * bindings.len() as u64
        + page_pixels * 2 * bindings.len() as u64
        + 64 * 1024;
    if store.remaining_bytes().is_some_and(|free| free < estimate) {
        return Err(RenderError::Artifact(ArtifactError::NoSpace));
    }

    let mut sinks = Sinks {
        logical: Vec::new(),
        physical: BTreeMap::new(),
        screened: BTreeMap::new(),
    };
    for name in LOGICAL_NAMES {
        sinks.logical.push(store.create(name)?);
    }
    for b in bindings {
        sinks.physical.insert(
            b.protocol,
            store.create(&format!("physical-p{}.f32", b.protocol.0))?,
        );
        sinks.screened.insert(
            b.protocol,
            store.create(&format!("screened-p{}.u16", b.protocol.0))?,
        );
    }

    let screen = ScreenSpec {
        policy: recipe.screen.clone(),
        dot_level: request.dot_level,
    };
    let mut buffer = Vec::new();
    let mut bands = 0u32;
    let mut peak = 0u64;
    let mut y0 = 0;
    while y0 < height {
        if request.cancellation.is_cancelled() {
            return Err(RenderError::Cancelled);
        }
        let y1 = (y0 + plan.band_height).min(height);
        let h0 = y0.saturating_sub(plan.halo);
        let h1 = (y1 + plan.halo).min(height);
        let offset = y0 - h0;
        let rows = y1 - y0;
        peak = peak.max((width * (h1 - h0)) as u64);

        // Halo-wide inputs for the operations that read neighboring rows.
        let alpha_halo = alpha_rows(request.source, h0..h1)?;
        let mask_band = |asset: Option<&SourceAsset>, id: Option<AssetId>| match (asset, id) {
            (Some(asset), Some(id)) => {
                mask_rows(asset, h0..h1).map(|plane| Some(SuppliedMask { id, plane }))
            }
            _ => Ok(None),
        };
        let supplied = mask_band(request.supplied_mask, plan.white_supplied)?;
        let eligible = mask_band(request.eligible_mask, plan.eligible)?;
        let clear_halo = build_clear_mask(
            &MaskInput {
                source: request.source,
                rows: Some(h0..h1),
                eligible_mask: eligible.as_ref(),
                origin_px: (0, 0),
                pitch: request.pitch,
            },
            &recipe.clear,
        )?;
        let white_halo = generate_white(
            &SeparationInput {
                alpha: alpha_halo.clone(),
                tonal: None,
                supplied_mask: supplied,
                clear_mask: Some(clear_halo.clone()),
            },
            &recipe.white,
            request.pitch,
        )?;

        // Keep only this band's own rows.
        let own = offset..offset + rows;
        let alpha = crop_rows(&alpha_halo, own.clone());
        let clear = ClearMask::from_plane(crop_rows(clear_halo.retained(), own.clone()))?;
        let white = crop_rows(&white_halo, own);

        let process = request.separator.separate(request.source, y0..y1)?;
        let band_size = (width, rows);
        let mut process_planes = [process.c, process.m, process.y, process.k];
        for plane in &mut process_planes {
            if (plane.width, plane.height) != band_size {
                return Err(RenderError::Separator(SeparatorError(
                    "separator returned a plane of the wrong size".into(),
                )));
            }
            plane.validate()?;
            // Straight alpha: hidden color under transparent pixels can never print.
            for (v, a) in plane.pixels.iter_mut().zip(&alpha.pixels) {
                *v = (*v * a).clamp(0.0, 1.0);
            }
        }
        let [c, m, y, k] = process_planes;
        let mut logical = LogicalPlanes {
            c,
            m,
            y,
            k,
            w: white,
        };
        enforce_clear_mask(&mut logical, &clear)?;
        for (sink, plane) in sinks
            .logical
            .iter_mut()
            .zip([&logical.c, &logical.m, &logical.y, &logical.k, &logical.w])
        {
            bytes_f32(plane, &mut buffer);
            sink.write(&buffer)?;
        }

        apply_calibration(&mut logical, &recipe.calibration)?;
        let physical = allocate_logical_to_physical(
            &logical,
            request.channel_map,
            &AllocationLimits {
                max_total_pixels: bindings.len() * width * rows,
            },
        )?;
        for (protocol, allocated) in &physical.slots {
            let sink = sinks.physical.get_mut(protocol).expect("sink per binding");
            bytes_f32(&allocated.plane, &mut buffer);
            sink.write(&buffer)?;
            let screened =
                screen_band(&allocated.plane, PageOrigin { x: 0, y: y0 as i64 }, &screen)?;
            buffer.clear();
            for code in &screened.pixels {
                buffer.extend_from_slice(&code.to_le_bytes());
            }
            sinks
                .screened
                .get_mut(protocol)
                .expect("sink per binding")
                .write(&buffer)?;
        }
        bands += 1;
        y0 = y1;
    }
    if request.cancellation.is_cancelled() {
        return Err(RenderError::Cancelled);
    }

    let finish = |sinks: Vec<Box<dyn ArtifactSink>>| -> Result<Vec<ArtifactFile>, RenderError> {
        sinks.into_iter().map(|s| Ok(s.finish()?)).collect()
    };
    let logical_files = finish(sinks.logical)?;
    let physical_files = finish(sinks.physical.into_values().collect())?;
    let screened_files = finish(sinks.screened.into_values().collect())?;
    let bytes_written = [&logical_files, &physical_files, &screened_files]
        .iter()
        .flat_map(|f| f.iter())
        .map(|f| f.bytes)
        .sum();
    let stage = |stage, format, files| StageArtifact::new(stage, width, height, format, files);
    let manifest = RenderManifest {
        schema_version: 1,
        evidence: "software_only".into(),
        job_id: job.id,
        snapshot_hash: request.snapshot.content_hash(),
        recipe_hash: job.recipe.content_hash(),
        source: source_ref,
        supplied_mask: supplied_ref,
        eligible_mask: eligible_ref,
        channel_map_version: request.channel_map.version(),
        pitch_x_nm: request.pitch.x_nm,
        pitch_y_nm: request.pitch.y_nm,
        page_width_px: width,
        page_height_px: height,
        dot_level: request.dot_level,
        logical_planes: stage(Stage::LogicalPlanes, SampleFormat::F32Le, logical_files),
        physical_planes: stage(Stage::PhysicalPlanes, SampleFormat::F32Le, physical_files),
        screened_planes: stage(Stage::ScreenedPlanes, SampleFormat::U16Le, screened_files),
        stats: RenderStats {
            bands,
            band_height: plan.band_height,
            halo_rows: plan.halo,
            page_pixels,
            peak_working_pixels: peak,
            bytes_written,
        },
    };
    let bytes =
        serde_json::to_vec_pretty(&manifest).map_err(|e| RenderError::Manifest(e.to_string()))?;
    let mut sink = store.create(MANIFEST_NAME)?;
    sink.write(&bytes)?;
    let manifest_file = sink.finish()?;
    Ok(RenderArtifacts {
        manifest,
        manifest_file,
    })
}
