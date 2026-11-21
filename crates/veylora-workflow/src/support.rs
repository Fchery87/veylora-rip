//! Support bundles: a compact, local, redacted account of one job for someone else to read.
//!
//! The bundle is written to a file the operator chooses and is never sent anywhere. Artwork is
//! included only when asked for. A section that cannot be read is named in the manifest instead
//! of being dropped, so a bundle never looks more complete than it is.

use crate::{
    error::{WorkflowError, WorkflowResult},
    workspace::Workspace,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use veylora_diagnostics::{
    bundle::{write_tar, BundleEntry},
    redact::Redactor,
};
use veylora_domain::{ids::JobId, provenance::ContentHash};
use veylora_persistence::JobStore;
use veylora_render::MANIFEST_NAME;

#[derive(Debug, Clone, Copy, Default)]
pub struct SupportBundleOptions {
    /// Include the stored source images and masks. Off by default: artwork is the customer's.
    pub include_artwork: bool,
}

#[derive(Serialize)]
struct Unavailable {
    section: String,
    reason: String,
}

#[derive(Serialize)]
struct Listed {
    path: String,
    bytes: usize,
    sha256: String,
}

#[derive(Serialize)]
struct BundleManifest<'a> {
    schema_version: u32,
    job_id: JobId,
    include_artwork: bool,
    redaction: &'static str,
    transmission: &'static str,
    /// Render runs that started but never wrote a manifest.
    incomplete_runs: Vec<String>,
    unavailable: &'a [Unavailable],
    files: Vec<Listed>,
}

fn redact_value(redactor: &Redactor, value: &mut Value) {
    match value {
        Value::String(s) => *s = redactor.redact(s),
        Value::Array(items) => items.iter_mut().for_each(|v| redact_value(redactor, v)),
        Value::Object(map) => map.values_mut().for_each(|v| redact_value(redactor, v)),
        _ => {}
    }
}

fn pretty(value: &Value) -> WorkflowResult<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

impl Workspace {
    /// Builds the archive in memory. The job must exist; everything else is best effort and any
    /// shortfall is listed under `unavailable` in `manifest.json`.
    pub fn support_bundle(
        &self,
        id: JobId,
        options: &SupportBundleOptions,
    ) -> WorkflowResult<Vec<u8>> {
        let snapshot = self.jobs.get(id)?;
        let mut roots = vec![self.root.display().to_string()];
        if let Ok(canonical) = self.root.canonicalize() {
            roots.push(
                canonical
                    .display()
                    .to_string()
                    .trim_start_matches("\\\\?\\")
                    .to_string(),
            );
        }
        let redactor = Redactor::new(&roots);
        let mut files: Vec<BundleEntry> = Vec::new();
        let mut unavailable: Vec<Unavailable> = Vec::new();
        let mut miss = |section: String, reason: String| {
            unavailable.push(Unavailable {
                section,
                reason: redactor.redact(&reason),
            });
        };

        match self.report(id).and_then(|r| Ok(serde_json::to_value(r)?)) {
            Ok(mut value) => {
                redact_value(&redactor, &mut value);
                files.push(BundleEntry {
                    path: "job-report.json".into(),
                    bytes: pretty(&value)?,
                });
            }
            Err(e) => miss("job-report".into(), e.to_string()),
        }

        match self
            .lifecycle_events(id)
            .and_then(|e| Ok(serde_json::to_value(e)?))
        {
            Ok(mut value) => {
                redact_value(&redactor, &mut value);
                files.push(BundleEntry {
                    path: "lifecycle.json".into(),
                    bytes: pretty(&value)?,
                });
            }
            Err(e) => miss("lifecycle".into(), e.to_string()),
        }

        let mut incomplete_runs = Vec::new();
        match self.run_names(id) {
            Ok(runs) => {
                for run in runs {
                    let path = self.artifacts_dir(id).join(&run).join(MANIFEST_NAME);
                    let section = format!("render-manifests/{run}.json");
                    match std::fs::read(&path) {
                        Ok(bytes) => {
                            // Manifests are hashed records and are included byte for byte. One
                            // that names a local path is withheld rather than rewritten.
                            // Checked on the decoded strings: JSON doubles the backslashes of a Windows path.
                            let leaks = serde_json::from_slice::<Value>(&bytes)
                                .map(|original| {
                                    let mut scrubbed = original.clone();
                                    redact_value(&redactor, &mut scrubbed);
                                    scrubbed != original
                                })
                                .unwrap_or(true);
                            if leaks {
                                miss(
                                    section,
                                    "contains a machine-specific path or is not JSON; withheld"
                                        .into(),
                                );
                            } else {
                                files.push(BundleEntry {
                                    path: section,
                                    bytes,
                                });
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                            incomplete_runs.push(run);
                        }
                        Err(e) => miss(section, e.to_string()),
                    }
                }
            }
            Err(e) => miss("render-manifests".into(), e.to_string()),
        }

        if options.include_artwork {
            let inputs = snapshot.inputs();
            let wanted: BTreeSet<ContentHash> = inputs
                .assets
                .iter()
                .chain(inputs.masks.iter())
                .map(|r| r.content_hash)
                .collect();
            for hash in wanted {
                match self.assets.get(&hash) {
                    Ok(bytes) => files.push(BundleEntry {
                        path: format!("artwork/{hash}"),
                        bytes,
                    }),
                    Err(e) => miss(format!("artwork/{hash}"), e.to_string()),
                }
            }
        }

        let mut listed: Vec<Listed> = files
            .iter()
            .map(|f| Listed {
                path: f.path.clone(),
                bytes: f.bytes.len(),
                sha256: ContentHash::digest(&f.bytes).to_string(),
            })
            .collect();
        listed.sort_by(|a, b| a.path.cmp(&b.path));
        unavailable.sort_by(|a, b| a.section.cmp(&b.section));
        let manifest = BundleManifest {
            schema_version: 1,
            job_id: id,
            include_artwork: options.include_artwork,
            redaction: "workspace location and user profile names are replaced in free text; \
                        hashed records are included unchanged or withheld",
            transmission: "none: this archive is written to a local file and is never uploaded",
            incomplete_runs,
            unavailable: &unavailable,
            files: listed,
        };
        files.push(BundleEntry {
            path: "manifest.json".into(),
            bytes: pretty(&serde_json::to_value(&manifest)?)?,
        });
        write_tar(&files).map_err(|e| WorkflowError::Invalid(e.to_string()))
    }
}
