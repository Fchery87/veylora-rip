//! The band pipeline. Each band of page rows runs the full chain
//! separation -> white -> clear mask -> calibration -> slot allocation -> screening and appends
//! its rows to disk, so memory is bounded by the band size, not the page size.
//!
//! Halos: white choke/spread and perforation edge protection read neighboring rows, so each band
//! is computed over its rows plus a halo of that many rows (clipped at the page edge) and only
//! the band's own rows are kept. Everything else is pointwise, and screening uses absolute page
//! coordinates, so the output does not depend on the band height.
//!
//! Layout: the job's layout is resolved against the device's mirror ownership, then every
//! placement is resampled and composited into one page raster (`veylora-compose`) before the
//! band chain runs. The composed page is held in memory, bounded by `max_page_pixels`; the
//! chain itself still streams bands to disk. The compose algorithm and composed-page hash are
//! recorded in the manifest. The perforation eligible mask is page-sized and is not transformed.
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
use veylora_compose::{compose_page, ComposeError, ComposeInput, ComposedPage};
use veylora_device::{
    capability::MirrorOwner,
    channels::{LogicalChannelMap, ProtocolChannel},
};
use veylora_domain::{errors::ErrorCode, preflight::Finding};
use veylora_domain::{
    ids::{AssetId, JobId},
    job::{AssetReference, RenderingSnapshot},
    provenance::ContentHash,
    recipe::WhiteSource,
};
use veylora_image::SourceAsset;
use veylora_layout::{resolve_transform, validate_layout, LayoutError, LayoutPolicy};
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
    /// Cap on page pixels (width x height) of the composed page.
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
    Layout(LayoutError),
    /// The layout has blocking findings. Acknowledgement cannot bypass these.
    LayoutBlocked(Vec<Finding>),
    /// The layout has findings that need an explicit operator acknowledgement.
    LayoutConfirmationRequired(Vec<Finding>),
    Compose(ComposeError),
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
from_error!(LayoutError => Layout, ComposeError => Compose, SeparationError => Separation, CalibrationError => Calibration,
    AllocationError => Allocation, ScreenError => Screen, SeparatorError => Separator,
    ArtifactError => Artifact);

/// An operator's acknowledgement of one confirmation-required layout finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcknowledgedFinding {
    pub code: ErrorCode,
    pub subject: Option<String>,
}

pub struct RenderRequest<'a> {
    pub snapshot: &'a RenderingSnapshot,
    /// Every image the layout's placements draw from.
    pub images: &'a [SourceAsset],
    /// Who mirrors the sheet; a layout mirror is refused unless this is the RIP.
    pub mirror_owner: MirrorOwner,
    /// Limits the layout is checked against before any pixel is composed.
    pub layout_policy: LayoutPolicy,
    /// Operator decisions for confirmation-required layout findings. A finding is accepted only
    /// when its code and subject both match; blocking findings can never be acknowledged.
    pub acknowledged: &'a [AcknowledgedFinding],
    /// The grayscale masks placements reference, normally the recipe's supplied white mask.
    pub supplied_masks: &'a [SourceAsset],
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
    pub layout_hash: ContentHash,
    /// Confirmation-required layout findings the operator acknowledged for this render.
    pub acknowledged_layout_findings: Vec<Finding>,
    /// Each distinct image the layout draws from, in first-use order.
    pub sources: Vec<ManifestAsset>,
    /// Resampling and compositing arithmetic that produced the page.
    pub compose_algorithm: String,
    /// Hash of the composed RGBA page the band chain separated.
    pub composed_page_hash: ContentHash,
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

/// Mask the job snapshot retains for `id`, matched against what the caller supplied.
fn check_mask(
    supplied: &[SourceAsset],
    id: AssetId,
    snapshot_masks: &[AssetReference],
    what: &'static str,
) -> Result<ManifestAsset, RenderError> {
    let retained = find(snapshot_masks, id).ok_or(RenderError::InputMismatch(what))?;
    let mask = supplied
        .iter()
        .find(|m| m.id == id)
        .ok_or(RenderError::InputMismatch(what))?;
    if mask.original_hash != retained.content_hash {
        return Err(RenderError::InputMismatch(
            "mask is not the mask the job snapshot retains",
        ));
    }
    Ok(ManifestAsset {
        id,
        content_hash: mask.original_hash,
    })
}

