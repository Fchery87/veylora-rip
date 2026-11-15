//! Headless application services. The CLI and, later, the desktop shell call these; neither owns
//! rendering, persistence or validation logic.
#![forbid(unsafe_code)]

mod error;
mod inspect;
mod spec;
mod workspace;

pub use error::{WorkflowError, WorkflowResult};
pub use inspect::{inspect_artifacts, inspect_asset, ArtifactReport, AssetReport, PlaneStats};
pub use spec::{CropSpec, JobSpec, LayoutSpec, PlacementSpec};
pub use workspace::{
    asset_id_for, AttemptSummary, JobCreated, JobReport, PreflightOptions, PreflightReport,
    RenderOptions, RenderOutcome, RenderSummary, Workspace, DIAGNOSTIC_BACKEND,
};
