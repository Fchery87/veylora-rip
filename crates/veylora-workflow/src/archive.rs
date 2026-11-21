//! Portable job archives: everything that defines a job, and its recorded history, in one file
//! that can move to another workspace or machine.
//!
//! The archive holds no paths, so it does not matter where it was made. It carries the job
//! snapshot, the device setup / print mode / recipe records the snapshot names, the source images
//! and masks, and the append-only lifecycle and submission history. It does not carry rendered
//! output (rendering is deterministic and is repeated) or device verification (that is evidence
//! about one physical machine; an imported setup starts unverified).
//!
//! History travels with the job on purpose. Import replays it through the same stores and state
//! machine, so a job that was submitted, or whose submission is ambiguous, is still that on the
//! other side and cannot be printed a second time by moving it.
//!
//! Import checks everything before it writes anything.

use crate::{
    error::{WorkflowError, WorkflowResult},
    workspace::Workspace,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use veylora_device::{
    mode::{ModeRecord, PrintMode},
    setup::{DeviceSetup, SetupRecord, VerificationState},
};
use veylora_diagnostics::bundle::{read_tar, write_tar, BundleEntry};
use veylora_domain::{
    ids::JobId,
    job::{JobSnapshot, SubmissionAttempt},
    lifecycle::{transition, JobState},
    provenance::ContentHash,
    recipe::PrintRecipe,
};
use veylora_persistence::{JobStore, LifecycleRecord, PersistenceError};

pub const ARCHIVE_FORMAT: &str = "veylora-job-archive";
/// The newest archive layout this build reads and writes.
pub const ARCHIVE_SCHEMA_VERSION: u32 = 1;

const MANIFEST: &str = "archive.json";
const SNAPSHOT: &str = "job/snapshot.json";
const SETUP: &str = "records/setup.json";
const MODE: &str = "records/mode.json";
const RECIPE: &str = "records/recipe.json";
const LIFECYCLE: &str = "history/lifecycle.json";
const ATTEMPTS: &str = "history/attempts.json";
const SOURCE_DIR: &str = "sources/";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Listed {
    path: String,
    bytes: u64,
    sha256: ContentHash,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArchiveManifest {
    format: String,
    schema_version: u32,
    job_id: JobId,
    snapshot_hash: ContentHash,
    software_version: String,
    files: Vec<Listed>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportedJob {
    pub job_id: JobId,
    pub snapshot_hash: ContentHash,
    pub state: JobState,
    pub lifecycle_events: usize,
    pub attempt_records: usize,
    /// Always false: verification is evidence about one machine and is never carried.
    pub setup_verification_carried: bool,
}

/// Verification is evidence about one physical machine, so it never travels: the setup's content
/// hash does not depend on it, and the imported setup has to be verified again where it lands.
fn unverified(mut record: SetupRecord) -> SetupRecord {
    record.verification = VerificationState::Unverified;
    record.evidence = None;
    record
}

fn bad(why: impl std::fmt::Display) -> WorkflowError {
    WorkflowError::Invalid(format!("not a usable job archive: {why}"))
}

fn json<T: Serialize>(value: &T) -> WorkflowResult<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

struct Files(BTreeMap<String, Vec<u8>>);

impl Files {
    fn take(&mut self, path: &str) -> WorkflowResult<Vec<u8>> {
        self.0
            .remove(path)
            .ok_or_else(|| bad(format!("{path} is missing")))
    }
    fn parse<T: serde::de::DeserializeOwned>(&mut self, path: &str) -> WorkflowResult<T> {
        let bytes = self.take(path)?;
        serde_json::from_slice(&bytes).map_err(|e| bad(format!("{path}: {e}")))
    }
}

impl Workspace {
    /// Builds the archive in memory. Refuses a job whose source files are not all present: an
    /// archive that could not be imported complete is not worth making.
    pub fn export_job_archive(&self, id: JobId) -> WorkflowResult<Vec<u8>> {
        let loaded = self.load(id)?;
        let inputs = loaded.snapshot.inputs();
        let mut entries: Vec<BundleEntry> = vec![
            BundleEntry {
                path: SNAPSHOT.into(),
                bytes: json(&loaded.snapshot)?,
            },
            BundleEntry {
                path: SETUP.into(),
                bytes: json(&unverified(loaded.setup.to_record()))?,
            },
            BundleEntry {
                path: MODE.into(),
                bytes: json(&loaded.mode.to_record())?,
            },
            BundleEntry {
                path: RECIPE.into(),
                bytes: json(&inputs.recipe)?,
            },
            BundleEntry {
                path: LIFECYCLE.into(),
                bytes: json(&self.lifecycle.events(id)?)?,
            },
            BundleEntry {
                path: ATTEMPTS.into(),
                bytes: json(&self.jobs.attempt_history(id)?)?,
            },
        ];
        let sources: BTreeSet<ContentHash> = inputs
            .assets
            .iter()
            .chain(inputs.masks.iter())
            .map(|r| r.content_hash)
            .collect();
        for hash in sources {
            let bytes = self.assets.get(&hash).map_err(|e| match e {
                PersistenceError::NotFound { .. } => WorkflowError::Invalid(format!(
                    "source {hash} is missing from the workspace; the job cannot be archived"
                )),
                other => other.into(),
            })?;
            entries.push(BundleEntry {
                path: format!("{SOURCE_DIR}{hash}"),
                bytes,
            });
        }
        let mut files: Vec<Listed> = entries
            .iter()
            .map(|e| Listed {
                path: e.path.clone(),
                bytes: e.bytes.len() as u64,
                sha256: ContentHash::digest(&e.bytes),
            })
            .collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        entries.push(BundleEntry {
            path: MANIFEST.into(),
            bytes: json(&ArchiveManifest {
                format: ARCHIVE_FORMAT.into(),
                schema_version: ARCHIVE_SCHEMA_VERSION,
                job_id: id,
                snapshot_hash: loaded.snapshot.content_hash(),
                software_version: veylora_domain::version().to_string(),
                files,
            })?,
        });
        write_tar(&entries).map_err(|e| WorkflowError::Invalid(e.to_string()))
    }

    /// Verifies the whole archive, then writes it into this workspace. A job with the same id
    /// that already exists here is never overwritten or merged.
    pub fn import_job_archive(&self, bytes: &[u8]) -> WorkflowResult<ImportedJob> {
        let entries = read_tar(bytes).map_err(bad)?;
        let mut files = Files(entries.into_iter().map(|e| (e.path, e.bytes)).collect());
        let manifest_bytes = files.take(MANIFEST)?;
        let head: Value =
            serde_json::from_slice(&manifest_bytes).map_err(|e| bad(format!("{MANIFEST}: {e}")))?;
        if head.get("format").and_then(Value::as_str) != Some(ARCHIVE_FORMAT) {
            return Err(bad("it is not a Veylora job archive"));
        }
        // New layouts get a case here that migrates them to the current one; until one exists,
        // anything but the current version is refused rather than guessed at.
        match head.get("schema_version").and_then(Value::as_u64) {
            Some(1) => {}
            Some(found) => {
                return Err(WorkflowError::ArchiveVersion {
                    found: u32::try_from(found).unwrap_or(u32::MAX),
                    supported: ARCHIVE_SCHEMA_VERSION,
                })
            }
            None => return Err(bad("the schema version is missing")),
        }
        let manifest: ArchiveManifest =
            serde_json::from_slice(&manifest_bytes).map_err(|e| bad(format!("{MANIFEST}: {e}")))?;

        // Every file is listed with its size and hash, and nothing else is present.
        let listed: BTreeSet<&str> = manifest.files.iter().map(|f| f.path.as_str()).collect();
        if listed.len() != manifest.files.len() {
            return Err(bad("a file is listed twice"));
        }
        let present: BTreeSet<&str> = files.0.keys().map(String::as_str).collect();
        if listed != present {
            return Err(bad("the file list does not match the archive contents"));
        }
        for f in &manifest.files {
            let content = &files.0[&f.path];
            if content.len() as u64 != f.bytes || ContentHash::digest(content) != f.sha256 {
                return Err(bad(format!("{} does not match its checksum", f.path)));
            }
        }

        let snapshot: JobSnapshot = files.parse(SNAPSHOT)?;
        let inputs = snapshot.inputs();
        let id = snapshot.job_id();
        if id != manifest.job_id || snapshot.content_hash() != manifest.snapshot_hash {
            return Err(bad("the job snapshot is not the one the archive describes"));
        }
        let setup = DeviceSetup::from_record(unverified(files.parse::<SetupRecord>(SETUP)?))
            .map_err(|f| bad(f.message))?;
        let mode =
            PrintMode::from_record(files.parse::<ModeRecord>(MODE)?).map_err(|f| bad(f.message))?;
        let recipe: PrintRecipe = files.parse(RECIPE)?;
        if setup.content_hash() != inputs.device_setup.content_hash
            || mode.content_hash() != inputs.print_mode.content_hash
            || recipe.content_hash() != inputs.recipe.content_hash()
            || mode.device_setup() != setup.id()
        {
            return Err(bad("a record does not match what the job snapshot names"));
        }

        let wanted: BTreeSet<ContentHash> = inputs
            .assets
            .iter()
            .chain(inputs.masks.iter())
            .map(|r| r.content_hash)
            .collect();
        let mut sources = Vec::new();
        for hash in &wanted {
            let content = files
                .take(&format!("{SOURCE_DIR}{hash}"))
                .map_err(|_| bad(format!("source {hash} is missing")))?;
            if ContentHash::digest(&content) != *hash {
                return Err(bad(format!("source {hash} is not the image the job names")));
            }
            sources.push(content);
        }

        let events: Vec<LifecycleRecord> = files.parse(LIFECYCLE)?;
        let attempts: Vec<SubmissionAttempt> = files.parse(ATTEMPTS)?;
        let mut state = JobState::Draft;
        for (i, r) in events.iter().enumerate() {
            let legal = transition(state, r.event).ok() == Some(r.to);
            if r.job_id != id || r.sequence as usize != i + 1 || r.from != state || !legal {
                return Err(bad(format!(
                    "lifecycle event {} is not a legal history",
                    i + 1
                )));
            }
            state = r.to;
        }
        if attempts
            .iter()
            .any(|a| a.job_id != id || a.snapshot_hash != manifest.snapshot_hash)
        {
            return Err(bad("a submission record belongs to a different job"));
        }
        if !files.0.is_empty() {
            return Err(bad("it holds files this version does not understand"));
        }

        match self.jobs.get(id) {
            Ok(_) => {
                return Err(WorkflowError::Invalid(format!(
                    "job {id} already exists in this workspace; it is never overwritten or merged"
                )))
            }
            Err(PersistenceError::NotFound { .. }) => {}
            Err(e) => return Err(e.into()),
        }

        // Nothing above wrote anything. From here the order is: content-addressed inputs first,
        // then the job, then its history, so a job never exists without its inputs.
        for content in &sources {
            self.assets.put(content)?;
        }
        self.devices.put_setup(&setup)?;
        self.devices.put_mode(&mode)?;
        self.records.put_recipe(&recipe)?;
        // The snapshot and attempts go in together and undo themselves if one fails. The history
        // is moved into place last, in one step; if that or the check after it fails, the job is
        // taken out again, so an import either completes or leaves nothing.
        let mark = match self.jobs.import_job(snapshot, &attempts) {
            Ok(mark) => mark,
            Err(e) if self.jobs.get(id).is_err() => {
                return Err(WorkflowError::Invalid(format!(
                    "import of job {id} failed ({e}); nothing was imported and the archive can be                      imported again"
                )))
            }
            Err(e) => {
                return Err(WorkflowError::Invalid(format!(
                    "import of job {id} stopped part-way ({e}) and could not be undone. The job                      exists here with incomplete history: do not print it. Import the archive                      into a clean workspace instead."
                )))
            }
        };
        let failed = |e: &dyn std::fmt::Display, history_in_place: bool| {
            let history_undone =
                !history_in_place || self.lifecycle.discard_imported_events(id).is_ok();
            let job_undone = history_undone && self.jobs.undo_import(id, &mark).is_ok();
            if job_undone {
                WorkflowError::Invalid(format!(
                    "import of job {id} failed ({e}); nothing was imported and the archive can be \
                     imported again"
                ))
            } else {
                WorkflowError::Invalid(format!(
                    "import of job {id} stopped part-way ({e}) and could not be undone. The job \
                     exists here with incomplete history: do not print it. Import the archive \
                     into a clean workspace instead."
                ))
            }
        };
        if let Err(e) = self.lifecycle.import_events(id, &events) {
            return Err(failed(&e, false));
        }
        match self.lifecycle.events(id) {
            Ok(written) if written == events => {}
            Ok(_) => {
                return Err(failed(
                    &"the written history differs from the archive",
                    true,
                ))
            }
            Err(e) => return Err(failed(&e, true)),
        }
        Ok(ImportedJob {
            job_id: id,
            snapshot_hash: manifest.snapshot_hash,
            state,
            lifecycle_events: events.len(),
            attempt_records: attempts.len(),
            setup_verification_carried: false,
        })
    }
}
