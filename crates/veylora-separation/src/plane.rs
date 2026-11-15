use serde::{Deserialize, Serialize};

/// Normalized coverage samples in row-major order. Valid planes are non-empty, consistently
/// sized and finite.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<f32>,
}
impl Plane {
    pub fn new(width: usize, height: usize, value: f32) -> Self {
        Self {
            width,
            height,
            pixels: vec![value; width.saturating_mul(height)],
        }
    }
    pub fn from_pixels(
        width: usize,
        height: usize,
        pixels: Vec<f32>,
    ) -> Result<Self, SeparationError> {
        if pixels.len() != width.saturating_mul(height) {
            return Err(SeparationError::DimensionMismatch);
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }
    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.pixels[y * self.width + x]
    }
    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        self.pixels[y * self.width + x] = value.clamp(0.0, 1.0);
    }
    pub fn validate(&self) -> Result<(), SeparationError> {
        if self.width == 0
            || self.height == 0
            || self.width.checked_mul(self.height) != Some(self.pixels.len())
            || self.pixels.iter().any(|v| !v.is_finite())
        {
            Err(SeparationError::InvalidPlane)
        } else {
            Ok(())
        }
    }
    pub(crate) fn same_size(&self, other: &Plane) -> bool {
        self.width == other.width && self.height == other.height
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeparationError {
    DimensionMismatch,
    InvalidPlane,
    /// The policy names a supplied mask but none was provided.
    MissingSuppliedMask,
    /// A supplied mask was provided, but not the one the policy references.
    SuppliedMaskMismatch,
    InvalidPolicy,
    /// A knockout or perforation rule has non-finite, out-of-range or inconsistent values.
    InvalidRule,
    /// The tonal plane needs a color engine and was not supplied.
    MissingTonal,
    /// Decoded samples do not match the declared dimensions and layout.
    MalformedSource,
    /// The source layout cannot be used for this purpose (for example an RGB supplied mask).
    UnsupportedSource,
    /// Choke/spread needs equal horizontal and vertical pixel pitch; it is never approximated.
    AnisotropicPitch,
}
