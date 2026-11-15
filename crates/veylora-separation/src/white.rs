//! White generation with the declared precedence:
//! source -> combination -> response curve and density -> choke/spread -> clear mask.
use crate::{
    morphology::{dilate, erode, MAX_MORPHOLOGY_RADIUS_PX},
    Plane, SeparationError,
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
    pub tonal: Plane,
    pub supplied_mask: Option<SuppliedMask>,
    pub clear_mask: Option<Plane>,
}
/// Physical size of one raster pixel in micrometres, used to turn `spread_um` into pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelPitch {
    pub x_um: u32,
    pub y_um: u32,
}

fn radius_px(spread_um: i32, pitch: PixelPitch) -> Result<(usize, bool), SeparationError> {
    if pitch.x_um == 0 || pitch.y_um == 0 {
        return Err(SeparationError::InvalidPolicy);
    }
    if pitch.x_um != pitch.y_um {
        return Err(SeparationError::AnisotropicPitch);
    }
    let magnitude = u64::from(spread_um.unsigned_abs());
    let pitch = u64::from(pitch.x_um);
    // Round half up so the radius never silently drops a requested whole pixel.
    let radius = (magnitude * 2 + pitch) / (pitch * 2);
    if radius > MAX_MORPHOLOGY_RADIUS_PX as u64 {
        return Err(SeparationError::InvalidPolicy);
    }
    Ok((radius as usize, spread_um < 0))
}

fn response(curve: &[u16], value: f32) -> f32 {
    let last = curve.len() - 1;
    let position = value.clamp(0.0, 1.0) * last as f32;
    let index = (position.floor() as usize).min(last - 1);
    let fraction = position - index as f32;
    let (a, b) = (f32::from(curve[index]), f32::from(curve[index + 1]));
    ((a + (b - a) * fraction) / f32::from(u16::MAX)).clamp(0.0, 1.0)
}

pub fn generate_white(
    input: &SeparationInput,
    policy: &WhitePolicy,
    pitch: PixelPitch,
) -> Result<Plane, SeparationError> {
    if policy.response.len() < 2 || policy.density_percent > 100 {
        return Err(SeparationError::InvalidPolicy);
    }
    let (radius, choke) = radius_px(policy.spread_um, pitch)?;
    input.alpha.validate()?;
    input.tonal.validate()?;
    if !input.alpha.same_size(&input.tonal) {
        return Err(SeparationError::DimensionMismatch);
    }
    for optional in [
        input.supplied_mask.as_ref().map(|m| &m.plane),
        input.clear_mask.as_ref(),
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
        WhiteSource::Tonal => input.tonal.clone(),
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
            *v = (response(&policy.response, *v) * density).clamp(0.0, 1.0);
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
        for (v, c) in out.pixels.iter_mut().zip(&clear.pixels) {
            *v *= c.clamp(0.0, 1.0);
        }
    }
    Ok(out)
}
