use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use veylora_screen::{screen_band, PageOrigin, ScreenError, ScreenSpec, ScreenedBand};
use veylora_separation::Plane;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderLimits {
    pub max_pixels: usize,
    pub band_height: usize,
}
/// Cooperative cancellation shared between the renderer and its owner. Clones observe the same flag.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderError {
    Cancelled,
    LimitExceeded,
    InvalidBandHeight,
    InvalidPlane,
    Screen(ScreenError),
}
impl From<ScreenError> for RenderError {
    fn from(value: ScreenError) -> Self {
        Self::Screen(value)
    }
}

pub fn render_bands(
    plane: &Plane,
    spec: &ScreenSpec,
    limits: RenderLimits,
    cancellation: &CancellationToken,
) -> Result<Vec<ScreenedBand>, RenderError> {
    plane.validate().map_err(|_| RenderError::InvalidPlane)?;
    if limits.band_height == 0 || plane.width.saturating_mul(plane.height) > limits.max_pixels {
        return Err(if limits.band_height == 0 {
            RenderError::InvalidBandHeight
        } else {
            RenderError::LimitExceeded
        });
    }
    let mut bands = Vec::new();
    let mut y = 0;
    while y < plane.height {
        if cancellation.is_cancelled() {
            return Err(RenderError::Cancelled);
        }
        let height = limits.band_height.min(plane.height - y);
        let pixels = plane.pixels[y * plane.width..(y + height) * plane.width].to_vec();
        let band = Plane::from_pixels(plane.width, height, pixels)
            .map_err(|_| RenderError::InvalidPlane)?;
        bands.push(screen_band(&band, PageOrigin { x: 0, y: y as i64 }, spec)?);
        y += height;
    }
    Ok(bands)
}
