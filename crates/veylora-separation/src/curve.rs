/// Piecewise-linear response over `curve` samples spread uniformly across 0..=65535, evaluated at a
/// normalized coverage. Requires at least two samples; callers validate that.
pub fn apply_response(curve: &[u16], value: f32) -> f32 {
    let last = curve.len() - 1;
    let position = value.clamp(0.0, 1.0) * last as f32;
    let index = (position.floor() as usize).min(last - 1);
    let fraction = position - index as f32;
    let (a, b) = (f32::from(curve[index]), f32::from(curve[index + 1]));
    ((a + (b - a) * fraction) / f32::from(u16::MAX)).clamp(0.0, 1.0)
}
