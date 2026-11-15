//! White generation with the declared precedence:
//! source -> combination -> response curve and density -> choke/spread -> clear mask.
use crate::{
    apply_response,
    clear_mask::ClearMask,
    morphology::{dilate, erode},
    PixelPitch, Plane, SeparationError,
};
use veylora_domain::{
    ids::AssetId,
    recipe::{MaskCombination, WhitePolicy, WhiteSource},
};

/// A supplied grayscale white mask and the asset it was imported as.
#[derive(Clone, Debug, PartialEq)]
pub struct SuppliedMask {
    pub id: AssetId,
    pub plane: Plane,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SeparationInput {
    pub alpha: Plane,
    /// Needs a color engine to derive; None makes a tonal white source an error.
    pub tonal: Option<Plane>,
    pub supplied_mask: Option<SuppliedMask>,
    pub clear_mask: Option<ClearMask>,
}
impl SuppliedMask {
    /// Imports a Gray8 asset as the supplied white mask it was loaded as.
    pub fn from_asset(asset: &veylora_image::SourceAsset) -> Result<Self, SeparationError> {
        Ok(Self {
            id: asset.id,
            plane: crate::source::mask_plane(asset)?,
        })
    }
}
pub fn generate_white(
    input: &SeparationInput,
    policy: &WhitePolicy,
    pitch: PixelPitch,
) -> Result<Plane, SeparationError> {
    if policy.response.len() < 2 || policy.density_percent > 100 {
        return Err(SeparationError::InvalidPolicy);
    }
    let radius = pitch.radius_px(policy.spread_um.unsigned_abs())?;
    let choke = policy.spread_um < 0;
    input.alpha.validate()?;
    if let Some(tonal) = &input.tonal {
        tonal.validate()?;
        if !tonal.same_size(&input.alpha) {
            return Err(SeparationError::DimensionMismatch);
        }
    }
    for optional in [
        input.supplied_mask.as_ref().map(|m| &m.plane),
        input.clear_mask.as_ref().map(ClearMask::retained),
    ]
    .into_iter()
    .flatten()
    {
        optional.validate()?;
        if !optional.same_size(&input.alpha) {
            return Err(SeparationError::DimensionMismatch);
        }
    }
    let supplied = input.supplied_mask.as_ref();
    let mut out = match &policy.source {
        WhiteSource::Alpha => input.alpha.clone(),
        WhiteSource::Tonal => input.tonal.clone().ok_or(SeparationError::MissingTonal)?,
        WhiteSource::Supplied { mask_id } => match supplied {
            None => return Err(SeparationError::MissingSuppliedMask),
            Some(mask) if mask.id != *mask_id => return Err(SeparationError::SuppliedMaskMismatch),
            Some(mask) => mask.plane.clone(),
        },
        WhiteSource::Disabled => Plane::new(input.alpha.width, input.alpha.height, 0.0),
    };
    // Combination merges a supplied mask into a generated source. With a supplied source or a
    // disabled source it is a no-op, so no policy field is silently ignored in a way that matters.
    if matches!(policy.source, WhiteSource::Alpha | WhiteSource::Tonal) {
        if let Some(mask) = supplied {
            for (v, s) in out.pixels.iter_mut().zip(&mask.plane.pixels) {
                *v = match policy.combine {
                    MaskCombination::Replace => *s,
                    MaskCombination::Union => v.max(*s),
                    MaskCombination::Intersection => v.min(*s),
                };
            }
        }
    }
    if !matches!(policy.source, WhiteSource::Disabled) {
        let density = f32::from(policy.density_percent) / 100.0;
        for v in &mut out.pixels {
            *v = (apply_response(&policy.response, *v) * density).clamp(0.0, 1.0);
        }
        // A response curve must not turn fully transparent pixels into ink unless the
        // operator deliberately supplied a mask that extends the white.
        if supplied.is_none() {
            for (v, a) in out.pixels.iter_mut().zip(&input.alpha.pixels) {
                if *a <= 0.0 {
                    *v = 0.0;
                }
            }
        }
    }
    if radius > 0 {
        out = if choke {
            erode(&out, radius)
        } else {
            dilate(&out, radius)
        };
    }
    if let Some(clear) = &input.clear_mask {
        for (v, c) in out.pixels.iter_mut().zip(&clear.retained().pixels) {
            *v *= c;
        }
    }
    Ok(out)
}
