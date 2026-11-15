//! One final clear mask shared by every plane. Values are the retained fraction:
//! 1.0 keeps ink, 0.0 clears it. Masks are separate from the artwork, so disabling one never
//! alters the source.
use crate::{LogicalPlanes, PhysicalPlanes};
use crate::{PixelPitch, Plane, SeparationError};
use veylora_image::{SampleLayout, SourceAsset};

pub use crate::perforation::PerforationRule;

#[derive(Clone, Debug, PartialEq)]
pub struct ClearMask(Plane);
impl ClearMask {
    /// Retains everything.
    pub fn open(width: usize, height: usize) -> Self {
        Self(Plane::new(width, height, 1.0))
    }
    pub fn from_plane(plane: Plane) -> Result<Self, SeparationError> {
        plane.validate()?;
        if plane.pixels.iter().any(|v| !(0.0..=1.0).contains(v)) {
            return Err(SeparationError::InvalidPlane);
        }
        Ok(Self(plane))
    }
    pub fn retained(&self) -> &Plane {
        &self.0
    }
    /// Intersection of clears: a pixel is retained only if both masks retain it.
    pub fn combine(&self, other: &ClearMask) -> Result<ClearMask, SeparationError> {
        if !self.0.same_size(&other.0) {
            return Err(SeparationError::DimensionMismatch);
        }
        let pixels = self
            .0
            .pixels
            .iter()
            .zip(&other.0.pixels)
            .map(|(a, b)| a.min(*b));
        Ok(Self(Plane {
            width: self.0.width,
            height: self.0.height,
            pixels: pixels.collect(),
        }))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KnockoutTarget {
    /// Source-encoded black, for black and near-black knockout.
    Black,
    /// A selected source color in the source's own 8-bit encoding. Garment color is a separate
    /// concept and is not modeled here.
    Color { rgb: [u8; 3] },
}
/// Clears pixels whose source color is within `tolerance` of the target, ramping back to fully
/// retained over `transition`. Distance is the largest per-channel difference, scaled to 0..=1,
/// in the source encoding (no color management is applied).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KnockoutRule {
    pub target: KnockoutTarget,
    pub tolerance: f32,
    /// Width of the soft edge. Zero is a hard clip.
    pub transition: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub enum MaskRule {
    Knockout(KnockoutRule),
    Perforation(PerforationRule),
}
pub struct MaskInput<'a> {
    pub source: &'a SourceAsset,
    /// Absolute page position of the source's top-left pixel, so holes stay phase-stable.
    pub origin_px: (i64, i64),
    pub pitch: PixelPitch,
}

pub(crate) fn pitch_um(pitch: PixelPitch) -> Result<u32, SeparationError> {
    if pitch.x_um == 0 || pitch.y_um == 0 {
        return Err(SeparationError::InvalidPolicy);
    }
    if pitch.x_um != pitch.y_um {
        return Err(SeparationError::AnisotropicPitch);
    }
    Ok(pitch.x_um)
}

fn knockout(source: &SourceAsset, rule: &KnockoutRule) -> Result<Plane, SeparationError> {
    let unit = |v: f32| v.is_finite() && (0.0..=1.0).contains(&v);
    if !unit(rule.tolerance) || !unit(rule.transition) {
        return Err(SeparationError::InvalidRule);
    }
    let alpha = crate::source::alpha_plane(source)?;
    let (n, has_alpha) = match source.decoded.layout {
        SampleLayout::Gray8 => (1, false),
        SampleLayout::GrayAlpha8 => (2, true),
        SampleLayout::Rgb8 => (3, false),
        SampleLayout::Rgba8 => (4, true),
    };
    let target = match rule.target {
        KnockoutTarget::Black => [0, 0, 0],
        KnockoutTarget::Color { rgb } => rgb,
    };
    let mut out = Plane::new(alpha.width, alpha.height, 1.0);
    for (i, px) in source.decoded.samples.chunks_exact(n).enumerate() {
        // Hidden color under fully transparent pixels is not visible artwork.
        if has_alpha && alpha.pixels[i] <= 0.0 {
            continue;
        }
        let rgb = if n <= 2 {
            [px[0]; 3]
        } else {
            [px[0], px[1], px[2]]
        };
        let distance = rgb
            .iter()
            .zip(target)
            .map(|(a, b)| a.abs_diff(b))
            .max()
            .unwrap_or(0) as f32
            / 255.0;
        out.pixels[i] = if distance <= rule.tolerance {
            0.0
        } else if rule.transition == 0.0 {
            1.0
        } else {
            ((distance - rule.tolerance) / rule.transition).min(1.0)
        };
    }
    Ok(out)
}

/// Builds one clear mask from all rules. Every rule can only clear more, never retain again.
pub fn build_clear_mask(
    input: &MaskInput<'_>,
    rules: &[MaskRule],
) -> Result<ClearMask, SeparationError> {
    let base = crate::source::alpha_plane(input.source)?;
    let mut mask = ClearMask::open(base.width, base.height);
    for rule in rules {
        let plane = match rule {
            MaskRule::Knockout(k) => knockout(input.source, k)?,
            MaskRule::Perforation(p) => {
                crate::perforation::holes(p, base.width, base.height, input)?
            }
        };
        mask = mask.combine(&ClearMask::from_plane(plane)?)?;
    }
    Ok(mask)
}

/// Plane collections that must all honor the clear mask.
pub trait PlaneSet {
    fn planes_mut(&mut self) -> Vec<&mut Plane>;
}
impl PlaneSet for LogicalPlanes {
    fn planes_mut(&mut self) -> Vec<&mut Plane> {
        vec![
            &mut self.c,
            &mut self.m,
            &mut self.y,
            &mut self.k,
            &mut self.w,
        ]
    }
}
impl PlaneSet for PhysicalPlanes {
    fn planes_mut(&mut self) -> Vec<&mut Plane> {
        self.slots.values_mut().map(|a| &mut a.plane).collect()
    }
}
/// Final enforcement over every plane. Sizes are checked for all planes before any is changed.
pub fn enforce_clear_mask<P: PlaneSet>(
    planes: &mut P,
    clear: &ClearMask,
) -> Result<(), SeparationError> {
    let mut all = planes.planes_mut();
    if all.iter().any(|p| !p.same_size(&clear.0)) {
        return Err(SeparationError::DimensionMismatch);
    }
    for plane in &mut all {
        for (v, keep) in plane.pixels.iter_mut().zip(&clear.0.pixels) {
            *v *= keep;
        }
    }
    Ok(())
}
