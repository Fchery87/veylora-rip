//! Bounded, cancellable, deterministic band rendering from a job snapshot to retained stage
//! artifacts: logical planes, calibrated physical planes and screened device rasters.
#![forbid(unsafe_code)]

mod artifacts;
mod pipeline;
mod process;

pub use artifacts::{
    ArtifactError, ArtifactFile, ArtifactSink, ArtifactStore, DirectoryStore, SampleFormat, Stage,
    StageArtifact,
};
pub use pipeline::{
    render, CancellationToken, RenderArtifacts, RenderError, RenderLimits, RenderManifest,
    RenderRequest, RenderStats, MANIFEST_NAME,
};
pub use process::{ProcessBand, ProcessSeparator, SeparatorError, WhiteOnly};
