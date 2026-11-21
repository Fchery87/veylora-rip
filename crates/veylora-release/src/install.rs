//! Side-by-side installs. `releases/<version>/` holds verified copies; `install-state.json` says
//! which is current. The state file is the only thing that changes what runs, and it is replaced
//! atomically, so a crash leaves either the old release or the new one, never a mixture.

use crate::{
    error::ReleaseError,
    manifest::{verify_release, ReleaseManifest, MANIFEST_NAME},
    version::Version,
};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

const STATE_NAME: &str = "install-state.json";
const LOCK_NAME: &str = "install.lock";
const STAGING_SUFFIX: &str = ".staging";

/// Something that makes an update unsafe right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blocker {
    pub subject: String,
    pub reason: String,
}

/// Asked before any update or rollback. An `Err` means the answer is unknown, which is treated as
/// unsafe: an update never proceeds on a guess.
pub trait UpdateGuard {
    fn blockers(&self) -> Result<Vec<Blocker>, String>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingUpdate {
    pub from: Option<String>,
    pub to: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallState {
    pub current: Option<String>,
    pub previous: Option<String>,
    /// Written before an update starts changing the install and cleared when it commits. One that
    /// is still here after a crash was never committed.
    pub pending: Option<PendingUpdate>,
}

pub struct UpdateRequest<'a> {
    pub source: &'a Path,
    /// The workspace's data schema version, when there is a workspace.
    pub data_schema: Option<u32>,
    pub guard: &'a dyn UpdateGuard,
    /// Runs against the installed copy after the switch. An `Err` restores the old release.
    pub smoke_test: &'a dyn Fn(&Path, &ReleaseManifest) -> Result<(), String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOutcome {
    pub installed: String,
    pub previous: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecoveryReport {
    pub abandoned_update: bool,
    pub fell_back_from_damaged_current: bool,
    pub removed: Vec<String>,
}

#[derive(Debug)]
pub struct InstallStore {
    root: PathBuf,
}

struct Lock(PathBuf);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

impl InstallStore {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, ReleaseError> {
        let root = root.into();
        std::fs::create_dir_all(root.join("releases"))?;
        Ok(Self { root })
    }

    fn releases(&self) -> PathBuf {
        self.root.join("releases")
    }

    fn release_dir(&self, version: &str) -> PathBuf {
        self.releases().join(version)
    }

    fn lock(&self) -> Result<Lock, ReleaseError> {
        let path = self.root.join(LOCK_NAME);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(_) => Ok(Lock(path)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Err(ReleaseError::Locked),
            Err(e) => Err(e.into()),
        }
    }

    pub fn state(&self) -> Result<InstallState, ReleaseError> {
        match std::fs::read(self.root.join(STATE_NAME)) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| ReleaseError::Malformed(format!("{STATE_NAME}: {e}"))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(InstallState::default()),
            Err(e) => Err(e.into()),
        }
    }

    fn write_state(&self, state: &InstallState) -> Result<(), ReleaseError> {
        let tmp = self.root.join(format!("{STATE_NAME}.tmp"));
        let bytes =
            serde_json::to_vec_pretty(state).map_err(|e| ReleaseError::Malformed(e.to_string()))?;
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, self.root.join(STATE_NAME))?;
        Ok(())
    }

    pub fn current_dir(&self) -> Result<Option<PathBuf>, ReleaseError> {
        Ok(self.state()?.current.map(|v| self.release_dir(&v)))
    }

    /// Verifies the current release in place, or `None` when nothing is installed.
    pub fn verify_current(&self) -> Result<Option<ReleaseManifest>, ReleaseError> {
        match self.current_dir()? {
            None => Ok(None),
            Some(dir) => Ok(Some(verify_release(&dir)?)),
        }
    }

    fn check_guard(guard: &dyn UpdateGuard) -> Result<(), ReleaseError> {
        let blockers = guard.blockers().map_err(ReleaseError::GuardUnavailable)?;
        if blockers.is_empty() {
            Ok(())
        } else {
            Err(ReleaseError::Blocked(blockers))
        }
    }

