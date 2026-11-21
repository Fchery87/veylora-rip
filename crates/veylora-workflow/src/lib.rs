//! Headless application services. The CLI and, later, the desktop shell call these; neither owns
//! rendering, persistence or validation logic.
#![forbid(unsafe_code)]

mod archive;
mod error;
mod inspect;
mod jobflow;
mod lifecycle;
mod spec;
mod support;
mod update;
mod workspace;

pub use archive::{ImportedJob, ARCHIVE_FORMAT, ARCHIVE_SCHEMA_VERSION};
pub use error::{WorkflowError, WorkflowResult};
pub use inspect::{inspect_artifacts, inspect_asset, ArtifactReport, AssetReport, PlaneStats};
pub use jobflow::{RecoveryReport, ReviewResolution, SubmitOutcome};
pub use lifecycle::{AttemptReview, ReviewReport, SetupDraft, SetupStatus};
pub use spec::{CropSpec, JobSpec, LayoutSpec, PlacementSpec};
pub use support::SupportBundleOptions;
pub use update::UpdateBlocker;
pub use workspace::{
    asset_id_for, AttemptSummary, JobCreated, JobReport, PreflightOptions, PreflightReport,
    RenderOptions, RenderOutcome, RenderSummary, Workspace, DIAGNOSTIC_BACKEND,
};