/// Everything verified before any pixel is composed.
struct Verified {
    sources: Vec<ManifestAsset>,
    supplied: Option<ManifestAsset>,
    white_supplied: Option<AssetId>,
    eligible: Option<AssetId>,
}

fn verify(request: &RenderRequest<'_>) -> Result<Verified, RenderError> {
    let job = request.snapshot.snapshot().inputs();
    let recipe = job.recipe.inputs();
    let layout = job.layout.inputs();

    // Each placement must draw from an image the snapshot retains, byte for byte.
    let mut sources: Vec<ManifestAsset> = Vec::new();
    for placement in &layout.placements {
        let retained = find(&job.assets, placement.asset_id).ok_or(RenderError::InputMismatch(
            "a placement's asset is not an asset of the job snapshot",
        ))?;
        let image = request
            .images
            .iter()
            .find(|i| i.id == placement.asset_id)
            .ok_or(RenderError::InputMismatch(
                "an image the layout draws from was not supplied",
            ))?;
        if image.original_hash != retained.content_hash {
            return Err(RenderError::InputMismatch(
                "source bytes differ from the job snapshot",
            ));
        }
        if !sources.iter().any(|s| s.id == placement.asset_id) {
            sources.push(ManifestAsset {
                id: placement.asset_id,
                content_hash: image.original_hash,
            });
        }
    }

    // A supplied white mask rides with its placements: every placement names it, or none does.
    let white_supplied = match &recipe.white.source {
        WhiteSource::Supplied { mask_id } => Some(*mask_id),
        _ => None,
    };
    match white_supplied {
        Some(id) => {
            if layout.placements.iter().any(|p| p.mask_id != Some(id)) {
                return Err(RenderError::InputMismatch(
                    "every placement must carry the recipe's supplied white mask",
                ));
            }
        }
        None => {
            if layout.placements.iter().any(|p| p.mask_id.is_some()) {
                return Err(RenderError::InputMismatch(
                    "a placement references a mask the recipe does not",
                ));
            }
            if !request.supplied_masks.is_empty() {
                return Err(RenderError::InputMismatch(
                    "a mask was supplied that the recipe does not reference",
                ));
            }
        }
    }
    let supplied = white_supplied
        .map(|id| {
            check_mask(
                request.supplied_masks,
                id,
                &job.masks,
                "the recipe's supplied white mask is missing",
            )
        })
        .transpose()?;
    Ok(Verified {
        sources,
        supplied,
        white_supplied,
        eligible: recipe.clear.perforation.and_then(|p| p.eligible_mask_id),
    })
}

/// Runs layout validation. Blocking findings refuse the render; confirmation-required findings
/// must each be acknowledged by code and subject. Returns the acknowledged findings.
fn check_layout(
    request: &RenderRequest<'_>,
    layout: &veylora_layout::ResolvedLayout,
) -> Result<Vec<Finding>, RenderError> {
    let findings = validate_layout(layout, &request.layout_policy);
    let blocking: Vec<Finding> = findings
        .iter()
        .filter(|f| f.severity.is_blocking())
        .cloned()
        .collect();
    if !blocking.is_empty() {
        return Err(RenderError::LayoutBlocked(blocking));
    }
    let needs: Vec<Finding> = findings
        .into_iter()
        .filter(|f| f.severity.requires_confirmation())
        .collect();
    let accepted = |f: &Finding| {
        request
            .acknowledged
            .iter()
            .any(|a| a.code == f.code && a.subject == f.subject)
    };
    let missing: Vec<Finding> = needs.iter().filter(|f| !accepted(f)).cloned().collect();
    if !missing.is_empty() {
        return Err(RenderError::LayoutConfirmationRequired(missing));
    }
    Ok(needs)
}

