use veylora_domain::recipe::{ScreenAlgorithm, ScreenPolicy};
use veylora_separation::Plane;

/// Cap on threshold-cell area so a recipe cannot demand an unbounded table.
pub const MAX_SCREEN_CELL_PIXELS: usize = 1 << 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageOrigin {
    pub x: i64,
    pub y: i64,
}
/// Binary ordered-threshold screen from the recipe, plus the code a printed dot takes.
///
/// A pixel prints a dot (`dot_level`) when its coverage, as a 16-bit integer, is strictly greater
/// than the threshold at its position; otherwise it prints nothing (0). The threshold cell repeats
/// from the recipe's phase in absolute page pixels, so any band of a page screens identically.
/// `seed` is retained in the recipe for future stochastic methods and is unused by this method.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenSpec {
    pub policy: ScreenPolicy,
    /// Nonzero code a printed dot takes in the device raster.
    pub dot_level: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenedBand {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u16>,
    pub origin: PageOrigin,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScreenError {
    InvalidLevels,
    InvalidPlane,
    /// Empty or oversize cell, wrong threshold count, or a threshold that could never print
    /// full coverage.
    InvalidScreen,
}

fn validate(spec: &ScreenSpec) -> Result<(usize, usize), ScreenError> {
    if spec.dot_level == 0 {
        return Err(ScreenError::InvalidLevels);
    }
    let p = &spec.policy;
    let (w, h) = (usize::from(p.cell_width), usize::from(p.cell_height));
    // Full coverage (65535) must print, so no threshold may equal it.
    if w == 0
        || h == 0
        || w * h > MAX_SCREEN_CELL_PIXELS
        || p.thresholds.len() != w * h
        || p.thresholds.contains(&u16::MAX)
    {
        return Err(ScreenError::InvalidScreen);
    }
    match p.algorithm {
        ScreenAlgorithm::OrderedThreshold => Ok((w, h)),
    }
}

pub fn screen_band(
    plane: &Plane,
    origin: PageOrigin,
    spec: &ScreenSpec,
) -> Result<ScreenedBand, ScreenError> {
    let (cell_w, cell_h) = validate(spec)?;
    plane.validate().map_err(|_| ScreenError::InvalidPlane)?;
    let p = &spec.policy;
    let mut pixels = Vec::with_capacity(plane.pixels.len());
    for y in 0..plane.height {
        let cy = (origin.y + y as i64 - i64::from(p.phase_y)).rem_euclid(cell_h as i64) as usize;
        for x in 0..plane.width {
            let cx =
                (origin.x + x as i64 - i64::from(p.phase_x)).rem_euclid(cell_w as i64) as usize;
            let threshold = p.thresholds[cy * cell_w + cx];
            let coverage =
                (plane.pixels[y * plane.width + x].clamp(0.0, 1.0) * 65535.0).round() as u16;
            pixels.push(if coverage > threshold {
                spec.dot_level
            } else {
                0
            });
        }
    }
    Ok(ScreenedBand {
        width: plane.width,
        height: plane.height,
        pixels,
        origin,
    })
}
