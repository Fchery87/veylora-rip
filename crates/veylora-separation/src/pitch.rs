use crate::{morphology::MAX_MORPHOLOGY_RADIUS_PX, SeparationError};

/// Physical size of one raster pixel in nanometres. Nanometres keep real printer resolutions
/// accurate: 1440 dpi is 17 639 nm, where whole micrometres would be off by about 2%.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelPitch {
    pub x_nm: u32,
    pub y_nm: u32,
}
impl PixelPitch {
    /// Pitch for a raster resolution, rounded to the nearest nanometre. None for zero dpi.
    pub fn from_dpi(dpi_x: u32, dpi_y: u32) -> Option<Self> {
        let nm = |dpi: u32| {
            (dpi != 0).then(|| ((25_400_000u64 * 2 + u64::from(dpi)) / (u64::from(dpi) * 2)) as u32)
        };
        Some(Self {
            x_nm: nm(dpi_x)?,
            y_nm: nm(dpi_y)?,
        })
    }
    /// The single pitch used by geometry operations. Zero or unequal pitches are rejected;
    /// anisotropic geometry is never approximated.
    pub(crate) fn isotropic_nm(self) -> Result<u64, SeparationError> {
        if self.x_nm == 0 || self.y_nm == 0 {
            return Err(SeparationError::InvalidPolicy);
        }
        if self.x_nm != self.y_nm {
            return Err(SeparationError::AnisotropicPitch);
        }
        Ok(u64::from(self.x_nm))
    }
    /// Pixels for a physical distance in micrometres, rounded half up and bounded by the
    /// morphology limit.
    pub fn radius_px(self, um: u32) -> Result<usize, SeparationError> {
        let pitch = self.isotropic_nm()?;
        let radius = (u64::from(um) * 1000 * 2 + pitch) / (pitch * 2);
        if radius > MAX_MORPHOLOGY_RADIUS_PX as u64 {
            return Err(SeparationError::InvalidPolicy);
        }
        Ok(radius as usize)
    }
}
