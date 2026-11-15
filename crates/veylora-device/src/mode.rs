//! Versioned, immutable print-mode record and compatibility checks.
//!
//! A `PrintMode` is a frozen bundle of raster geometry, protocol shape, media
//! settings, deposition targets, and quality targets. It is immutable: any edit
//! that would change hashed bytes must produce a new mode revision. Only backend
//! capability data authorizes a mode to be exposed in UI; only physical evidence
//! authorizes transitioning `Experimental -> Validated`.
//!
//! No Epson-specific numeric defaults live in this file. All values originate from
//! configuration input or test fixtures, not from guesses.

use crate::{
    capability::{Capability, ControlId, DeviceCapability},
    channels::invalid,
    setup::DeviceSetup,
};
use veylora_domain::{
    ids::{DeviceSetupId, PrintModeId},
    preflight::Finding,
    provenance::ContentHash,
    recipe::{PrintRecipe, WhiteSource},
};

/// Raster geometry: grid density and printable-box extent. Values are recorded,
/// not interpreted by this crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RasterSpec {
    pub dpi_x: u32,
    pub dpi_y: u32,
    pub printable_width_px: u32,
    pub printable_height_px: u32,
    pub band_height_px: u32,
}

/// Protocol shape negotiated with the backend for this mode. Sequence is the
/// ordered list of protocol-channel addresses the backend expects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtocolSpec {
    pub label: String,
    pub channel_sequence: Vec<u8>,
    /// Required firmware fingerprint, recorded verbatim from physical observation.
    pub firmware_fingerprint: String,
}

/// Media and feed settings this mode requires. Strings are recorded literally.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaSpec {
    pub media_label: String,
    pub feed_path: String,
    pub film_fingerprint: String,
}

/// Target ink deposition: pass structure and per-channel coverage caps.
#[derive(Clone, Debug, PartialEq)]
pub struct DepositionSpec {
    pub passes: u32,
    pub overlap_percent: u16,
    /// Per-logical-channel coverage cap, 0.0..=1.0. Values outside that range are rejected.
    pub coverage_caps: Vec<(String, f64)>,
}

/// Target quality parameters. Numbers are recorded verbatim; validators live elsewhere.
#[derive(Clone, Debug, PartialEq)]
pub struct QualitySpec {
    pub target_label: String,
    pub notes: String,
    pub tolerance_percent: f64,
}

/// Lifecycle status for a print mode. Only physical evidence may transition to `Validated`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeStatus {
    Experimental,
    Validated,
    Retired,
}

/// Immutable mode record. Construct through [`PrintMode::new`] or [`PrintMode::revise`].
#[derive(Clone, Debug)]
pub struct PrintMode {
    id: PrintModeId,
    revision: u32,
    device_setup: DeviceSetupId,
    raster: RasterSpec,
    protocol: ProtocolSpec,
    media: MediaSpec,
    deposition: DepositionSpec,
    quality: QualitySpec,
    status: ModeStatus,
    content_hash: ContentHash,
}

impl PrintMode {
    pub fn new(
        id: PrintModeId,
        device_setup: DeviceSetupId,
        raster: RasterSpec,
        protocol: ProtocolSpec,
        media: MediaSpec,
        deposition: DepositionSpec,
        quality: QualitySpec,
    ) -> Result<Self, Finding> {
        validate_raster(&raster)?;
        validate_protocol(&protocol)?;
        validate_media(&media)?;
        validate_deposition(&deposition)?;
        validate_quality(&quality)?;
        let content_hash = hash_mode(
            id,
            1,
            device_setup,
            &raster,
            &protocol,
            &media,
            &deposition,
            &quality,
            ModeStatus::Experimental,
        );
        Ok(Self {
            id,
            revision: 1,
            device_setup,
            raster,
            protocol,
            media,
            deposition,
            quality,
            status: ModeStatus::Experimental,
            content_hash,
        })
    }

    pub fn id(&self) -> PrintModeId {
        self.id
    }
    pub fn revision(&self) -> u32 {
        self.revision
    }
    pub fn device_setup(&self) -> DeviceSetupId {
        self.device_setup
    }
    pub fn raster(&self) -> &RasterSpec {
        &self.raster
    }
    pub fn protocol(&self) -> &ProtocolSpec {
        &self.protocol
    }
    pub fn media(&self) -> &MediaSpec {
        &self.media
    }
    pub fn deposition(&self) -> &DepositionSpec {
        &self.deposition
    }
    pub fn quality(&self) -> &QualitySpec {
        &self.quality
    }
    pub fn status(&self) -> ModeStatus {
        self.status
    }
    pub fn content_hash(&self) -> ContentHash {
        self.content_hash
    }

