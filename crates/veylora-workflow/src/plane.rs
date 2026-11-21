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

fn overview_size(
    plane: &Plane,
    max_px: usize,
    name: &str,
) -> WorkflowResult<(usize, usize, usize)> {
    if max_px == 0 || max_px > MAX_OVERVIEW_PX {
        return Err(invalid(format!(
            "An overview edge must be 1 to {MAX_OVERVIEW_PX} pixels."
        )));
    }
    if plane.width == 0 || plane.height == 0 {
        return Err(invalid(format!("{name} is empty.")));
    }
    let scale = plane.width.max(plane.height).div_ceil(max_px);
    Ok((
        plane.width.div_ceil(scale),
        plane.height.div_ceil(scale),
        scale,
    ))
}

/// Box average of `scale` x `scale` source pixels per picture pixel, as 0 to 1 values.
fn reduce(plane: &Plane, w: usize, h: usize, scale: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(w * h);
    for oy in 0..h {
        for ox in 0..w {
            let (mut sum, mut n) = (0.0f64, 0u32);
            for y in oy * scale..((oy + 1) * scale).min(plane.height) {
                for x in ox * scale..((ox + 1) * scale).min(plane.width) {
                    sum += f64::from(plane.values[y * plane.width + x]);
                    n += 1;
                }
            }
            out.push((sum / f64::from(n)) as f32);
        }
    }
    out
}

pub fn overview(run_dir: &Path, name: &str, max_px: usize) -> WorkflowResult<PlaneOverview> {
    let plane = load(run_dir, name)?;
    let (w, h, scale) = overview_size(&plane, max_px, name)?;
    let pixels: Vec<u8> = reduce(&plane, w, h, scale).into_iter().map(gray).collect();
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

/// Logical planes the composite draws, in the order `composite_color` takes them.
const COMPOSITE_PLANES: [&str; 5] = [
    "logical-c.f32",
    "logical-m.f32",
    "logical-y.f32",
    "logical-k.f32",
    "logical-w.f32",
];

/// Brightness of the fixed backdrop the white ink sits on: mid-gray, so white ink is visible.
const BACKDROP: f64 = 0.5;

/// The simulation's arithmetic for one pixel from cyan, magenta, yellow, black and white coverage
/// (0 to 1). White ink lifts the backdrop toward white; each colored ink then multiplies what is
/// under it. This is a plain subtractive approximation, not color managed: it shows where ink goes
/// and how the layers stack, not what the print will look like.
pub fn composite_color(c: f32, m: f32, y: f32, k: f32, w: f32) -> [u8; 3] {
    let unit = |v: f32| f64::from(v).clamp(0.0, 1.0);
    let base = BACKDROP + (1.0 - BACKDROP) * unit(w);
    let under_k = 1.0 - unit(k);
    [c, m, y].map(|ink| gray((base * (1.0 - unit(ink)) * under_k) as f32))
}

/// A colour picture of the logical planes of one run, composed by `composite_color`. Every plane is
/// verified against the manifest first.
pub fn composite_overview(run_dir: &Path, max_px: usize) -> WorkflowResult<PlaneOverview> {
    let planes = COMPOSITE_PLANES
        .iter()
        .map(|name| load(run_dir, name))
        .collect::<WorkflowResult<Vec<_>>>()?;
    let first = &planes[0];
    if planes
        .iter()
        .any(|p| (p.width, p.height) != (first.width, first.height))
    {
        return Err(invalid(
            "The logical planes differ in size and cannot be composited.",
        ));
    }
    let (w, h, scale) = overview_size(first, max_px, "the composite")?;
    let r: Vec<Vec<f32>> = planes.iter().map(|p| reduce(p, w, h, scale)).collect();
    let mut pixels = Vec::with_capacity(w * h * 3);
    for i in 0..w * h {
        pixels.extend(composite_color(r[0][i], r[1][i], r[2][i], r[3][i], r[4][i]));
    }
    let png = veylora_image::encode_rgb8_png(w as u32, h as u32, &pixels).map_err(invalid)?;
    Ok(PlaneOverview {
        png,
        width: w,
        height: h,
        scale,
        source_width: first.width,
        source_height: first.height,
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

#[cfg(test)]
mod tests {
    use super::composite_color;

    #[test]
    fn bare_backdrop_white_ink_and_each_color_follow_the_stated_arithmetic() {
        assert_eq!(composite_color(0.0, 0.0, 0.0, 0.0, 0.0), [128, 128, 128]);
        assert_eq!(composite_color(0.0, 0.0, 0.0, 0.0, 1.0), [255, 255, 255]);
        // Full cyan over white ink removes red only; full black removes everything.
        assert_eq!(composite_color(1.0, 0.0, 0.0, 0.0, 1.0), [0, 255, 255]);
        assert_eq!(composite_color(0.0, 1.0, 0.0, 0.0, 1.0), [255, 0, 255]);
        assert_eq!(composite_color(0.0, 0.0, 1.0, 0.0, 1.0), [255, 255, 0]);
        assert_eq!(composite_color(0.0, 0.0, 0.0, 1.0, 1.0), [0, 0, 0]);
        // Half black over full white ink is mid-gray; out-of-range input is clamped, not wrapped.
        assert_eq!(composite_color(0.0, 0.0, 0.0, 0.5, 1.0), [128, 128, 128]);
        assert_eq!(composite_color(2.0, -1.0, 0.0, 0.0, 1.0), [0, 255, 255]);
    }
}
