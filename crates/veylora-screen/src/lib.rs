use veylora_separation::Plane;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageOrigin {
    pub x: i64,
    pub y: i64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenMethod {
    OrderedBayer2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenSpec {
    pub method: ScreenMethod,
    pub phase_origin: PageOrigin,
    pub supported_levels: u16,
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
}

pub fn screen_band(
    plane: &Plane,
    origin: PageOrigin,
    spec: &ScreenSpec,
) -> Result<ScreenedBand, ScreenError> {
    if spec.supported_levels == 0 {
        return Err(ScreenError::InvalidLevels);
    }
    plane.validate().map_err(|_| ScreenError::InvalidPlane)?;
    let mut pixels = Vec::with_capacity(plane.pixels.len());
    let matrix = [[0u16, 2], [3, 1]];
    for y in 0..plane.height {
        for x in 0..plane.width {
            let px = (origin.x + x as i64 - spec.phase_origin.x).rem_euclid(2) as usize;
            let py = (origin.y + y as i64 - spec.phase_origin.y).rem_euclid(2) as usize;
            let threshold = matrix[py][px] as f32 / 4.0;
            let value = plane.pixels[y * plane.width + x].clamp(0.0, 1.0);
            pixels.push(if value > threshold {
                spec.supported_levels
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

#[cfg(test)]
mod tests {
    use super::*;
    fn spec() -> ScreenSpec {
        ScreenSpec {
            method: ScreenMethod::OrderedBayer2,
            phase_origin: PageOrigin { x: 0, y: 0 },
            supported_levels: 255,
        }
    }
    fn origin() -> PageOrigin {
        PageOrigin { x: 0, y: 0 }
    }
    #[test]
    fn extremes_map_to_off_and_full_levels() {
        let off = Plane::new(4, 4, 0.0);
        let on = Plane::new(4, 4, 1.0);
        assert!(screen_band(&off, origin(), &spec())
            .unwrap()
            .pixels
            .iter()
            .all(|p| *p == 0));
        assert!(screen_band(&on, origin(), &spec())
            .unwrap()
            .pixels
            .iter()
            .all(|p| *p == 255));
    }
    #[test]
    fn output_depends_on_absolute_page_position_not_band_position() {
        let p = Plane::new(2, 2, 0.5);
        let a = screen_band(&p, PageOrigin { x: 0, y: 0 }, &spec()).unwrap();
        let b = screen_band(&p, PageOrigin { x: 1, y: 0 }, &spec()).unwrap();
        assert_ne!(a.pixels, b.pixels);
        let c = screen_band(&p, PageOrigin { x: 2, y: 2 }, &spec()).unwrap();
        assert_eq!(a.pixels, c.pixels);
    }
    #[test]
    fn rejects_invalid_levels_and_planes_and_is_deterministic() {
        let p = Plane::new(3, 3, 0.4);
        let zero = ScreenSpec {
            supported_levels: 0,
            ..spec()
        };
        assert_eq!(
            screen_band(&p, origin(), &zero),
            Err(ScreenError::InvalidLevels)
        );
        let mut nan = p.clone();
        nan.pixels[4] = f32::NAN;
        assert_eq!(
            screen_band(&nan, origin(), &spec()),
            Err(ScreenError::InvalidPlane)
        );
        assert_eq!(
            screen_band(&p, origin(), &spec()),
            screen_band(&p, origin(), &spec())
        );
    }
}