    /// Promote an experimental mode to validated. This is only called after physical
    /// evidence has been recorded elsewhere; it does not itself authorize submission.
    pub fn mark_validated(&mut self) -> Result<(), Finding> {
        if matches!(self.status, ModeStatus::Retired) {
            return Err(invalid("Retired modes cannot be revalidated."));
        }
        self.status = ModeStatus::Validated;
        self.content_hash = hash_mode(
            self.id,
            self.revision,
            self.device_setup,
            &self.raster,
            &self.protocol,
            &self.media,
            &self.deposition,
            &self.quality,
            self.status,
        );
        Ok(())
    }

    /// Any change producing different hashed bytes must create a new revision that
    /// starts from `Experimental` again.
    #[allow(clippy::too_many_arguments)]
    pub fn revise(
        &self,
        raster: RasterSpec,
        protocol: ProtocolSpec,
        media: MediaSpec,
        deposition: DepositionSpec,
        quality: QualitySpec,
    ) -> Result<Self, Finding> {
        validate_raster(&raster)?;
        validate_protocol(&protocol)?;
        validate_media(&media)?;
        validate_deposition(&deposition)?;
        validate_quality(&quality)?;
        let revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("Mode revision overflow."))?;
        let content_hash = hash_mode(
            self.id,
            revision,
            self.device_setup,
            &raster,
            &protocol,
            &media,
            &deposition,
            &quality,
            ModeStatus::Experimental,
        );
        Ok(Self {
            id: self.id,
            revision,
            device_setup: self.device_setup,
            raster,
            protocol,
            media,
            deposition,
            quality,
            status: ModeStatus::Experimental,
            content_hash,
        })
    }
}

fn validate_raster(r: &RasterSpec) -> Result<(), Finding> {
    if r.dpi_x == 0
        || r.dpi_y == 0
        || r.printable_width_px == 0
        || r.printable_height_px == 0
        || r.band_height_px == 0
        || r.band_height_px > r.printable_height_px
    {
        return Err(invalid(
            "Raster spec requires positive DPI, printable extent, and a band height within the page.",
        ));
    }
    Ok(())
}

