//! Schema migrations for the local durable store.
//!
//! The store carries a monotonic `u32` schema version. Migrations declare the
//! `from` version they upgrade and the `to` version they produce; the runner
//! replays them in order until the store reaches the current version. The
//! migration set is open for extension but never edits in place: once a
//! migration has shipped it must not change.

use crate::errors::{PersistenceError, PersistenceResult};
use std::path::Path;

/// Current schema version understood by this build.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Current schema version. Exposed as a function for forward-compatible call
/// sites that must not depend on the constant being evaluated at compile time.
pub const fn current_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

/// A single schema migration. Migrations are pure state transitions and must be
/// idempotent when re-run from the same starting version.
pub trait Migration: Send + Sync {
    #[allow(clippy::wrong_self_convention)]
    fn from_version(&self) -> u32;
    fn to_version(&self) -> u32;
    fn apply(&self, root: &Path) -> PersistenceResult<()>;
}

/// Ordered, append-only migration set. Insertion rejects duplicates and gaps so
/// a corrupted release cannot accidentally leap over a version.
pub struct MigrationSet {
    migrations: Vec<Box<dyn Migration>>,
}

impl MigrationSet {
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    pub fn register(&mut self, migration: Box<dyn Migration>) -> PersistenceResult<()> {
        let expected_from = self.migrations.last().map(|m| m.to_version()).unwrap_or(0);
        if migration.from_version() != expected_from {
            return Err(PersistenceError::Corrupt {
                detail: format!(
                    "migration from {} does not follow previous to-version {expected_from}",
                    migration.from_version()
                ),
            });
        }
        if migration.to_version() != migration.from_version() + 1 {
            return Err(PersistenceError::Corrupt {
                detail: "migrations must advance exactly one version at a time".into(),
            });
        }
        self.migrations.push(migration);
        Ok(())
    }

    pub fn target_version(&self) -> u32 {
        self.migrations
            .last()
            .map(|m| m.to_version())
            .unwrap_or(CURRENT_SCHEMA_VERSION)
    }

    pub fn run(&self, root: &Path, mut current: u32) -> PersistenceResult<u32> {
        for migration in &self.migrations {
            if migration.from_version() < current {
                continue;
            }
            if migration.from_version() != current {
                return Err(PersistenceError::Corrupt {
                    detail: format!(
                        "migration expected from-version {} but store is at {current}",
                        migration.from_version()
                    ),
                });
            }
            migration.apply(root)?;
            current = migration.to_version();
        }
        Ok(current)
    }
}

impl Default for MigrationSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Migration from version 0 (empty directory) to version 1 (jobs + attempts).
pub(crate) struct InitialSchema;

impl Migration for InitialSchema {
    fn from_version(&self) -> u32 {
        0
    }
    fn to_version(&self) -> u32 {
        1
    }
    fn apply(&self, root: &Path) -> PersistenceResult<()> {
        let jobs_dir = root.join("jobs");
        std::fs::create_dir_all(&jobs_dir)?;
        let attempts = root.join("attempts.log");
        if !attempts.exists() {
            crate::jobs::create_empty_log(&attempts)?;
        }
        Ok(())
    }
}

/// The migration set the current build ships with.
pub fn shipped_migrations() -> PersistenceResult<MigrationSet> {
    let mut set = MigrationSet::new();
    set.register(Box::new(InitialSchema))?;
    Ok(set)
}
