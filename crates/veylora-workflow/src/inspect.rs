//! Read-only inspection of imported assets and of retained render artifacts.
use crate::error::WorkflowResult;
use serde::Serialize;
use std::path::Path;
use veylora_domain::provenance::ContentHash;
use veylora_image::{decode_png_with_policy, DecodeLimits, SampleLayout, UntaggedPolicy};
use veylora_render::{ArtifactFile, RenderManifest, SampleFormat, MANIFEST_NAME};

#[derive(Debug, Clone, Serialize)]
pub struct AssetReport {
    pub sha256: ContentHash,
    pub width: u32,
    pub height: u32,
    pub layout: &'static str,
    pub color: String,
    pub warnings: Vec<String>,
    pub file_type: &'static str,
    /// Proposal from the file's own resolution metadata only; never a placement instruction.
    pub physical_size_mm: Option<(f64, f64)>,
}

pub fn inspect_asset(bytes: &[u8], policy: UntaggedPolicy) -> WorkflowResult<AssetReport> {
    let asset = decode_png_with_policy(bytes, DecodeLimits::default(), policy)?;
    Ok(AssetReport {
        sha256: asset.original_hash,
        width: asset.metadata.width,
        height: asset.metadata.height,
        layout: match asset.decoded.layout {
            SampleLayout::Gray8 => "gray8",
            SampleLayout::GrayAlpha8 => "gray_alpha8",
            SampleLayout::Rgb8 => "rgb8",
            SampleLayout::Rgba8 => "rgba8",
        },
        color: format!("{:?}", asset.metadata.color),
        warnings: asset
            .metadata
            .warnings
            .iter()
            .map(|w| format!("{w:?}"))
            .collect(),
        file_type: asset.metadata.file_type,
        physical_size_mm: asset.metadata.physical_size_mm(),
    })
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlaneStats {
    pub name: String,
    pub pixels: u64,
    pub nonzero: u64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactReport {
    /// True when every retained file matches the manifest's size and hash.
    pub verified: bool,
    pub problems: Vec<String>,
    pub manifest: RenderManifest,
    pub manifest_hash: ContentHash,
    /// Statistics computed from the retained bytes, not from a recomputed preview.
    pub planes: Vec<PlaneStats>,
}

impl ArtifactReport {
    pub fn plane(&self, name: &str) -> Option<&PlaneStats> {
        self.planes.iter().find(|p| p.name == name)
    }
}

fn stats(name: &str, format: SampleFormat, bytes: &[u8]) -> PlaneStats {
    let values: Vec<f64> = match format {
        SampleFormat::F32Le => bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| f64::from(f32::from_le_bytes(*c)))
            .collect(),
        SampleFormat::U16Le => bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| f64::from(u16::from_le_bytes(*c)))
            .collect(),
    };
    let pixels = values.len() as u64;
    PlaneStats {
        name: name.to_string(),
        pixels,
        nonzero: values.iter().filter(|v| **v != 0.0).count() as u64,
        min: values.iter().copied().fold(f64::INFINITY, f64::min),
        max: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        mean: if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        },
    }
}

/// Re-reads every file the manifest names, checks size and hash, and summarizes the bytes. A
/// mismatch is reported in `problems`; it is not an error, so a damaged run can still be examined.
pub fn inspect_artifacts(dir: &Path) -> WorkflowResult<ArtifactReport> {
    let manifest_bytes = std::fs::read(dir.join(MANIFEST_NAME))?;
    let manifest: RenderManifest = serde_json::from_slice(&manifest_bytes)?;
    let mut problems = Vec::new();
    let mut planes = Vec::new();
    for stage in [
        &manifest.logical_planes,
        &manifest.physical_planes,
        &manifest.screened_planes,
    ] {
        let sample = match stage.format {
            SampleFormat::F32Le => 4u64,
            SampleFormat::U16Le => 2,
        };
        let expected_len = (stage.width as u64) * (stage.height as u64) * sample;
        for ArtifactFile { name, bytes, hash } in &stage.files {
            let data = match std::fs::read(dir.join(name)) {
                Ok(d) => d,
                Err(e) => {
                    problems.push(format!("{name}: cannot be read ({e})"));
                    continue;
                }
            };
            if data.len() as u64 != *bytes || data.len() as u64 != expected_len {
                problems.push(format!(
                    "{name}: {} bytes on disk, manifest says {bytes}, geometry needs {expected_len}",
                    data.len()
                ));
            }
            if ContentHash::digest(&data) != *hash {
                problems.push(format!("{name}: content does not match the manifest hash"));
            }
            planes.push(stats(name, stage.format, &data));
        }
    }
    Ok(ArtifactReport {
        verified: problems.is_empty(),
        problems,
        manifest,
        manifest_hash: ContentHash::digest(&manifest_bytes),
        planes,
    })
}
