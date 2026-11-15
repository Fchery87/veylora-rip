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
            &CancellationToken::new(),
        )
        .unwrap();
        let b = render_bands(
            &p,
            &s,
            RenderLimits {
                max_pixels: 100,
                band_height: 2,
            },
            &CancellationToken::new(),
        )
        .unwrap();
        let aa: Vec<_> = a.into_iter().flat_map(|x| x.pixels).collect();
        let bb: Vec<_> = b.into_iter().flat_map(|x| x.pixels).collect();
        assert_eq!(aa, bb);
    }

    fn plane(w: usize, h: usize) -> Plane {
        Plane::from_pixels(w, h, (0..w * h).map(|i| (i % 7) as f32 / 7.0).collect()).unwrap()
    }
    fn spec() -> ScreenSpec {
        ScreenSpec {
            method: ScreenMethod::OrderedBayer2,
            phase_origin: PageOrigin { x: 1, y: 1 },
            supported_levels: 255,
        }
    }
    fn limits(max_pixels: usize, band_height: usize) -> RenderLimits {
        RenderLimits {
            max_pixels,
            band_height,
        }
    }
    #[test]
    fn band_height_does_not_change_pixels_for_any_split() {
        let p = plane(5, 11);
        let whole: Vec<_> = render_bands(&p, &spec(), limits(100, 11), &CancellationToken::new())
            .unwrap()
            .into_iter()
            .flat_map(|b| b.pixels)
            .collect();
        for h in 1..=11 {
            let split: Vec<_> =
                render_bands(&p, &spec(), limits(100, h), &CancellationToken::new())
                    .unwrap()
                    .into_iter()
                    .flat_map(|b| b.pixels)
                    .collect();
            assert_eq!(whole, split, "band height {h}");
        }
    }
    #[test]
    fn rejects_oversize_zero_band_and_invalid_planes() {
        let t = CancellationToken::new();
        assert_eq!(
            render_bands(&plane(4, 4), &spec(), limits(15, 2), &t),
            Err(RenderError::LimitExceeded)
        );
        assert_eq!(
            render_bands(&plane(4, 4), &spec(), limits(100, 0), &t),
            Err(RenderError::InvalidBandHeight)
        );
        let mut nan = plane(2, 2);
        nan.pixels[0] = f32::NAN;
        assert_eq!(
            render_bands(&nan, &spec(), limits(100, 1), &t),
            Err(RenderError::InvalidPlane)
        );
        let empty = Plane::new(0, 0, 0.0);
        assert_eq!(
            render_bands(&empty, &spec(), limits(100, 1), &t),
            Err(RenderError::InvalidPlane)
        );
    }
    #[test]
    fn cancellation_from_a_clone_stops_rendering() {
        let t = CancellationToken::new();
        let owner = t.clone();
        owner.cancel();
        assert_eq!(
            render_bands(&plane(4, 4), &spec(), limits(100, 1), &t),
            Err(RenderError::Cancelled)
        );
    }
}
