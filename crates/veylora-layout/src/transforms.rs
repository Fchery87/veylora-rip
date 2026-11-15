//! One resolved transform for preview and production. The sheet mirror is
//! applied here exactly once, to every placement, after rotation.

use crate::geometry::{placement_rect, Rect};
use veylora_device::capability::MirrorOwner;
use veylora_domain::{
    ids::{AssetId, PlacementId},
    layout::{LayoutSnapshot, Rotation},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayoutError {
    /// The layout asks for a mirror the device setup says the RIP does not own,
    /// or does not permit. Applying it would mirror twice or mirror silently.
    MirrorOwnerConflict { owner: MirrorOwner },
    /// Page-space coordinates exceed the supported integer range.
    CoordinateOverflow { placement: PlacementId },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPlacement {
    pub id: PlacementId,
    pub asset_id: AssetId,
    pub mask_id: Option<AssetId>,
    /// Final page-space box, after rotation and any sheet mirror.
    pub rect: Rect,
    pub rotation: Rotation,
    pub mirrored: bool,
    /// Pre-rotation scaled crop size, the domain of `map_source_um`.
    pub source_width_um: u32,
    pub source_height_um: u32,
    pub crop_x_px: u32,
    pub crop_y_px: u32,
    pub crop_width_px: u32,
    pub crop_height_px: u32,
    page_width_um: i64,
    origin_x: i64,
    origin_y: i64,
}

impl ResolvedPlacement {
    /// Map a point in the pre-rotation scaled crop (micrometres from its top-left)
    /// to final page micrometres: rotate clockwise, translate, then mirror.
    pub fn map_source_um(&self, sx: u32, sy: u32) -> (i64, i64) {
        let (sx, sy) = (i64::from(sx), i64::from(sy));
        let (w, h) = (
            i64::from(self.source_width_um),
            i64::from(self.source_height_um),
        );
        let (rx, ry) = match self.rotation {
            Rotation::Deg0 => (sx, sy),
            Rotation::Deg90 => (h - sy, sx),
            Rotation::Deg180 => (w - sx, h - sy),
            Rotation::Deg270 => (sy, w - sx),
        };
        let (x, y) = (self.origin_x + rx, self.origin_y + ry);
        if self.mirrored {
            (self.page_width_um - x, y)
        } else {
            (x, y)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedLayout {
    pub page_width_um: u32,
    pub page_height_um: u32,
    /// True when the RIP mirrored the sheet. Never true for operator-owned mirrors.
    pub mirror_applied: bool,
    pub placements: Vec<ResolvedPlacement>,
}

/// Resolve a layout against the device's mirror ownership.
///
/// A requested mirror is honoured only when the RIP owns it; any other owner
/// would double-mirror or mirror silently, so it is refused, not approximated.
pub fn resolve_transform(
    layout: &LayoutSnapshot,
    owner: MirrorOwner,
) -> Result<ResolvedLayout, LayoutError> {
    let input = layout.inputs();
    if input.mirror && owner != MirrorOwner::Rip {
        return Err(LayoutError::MirrorOwnerConflict { owner });
    }
    let page = i64::from(input.page_width_um);
    let mut placements = Vec::with_capacity(input.placements.len());
    for p in &input.placements {
        let base = placement_rect(p);
        let overflow = || LayoutError::CoordinateOverflow { placement: p.id };
        let right = base
            .x
            .checked_add(i64::from(base.width))
            .ok_or_else(overflow)?;
        base.y
            .checked_add(i64::from(base.height))
            .ok_or_else(overflow)?;
        let rect = if input.mirror {
            Rect {
                x: page.checked_sub(right).ok_or_else(overflow)?,
                ..base
            }
        } else {
            base
        };
        placements.push(ResolvedPlacement {
            id: p.id,
            asset_id: p.asset_id,
            mask_id: p.mask_id,
            rect,
            rotation: p.rotation,
            mirrored: input.mirror,
            source_width_um: p.width_um,
            source_height_um: p.height_um,
            crop_x_px: p.crop_x_px,
            crop_y_px: p.crop_y_px,
            crop_width_px: p.crop_width_px,
            crop_height_px: p.crop_height_px,
            page_width_um: page,
            origin_x: p.x_um,
            origin_y: p.y_um,
        });
    }
    Ok(ResolvedLayout {
        page_width_um: input.page_width_um,
        page_height_um: input.page_height_um,
        mirror_applied: input.mirror,
        placements,
    })
}
