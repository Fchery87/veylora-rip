use crate::{error::ReleaseError, version::Version};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use veylora_domain::provenance::ContentHash;

pub const MANIFEST_NAME: &str = "release-manifest.json";
pub const MANIFEST_SCHEMA: u32 = 1;
pub const PRODUCT: &str = "veylora-rip";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseFile {
    pub path: String,
    pub bytes: u64,
    pub sha256: ContentHash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseManifest {
    pub schema_version: u32,
    pub product: String,
    pub version: String,
    /// The newest workspace data schema this release can read.
    pub max_data_schema: u32,
    /// Program (relative to the release) and arguments run after install to prove it starts.
    pub smoke_test: Option<Vec<String>>,
    pub files: Vec<ReleaseFile>,
}

/// A relative, `/`-separated path that cannot leave the release directory.
pub(crate) fn check_rel(path: &str) -> Result<(), ReleaseError> {
    let bad = || Err(ReleaseError::UnsafePath(path.to_string()));
    if path.is_empty() || path.len() > 240 {
        return bad();
    }
    if path
        .chars()
        .any(|c| c.is_control() || matches!(c, '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*'))
    {
        return bad();
    }
    if path
        .split('/')
        .any(|c| c.is_empty() || c == "." || c == ".." || c.ends_with('.') || c.ends_with(' '))
    {
        return bad();
    }
    Ok(())
}

const DEV_DIRS: [&str; 3] = ["node_modules", ".git", "target"];
const DEV_FILES: [&str; 8] = [
    "cargo.toml",
    "cargo.lock",
    "package.json",
    "pnpm-lock.yaml",
    "package-lock.json",
    "yarn.lock",
    "tsconfig.json",
    "rust-toolchain",
];
const DEV_EXTENSIONS: [&str; 6] = ["rs", "ts", "pdb", "rlib", "rmeta", "ilk"];

fn dev_tool(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let mut parts: Vec<&str> = lower.split('/').collect();
    let file = parts.pop().unwrap_or("");
    if parts.iter().any(|d| DEV_DIRS.contains(d)) {
        return true;
    }
    let stem = file.split('.').next().unwrap_or("");
    DEV_FILES.contains(&file)
        || stem == "rust-toolchain"
        || (stem == "vite" && file.contains(".config."))
        || file
            .rsplit_once('.')
            .is_some_and(|(_, e)| DEV_EXTENSIONS.contains(&e))
}

fn guard_path(path: &str) -> Result<(), ReleaseError> {
    check_rel(path)?;
    if dev_tool(path) {
        return Err(ReleaseError::DevTool(path.to_string()));
    }
    Ok(())
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) -> Result<(), ReleaseError> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path)?;
        let rel = path
            .strip_prefix(root)
            .map_err(|e| ReleaseError::Io(e.to_string()))?
            .to_string_lossy()
            .replace('\\', "/");
        if meta.is_dir() {
            collect(root, &path, out)?;
        } else if meta.is_file() {
            if rel != MANIFEST_NAME {
                out.push((rel, path));
            }
        } else {
            return Err(ReleaseError::UnsafePath(format!(
                "{rel} (not a regular file)"
            )));
        }
    }
    Ok(())
}

fn files_on_disk(dir: &Path) -> Result<Vec<(String, PathBuf)>, ReleaseError> {
    let mut out = Vec::new();
    collect(dir, dir, &mut out)?;
    out.sort();
    Ok(out)
}

/// Hashes every file under `dir` into a manifest. Refuses development tooling and anything that
/// is not a plain file.
pub fn build_manifest(
    dir: &Path,
    version: &str,
    max_data_schema: u32,
    smoke_test: Option<Vec<String>>,
) -> Result<ReleaseManifest, ReleaseError> {
    Version::parse(version)?;
    let mut files = Vec::new();
    for (rel, path) in files_on_disk(dir)? {
        guard_path(&rel)?;
        let bytes = std::fs::read(&path)?;
        files.push(ReleaseFile {
            path: rel,
            bytes: bytes.len() as u64,
            sha256: ContentHash::digest(&bytes),
        });
    }
    if files.is_empty() {
        return Err(ReleaseError::Malformed(
            "the release directory has no files".into(),
        ));
    }
    let manifest = ReleaseManifest {
        schema_version: MANIFEST_SCHEMA,
        product: PRODUCT.into(),
        version: version.into(),
        max_data_schema,
        smoke_test,
        files,
    };
    check_smoke(&manifest)?;
    Ok(manifest)
}

fn check_smoke(m: &ReleaseManifest) -> Result<(), ReleaseError> {
    if let Some(cmd) = &m.smoke_test {
        let program = cmd
            .first()
            .ok_or_else(|| ReleaseError::Malformed("smoke_test is empty".into()))?;
        check_rel(program)?;
        if !m.files.iter().any(|f| &f.path == program) {
            return Err(ReleaseError::Malformed(format!(
                "smoke_test names {program}, which is not in the release"
            )));
        }
    }
    Ok(())
}

pub fn write_manifest(dir: &Path, manifest: &ReleaseManifest) -> Result<(), ReleaseError> {
    let mut bytes =
        serde_json::to_vec_pretty(manifest).map_err(|e| ReleaseError::Malformed(e.to_string()))?;
    bytes.push(b'\n');
    std::fs::write(dir.join(MANIFEST_NAME), bytes)?;
    Ok(())
}

/// Checks that `dir` is exactly what its manifest says: same files, sizes and hashes, nothing
/// extra, nothing unsafe, no development tooling.
pub fn verify_release(dir: &Path) -> Result<ReleaseManifest, ReleaseError> {
    let bytes = std::fs::read(dir.join(MANIFEST_NAME))
        .map_err(|e| ReleaseError::Malformed(format!("{MANIFEST_NAME}: {e}")))?;
    let manifest: ReleaseManifest = serde_json::from_slice(&bytes)
        .map_err(|e| ReleaseError::Malformed(format!("{MANIFEST_NAME}: {e}")))?;
    if manifest.schema_version != MANIFEST_SCHEMA {
        return Err(ReleaseError::Malformed(format!(
            "manifest schema {} is not supported (expected {MANIFEST_SCHEMA})",
            manifest.schema_version
        )));
    }
    if manifest.product != PRODUCT {
        return Err(ReleaseError::Malformed(format!(
            "the release is for {:?}, not {PRODUCT}",
            manifest.product
        )));
    }
    Version::parse(&manifest.version)?;
    let mut seen = std::collections::BTreeSet::new();
    for f in &manifest.files {
        guard_path(&f.path)?;
        if !seen.insert(f.path.as_str()) {
            return Err(ReleaseError::Malformed(format!(
                "{} is listed twice",
                f.path
            )));
        }
    }
    check_smoke(&manifest)?;
    let on_disk = files_on_disk(dir)?;
    for (rel, _) in &on_disk {
        if !seen.contains(rel.as_str()) {
            return Err(ReleaseError::Mismatch {
                path: rel.clone(),
                what: "is present but not listed in the manifest",
            });
        }
    }
    for f in &manifest.files {
        let content = std::fs::read(dir.join(&f.path)).map_err(|_| ReleaseError::Mismatch {
            path: f.path.clone(),
            what: "is listed but missing",
        })?;
        if content.len() as u64 != f.bytes || ContentHash::digest(&content) != f.sha256 {
            return Err(ReleaseError::Mismatch {
                path: f.path.clone(),
                what: "does not match its recorded size and hash",
            });
        }
    }
    Ok(manifest)
}
