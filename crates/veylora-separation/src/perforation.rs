//! Deterministic perforation in physical units. Holes sit on a square grid anchored to absolute
//! page coordinates, so any band or tile of a page gets the same holes.
use crate::{
    clear_mask::{pitch_um, MaskInput},
    morphology::{erode, MAX_MORPHOLOGY_RADIUS_PX},
    Plane, SeparationError,
};
use veylora_domain::recipe::PerforationPolicy;

/// Erodes eligibility treating everything outside the plane as ineligible, so the page edge is
/// protected the same way as the edge of an eligible region.
fn erode_with_edge(plane: &Plane, r: usize) -> Plane {
    let (pw, ph) = (plane.width + 2 * r, plane.height + 2 * r);
    let mut padded = Plane::new(pw, ph, 0.0);
    for y in 0..plane.height {
        for x in 0..plane.width {
            padded.set(x + r, y + r, plane.get(x, y));
        }
    }
    let eroded = erode(&padded, r);
    let mut out = Plane::new(plane.width, plane.height, 0.0);
    for y in 0..plane.height {
        for x in 0..plane.width {
            out.set(x, y, eroded.get(x + r, y + r));
        }
    }
    out
}

pub(crate) fn holes(
    rule: &PerforationPolicy,
    width: usize,
    height: usize,
    input: &MaskInput<'_>,
) -> Result<Plane, SeparationError> {
    let pitch = u64::from(pitch_um(input.pitch)?);
    if rule.diameter_um == 0 || rule.spacing_um <= rule.diameter_um {
        return Err(SeparationError::InvalidRule);
    }
    if rule.edge_protection_um > 0 && rule.eligible_mask_id.is_none() {
        return Err(SeparationError::InvalidRule);
    }
    let mut eligible = match (rule.eligible_mask_id, input.eligible_mask) {
        (None, _) => Plane::new(width, height, 1.0),
        (Some(_), None) => return Err(SeparationError::MissingSuppliedMask),
        (Some(id), Some(mask)) if mask.id != id => {
            return Err(SeparationError::SuppliedMaskMismatch)
        }
        (Some(_), Some(mask)) => {
            mask.plane.validate()?;
            if mask.plane.width != width || mask.plane.height != height {
                return Err(SeparationError::DimensionMismatch);
            }
            mask.plane.clone()
        }
    };
    let protect = (u64::from(rule.edge_protection_um) * 2 + pitch) / (pitch * 2);
    if protect > MAX_MORPHOLOGY_RADIUS_PX as u64 {
        return Err(SeparationError::InvalidRule);
    }
    if protect > 0 {
        eligible = erode_with_edge(&eligible, protect as usize);
    }
    let (spacing, diameter) = (i128::from(rule.spacing_um), i128::from(rule.diameter_um));
    let phase = (
        i128::from(rule.phase_x_px) * i128::from(pitch),
        i128::from(rule.phase_y_px) * i128::from(pitch),
    );
    let axis = |index: usize, origin: i64, phase: i128| {
        let center =
            (i128::from(origin) + index as i128) * i128::from(pitch) + i128::from(pitch) / 2;
        let r = (center - phase).rem_euclid(spacing);
        r.min(spacing - r)
    };
    let mut out = Plane::new(width, height, 1.0);
    for y in 0..height {
        let dy = axis(y, input.origin_px.1, phase.1);
        for x in 0..width {
            let dx = axis(x, input.origin_px.0, phase.0);
            // Pixel center within the hole radius (compared as 4*r^2 <= d^2 in integers).
            if 4 * (dx * dx + dy * dy) <= diameter * diameter {
                out.pixels[y * width + x] = 1.0 - eligible.pixels[y * width + x];
            }
        }
    }
    Ok(out)
}