    pub fn install(&self, req: &UpdateRequest<'_>) -> Result<InstallOutcome, ReleaseError> {
        let _lock = self.lock()?;
        let before = self.state()?;
        Self::check_guard(req.guard)?;
        let manifest = verify_release(req.source)?;
        let offered = Version::parse(&manifest.version)?;
        if let Some(installed) = &before.current {
            if offered <= Version::parse(installed)? {
                return Err(ReleaseError::NotNewer {
                    installed: installed.clone(),
                    offered: manifest.version.clone(),
                });
            }
        }
        if let Some(data) = req.data_schema {
            if data > manifest.max_data_schema {
                return Err(ReleaseError::DataTooNew {
                    data,
                    supported: manifest.max_data_schema,
                });
            }
        }

        let version = manifest.version.clone();
        let target = self.release_dir(&version);
        let staging = self.releases().join(format!("{version}{STAGING_SUFFIX}"));
        let _ = std::fs::remove_dir_all(&staging);
        let _ = std::fs::remove_dir_all(&target);
        let staged = copy_release(req.source, &staging, &manifest)
            .and_then(|()| verify_release(&staging).map(|_| ()));
        if let Err(e) = staged {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(e);
        }

        // Journal first, so a crash from here until the commit is recognisable as unfinished.
        let mut journaled = before.clone();
        journaled.pending = Some(PendingUpdate {
            from: before.current.clone(),
            to: version.clone(),
        });
        let switched = self
            .write_state(&journaled)
            .and_then(|()| std::fs::rename(&staging, &target).map_err(Into::into))
            .and_then(|()| verify_release(&target).map(|_| ()))
            .and_then(|()| {
                self.write_state(&InstallState {
                    current: Some(version.clone()),
                    previous: before.current.clone(),
                    pending: None,
                })
            });
        if let Err(e) = switched {
            let _ = self.write_state(&before);
            let _ = std::fs::remove_dir_all(&staging);
            let _ = std::fs::remove_dir_all(&target);
            return Err(e);
        }

        if let Err(reason) = (req.smoke_test)(&target, &manifest) {
            self.write_state(&before)?;
            let _ = std::fs::remove_dir_all(&target);
            return Err(ReleaseError::RolledBack { version, reason });
        }
        // Only the current and previous releases are kept.
        if let Some(old) = &before.previous {
            let _ = std::fs::remove_dir_all(self.release_dir(old));
        }
        Ok(InstallOutcome {
            installed: version,
            previous: before.current,
        })
    }

    /// Switches back to the previous release. That release is verified first, and the release
    /// being left is removed, so a rollback can only be done once.
    pub fn rollback(&self, guard: &dyn UpdateGuard) -> Result<InstallOutcome, ReleaseError> {
        let _lock = self.lock()?;
        let before = self.state()?;
        Self::check_guard(guard)?;
        let previous = before
            .previous
            .clone()
            .ok_or(ReleaseError::NothingToRollBackTo)?;
        verify_release(&self.release_dir(&previous))?;
        self.write_state(&InstallState {
            current: Some(previous.clone()),
            previous: None,
            pending: None,
        })?;
        if let Some(left) = &before.current {
            let _ = std::fs::remove_dir_all(self.release_dir(left));
        }
        Ok(InstallOutcome {
            installed: previous,
            previous: None,
        })
    }

    /// Run at startup. Clears what an interrupted update left behind and never commits anything
    /// new: an unfinished update is abandoned, and a damaged current release falls back to the
    /// previous one only if that verifies.
    pub fn recover(&self) -> Result<RecoveryReport, ReleaseError> {
        // A lock left by a crash would block every later update. Recovery runs when nothing else
        // should be updating, so it clears the lock before taking its own.
        let _ = std::fs::remove_file(self.root.join(LOCK_NAME));
        let _lock = self.lock()?;
        let mut state = self.state()?;
        let mut report = RecoveryReport::default();

        let mut leftovers: Vec<String> = std::fs::read_dir(self.releases())?
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(STAGING_SUFFIX))
            .collect();
        leftovers.sort();
        for name in leftovers {
            std::fs::remove_dir_all(self.releases().join(&name))?;
            report.removed.push(name);
        }

        if let Some(pending) = state.pending.take() {
            report.abandoned_update = true;
            if state.current.as_deref() != Some(pending.to.as_str())
                && state.previous.as_deref() != Some(pending.to.as_str())
            {
                let _ = std::fs::remove_dir_all(self.release_dir(&pending.to));
            }
            self.write_state(&state)?;
        }

        if let Some(current) = state.current.clone() {
            if let Err(damage) = verify_release(&self.release_dir(&current)) {
                let fallback = state
                    .previous
                    .clone()
                    .filter(|p| verify_release(&self.release_dir(p)).is_ok());
                let Some(previous) = fallback else {
                    return Err(ReleaseError::CurrentDamaged(damage.to_string()));
                };
                self.write_state(&InstallState {
                    current: Some(previous),
                    previous: None,
                    pending: None,
                })?;
                let _ = std::fs::remove_dir_all(self.release_dir(&current));
                report.fell_back_from_damaged_current = true;
            }
        }
        Ok(report)
    }
}

fn copy_release(
    source: &Path,
    dest: &Path,
    manifest: &ReleaseManifest,
) -> Result<(), ReleaseError> {
    for f in &manifest.files {
        let to = dest.join(&f.path);
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(source.join(&f.path), &to)?;
    }
    std::fs::copy(source.join(MANIFEST_NAME), dest.join(MANIFEST_NAME))?;
    Ok(())
}
