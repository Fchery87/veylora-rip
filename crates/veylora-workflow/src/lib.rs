//! Headless application services. The CLI and, later, the desktop shell call these; neither owns
//! rendering, persistence or validation logic.
#![forbid(unsafe_code)]

mod archive;
mod derive;
mod error;
mod inspect;
mod jobflow;
mod lifecycle;
mod lock;
mod plane;
mod recipe_validation;
mod spec;
mod support;
mod update;
mod workspace;

pub use archive::{ImportedJob, ARCHIVE_FORMAT, ARCHIVE_SCHEMA_VERSION};
pub use derive::{JobParent, WhiteEdit};
pub use error::{WorkflowError, WorkflowResult};
pub use inspect::{inspect_artifacts, inspect_asset, ArtifactReport, AssetReport, PlaneStats};
pub use jobflow::{RecoveryReport, ReviewResolution, SubmitOutcome};
pub use lifecycle::{AttemptReview, ReviewReport, SetupDraft, SetupStatus};
pub use lock::JobLock;
pub use plane::{PlaneOverview, PlaneWindow, MAX_OVERVIEW_PX, MAX_WINDOW_PX};
pub use recipe_validation::{RecipeValidationRequest, ValidatedRecipe};
pub use spec::{CropSpec, JobSpec, LayoutSpec, PlacementSpec};
pub use support::SupportBundleOptions;
pub use update::UpdateBlocker;
pub use veylora_persistence::AttemptView;
pub use workspace::{
    asset_id_for, AttemptSummary, Catalog, CatalogMode, CatalogRecipe, CatalogSetup, ExportedRun,
    InspectedRun, JobChannel, JobCreated, JobDevice, JobReport, PreflightOptions, PreflightReport,
    RenderOptions, RenderOutcome, RenderSummary, Workspace, DIAGNOSTIC_BACKEND,
};