fn validate_protocol(p: &ProtocolSpec) -> Result<(), Finding> {
    if p.label.trim().is_empty()
        || p.firmware_fingerprint.trim().is_empty()
        || p.channel_sequence.is_empty()
    {
        return Err(invalid(
            "Protocol spec requires a label, firmware fingerprint, and non-empty channel sequence.",
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for channel in &p.channel_sequence {
        if !seen.insert(*channel) {
            return Err(invalid(
                "Protocol channel sequence must not contain duplicates.",
            ));
        }
    }
    Ok(())
}

fn validate_media(m: &MediaSpec) -> Result<(), Finding> {
    if m.media_label.trim().is_empty()
        || m.feed_path.trim().is_empty()
        || m.film_fingerprint.trim().is_empty()
    {
        return Err(invalid(
            "Media spec requires label, feed path, and film fingerprint.",
        ));
    }
    Ok(())
}

fn validate_deposition(d: &DepositionSpec) -> Result<(), Finding> {
    if d.passes == 0 || d.overlap_percent > 100 || d.coverage_caps.is_empty() {
        return Err(invalid(
            "Deposition spec requires at least one pass, overlap <= 100%, and non-empty coverage caps.",
        ));
    }
    let mut labels = std::collections::BTreeSet::new();
    for (label, cap) in &d.coverage_caps {
        if label.trim().is_empty() || !labels.insert(label.clone()) {
            return Err(invalid("Coverage cap labels must be unique and non-empty."));
        }
        if !cap.is_finite() || !(0.0..=1.0).contains(cap) {
            return Err(invalid("Coverage caps must be finite and in [0, 1]."));
        }
    }
    Ok(())
}

fn validate_quality(q: &QualitySpec) -> Result<(), Finding> {
    if q.target_label.trim().is_empty()
        || !q.tolerance_percent.is_finite()
        || q.tolerance_percent < 0.0
    {
        return Err(invalid(
            "Quality spec requires a non-empty target label and a finite non-negative tolerance.",
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn hash_mode(
    id: PrintModeId,
    revision: u32,
    device_setup: DeviceSetupId,
    raster: &RasterSpec,
    protocol: &ProtocolSpec,
    media: &MediaSpec,
    deposition: &DepositionSpec,
    quality: &QualitySpec,
    status: ModeStatus,
) -> ContentHash {
    let mut bytes = b"veylora-print-mode-v1\0".to_vec();
    bytes.extend_from_slice(&revision.to_le_bytes());
    push_str(&mut bytes, &id.to_string());
    push_str(&mut bytes, &device_setup.to_string());
    for value in [
        raster.dpi_x,
        raster.dpi_y,
        raster.printable_width_px,
        raster.printable_height_px,
        raster.band_height_px,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    push_str(&mut bytes, &protocol.label);
    push_str(&mut bytes, &protocol.firmware_fingerprint);
    bytes.extend_from_slice(&(protocol.channel_sequence.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&protocol.channel_sequence);
    push_str(&mut bytes, &media.media_label);
    push_str(&mut bytes, &media.feed_path);
    push_str(&mut bytes, &media.film_fingerprint);
    bytes.extend_from_slice(&deposition.passes.to_le_bytes());
    bytes.extend_from_slice(&deposition.overlap_percent.to_le_bytes());
    bytes.extend_from_slice(&(deposition.coverage_caps.len() as u64).to_le_bytes());
    for (label, cap) in &deposition.coverage_caps {
        push_str(&mut bytes, label);
        bytes.extend_from_slice(&cap.to_le_bytes());
    }
    push_str(&mut bytes, &quality.target_label);
    push_str(&mut bytes, &quality.notes);
    bytes.extend_from_slice(&quality.tolerance_percent.to_le_bytes());
    bytes.push(status as u8);
    ContentHash::digest(&bytes)
}

fn push_str(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(&(value.len() as u64).to_le_bytes());
    out.extend_from_slice(value.as_bytes());
}

/// Compatibility report. Severity on each `Finding` describes the required action.
pub fn check_compatibility(
    mode: &PrintMode,
    setup: &DeviceSetup,
    capability: &DeviceCapability,
    recipe: &PrintRecipe,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if mode.device_setup() != setup.id() {
        findings.push(invalid(
            "Print mode is bound to a different device setup and cannot be used with this one.",
        ));
    }
    if setup.require_production().is_err() && matches!(mode.status(), ModeStatus::Validated) {
        findings.push(invalid(
            "A validated mode requires a physically verified device setup.",
        ));
    }
    if matches!(mode.status(), ModeStatus::Retired) {
        findings.push(invalid("Retired modes may not be used for new jobs."));
    }
    if matches!(capability.independent_white, Capability::Unsupported) {
        if mode
            .deposition()
            .coverage_caps
            .iter()
            .any(|(label, _)| label.eq_ignore_ascii_case("white"))
        {
            findings.push(invalid(
                "Backend does not offer independent white but the mode declares a white coverage cap.",
            ));
        }
        if !matches!(recipe.inputs().white.source, WhiteSource::Disabled) {
            findings.push(invalid(
                "Backend does not offer independent white but the recipe prints white.",
            ));
        }
    }
    for (label, _) in &mode.deposition().coverage_caps {
        if !label.eq_ignore_ascii_case("white")
            && !capability
                .process_channels
                .iter()
                .any(|c| c.eq_ignore_ascii_case(label))
        {
            findings.push(invalid(
                "Backend capability does not expose a required process channel.",
            ));
        }
    }
    // Firmware fingerprint recorded on the mode must exactly match the setup's
    // recorded firmware when the setup has recorded one.
    if let Some(firmware) = setup.details().firmware.as_deref() {
        if firmware != mode.protocol().firmware_fingerprint {
            findings.push(invalid(
                "Mode firmware fingerprint does not match the device setup firmware observation.",
            ));
        }
    }
    // Media/feed path must match the setup's recorded feed path.
    if setup.details().feed_path != mode.media().feed_path {
        findings.push(invalid(
            "Mode feed path does not match the recorded device setup feed path.",
        ));
    }
    // A validated mode must not be driven by a recipe with an unversioned calibration.
    if matches!(mode.status(), ModeStatus::Validated) && recipe.inputs().calibration.revision == 0 {
        findings.push(invalid(
            "A validated mode requires a versioned calibration on the recipe.",
        ));
    }
    findings
}

/// UI-side gate: a control is permitted only when the backend declares it supported.
pub fn control_offered(capability: &DeviceCapability, control: &ControlId) -> bool {
    capability.offers(control)
}
