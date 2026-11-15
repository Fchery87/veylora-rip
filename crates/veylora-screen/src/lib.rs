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
    if spec.supported_levels == 0 || spec.supported_levels > u16::MAX {
        return Err(ScreenError::InvalidLevels);
    }
    if plane.width == 0 || plane.height == 0 || plane.pixels.len() != plane.width * plane.height {
        return Err(ScreenError::InvalidPlane);
    }
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
