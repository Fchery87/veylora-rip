//! Narrowly scoped durable store for immutable job snapshots and append-only
//! submission attempts.
//!
//! The on-disk layout is deliberately boring:
//!
//! - `<root>/VERSION`            current schema version, written atomically.
//! - `<root>/jobs/<job_id>.json` one immutable snapshot per job, write-once.
//! - `<root>/attempts.log`       append-only JSON-Lines event log.
//!
//! Every mutation `fsync`s both the file and the parent directory so a crash,
//! power loss, or process exit never leaves a torn record. The store exposes
//! only read-and-append operations; it never rewrites or deletes history.
//!
//! The implementation is SQLite-style in spirit (strict schema, migrations,
//! durable writes, no auto-retry) without pulling in a native dependency this
//! phase of the project has not accepted yet.
#![forbid(unsafe_code)]

pub mod calibration;
pub mod devices;
pub mod errors;
pub mod jobs;
pub mod migrations;
pub mod records;

pub use calibration::{CalibrationStore, StoreError};
pub use devices::FileDeviceStore;
pub use errors::{PersistenceError, PersistenceResult};
pub use jobs::{AttemptView, FileJobStore, JobStore};
pub use migrations::{current_schema_version, Migration, MigrationSet};
pub use records::FileRecordStore;
