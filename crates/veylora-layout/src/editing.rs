//! Deterministic placement edits. Each edit keeps placement and asset identity so
//! a selective reprint still names the same placement.

use crate::geometry::{placement_rect, rotated_extent, Rect};
use veylora_domain::{
    ids::PlacementId,
    layout::{Placement, Rotation},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditError {
    /// A size would be zero. Sizes are never clamped silently.
    ZeroSize,
    /// The scaled size does not fit the supported integer range.
    SizeOverflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Right,
    Top,
    Bottom,
    HorizontalCenter,
    VerticalCenter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

pub fn set_size(
    placement: &Placement,
    width_um: u32,
    height_um: u32,
) -> Result<Placement, EditError> {
    if width_um == 0 || height_um == 0 {
        return Err(EditError::ZeroSize);
    }
    Ok(Placement {
        width_um,
        height_um,
        ..placement.clone()
    })
}

/// Resize to a new width keeping the current aspect ratio. The height is rounded
/// half up so the same inputs always give the same size.
pub fn resize_width_locked(placement: &Placement, width_um: u32) -> Result<Placement, EditError> {
    if width_um == 0 {
        return Err(EditError::ZeroSize);
    }
    let numerator = u128::from(placement.height_um) * u128::from(width_um);
    let denominator = u128::from(placement.width_um);
    let height = (numerator * 2 + denominator) / (denominator * 2);
    let height = u32::try_from(height).map_err(|_| EditError::SizeOverflow)?;
    set_size(placement, width_um, height)
}

/// Rotate clockwise by quarter turns (negative turns go counter-clockwise). The
/// top-left of the bounding box stays put, matching the domain's anchor rule.
pub fn rotate_quarter_turns(placement: &Placement, turns: i32) -> Placement {
    let index = |r: Rotation| match r {
        Rotation::Deg0 => 0,
        Rotation::Deg90 => 1,
        Rotation::Deg180 => 2,
        Rotation::Deg270 => 3,
    };
    let rotation = match (index(placement.rotation) + turns).rem_euclid(4) {
        0 => Rotation::Deg0,
        1 => Rotation::Deg90,
        2 => Rotation::Deg180,
        _ => Rotation::Deg270,
    };
    Placement {
        rotation,
        ..placement.clone()
    }
}

fn selected<'a, 'b>(
    items: &'a mut [&'b mut Placement],
    ids: &[PlacementId],
) -> Vec<&'a mut &'b mut Placement> {
    items.iter_mut().filter(|p| ids.contains(&p.id)).collect()
}

fn span(rects: &[Rect], axis: Axis) -> (i64, i64) {
    let (lo, hi): (Vec<i64>, Vec<i64>) = rects
        .iter()
        .map(|r| match axis {
            Axis::Horizontal => (r.x, r.right()),
            Axis::Vertical => (r.y, r.bottom()),
        })
        .unzip();
    (
        lo.into_iter().min().unwrap_or(0),
        hi.into_iter().max().unwrap_or(0),
    )
}

fn set_pos(p: &mut Placement, axis: Axis, value: i64) {
    match axis {
        Axis::Horizontal => p.x_um = value,
        Axis::Vertical => p.y_um = value,
    }
}

fn extent(r: &Rect, axis: Axis) -> i64 {
    match axis {
        Axis::Horizontal => i64::from(r.width),
        Axis::Vertical => i64::from(r.height),
    }
}

/// Align the listed placements to the extent of their own selection. Unknown ids
/// and unlisted placements are ignored. Centres floor, so results are stable.
pub fn align(items: &mut [&mut Placement], ids: &[PlacementId], alignment: Alignment) {
    let mut sel = selected(items, ids);
    let rects: Vec<Rect> = sel.iter().map(|p| placement_rect(p)).collect();
    let axis = match alignment {
        Alignment::Left | Alignment::Right | Alignment::HorizontalCenter => Axis::Horizontal,
        _ => Axis::Vertical,
    };
    let (lo, hi) = span(&rects, axis);
    for (p, r) in sel.iter_mut().zip(&rects) {
        let size = extent(r, axis);
        let pos = match alignment {
            Alignment::Left | Alignment::Top => lo,
            Alignment::Right | Alignment::Bottom => hi - size,
            Alignment::HorizontalCenter | Alignment::VerticalCenter => {
                lo + (hi - lo) / 2 - size / 2
            }
        };
        set_pos(p, axis, pos);
    }
}

/// Equalise the gaps between the listed placements along one axis. The first and
/// last (by position) stay fixed; leftover micrometres go to the earliest gaps.
pub fn distribute(items: &mut [&mut Placement], ids: &[PlacementId], axis: Axis) {
    let mut sel = selected(items, ids);
    if sel.len() < 3 {
        return;
    }
    let key = |p: &Placement| {
        let r = placement_rect(p);
        match axis {
            Axis::Horizontal => r.x,
            Axis::Vertical => r.y,
        }
    };
    sel.sort_by_key(|p| key(p));
    let rects: Vec<Rect> = sel.iter().map(|p| placement_rect(p)).collect();
    let (lo, hi) = span(&rects, axis);
    let content: i64 = rects.iter().map(|r| extent(r, axis)).sum();
    let gaps = (sel.len() - 1) as i64;
    let free = (hi - lo - content).max(0);
    let (base, extra) = (free / gaps, free % gaps);
    let mut cursor = lo;
    for (i, (p, r)) in sel.iter_mut().zip(&rects).enumerate() {
        set_pos(p, axis, cursor);
        let gap = base + i64::from((i as i64) < extra);
        cursor += extent(r, axis) + gap;
    }
}

/// Convenience for callers that only need the rotated size.
pub fn extent_of(placement: &Placement) -> (u32, u32) {
    rotated_extent(placement.width_um, placement.height_um, placement.rotation)
}
