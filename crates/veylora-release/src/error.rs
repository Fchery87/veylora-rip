use crate::install::Blocker;

#[derive(Debug)]
pub enum ReleaseError {
    Io(String),
    Malformed(String),
    UnsafePath(String),
    /// A development tool or build leftover would ship in the release.
    DevTool(String),
    Mismatch {
        path: String,
        what: &'static str,
    },
    Version(String),
    NotNewer {
        installed: String,
        offered: String,
    },
    /// Work that an update would interrupt is in flight.
    Blocked(Vec<Blocker>),
    /// The job state could not be read, so it cannot be shown safe to update.
    GuardUnavailable(String),
    DataTooNew {
        data: u32,
        supported: u32,
    },
    /// The new release failed its smoke test and the previous release was restored.
    RolledBack {
        version: String,
        reason: String,
    },
    NothingToRollBackTo,
    CurrentDamaged(String),
    /// Another update, rollback or recovery holds the install lock.
    Locked,
}

impl std::fmt::Display for ReleaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Malformed(e) => write!(f, "malformed release data: {e}"),
            Self::UnsafePath(p) => write!(f, "unsafe release path {p:?}"),
            Self::DevTool(p) => write!(f, "{p} is a development tool or build leftover and must not ship"),
            Self::Mismatch { path, what } => write!(f, "{path}: {what}"),
            Self::Version(v) => write!(f, "{v:?} is not a plain MAJOR.MINOR.PATCH version"),
            Self::NotNewer { installed, offered } => write!(
                f,
                "version {offered} is not newer than the installed {installed}"
            ),
            Self::Blocked(b) => {
                write!(f, "update refused while work is in flight:")?;
                for x in b {
                    write!(f, " [{}] {}", x.subject, x.reason)?;
                }
                Ok(())
            }
            Self::GuardUnavailable(e) => write!(
                f,
                "update refused: cannot confirm that no submission is in flight ({e})"
            ),
            Self::DataTooNew { data, supported } => write!(
                f,
                "the workspace data is at schema {data} but this release reads up to {supported}"
            ),
            Self::RolledBack { version, reason } => write!(
                f,
                "release {version} failed its smoke test ({reason}); the previous release was restored"
            ),
            Self::NothingToRollBackTo => write!(f, "there is no previous release to roll back to"),
            Self::CurrentDamaged(e) => write!(f, "the current release is damaged: {e}"),
            Self::Locked => write!(
                f,
                "another update is in progress (install.lock exists); run recovery if none is"
            ),
        }
    }
}

impl std::error::Error for ReleaseError {}

impl From<std::io::Error> for ReleaseError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}
