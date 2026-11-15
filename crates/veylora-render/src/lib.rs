use veylora_screen::{screen_band, PageOrigin, ScreenError, ScreenSpec, ScreenedBand};
use veylora_separation::Plane;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderLimits {
    pub max_pixels: usize,
    pub band_height: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CancellationToken {
    cancelled: bool,
}
impl CancellationToken {
    pub const fn new() -> Self {
        Self { cancelled: false }
    }
    pub const fn cancelled() -> Self {
        Self { cancelled: true }
    }
    pub const fn is_cancelled(self) -> bool {
        self.cancelled
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderError {
    Cancelled,
    LimitExceeded,
    InvalidBandHeight,
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
    cancellation: CancellationToken,
) -> Result<Vec<ScreenedBand>, RenderError> {
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
            .map_err(|_| RenderError::LimitExceeded)?;
        bands.push(screen_band(&band, PageOrigin { x: 0, y: y as i64 }, spec)?);
        y += height;
    }
    Ok(bands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use veylora_screen::{PageOrigin, ScreenMethod};
    #[test]
    fn bands_concatenate_to_full_frame() {
        let p = Plane::from_pixels(4, 5, (0..20).map(|i| i as f32 / 20.0).collect()).unwrap();
        let s = ScreenSpec {
            method: ScreenMethod::OrderedBayer2,
            phase_origin: PageOrigin { x: 0, y: 0 },
            supported_levels: 255,
        };
        let a = render_bands(
            &p,
            &s,
            RenderLimits {
                max_pixels: 100,
                band_height: 5,
            },
            CancellationToken::new(),
        )
        .unwrap();
        let b = render_bands(
            &p,
            &s,
            RenderLimits {
                max_pixels: 100,
                band_height: 2,
            },
            CancellationToken::new(),
        )
        .unwrap();
        let aa: Vec<_> = a.into_iter().flat_map(|x| x.pixels).collect();
        let bb: Vec<_> = b.into_iter().flat_map(|x| x.pixels).collect();
        assert_eq!(aa, bb);
    }
}
