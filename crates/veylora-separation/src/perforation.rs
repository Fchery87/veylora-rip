//! Deterministic perforation in physical units. Holes sit on a square grid anchored to absolute
//! page coordinates, so any band or tile of a page gets the same holes.
use crate::{
    clear_mask::{pitch_um, MaskInput},
    morphology::{erode, MAX_MORPHOLOGY_RADIUS_PX},
    Plane, SeparationError,
};

#[derive(Clone, Debug, PartialEq)]
pub struct PerforationRule {
    pub diameter_um: u32,
    /// Center-to-center distance. Must exceed the diameter so holes never overlap.
    pub spacing_um: u32,
    /// Grid anchor in absolute page pixels.
    pub phase_px: (i64, i64),
    /// Areas where holes may be punched (coverage 0..=1). None makes the whole source eligible.
    pub eligible: Option<Plane>,
    /// Keeps holes this far from the edge of the eligible area. Requires `eligible`.
    pub edge_protection_um: u32,
}

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
    rule: &PerforationRule,
    width: usize,
    height: usize,
    input: &MaskInput<'_>,
) -> Result<Plane, SeparationError> {
    let pitch = u64::from(pitch_um(input.pitch)?);
    if rule.diameter_um == 0 || rule.spacing_um <= rule.diameter_um {
        return Err(SeparationError::InvalidRule);
    }
    if rule.edge_protection_um > 0 && rule.eligible.is_none() {
        return Err(SeparationError::InvalidRule);
    }
    let mut eligible = match &rule.eligible {
        Some(p) => {
            p.validate()?;
            if p.width != width || p.height != height {
                return Err(SeparationError::DimensionMismatch);
            }
            p.clone()
        }
        None => Plane::new(width, height, 1.0),
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
        i128::from(rule.phase_px.0) * i128::from(pitch),
        i128::from(rule.phase_px.1) * i128::from(pitch),
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
