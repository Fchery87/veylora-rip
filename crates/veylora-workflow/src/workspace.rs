//! The application service layer behind the headless commands. It owns the stores and the order
//! of operations; the CLI only parses arguments and prints results.
use crate::{
    error::{WorkflowError, WorkflowResult},
    spec::{CropSpec, JobSpec},
};
use serde::Serialize;
use std::path::{Path, PathBuf};
use veylora_device::{
    capability::{Capability, DeviceCapability, MirrorOwner},
    mode::{check_compatibility, ModeRecord, PrintMode},
    setup::{DeviceSetup, SetupRecord},
};
use veylora_domain::{
    errors::ErrorCode,
    ids::{AssetId, JobId, PlacementId},
    job::{AssetReference, JobSnapshot, JobSnapshotInput},
    layout::{LayoutInput, LayoutSnapshot, Placement},
    lifecycle::{JobState, LifecycleEvent},
    preflight::{Finding, Severity},
    provenance::{ContentHash, Provenance},
    recipe::{PrintRecipe, UntaggedRgbPolicy, WhiteSource},
};
use veylora_image::{
    decode_png_with_policy, DecodeLimits, SampleLayout, SourceAsset, UntaggedPolicy,
};
use veylora_layout::{resolve_transform, validate_layout, LayoutPolicy};
use veylora_persistence::{
    FileAssetStore, FileDeviceStore, FileJobStore, FileLifecycleStore, FileRecordStore, JobStore,
};
use veylora_render::{
    render, AcknowledgedFinding, CancellationToken, DirectoryStore, RenderLimits, RenderManifest,
    RenderRequest, WhiteOnly, MANIFEST_NAME,
};
use veylora_separation::PixelPitch;

/// Backend label recorded in every job. These jobs target a file sink, not a printer.
pub const DIAGNOSTIC_BACKEND: &str = "file-sink-diagnostic";

