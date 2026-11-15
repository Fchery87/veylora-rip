use serde::{Deserialize, Serialize};

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
            || self.pixels.len() != self.width * self.height
            || self.pixels.iter().any(|v| !v.is_finite())
        {
            Err(SeparationError::InvalidPlane)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhiteSource {
    Alpha,
    Tonal,
    SuppliedMask,
    Disabled,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum MaskCombination {
    Replace,
    UnionMax,
    IntersectionMin,
    Blend { supplied_weight: f32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurveId {
    Identity,
    Invert,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WhitePolicy {
    pub source: WhiteSource,
    pub combination: MaskCombination,
    pub curve: CurveId,
    pub choke_px: usize,
    pub spread_px: usize,
    pub max_coverage: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SeparationInput {
    pub alpha: Plane,
    pub tonal: Plane,
    pub supplied_mask: Option<Plane>,
    pub clear_mask: Option<Plane>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeparationError {
    DimensionMismatch,
    InvalidPlane,
    MissingSuppliedMask,
    InvalidPolicy,
}

/// Upper bound on choke/spread radius so morphology cost stays bounded (O(w*h*r^2)).
pub const MAX_MORPHOLOGY_RADIUS_PX: usize = 64;

fn validate_policy(policy: &WhitePolicy) -> Result<(), SeparationError> {
    let blend_ok = match policy.combination {
        MaskCombination::Blend { supplied_weight } => supplied_weight.is_finite(),
        _ => true,
    };
    if !blend_ok
        || !policy.max_coverage.is_finite()
        || policy.choke_px > MAX_MORPHOLOGY_RADIUS_PX
        || policy.spread_px > MAX_MORPHOLOGY_RADIUS_PX
    {
        return Err(SeparationError::InvalidPolicy);
    }
    Ok(())
}

pub fn generate_white(
    input: &SeparationInput,
    policy: &WhitePolicy,
) -> Result<Plane, SeparationError> {
    validate_policy(policy)?;
    input.alpha.validate()?;
    input.tonal.validate()?;
    if input.alpha.width != input.tonal.width || input.alpha.height != input.tonal.height {
        return Err(SeparationError::DimensionMismatch);
    }
    for optional in [&input.supplied_mask, &input.clear_mask]
        .into_iter()
        .flatten()
    {
        optional.validate()?;
        if optional.width != input.alpha.width || optional.height != input.alpha.height {
            return Err(SeparationError::DimensionMismatch);
        }
    }
    let base = match policy.source {
        WhiteSource::Alpha => input.alpha.clone(),
        WhiteSource::Tonal => input.tonal.clone(),
        WhiteSource::SuppliedMask => input
            .supplied_mask
            .clone()
            .ok_or(SeparationError::MissingSuppliedMask)?,
        WhiteSource::Disabled => Plane::new(input.alpha.width, input.alpha.height, 0.0),
    };
    if base.width != input.alpha.width || base.height != input.alpha.height {
        return Err(SeparationError::DimensionMismatch);
    }
    let supplied = input.supplied_mask.as_ref();
    let mut out = base;
    if !matches!(policy.source, WhiteSource::Disabled) {
        for i in 0..out.pixels.len() {
            out.pixels[i] = match (policy.combination, supplied) {
                (MaskCombination::Replace, Some(s)) => s.pixels[i],
                (MaskCombination::UnionMax, Some(s)) => out.pixels[i].max(s.pixels[i]),
                (MaskCombination::IntersectionMin, Some(s)) => out.pixels[i].min(s.pixels[i]),
                (MaskCombination::Blend { supplied_weight }, Some(s)) => {
                    out.pixels[i] * (1.0 - supplied_weight.clamp(0.0, 1.0))
                        + s.pixels[i] * supplied_weight.clamp(0.0, 1.0)
                }
                (_, None) => out.pixels[i],
            }
            .clamp(0.0, 1.0);
        }
    }
    if matches!(policy.curve, CurveId::Invert) {
        for v in &mut out.pixels {
            *v = 1.0 - *v;
        }
    }
    if policy.choke_px > 0 {
        out = erode(&out, policy.choke_px);
    }
    if policy.spread_px > 0 {
        out = dilate(&out, policy.spread_px);
    }
    for v in &mut out.pixels {
        *v = v.min(policy.max_coverage.clamp(0.0, 1.0));
    }
    if let Some(clear) = &input.clear_mask {
        if clear.width != out.width || clear.height != out.height {
            return Err(SeparationError::DimensionMismatch);
        }
        for (v, c) in out.pixels.iter_mut().zip(&clear.pixels) {
            *v *= c.clamp(0.0, 1.0);
        }
    }
    Ok(out)
}

fn neighborhood(plane: &Plane, x: usize, y: usize, radius: usize, max: bool) -> f32 {
    let x0 = x.saturating_sub(radius);
    let y0 = y.saturating_sub(radius);
    let x1 = (x + radius).min(plane.width - 1);
    let y1 = (y + radius).min(plane.height - 1);
    let mut value: f32 = if max { 0.0 } else { 1.0 };
    for yy in y0..=y1 {
        for xx in x0..=x1 {
            value = if max {
                value.max(plane.get(xx, yy))
            } else {
                value.min(plane.get(xx, yy))
            };
        }
    }
    value
}
fn dilate(p: &Plane, r: usize) -> Plane {
    let mut o = p.clone();
    for y in 0..p.height {
        for x in 0..p.width {
            o.set(x, y, neighborhood(p, x, y, r, true));
        }
    }
    o
}
fn erode(p: &Plane, r: usize) -> Plane {
    let mut o = p.clone();
    for y in 0..p.height {
        for x in 0..p.width {
            o.set(x, y, neighborhood(p, x, y, r, false));
        }
    }
    o
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalSlot {
    pub protocol_channel: String,
    pub logical_channel: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogicalChannelMap {
    pub slots: Vec<PhysicalSlot>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AllocationError {
    MissingChannel(String),
    DuplicateProtocolChannel,
    DimensionMismatch,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LogicalPlanes {
    pub c: Plane,
    pub m: Plane,
    pub y: Plane,
    pub k: Plane,
    pub w: Plane,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PhysicalPlanes {
    pub slots: std::collections::BTreeMap<String, Plane>,
}
pub fn allocate_logical_to_physical(
    logical: &LogicalPlanes,
    map: &LogicalChannelMap,
) -> Result<PhysicalPlanes, AllocationError> {
    let planes = [&logical.c, &logical.m, &logical.y, &logical.k, &logical.w];
    if planes
        .iter()
        .any(|p| p.width != logical.c.width || p.height != logical.c.height)
    {
        return Err(AllocationError::DimensionMismatch);
    }
    let mut out = std::collections::BTreeMap::new();
    for slot in &map.slots {
        let plane = match slot.logical_channel.as_str() {
            "C" => &logical.c,
            "M" => &logical.m,
            "Y" => &logical.y,
            "K" => &logical.k,
            "W" => &logical.w,
            other => return Err(AllocationError::MissingChannel(other.to_owned())),
        };
        if out
            .insert(slot.protocol_channel.clone(), plane.clone())
            .is_some()
        {
            return Err(AllocationError::DuplicateProtocolChannel);
        }
    }
    Ok(PhysicalPlanes { slots: out })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn morphology_is_bounded() {
        let p = Plane::from_pixels(3, 1, vec![0.0, 1.0, 0.0]).unwrap();
        let i = SeparationInput {
            alpha: p.clone(),
            tonal: p.clone(),
            supplied_mask: None,
            clear_mask: None,
        };
        let w = generate_white(
            &i,
            &WhitePolicy {
                source: WhiteSource::Alpha,
                combination: MaskCombination::Replace,
                curve: CurveId::Identity,
                choke_px: 0,
                spread_px: 1,
                max_coverage: 1.0,
            },
        )
        .unwrap();
        assert_eq!(w.pixels, vec![1.0, 1.0, 1.0]);
    }

    fn input(w: usize, h: usize) -> SeparationInput {
        let p = Plane::new(w, h, 1.0);
        SeparationInput {
            alpha: p.clone(),
            tonal: p,
            supplied_mask: None,
            clear_mask: None,
        }
    }
    fn policy() -> WhitePolicy {
        WhitePolicy {
            source: WhiteSource::Alpha,
            combination: MaskCombination::UnionMax,
            curve: CurveId::Identity,
            choke_px: 0,
            spread_px: 0,
            max_coverage: 1.0,
        }
    }
    #[test]
    fn mismatched_or_non_finite_supplied_mask_is_an_error_not_a_panic() {
        let mut i = input(3, 3);
        i.supplied_mask = Some(Plane::new(2, 2, 1.0));
        assert_eq!(
            generate_white(&i, &policy()),
            Err(SeparationError::DimensionMismatch)
        );
        let mut bad = Plane::new(3, 3, 1.0);
        bad.pixels[0] = f32::NAN;
        i.supplied_mask = Some(bad);
        assert_eq!(
            generate_white(&i, &policy()),
            Err(SeparationError::InvalidPlane)
        );
    }
    #[test]
    fn non_finite_policy_values_and_unbounded_radius_are_rejected() {
        let i = input(3, 3);
        let nan_cov = WhitePolicy {
            max_coverage: f32::NAN,
            ..policy()
        };
        assert_eq!(
            generate_white(&i, &nan_cov),
            Err(SeparationError::InvalidPolicy)
        );
        let nan_blend = WhitePolicy {
            combination: MaskCombination::Blend {
                supplied_weight: f32::NAN,
            },
            ..policy()
        };
        assert_eq!(
            generate_white(&i, &nan_blend),
            Err(SeparationError::InvalidPolicy)
        );
        let huge = WhitePolicy {
            spread_px: MAX_MORPHOLOGY_RADIUS_PX + 1,
            ..policy()
        };
        assert_eq!(
            generate_white(&i, &huge),
            Err(SeparationError::InvalidPolicy)
        );
        let huge_choke = WhitePolicy {
            choke_px: usize::MAX,
            ..policy()
        };
        assert_eq!(
            generate_white(&i, &huge_choke),
            Err(SeparationError::InvalidPolicy)
        );
    }
    #[test]
    fn clear_mask_knocks_out_white_and_checks_size() {
        let mut i = input(2, 1);
        i.clear_mask = Some(Plane::from_pixels(2, 1, vec![0.0, 1.0]).unwrap());
        let w = generate_white(&i, &policy()).unwrap();
        assert_eq!(w.pixels, vec![0.0, 1.0]);
        i.clear_mask = Some(Plane::new(3, 1, 1.0));
        assert_eq!(
            generate_white(&i, &policy()),
            Err(SeparationError::DimensionMismatch)
        );
    }
    #[test]
    fn allocation_rejects_mismatched_planes_duplicates_and_unknown_channels() {
        let p = |w| Plane::new(w, 1, 0.5);
        let logical = LogicalPlanes {
            c: p(2),
            m: p(2),
            y: p(2),
            k: p(2),
            w: p(3),
        };
        let slot = |proto: &str, l: &str| PhysicalSlot {
            protocol_channel: proto.into(),
            logical_channel: l.into(),
        };
        let map = LogicalChannelMap {
            slots: vec![slot("P0", "C")],
        };
        assert_eq!(
            allocate_logical_to_physical(&logical, &map),
            Err(AllocationError::DimensionMismatch)
        );
        let ok = LogicalPlanes { w: p(2), ..logical };
        assert!(allocate_logical_to_physical(&ok, &map).is_ok());
        let dup = LogicalChannelMap {
            slots: vec![slot("P0", "C"), slot("P0", "M")],
        };
        assert_eq!(
            allocate_logical_to_physical(&ok, &dup),
            Err(AllocationError::DuplicateProtocolChannel)
        );
        let unknown = LogicalChannelMap {
            slots: vec![slot("P0", "X")],
        };
        assert_eq!(
            allocate_logical_to_physical(&ok, &unknown),
            Err(AllocationError::MissingChannel("X".into()))
        );
    }
}
