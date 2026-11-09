//! Frozen input graph, independent of rendering, storage and device implementations.
use crate::{
    errors::ValidationError,
    ids::{AssetId, DeviceSetupId, JobId, PrintModeId},
    layout::LayoutSnapshot,
    provenance::{ContentHash, Provenance},
    recipe::{record_hash, PrintRecipe, VersionedReference},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetReference {
    pub id: AssetId,
    pub content_hash: ContentHash,
}

/// Construction data. All referenced bytes must be retained and hash-checked by
/// the application before use. Reference existence and device compatibility are
/// preflight responsibilities, not claims made by this structural contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobSnapshotInput {
    pub schema_version: u32,
    pub id: JobId,
    /// UTC Unix milliseconds, explicitly supplied and retained.
    pub created_at_ms: u64,
    pub assets: Vec<AssetReference>,
    pub masks: Vec<AssetReference>,
    /// Complete layout record including placement identity and resolved transforms.
    pub layout: LayoutSnapshot,
    pub device_setup_id: DeviceSetupId,
    pub device_setup: VersionedReference,
    pub print_mode_id: PrintModeId,
    pub print_mode: VersionedReference,
    pub recipe: PrintRecipe,
    pub provenance: Provenance,
}

/// Immutable from construction, which is stronger than freezing only at render.
/// Edits require constructing a new snapshot, never modifying the retained one.
///
/// ```compile_fail
/// use veylora_domain::job::JobSnapshot;
/// fn mutate(snapshot: &mut JobSnapshot) {
///     snapshot.inputs().assets.clear();
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "JobSnapshotInput", into = "JobSnapshotInput")]
pub struct JobSnapshot(JobSnapshotInput);

impl JobSnapshot {
    pub fn new(input: JobSnapshotInput) -> Result<Self, ValidationError> {
        if input.schema_version != 1 {
            return Err(ValidationError::invalid(
                "job.schema_version",
                "Only job schema version 1 is supported.",
            ));
        }
        if input.assets.is_empty() {
            return Err(ValidationError::invalid(
                "job.assets",
                "Supply at least one source asset.",
            ));
        }
        if input.created_at_ms == 0 {
            return Err(ValidationError::invalid(
                "job.created_at_ms",
                "Supply a positive UTC Unix timestamp.",
            ));
        }
        let mut asset_ids = std::collections::BTreeSet::new();
        for asset in input.assets.iter().chain(input.masks.iter()) {
            if !asset_ids.insert(asset.id) {
                return Err(ValidationError::invalid(
                    "job.assets",
                    "Asset and mask identities must be unique within a snapshot.",
                ));
            }
        }
        let sources: std::collections::BTreeSet<_> = input.assets.iter().map(|a| a.id).collect();
        let masks: std::collections::BTreeSet<_> = input.masks.iter().map(|a| a.id).collect();
        for placement in input.layout.placements() {
            if !sources.contains(&placement.asset_id)
                || placement.mask_id.is_some_and(|id| !masks.contains(&id))
            {
                return Err(ValidationError::invalid(
                    "job.layout",
                    "Restore the source or mask referenced by each placement.",
                ));
            }
        }
        if let crate::recipe::WhiteSource::Supplied { mask_id } =
            &input.recipe.inputs().white.source
        {
            if !masks.contains(mask_id) {
                return Err(ValidationError::invalid(
                    "job.recipe.white",
                    "Restore the supplied white mask.",
                ));
            }
        }
        let mut labels = std::collections::BTreeSet::new();
        for (label, _) in &input.provenance.content_hashes {
            if label.trim().is_empty() || !labels.insert(label) {
                return Err(ValidationError::invalid(
                    "job.provenance",
                    "Use nonempty unique provenance labels.",
                ));
            }
        }
        for (subject, reference) in [
            ("job.device_setup", &input.device_setup),
            ("job.print_mode", &input.print_mode),
        ] {
            reference.validate(subject)?;
        }
        if input.provenance.software_version.trim().is_empty()
            || input.provenance.backend_version.trim().is_empty()
        {
            return Err(ValidationError::invalid(
                "job.provenance",
                "Record nonempty software and backend versions.",
            ));
        }
        Ok(Self(input))
    }

    pub fn inputs(&self) -> &JobSnapshotInput {
        &self.0
    }

    pub fn job_id(&self) -> JobId {
        self.0.id
    }

    /// Uses the same versioned compact-JSON contract as PrintRecipe. Lifecycle
    /// state and later artifacts are deliberately not part of the input hash.
    pub fn content_hash(&self) -> ContentHash {
        record_hash(b"veylora-job-snapshot-v1\0", &self.0)
    }

    /// Freezes ownership for a render. This does not authorize native output or
    /// replace application preflight/lifecycle checks. Validation already occurred
    /// at construction, so schema 1 has no remaining fallible render-freeze step.
    pub fn begin_render(self) -> Result<RenderingSnapshot, SnapshotError> {
        Ok(RenderingSnapshot(self))
    }
}

impl TryFrom<JobSnapshotInput> for JobSnapshot {
    type Error = ValidationError;
    fn try_from(input: JobSnapshotInput) -> Result<Self, Self::Error> {
        Self::new(input)
    }
}
impl From<JobSnapshot> for JobSnapshotInput {
    fn from(snapshot: JobSnapshot) -> Self {
        snapshot.0
    }
}

/// Read-only render input. No mutable accessor or in-place update exists.
///
/// ```compile_fail
/// use veylora_domain::job::RenderingSnapshot;
/// fn mutate(rendering: &mut RenderingSnapshot) {
///     rendering.snapshot().inputs().masks.clear();
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RenderingSnapshot(JobSnapshot);

impl RenderingSnapshot {
    pub fn snapshot(&self) -> &JobSnapshot {
        &self.0
    }
    pub fn content_hash(&self) -> ContentHash {
        self.0.content_hash()
    }
}

/// Schema 1 makes invalid freeze operations unrepresentable. Kept as the error
/// type of the planned fallible boundary without inventing a failure condition.
pub type SnapshotError = std::convert::Infallible;

/// Recorded transport state, not proof of physical output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptState {
    Submitting,
    Submitted,
    Ambiguous,
    Failed,
    Cancelled,
    OperatorConfirmedFinished,
}

impl AttemptState {
    pub const fn is_terminal(self) -> bool {
        !matches!(self, Self::Submitting)
    }

    pub const fn is_ambiguous(self) -> bool {
        matches!(self, Self::Ambiguous)
    }
}

/// An event retained by the append-only attempt store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmissionAttempt {
    pub attempt_id: crate::ids::AttemptId,
    pub job_id: JobId,
    pub snapshot_hash: ContentHash,
    pub sequence: u32,
    pub state: AttemptState,
    pub bytes_written: u64,
    pub note: String,
    pub recorded_at: String,
}
