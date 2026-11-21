//! Read-only views of retained planes. Every read first checks the file against the manifest, so a
//! picture never shows bytes the manifest does not vouch for. Nothing here changes a run.
use crate::error::{WorkflowError, WorkflowResult};
use std::path::Path;
use veylora_domain::provenance::ContentHash;
use veylora_render::{RenderManifest, SampleFormat, MANIFEST_NAME};

/// Largest overview edge, in pixels.
pub const MAX_OVERVIEW_PX: usize = 4096;
/// Largest loupe window edge, in pixels.
pub const MAX_WINDOW_PX: usize = 64;

/// A gray PNG of one plane, reduced by `scale` (box average) so it fits the requested edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaneOverview {
    pub png: Vec<u8>,
    pub width: usize,
    pub height: usize,
    /// Source pixels per picture pixel along each axis; 1 means the picture is exact.
    pub scale: usize,
    pub source_width: usize,
    pub source_height: usize,
}

/// Exact values (0 to 1) of a rectangle of one plane, clipped to the plane.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaneWindow {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub values: Vec<f64>,
}

struct Plane {
    width: usize,
    height: usize,
    values: Vec<f32>,
}

fn invalid(message: impl Into<String>) -> WorkflowError {
    WorkflowError::Invalid(message.into())
}

fn load(run_dir: &Path, name: &str) -> WorkflowResult<Plane> {
    let manifest_bytes = std::fs::read(run_dir.join(MANIFEST_NAME))?;
    let manifest: RenderManifest = serde_json::from_slice(&manifest_bytes)?;
    let (stage, file) = [
        &manifest.logical_planes,
        &manifest.physical_planes,
        &manifest.screened_planes,
    ]
    .into_iter()
    .find_map(|s| s.files.iter().find(|f| f.name == name).map(|f| (s, f)))
    .ok_or_else(|| invalid(format!("'{name}' is not a plane of this run.")))?;
    let data = std::fs::read(run_dir.join(name))
        .map_err(|e| invalid(format!("{name}: cannot be read ({e})")))?;
    let sample = match stage.format {
        SampleFormat::F32Le => 4,
        SampleFormat::U16Le => 2,
    };
    if data.len() as u64 != file.bytes
        || data.len() != stage.width * stage.height * sample
        || ContentHash::digest(&data) != file.hash
    {
        return Err(invalid(format!(
            "{name} does not match the manifest and is not shown."
        )));
    }
    let level = f32::from(manifest.dot_level.max(1));
    let values = match stage.format {
        SampleFormat::F32Le => data
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect(),
        SampleFormat::U16Le => data
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| f32::from(u16::from_le_bytes(*c)) / level)
            .collect(),
    };
    Ok(Plane {
        width: stage.width,
        height: stage.height,
        values,
    })
}

fn gray(v: f32) -> u8 {
    (f64::from(v).clamp(0.0, 1.0) * 255.0).round() as u8
}

pub fn overview(run_dir: &Path, name: &str, max_px: usize) -> WorkflowResult<PlaneOverview> {
    if max_px == 0 || max_px > MAX_OVERVIEW_PX {
        return Err(invalid(format!(
            "An overview edge must be 1 to {MAX_OVERVIEW_PX} pixels."
        )));
    }
    let plane = load(run_dir, name)?;
    if plane.width == 0 || plane.height == 0 {
        return Err(invalid(format!("{name} is empty.")));
    }
    let scale = plane.width.max(plane.height).div_ceil(max_px);
    let (w, h) = (plane.width.div_ceil(scale), plane.height.div_ceil(scale));
    let mut pixels = Vec::with_capacity(w * h);
    for oy in 0..h {
        for ox in 0..w {
            let (mut sum, mut n) = (0.0f64, 0u32);
            for y in oy * scale..((oy + 1) * scale).min(plane.height) {
                for x in ox * scale..((ox + 1) * scale).min(plane.width) {
                    sum += f64::from(plane.values[y * plane.width + x]);
                    n += 1;
                }
            }
            pixels.push(gray((sum / f64::from(n)) as f32));
        }
    }
    let png = veylora_image::encode_gray8_png(w as u32, h as u32, &pixels).map_err(invalid)?;
    Ok(PlaneOverview {
        png,
        width: w,
        height: h,
        scale,
        source_width: plane.width,
        source_height: plane.height,
    })
}

pub fn window(
    run_dir: &Path,
    name: &str,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> WorkflowResult<PlaneWindow> {
    if width == 0 || height == 0 || width > MAX_WINDOW_PX || height > MAX_WINDOW_PX {
        return Err(invalid(format!(
            "A window edge must be 1 to {MAX_WINDOW_PX} pixels."
        )));
    }
    let plane = load(run_dir, name)?;
    if x >= plane.width || y >= plane.height {
        return Err(invalid(format!(
            "({x}, {y}) is outside the {} x {} plane.",
            plane.width, plane.height
        )));
    }
    let (w, h) = (width.min(plane.width - x), height.min(plane.height - y));
    let mut values = Vec::with_capacity(w * h);
    for row in y..y + h {
        for col in x..x + w {
            values.push(f64::from(plane.values[row * plane.width + col]).clamp(0.0, 1.0));
        }
    }
    Ok(PlaneWindow {
        x,
        y,
        width: w,
        height: h,
        values,
    })
}