/// Limits and sizes, known once the page is composed.
fn plan(
    request: &RenderRequest<'_>,
    page: &ComposedPage,
    verified: &Verified,
) -> Result<(Plan, Option<ManifestAsset>), RenderError> {
    let job = request.snapshot.snapshot().inputs();
    let recipe = job.recipe.inputs();
    let (width, height) = (page.width_px, page.height_px);

    let eligible_ref = match (verified.eligible, request.eligible_mask) {
        (None, None) => None,
        (None, Some(_)) => {
            return Err(RenderError::InputMismatch(
                "a mask was supplied that the recipe does not reference",
            ))
        }
        (Some(id), _) => {
            let eligible = std::slice::from_ref(request.eligible_mask.ok_or(
                RenderError::InputMismatch("the recipe's perforation eligible mask is missing"),
            )?);
            let r = check_mask(
                eligible,
                id,
                &job.masks,
                "the recipe's perforation eligible mask is missing",
            )?;
            let mask = &eligible[0];
            if mask.metadata.width as usize != width || mask.metadata.height as usize != height {
                return Err(RenderError::InputMismatch(
                    "mask dimensions differ from the page",
                ));
            }
            Some(r)
        }
    };

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
    let pitch = request.pitch;
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
            white_supplied: verified.white_supplied,
            eligible: verified.eligible,
        },
        eligible_ref,
    ))
}

/// Bytes a render retains: five logical planes, then a physical and a screened plane for every
/// bound channel, plus the manifest. Used to refuse early when the known capacity is too small.
pub fn estimate_artifact_bytes(page_pixels: u64, bound_channels: usize) -> u64 {
    page_pixels * 4 * 5
        + page_pixels * 4 * bound_channels as u64
        + page_pixels * 2 * bound_channels as u64
        + 64 * 1024
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
    let verified = verify(request)?;
    let job = request.snapshot.snapshot().inputs();
    let recipe = job.recipe.inputs();
    let layout = resolve_transform(&job.layout, request.mirror_owner)?;
    let acknowledged = check_layout(request, &layout)?;
    let cancelled = || request.cancellation.is_cancelled();
    let page = compose_page(
        &ComposeInput {
            layout: &layout,
            images: &request.images.iter().collect::<Vec<_>>(),
            masks: &request.supplied_masks.iter().collect::<Vec<_>>(),
            pitch: request.pitch,
            max_page_pixels: request.limits.max_page_pixels,
        },
        &cancelled,
    )
    .map_err(|e| match e {
        ComposeError::Cancelled => RenderError::Cancelled,
        ComposeError::PageTooLarge => RenderError::LimitExceeded("page exceeds the pixel limit"),
        other => RenderError::Compose(other),
    })?;
    let (plan, eligible_ref) = plan(request, &page, &verified)?;
    let (width, height) = (plan.width, plan.height);
    let page_pixels = (width * height) as u64;
    let bindings = request.channel_map.bindings();

    // Refuse early when the known capacity cannot hold the output.
    let estimate = estimate_artifact_bytes(page_pixels, bindings.len());
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
        let alpha_halo = alpha_rows(&page.image, h0..h1)?;
        let mask_band = |asset: Option<&SourceAsset>, id: Option<AssetId>| match (asset, id) {
            (Some(asset), Some(id)) => {
                mask_rows(asset, h0..h1).map(|plane| Some(SuppliedMask { id, plane }))
            }
            _ => Ok(None),
        };
        let supplied = mask_band(page.mask.as_ref(), plan.white_supplied)?;
        let eligible = mask_band(request.eligible_mask, plan.eligible)?;
        let clear_halo = build_clear_mask(
            &MaskInput {
                source: &page.image,
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

        let process = request.separator.separate(&page.image, y0..y1)?;
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
        layout_hash: job.layout.content_hash(),
        acknowledged_layout_findings: acknowledged,
        sources: verified.sources,
        compose_algorithm: veylora_compose::ALGORITHM.into(),
        composed_page_hash: page.image.original_hash,
        supplied_mask: verified.supplied,
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