/// Deterministic asset identity derived from the content hash, so one file always has one id and
/// a recipe can name a mask before any job exists.
pub fn asset_id_for(hash: &ContentHash) -> AssetId {
    let hex = hash.to_string();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
    .parse()
    .expect("32 hex digits form a UUID")
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JobCreated {
    pub job_id: JobId,
    pub snapshot_hash: ContentHash,
}

/// Operator layout policy. Not device truth: the media spec carries no margin or size data, so
/// these are stated, echoed in every report and recorded with the findings they produce.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PreflightOptions {
    pub min_effective_ppi: u32,
    pub margin_um: u32,
    pub supported_page_sizes_um: Vec<(u32, u32)>,
    /// Largest composed page, in pixels, a render may hold in memory.
    pub max_page_pixels: usize,
    /// Bytes the retained artifacts may occupy. Unlimited when absent. This is a budget the
    /// operator states, not a measurement of free disk space.
    pub disk_budget_bytes: Option<u64>,
}
impl Default for PreflightOptions {
    fn default() -> Self {
        Self {
            min_effective_ppi: 150,
            margin_um: 0,
            supported_page_sizes_um: Vec::new(),
            max_page_pixels: 1 << 26,
            disk_budget_bytes: None,
        }
    }
}
impl PreflightOptions {
    fn policy(&self) -> LayoutPolicy {
        LayoutPolicy {
            min_effective_ppi: self.min_effective_ppi,
            margin_um: self.margin_um,
            supported_page_sizes_um: self.supported_page_sizes_um.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PreflightReport {
    pub job_id: JobId,
    pub snapshot_hash: ContentHash,
    pub policy: PreflightOptions,
    /// False when any finding is blocking.
    pub can_render: bool,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub layout: PreflightOptions,
    /// Confirmation-required findings the operator accepts, by code and subject.
    pub acknowledged: Vec<(ErrorCode, Option<String>)>,
    /// Rows per band. The mode's own band height when absent.
    pub band_height: Option<usize>,
    /// Code a printed dot takes in the screened raster. A diagnostic default until the print
    /// mode carries it.
    pub dot_level: u16,
    pub max_working_pixels: usize,
}
impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            layout: PreflightOptions::default(),
            acknowledged: Vec::new(),
            band_height: None,
            dot_level: 65535,
            max_working_pixels: 1 << 26,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderOutcome {
    pub dir: PathBuf,
    pub manifest: RenderManifest,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogSetup {
    pub hash: ContentHash,
    pub id: String,
    pub physically_verified: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogMode {
    pub hash: ContentHash,
    pub id: String,
    pub setup_id: String,
    pub status: veylora_device::mode::ModeStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogRecipe {
    pub hash: ContentHash,
    pub id: String,
    pub revision: u32,
    pub retired: bool,
}

/// Stored records a new job can be built from.
#[derive(Debug, Clone, PartialEq)]
pub struct Catalog {
    pub setups: Vec<CatalogSetup>,
    pub modes: Vec<CatalogMode>,
    pub recipes: Vec<CatalogRecipe>,
}

/// A run copied out of the workspace by `export_latest_run`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedRun {
    pub dir: PathBuf,
    pub run: String,
    pub files: usize,
    pub bytes: u64,
}

/// What a job's stored device setup and print mode say about themselves.
#[derive(Debug, Clone, PartialEq)]
pub struct JobDevice {
    pub setup_id: String,
    /// True only when the setup carries physical channel evidence.
    pub setup_physically_verified: bool,
    pub mode_id: String,
    pub mode_status: veylora_device::mode::ModeStatus,
    /// Largest page the mode can print, in micrometres (width, height).
    pub printable_um: (u64, u64),
    /// True when the mode caps a white channel above zero, so white can be printed at all.
    pub white_capable: bool,
    /// Version of the setup's logical-channel map, which says what each protocol channel is.
    pub channel_map_version: u32,
    /// The setup's channel bindings, in the order the map stores them.
    pub channels: Vec<JobChannel>,
}

/// One binding of a setup's channel map with the labels of the physical slot it names.
#[derive(Debug, Clone, PartialEq)]
pub struct JobChannel {
    pub logical: veylora_device::channels::LogicalChannel,
    pub slot: u8,
    pub protocol: u8,
    /// Fraction of the logical coverage this slot carries.
    pub fraction: f64,
    pub tank_label: String,
    pub head_label: String,
}

/// One completed render run and what re-reading its retained bytes found.
#[derive(Debug, Clone)]
pub struct InspectedRun {
    pub run: String,
    pub report: crate::inspect::ArtifactReport,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderSummary {
    pub run: String,
    pub manifest_hash: ContentHash,
    pub snapshot_hash: ContentHash,
    pub page_width_px: usize,
    pub page_height_px: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttemptSummary {
    pub attempt_id: String,
    pub sequence: u32,
    pub state: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JobReport {
    pub job_id: JobId,
    pub snapshot_hash: ContentHash,
    pub created_at_ms: u64,
    pub setup: ContentHash,
    pub mode: ContentHash,
    pub recipe: ContentHash,
    pub assets: Vec<AssetReference>,
    pub masks: Vec<AssetReference>,
    pub layout_hash: ContentHash,
    pub renders: Vec<RenderSummary>,
    pub submission_attempts: Vec<AttemptSummary>,
}

pub struct Workspace {
    pub(crate) root: PathBuf,
    pub(crate) records: FileRecordStore,
    pub(crate) devices: FileDeviceStore,
    pub(crate) jobs: FileJobStore,
    pub(crate) lifecycle: FileLifecycleStore,
    pub(crate) assets: FileAssetStore,
}

fn untagged(policy: UntaggedRgbPolicy) -> UntaggedPolicy {
    match policy {
        UntaggedRgbPolicy::Reject => UntaggedPolicy::Reject,
        UntaggedRgbPolicy::AssumeSrgb => UntaggedPolicy::AssumeSrgb,
    }
}

/// What the file-sink backend offers: what the setup physically has, nothing more. The RIP owns
/// the mirror because it produces the raster itself. This is not a printer capability claim.
pub(crate) fn diagnostic_capability(setup: &DeviceSetup) -> DeviceCapability {
    use veylora_device::channels::LogicalChannel::*;
    let bindings = setup.details().channel_map.bindings();
    let has = |c| bindings.iter().any(|b| b.logical == c);
    DeviceCapability {
        independent_white: if has(White) {
            Capability::Experimental
        } else {
            Capability::Unsupported
        },
        process_channels: [
            (Cyan, "cyan"),
            (Magenta, "magenta"),
            (Yellow, "yellow"),
            (Black, "black"),
        ]
        .into_iter()
        .filter(|(c, _)| has(*c))
        .map(|(_, n)| n.to_string())
        .collect(),
        mirror_owner: MirrorOwner::Rip,
        supported_controls: Default::default(),
    }
}

fn finding(code: ErrorCode, severity: Severity, subject: &str, message: &str) -> Finding {
    Finding {
        code,
        severity,
        message: message.to_string(),
        subject: Some(subject.to_string()),
    }
}

pub(crate) struct Loaded {
    pub(crate) snapshot: JobSnapshot,
    pub(crate) setup: DeviceSetup,
    pub(crate) mode: PrintMode,
}

impl Workspace {
    pub fn open(root: impl Into<PathBuf>) -> WorkflowResult<Self> {
        let root = root.into();
        Ok(Self {
            records: FileRecordStore::open(root.join("records"))?,
            devices: FileDeviceStore::open(root.join("devices"))?,
            jobs: FileJobStore::open(root.join("jobs"))?,
            lifecycle: FileLifecycleStore::open(root.join("state"))?,
            assets: FileAssetStore::open(root.join("assets"))?,
            root,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn import_setup(&self, record: &SetupRecord) -> WorkflowResult<ContentHash> {
        let setup = DeviceSetup::from_record(record.clone())
            .map_err(|f| WorkflowError::Invalid(f.message))?;
        Ok(self.devices.put_setup(&setup)?)
    }
    pub fn import_mode(&self, record: &ModeRecord) -> WorkflowResult<ContentHash> {
        let mode = PrintMode::from_record(record.clone())
            .map_err(|f| WorkflowError::Invalid(f.message))?;
        Ok(self.devices.put_mode(&mode)?)
    }
    pub fn import_recipe(&self, recipe: &PrintRecipe) -> WorkflowResult<ContentHash> {
        Ok(self.records.put_recipe(recipe)?)
    }
    pub fn import_asset(&self, bytes: &[u8]) -> WorkflowResult<ContentHash> {
        Ok(self.assets.put(bytes)?)
    }

    fn decode(&self, hash: &ContentHash, policy: UntaggedPolicy) -> WorkflowResult<SourceAsset> {
        let bytes = self.assets.get(hash)?;
        let mut asset = decode_png_with_policy(&bytes, DecodeLimits::default(), policy)?;
        // The snapshot refers to assets by their content-derived id.
        asset.id = asset_id_for(hash);
        Ok(asset)
    }

    pub fn create_job(&self, spec: &JobSpec) -> WorkflowResult<JobCreated> {
        let invalid = |m: &str| WorkflowError::Invalid(m.to_string());
        if spec.schema_version != 1 {
            return Err(invalid("Only job spec schema version 1 is supported."));
        }
        let setup = self.devices.get_setup(&spec.setup)?;
        let mode = self.devices.get_mode(&spec.mode)?;
        let recipe = self.records.get_recipe(&spec.recipe)?;
        if mode.device_setup() != setup.id() {
            return Err(invalid(
                "The print mode belongs to a different device setup.",
            ));
        }
        let policy = untagged(recipe.inputs().color.untagged_rgb_policy);

        // Decoding checks every referenced file and gives the size a crop defaults to.
        let mut sizes = std::collections::BTreeMap::new();
        let mut assets = Vec::new();
        for (name, hash) in &spec.assets {
            let a = self.decode(hash, policy)?;
            sizes.insert(name.clone(), (a.metadata.width, a.metadata.height));
            assets.push(AssetReference {
                id: asset_id_for(hash),
                content_hash: *hash,
            });
        }
        let mut masks = Vec::new();
        for (name, hash) in &spec.masks {
            let m = self.decode(hash, policy)?;
            if m.decoded.layout != SampleLayout::Gray8 {
                return Err(WorkflowError::Invalid(format!(
                    "Mask '{name}' must be a Gray8 image."
                )));
            }
            masks.push(AssetReference {
                id: asset_id_for(hash),
                content_hash: *hash,
            });
        }

        let mut placements = Vec::new();
        for p in &spec.layout.placements {
            let hash = spec.assets.get(&p.asset).ok_or_else(|| {
                WorkflowError::Invalid(format!("Placement names unknown asset '{}'.", p.asset))
            })?;
            let mask_id = match &p.mask {
                None => None,
                Some(name) => Some(asset_id_for(spec.masks.get(name).ok_or_else(|| {
                    WorkflowError::Invalid(format!("Placement names unknown mask '{name}'."))
                })?)),
            };
            let (w, h) = sizes[&p.asset];
            let crop = p.crop.unwrap_or(CropSpec {
                x: 0,
                y: 0,
                width: w,
                height: h,
            });
            placements.push(Placement {
                id: PlacementId::new(),
                asset_id: asset_id_for(hash),
                mask_id,
                x_um: p.x_um,
                y_um: p.y_um,
                width_um: p.width_um,
                height_um: p.height_um,
                rotation: p.rotation,
                crop_x_px: crop.x,
                crop_y_px: crop.y,
                crop_width_px: crop.width,
                crop_height_px: crop.height,
            });
        }
        let layout = LayoutSnapshot::new(LayoutInput {
            schema_version: 1,
            page_width_um: spec.layout.page_width_um,
            page_height_um: spec.layout.page_height_um,
            mirror: spec.layout.mirror,
            placements,
        })
        .map_err(|e| WorkflowError::Invalid(e.message))?;

        let mut content_hashes = vec![
            ("device_setup".to_string(), setup.content_hash()),
            ("print_mode".to_string(), mode.content_hash()),
            ("recipe".to_string(), recipe.content_hash()),
        ];
        content_hashes.extend(
            assets
                .iter()
                .map(|a| (format!("asset:{}", a.id), a.content_hash)),
        );
        content_hashes.extend(
            masks
                .iter()
                .map(|a| (format!("mask:{}", a.id), a.content_hash)),
        );
        let snapshot = JobSnapshot::new(JobSnapshotInput {
            schema_version: 1,
            id: spec.job_id.unwrap_or_default(),
            created_at_ms: spec.created_at_ms,
            assets,
            masks,
            layout,
            device_setup_id: setup.id(),
            device_setup: veylora_domain::recipe::VersionedReference {
                version: setup.revision(),
                content_hash: setup.content_hash(),
            },
            print_mode_id: mode.id(),
            print_mode: veylora_domain::recipe::VersionedReference {
                version: mode.revision(),
                content_hash: mode.content_hash(),
            },
            recipe,
            provenance: Provenance {
                software_version: veylora_domain::version().to_string(),
                backend_version: DIAGNOSTIC_BACKEND.to_string(),
                content_hashes,
            },
        })
        .map_err(|e| WorkflowError::Invalid(e.message))?;
        let created = JobCreated {
            job_id: snapshot.job_id(),
            snapshot_hash: snapshot.content_hash(),
        };
        self.jobs.create_snapshot(snapshot)?;
        Ok(created)
    }

    pub(crate) fn load(&self, id: JobId) -> WorkflowResult<Loaded> {
        let snapshot = self.jobs.get(id)?;
        let inputs = snapshot.inputs();
        let setup = self.devices.get_setup(&inputs.device_setup.content_hash)?;
        let mode = self.devices.get_mode(&inputs.print_mode.content_hash)?;
        Ok(Loaded {
            snapshot,
            setup,
            mode,
        })
    }

    pub fn preflight(
        &self,
        id: JobId,
        options: &PreflightOptions,
    ) -> WorkflowResult<PreflightReport> {
        let loaded = self.load(id)?;
        let findings = self.preflight_findings(&loaded, options);
        let can_render = !findings.iter().any(|f| f.severity.is_blocking());
        self.note_preflight(id, !can_render)?;
        Ok(PreflightReport {
            job_id: id,
            snapshot_hash: loaded.snapshot.content_hash(),
            policy: options.clone(),
            can_render,
            findings,
        })
    }

    fn preflight_findings(&self, loaded: &Loaded, options: &PreflightOptions) -> Vec<Finding> {
        let inputs = loaded.snapshot.inputs();
        let recipe = &inputs.recipe;
        let mut findings = check_compatibility(
            &loaded.mode,
            &loaded.setup,
            &diagnostic_capability(&loaded.setup),
            recipe,
        );
        if loaded.setup.require_production().is_err() {
            findings.push(finding(
                ErrorCode::UnverifiedDeviceSetup,
                Severity::Informational,
                "device_setup",
                "The device setup has no physical channel evidence. Output is diagnostic only.",
            ));
        }

        // The page must fit the mode's printable area.
        let raster = loaded.mode.raster();
        let printable_um = |px: u32, dpi: u32| u64::from(px) * 25_400 / u64::from(dpi);
        let layout = inputs.layout.inputs();
        if u64::from(layout.page_width_um) > printable_um(raster.printable_width_px, raster.dpi_x)
            || u64::from(layout.page_height_um)
                > printable_um(raster.printable_height_px, raster.dpi_y)
        {
            findings.push(finding(
                ErrorCode::UnsupportedMediaSize,
                Severity::Blocking,
                "layout.page",
                "The page is larger than the print mode's printable area.",
            ));
        }

        self.source_findings(inputs, &mut findings);
        if recipe.inputs().color.profile.is_some() {
            findings.push(finding(
                ErrorCode::UnsupportedProfile,
                Severity::Blocking,
                "recipe.color.profile",
                "The recipe names a color profile, but no color engine is connected to this                  workflow. It would be ignored, so the render is refused.",
            ));
        }
        self.resource_findings(loaded, options, &mut findings);

        match resolve_transform(&inputs.layout, MirrorOwner::Rip) {
            Ok(resolved) => findings.extend(validate_layout(&resolved, &options.policy())),
            Err(e) => findings.push(finding(
                ErrorCode::UnsupportedControl,
                Severity::Blocking,
                "layout.mirror",
                &format!("The layout cannot be resolved: {e:?}"),
            )),
        }
        findings
    }

    /// Every file the snapshot names must still be in the store and still match its hash.
    fn source_findings(
        &self,
        inputs: &veylora_domain::job::JobSnapshotInput,
        out: &mut Vec<Finding>,
    ) {
        for (kind, list) in [("asset", &inputs.assets), ("mask", &inputs.masks)] {
            for a in list {
                let subject = format!("{kind}:{}", a.id);
                match self.assets.get(&a.content_hash) {
                    Ok(_) => {}
                    Err(veylora_persistence::PersistenceError::NotFound { .. }) => {
                        out.push(finding(
                            ErrorCode::MissingAsset,
                            Severity::Blocking,
                            &subject,
                            "The source file is no longer in the asset store.",
                        ))
                    }
                    Err(e) => out.push(finding(
                        ErrorCode::UnsupportedInput,
                        Severity::Blocking,
                        &subject,
                        &format!("The stored source file cannot be trusted: {e}"),
                    )),
                }
            }
        }
    }

    /// Limits that a render would otherwise only discover by failing part of the way through.
    fn resource_findings(
        &self,
        loaded: &Loaded,
        options: &PreflightOptions,
        out: &mut Vec<Finding>,
    ) {
        let layout = loaded.snapshot.inputs().layout.inputs();
        let raster = loaded.mode.raster();
        let Some(pitch) = PixelPitch::from_dpi(raster.dpi_x, raster.dpi_y) else {
            return; // Rendering reports the unusable resolution itself.
        };
        let (w, h) =
            veylora_compose::page_size_px(layout.page_width_um, layout.page_height_um, pitch);
        let pixels = (w as u64).saturating_mul(h as u64);
        if pixels > options.max_page_pixels as u64 {
            out.push(finding(
                ErrorCode::UnsupportedMediaSize,
                Severity::Blocking,
                "layout.page_pixels",
                &format!(
                    "The page is {pixels} pixels, over the limit of {}.",
                    options.max_page_pixels
                ),
            ));
        }
        let bound = loaded.setup.details().channel_map.bindings().len();
        let needed = veylora_render::estimate_artifact_bytes(pixels, bound);
        if options
            .disk_budget_bytes
            .is_some_and(|budget| needed > budget)
        {
            out.push(finding(
                ErrorCode::InsufficientDiskSpace,
                Severity::Blocking,
                "artifacts",
                &format!(
                    "The retained artifacts need about {needed} bytes, over the budget of {}.",
                    options.disk_budget_bytes.unwrap_or_default()
                ),
            ));
        }
    }

    /// Renders the job into a new `run-NNNN` directory and records each step of the lifecycle.
    /// Refusals that happen before rendering starts (state, blocking findings, unacknowledged
    /// confirmations) leave the job where it was. A failure once rendering has begun marks the
    /// job failed; an operator can render again, which releases it.
    pub fn render(&self, id: JobId, options: &RenderOptions) -> WorkflowResult<RenderOutcome> {
        let loaded = self.load(id)?;
        self.begin_render(id, &loaded, options)?;
        match self.render_run(id, &loaded, options) {
            Ok(out) => {
                self.record(id, LifecycleEvent::RenderCompleted, "", None)?;
                Ok(out)
            }
            Err(e) => {
                let _ = self.record(id, LifecycleEvent::RenderFailed, &e.to_string(), None);
                Err(e)
            }
        }
    }

    /// Checks everything that can be refused without side effects, then moves the job into
    /// `Rendering`.
    fn begin_render(
        &self,
        id: JobId,
        loaded: &Loaded,
        options: &RenderOptions,
    ) -> WorkflowResult<()> {
        use LifecycleEvent::*;
        let state = self.state(id)?;
        if !matches!(
            state,
            JobState::Draft
                | JobState::PreflightFailed
                | JobState::Held
                | JobState::Ready
                | JobState::Failed
        ) {
            return Err(WorkflowError::Invalid(format!(
                "A job that is {state:?} cannot be rendered."
            )));
        }
        let findings = self.preflight_findings(loaded, &options.layout);
        let blocking: Vec<Finding> = findings
            .iter()
            .filter(|f| f.severity.is_blocking())
            .cloned()
            .collect();
        self.note_preflight(id, !blocking.is_empty())?;
        if !blocking.is_empty() {
            return Err(WorkflowError::PreflightBlocked(blocking));
        }
        // The renderer enforces this too; checking first keeps a refusal from failing the job.
        let unacknowledged: Vec<Finding> = findings
            .into_iter()
            .filter(|f| f.severity == Severity::ConfirmationRequired)
            .filter(|f| {
                !options
                    .acknowledged
                    .iter()
                    .any(|(code, subject)| *code == f.code && *subject == f.subject)
            })
            .collect();
        if !unacknowledged.is_empty() {
            return Err(WorkflowError::PreflightBlocked(unacknowledged));
        }
        match self.state(id)? {
            JobState::Ready => {
                self.record(id, Hold, "held for another render", None)?;
            }
            JobState::Failed => {
                self.record(id, Release, "released after a failure", None)?;
            }
            _ => {}
        }
        self.record(id, BeginRender, "", None)?;
        Ok(())
    }

    fn render_run(
        &self,
        id: JobId,
        loaded: &Loaded,
        options: &RenderOptions,
    ) -> WorkflowResult<RenderOutcome> {
        let inputs = loaded.snapshot.inputs();
        let recipe = inputs.recipe.inputs();
        let policy = untagged(recipe.color.untagged_rgb_policy);
        let images = inputs
            .assets
            .iter()
            .map(|a| self.decode(&a.content_hash, policy))
            .collect::<WorkflowResult<Vec<_>>>()?;
        let mut masks = inputs
            .masks
            .iter()
            .map(|a| self.decode(&a.content_hash, policy))
            .collect::<WorkflowResult<Vec<_>>>()?;
        let eligible_id = recipe.clear.perforation.and_then(|p| p.eligible_mask_id);
        let eligible = eligible_id
            .and_then(|id| masks.iter().position(|m| m.id == id))
            .map(|i| masks.swap_remove(i));
        let supplied: Vec<SourceAsset> = match &recipe.white.source {
            WhiteSource::Supplied { mask_id } => {
                masks.into_iter().filter(|m| m.id == *mask_id).collect()
            }
            _ => Vec::new(),
        };

        let pitch = PixelPitch::from_dpi(loaded.mode.raster().dpi_x, loaded.mode.raster().dpi_y)
            .ok_or_else(|| {
                WorkflowError::Invalid("The print mode has a zero resolution.".into())
            })?;
        let band_height = options
            .band_height
            .unwrap_or(loaded.mode.raster().band_height_px as usize);
        let acknowledged: Vec<AcknowledgedFinding> = options
            .acknowledged
            .iter()
            .map(|(code, subject)| AcknowledgedFinding {
                code: *code,
                subject: subject.clone(),
            })
            .collect();
        let rendering = loaded
            .snapshot
            .clone()
            .begin_render()
            .map_err(|e| WorkflowError::Invalid(format!("{e:?}")))?;
        let token = CancellationToken::new();
        let request = RenderRequest {
            snapshot: &rendering,
            images: &images,
            mirror_owner: MirrorOwner::Rip,
            layout_policy: options.layout.policy(),
            acknowledged: &acknowledged,
            supplied_masks: &supplied,
            eligible_mask: eligible.as_ref(),
            separator: &WhiteOnly,
            channel_map: &loaded.setup.details().channel_map,
            pitch,
            dot_level: options.dot_level,
            limits: RenderLimits {
                max_page_pixels: options.layout.max_page_pixels,
                band_height,
                max_working_pixels: options.max_working_pixels,
            },
            cancellation: &token,
        };

        let (run, dir) = self.new_run_dir(id)?;
        let mut store = DirectoryStore::new(&dir, options.layout.disk_budget_bytes);
        match render(&request, &mut store) {
            Ok(artifacts) => {
                let _ = run;
                Ok(RenderOutcome {
                    dir,
                    manifest: artifacts.manifest,
                })
            }
            Err(e) => {
                // The store already removed its files; drop the now-empty run directory.
                let _ = std::fs::remove_dir(&dir);
                Err(WorkflowError::Render(e))
            }
        }
    }

    pub(crate) fn artifacts_dir(&self, id: JobId) -> PathBuf {
        self.root.join("artifacts").join(id.to_string())
    }

    /// Creates the next `run-NNNN` directory. Existing runs are never reused.
    fn new_run_dir(&self, id: JobId) -> WorkflowResult<(String, PathBuf)> {
        let base = self.artifacts_dir(id);
        std::fs::create_dir_all(&base)?;
        let mut n = self.run_names(id)?.len() as u32 + 1;
        loop {
            let name = format!("run-{n:04}");
            match std::fs::create_dir(base.join(&name)) {
                Ok(()) => return Ok((name.clone(), base.join(name))),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => n += 1,
                Err(e) => return Err(e.into()),
            }
        }
    }

    pub(crate) fn run_names(&self, id: JobId) -> WorkflowResult<Vec<String>> {
        let mut names = Vec::new();
        match std::fs::read_dir(self.artifacts_dir(id)) {
            Ok(entries) => {
                for e in entries {
                    let e = e?;
                    if let Some(n) = e.file_name().to_str().filter(|n| n.starts_with("run-")) {
                        names.push(n.to_string());
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        names.sort();
        Ok(names)
    }

    /// Every stored job, newest first (ties broken by id so the order is stable). A snapshot that
    /// cannot be read is still listed here and fails when opened.
    pub fn list_jobs(&self) -> WorkflowResult<Vec<JobId>> {
        let mut keyed: Vec<(u64, JobId)> = Vec::new();
        for id in self.jobs.list_ids()? {
            // An unreadable snapshot sorts last rather than vanishing from the list.
            let created = self.jobs.get(id).map_or(0, |s| s.inputs().created_at_ms);
            keyed.push((created, id));
        }
        keyed.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| a.1.to_string().cmp(&b.1.to_string()))
        });
        Ok(keyed.into_iter().map(|(_, id)| id).collect())
    }

    /// What is stored and can be chosen when creating a job.
    pub fn catalog(&self) -> WorkflowResult<Catalog> {
        let mut setups = Vec::new();
        for hash in self.devices.list_setups()? {
            let setup = self.devices.get_setup(&hash)?;
            setups.push(CatalogSetup {
                hash,
                id: setup.id().to_string(),
                physically_verified: setup.require_production().is_ok(),
            });
        }
        let mut modes = Vec::new();
        for hash in self.devices.list_modes()? {
            let mode = self.devices.get_mode(&hash)?;
            modes.push(CatalogMode {
                hash,
                id: mode.id().to_string(),
                setup_id: mode.device_setup().to_string(),
                status: mode.status(),
            });
        }
        let mut recipes = Vec::new();
        for hash in self.records.list_recipes()? {
            let recipe = self.records.get_recipe(&hash)?;
            recipes.push(CatalogRecipe {
                hash,
                id: recipe.inputs().id.to_string(),
                revision: recipe.inputs().revision,
                retired: matches!(
                    recipe.inputs().status,
                    veylora_domain::recipe::RecipeStatus::Retired
                ),
            });
        }
        Ok(Catalog {
            setups,
            modes,
            recipes,
        })
    }

    fn exports_dir(&self) -> PathBuf {
        self.root.join("exports")
    }

    /// Folders this job's runs were exported to, oldest first.
    pub fn exports(&self, id: JobId) -> WorkflowResult<Vec<PathBuf>> {
        self.jobs.get(id)?;
        let prefix = format!("{id}-");
        let mut found = Vec::new();
        match std::fs::read_dir(self.exports_dir()) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry?;
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if name.starts_with(&prefix)
                        && !name.ends_with(".partial")
                        && entry.path().is_dir()
                    {
                        found.push(entry.path());
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        found.sort();
        Ok(found)
    }

    /// Copies the newest completed run to `exports/<job>-<run>`: the manifest and every file it
    /// names, byte for byte. Diagnostic output only; these are not printer bytes and nothing is
    /// sent anywhere, so no submission attempt is recorded. A damaged run is refused, an existing
    /// export is never overwritten, and the copy is re-verified before it is shown to exist.
    pub fn export_latest_run(&self, id: JobId) -> WorkflowResult<ExportedRun> {
        let inspected = self.inspect_latest_run(id)?.ok_or_else(|| {
            WorkflowError::Invalid("Nothing to export: the job has no completed render.".into())
        })?;
        if !inspected.report.verified {
            return Err(WorkflowError::Invalid(format!(
                "Run {} failed verification and is not exported: {}",
                inspected.run,
                inspected.report.problems.join("; ")
            )));
        }
        let source = self.artifacts_dir(id).join(&inspected.run);
        let dest = self.exports_dir().join(format!("{id}-{}", inspected.run));
        if dest.exists() {
            return Err(WorkflowError::Invalid(format!(
                "Run {} is already exported to {}.",
                inspected.run,
                dest.display()
            )));
        }
        let partial = self
            .exports_dir()
            .join(format!("{id}-{}.partial", inspected.run));
        let _ = std::fs::remove_dir_all(&partial);
        std::fs::create_dir_all(&partial)?;
        let m = &inspected.report.manifest;
        let mut names: Vec<&str> = vec![MANIFEST_NAME];
        for stage in [&m.logical_planes, &m.physical_planes, &m.screened_planes] {
            names.extend(stage.files.iter().map(|f| f.name.as_str()));
        }
        let copied = (|| -> WorkflowResult<u64> {
            let mut bytes = 0;
            for name in &names {
                bytes += std::fs::copy(source.join(name), partial.join(name))?;
            }
            let again = crate::inspect::inspect_artifacts(&partial)?;
            if !again.verified {
                return Err(WorkflowError::Invalid(format!(
                    "The exported copy did not verify: {}",
                    again.problems.join("; ")
                )));
            }
            std::fs::rename(&partial, &dest)?;
            Ok(bytes)
        })();
        match copied {
            Ok(bytes) => Ok(ExportedRun {
                dir: dest,
                run: inspected.run,
                files: names.len(),
                bytes,
            }),
            Err(e) => {
                let _ = std::fs::remove_dir_all(&partial);
                Err(e)
            }
        }
    }

    /// Every submission attempt of a job at its latest recorded state, oldest first.
    pub fn job_attempts(&self, id: JobId) -> WorkflowResult<Vec<veylora_persistence::AttemptView>> {
        self.jobs.get(id)?;
        Ok(self.jobs.list_attempts(id)?)
    }

    /// The device facts a job was created against, read from its stored setup and mode records.
    pub fn job_device(&self, id: JobId) -> WorkflowResult<JobDevice> {
        let loaded = self.load(id)?;
        Ok(JobDevice {
            setup_id: loaded.setup.id().to_string(),
            setup_physically_verified: loaded.setup.require_production().is_ok(),
            mode_id: loaded.mode.id().to_string(),
            mode_status: loaded.mode.status(),
            printable_um: {
                let r = loaded.mode.raster();
                (
                    u64::from(r.printable_width_px) * 25_400 / u64::from(r.dpi_x),
                    u64::from(r.printable_height_px) * 25_400 / u64::from(r.dpi_y),
                )
            },
            white_capable: loaded
                .mode
                .deposition()
                .coverage_caps
                .iter()
                .any(|(name, cap)| name.eq_ignore_ascii_case("white") && *cap > 0.0),
            channel_map_version: loaded.setup.details().channel_map.version(),
            channels: {
                let details = loaded.setup.details();
                details
                    .channel_map
                    .bindings()
                    .iter()
                    .map(|b| {
                        let slot = details.physical_slots.iter().find(|s| s.id == b.slot);
                        JobChannel {
                            logical: b.logical,
                            slot: b.slot.0,
                            protocol: b.protocol.0,
                            fraction: b.fraction,
                            tank_label: slot.map(|s| s.tank_label.clone()).unwrap_or_default(),
                            head_label: slot.map(|s| s.head_label.clone()).unwrap_or_default(),
                        }
                    })
                    .collect()
            },
        })
    }

    /// The immutable record of what a job was created from.
    pub fn job_snapshot(&self, id: JobId) -> WorkflowResult<veylora_domain::job::JobSnapshot> {
        Ok(self.jobs.get(id)?)
    }

    /// Inspects the newest run that completed (wrote a manifest), from its retained bytes. A run
    /// directory without a manifest never finished and is skipped. `None` when nothing was ever
    /// rendered; a damaged run is returned with its problems, not hidden.
    pub fn inspect_latest_run(&self, id: JobId) -> WorkflowResult<Option<InspectedRun>> {
        self.jobs.get(id)?;
        let mut runs = self.run_names(id)?;
        runs.reverse();
        for run in runs {
            let dir = self.artifacts_dir(id).join(&run);
            if dir.join(MANIFEST_NAME).is_file() {
                let report = crate::inspect::inspect_artifacts(&dir)?;
                return Ok(Some(InspectedRun { run, report }));
            }
        }
        Ok(None)
    }

    fn latest_run_dir(&self, id: JobId) -> WorkflowResult<PathBuf> {
        self.jobs.get(id)?;
        let mut runs = self.run_names(id)?;
        runs.reverse();
        runs.into_iter()
            .map(|r| self.artifacts_dir(id).join(r))
            .find(|d| d.join(MANIFEST_NAME).is_file())
            .ok_or_else(|| WorkflowError::Invalid("The job has no completed render.".into()))
    }

    /// A gray picture of one plane of the newest run, no larger than `max_px` on a side. The file is
    /// checked against the manifest first; a damaged plane is refused rather than drawn.
    pub fn plane_overview(
        &self,
        id: JobId,
        plane: &str,
        max_px: usize,
    ) -> WorkflowResult<crate::plane::PlaneOverview> {
        crate::plane::overview(&self.latest_run_dir(id)?, plane, max_px)
    }

    /// A colour picture of the newest run's logical planes stacked on a fixed backdrop. It is a
    /// simulation of where ink goes (see `plane::composite_color`), not a colour-managed preview.
    pub fn composite_overview(
        &self,
        id: JobId,
        max_px: usize,
    ) -> WorkflowResult<crate::plane::PlaneOverview> {
        crate::plane::composite_overview(&self.latest_run_dir(id)?, max_px)
    }

    /// Exact values of a small rectangle of one plane of the newest run, clipped to the plane.
    pub fn plane_window(
        &self,
        id: JobId,
        plane: &str,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> WorkflowResult<crate::plane::PlaneWindow> {
        crate::plane::window(&self.latest_run_dir(id)?, plane, x, y, width, height)
    }

    pub fn report(&self, id: JobId) -> WorkflowResult<JobReport> {
        let snapshot = self.jobs.get(id)?;
        let inputs = snapshot.inputs();
        let mut renders = Vec::new();
        for run in self.run_names(id)? {
            let path = self.artifacts_dir(id).join(&run).join(MANIFEST_NAME);
            // A run without a manifest never completed; it is not a render.
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            let manifest: RenderManifest = serde_json::from_slice(&bytes)?;
            renders.push(RenderSummary {
                run,
                manifest_hash: ContentHash::digest(&bytes),
                snapshot_hash: manifest.snapshot_hash,
                page_width_px: manifest.page_width_px,
                page_height_px: manifest.page_height_px,
            });
        }
        let attempts = self
            .jobs
            .list_attempts(id)?
            .into_iter()
            .map(|a| AttemptSummary {
                attempt_id: a.attempt_id.to_string(),
                sequence: a.sequence,
                state: format!("{:?}", a.state),
                note: a.note,
            })
            .collect();
        Ok(JobReport {
            job_id: id,
            snapshot_hash: snapshot.content_hash(),
            created_at_ms: inputs.created_at_ms,
            setup: inputs.device_setup.content_hash,
            mode: inputs.print_mode.content_hash,
            recipe: inputs.recipe.content_hash(),
            assets: inputs.assets.clone(),
            masks: inputs.masks.clone(),
            layout_hash: inputs.layout.content_hash(),
            renders,
            submission_attempts: attempts,
        })
    }
}
