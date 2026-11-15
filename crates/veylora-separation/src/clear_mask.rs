//! One final clear mask shared by every plane. Values are the retained fraction:
//! 1.0 keeps ink, 0.0 clears it. Masks are separate from the artwork, so disabling one never
//! alters the source.
use crate::{LogicalPlanes, PhysicalPlanes, PixelPitch, Plane, SeparationError, SuppliedMask};
use veylora_domain::recipe::{ClearPolicy, KnockoutPolicy, KnockoutTarget};
use veylora_image::{SampleLayout, SourceAsset};

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

pub struct MaskInput<'a> {
    pub source: &'a SourceAsset,
    /// Source rows to build the mask for; None means the whole source. An eligible mask, if the
    /// policy needs one, must cover exactly these rows.
    pub rows: Option<std::ops::Range<usize>>,
    /// The mask the policy names as the perforation eligible area, if it names one.
    pub eligible_mask: Option<&'a SuppliedMask>,
    /// Absolute page position of the source's top-left pixel (row 0), so holes stay phase-stable.
    pub origin_px: (i64, i64),
    pub pitch: PixelPitch,
}

fn knockout(
    source: &SourceAsset,
    rows: &std::ops::Range<usize>,
    rule: &KnockoutPolicy,
) -> Result<Plane, SeparationError> {
    if rule.tolerance_permille > 1000 || rule.transition_permille > 1000 {
        return Err(SeparationError::InvalidRule);
    }
    let alpha = crate::source::alpha_rows(source, rows.clone())?;
    let (_, samples, _) = crate::source::band(source, rows)?;
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
    // Distances are compared as integers: diff/255 <= permille/1000  <=>  diff*1000 <= permille*255.
    let tolerance = u32::from(rule.tolerance_permille) * 255;
    let transition = u32::from(rule.transition_permille) * 255;
    let mut out = Plane::new(alpha.width, alpha.height, 1.0);
    for (i, px) in samples.chunks_exact(n).enumerate() {
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
            .map(|(a, b)| u32::from(a.abs_diff(b)))
            .max()
            .unwrap_or(0)
            * 1000;
        out.pixels[i] = if distance <= tolerance {
            0.0
        } else if transition == 0 {
            1.0
        } else {
            ((distance - tolerance) as f32 / transition as f32).min(1.0)
        };
    }
    Ok(out)
}

/// Builds one clear mask from the recipe's policy. Every rule can only clear more.
pub fn build_clear_mask(
    input: &MaskInput<'_>,
    policy: &ClearPolicy,
) -> Result<ClearMask, SeparationError> {
    let height = input.source.metadata.height as usize;
    let rows = input.rows.clone().unwrap_or(0..height);
    let base = crate::source::alpha_rows(input.source, rows.clone())?;
    let mut mask = ClearMask::open(base.width, base.height);
    for rule in &policy.knockouts {
        mask = mask.combine(&ClearMask::from_plane(knockout(
            input.source,
            &rows,
            rule,
        )?)?)?;
    }
    if let Some(rule) = &policy.perforation {
        let holes = crate::perforation::holes(rule, base.width, base.height, rows.start, input)?;
        mask = mask.combine(&ClearMask::from_plane(holes)?)?;
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
