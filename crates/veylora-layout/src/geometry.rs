//! Integer micrometre rectangles in page space (origin top-left, y down).

use veylora_domain::layout::{Placement, Rotation};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i64,
    pub y: i64,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn right(&self) -> i64 {
        self.x + i64::from(self.width)
    }
    pub fn bottom(&self) -> i64 {
        self.y + i64::from(self.height)
    }
    pub fn contains(&self, inner: &Rect) -> bool {
        inner.x >= self.x
            && inner.y >= self.y
            && inner.right() <= self.right()
            && inner.bottom() <= self.bottom()
    }
    /// True when the interiors overlap. Touching edges do not intersect.
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }
}

/// Extent after rotation. Quarter turns swap the pre-rotation dimensions.
pub fn rotated_extent(width_um: u32, height_um: u32, rotation: Rotation) -> (u32, u32) {
    match rotation {
        Rotation::Deg0 | Rotation::Deg180 => (width_um, height_um),
        Rotation::Deg90 | Rotation::Deg270 => (height_um, width_um),
    }
}

/// The placement's bounding box before any sheet mirror. `x_um`/`y_um` are the
/// top-left of the rotated bounding box.
pub fn placement_rect(placement: &Placement) -> Rect {
    let (width, height) =
        rotated_extent(placement.width_um, placement.height_um, placement.rotation);
    Rect {
        x: placement.x_um,
        y: placement.y_um,
        width,
        height,
    }
}
