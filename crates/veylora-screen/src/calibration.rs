use veylora_domain::recipe::CalibrationPolicy;
use veylora_separation::{apply_response, LogicalPlanes, Plane};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CalibrationError {
    /// A process curve has fewer than two samples or does not start at zero. A curve that lifts
    /// zero would ink empty pixels, so it is rejected rather than silently corrected.
    InvalidCurve,
    /// A channel or combined limit is outside its domain.
    InvalidLimit,
    InvalidPlane,
    DimensionMismatch,
}

/// Applies the recipe's calibration to the C, M, Y and K planes in place:
/// response curve, then the per-channel limit, then the combined limit.
///
/// The combined limit caps C+M+Y+K per pixel by scaling all four down proportionally. White is
/// measured physical deposition with its own response in the white policy, so it is neither
/// calibrated nor counted here. Nothing is modified unless the whole policy and all planes are
/// valid.
pub fn apply_calibration(
    planes: &mut LogicalPlanes,
    policy: &CalibrationPolicy,
) -> Result<(), CalibrationError> {
    if policy
        .process_curves
        .iter()
        .any(|c| c.len() < 2 || c[0] != 0)
    {
        return Err(CalibrationError::InvalidCurve);
    }
    if policy.channel_limits_percent.iter().any(|v| *v > 100) || policy.total_coverage_percent > 400
    {
        return Err(CalibrationError::InvalidLimit);
    }
    let size_of = |p: &Plane| (p.width, p.height);
    let process = [&planes.c, &planes.m, &planes.y, &planes.k];
    for plane in process {
        plane
            .validate()
            .map_err(|_| CalibrationError::InvalidPlane)?;
        if size_of(plane) != size_of(&planes.c) {
            return Err(CalibrationError::DimensionMismatch);
        }
    }
    let [c, m, y, k] = [&mut planes.c, &mut planes.m, &mut planes.y, &mut planes.k];
    let channel_caps = policy.channel_limits_percent.map(|p| f32::from(p) / 100.0);
    let total_cap = f32::from(policy.total_coverage_percent) / 100.0;
    for (index, plane) in [&mut *c, &mut *m, &mut *y, &mut *k].into_iter().enumerate() {
        for v in &mut plane.pixels {
            *v = apply_response(&policy.process_curves[index], *v).min(channel_caps[index]);
        }
    }
    for i in 0..c.pixels.len() {
        let sum = c.pixels[i] + m.pixels[i] + y.pixels[i] + k.pixels[i];
        if sum > total_cap {
            let scale = if sum > 0.0 { total_cap / sum } else { 0.0 };
            for plane in [&mut *c, &mut *m, &mut *y, &mut *k] {
                plane.pixels[i] *= scale;
            }
        }
    }
    Ok(())
}
