use crate::Plane;

/// Upper bound on choke/spread radius so morphology cost stays bounded (O(w*h*r^2)).
pub const MAX_MORPHOLOGY_RADIUS_PX: usize = 64;

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
pub(crate) fn dilate(p: &Plane, r: usize) -> Plane {
    morph(p, r, true)
}
pub(crate) fn erode(p: &Plane, r: usize) -> Plane {
    morph(p, r, false)
}
fn morph(p: &Plane, r: usize, max: bool) -> Plane {
    let mut o = p.clone();
    for y in 0..p.height {
        for x in 0..p.width {
            o.set(x, y, neighborhood(p, x, y, r, max));
        }
    }
    o
}
