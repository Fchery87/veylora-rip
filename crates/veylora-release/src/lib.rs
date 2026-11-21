//! Release manifests and side-by-side installs with a journaled switch, rollback and recovery.
//!
//! Nothing here runs the network or an installer. It decides, from files on disk, whether a
//! release is intact, whether an update may start, and how to get back to a known-good release.
#![forbid(unsafe_code)]

mod error;
mod install;
mod manifest;
mod version;

pub use error::ReleaseError;
pub use install::{
    Blocker, InstallOutcome, InstallState, InstallStore, PendingUpdate, RecoveryReport,
    UpdateGuard, UpdateRequest,
};
pub use manifest::{
    build_manifest, verify_release, write_manifest, ReleaseFile, ReleaseManifest, MANIFEST_NAME,
    MANIFEST_SCHEMA, PRODUCT,
};
pub use version::Version;
