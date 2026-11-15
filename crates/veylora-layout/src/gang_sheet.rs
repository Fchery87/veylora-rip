//! Gang sheet: several placements on one page with exact gutters, deterministic
//! nesting, duplication and selective-reprint subsets.
//!
//! Notes are non-printing annotations kept beside the placements. They are not
//! part of the layout snapshot or its hash and never reach the raster; printed
//! labels would be real artwork and need their own asset.

use crate::geometry::{placement_rect, rotated_extent};
use std::collections::BTreeMap;
use veylora_domain::{
    errors::ValidationError,
    ids::PlacementId,
    layout::{LayoutInput, LayoutSnapshot, Placement},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SheetError {
    UnknownPlacement(PlacementId),
    /// This placement does not fit within the margins. The sheet is left unchanged.
    DoesNotFit(PlacementId),
    Overflow,
}

impl std::fmt::Display for SheetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownPlacement(id) => write!(f, "Unknown placement {id}."),
            Self::DoesNotFit(id) => write!(f, "Placement {id} does not fit on the sheet."),
            Self::Overflow => f.write_str("Layout coordinates exceed the supported range."),
        }
    }
}

impl std::error::Error for SheetError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GangSheet {
    pub page_width_um: u32,
    pub page_height_um: u32,
    pub placements: Vec<Placement>,
    pub notes: BTreeMap<PlacementId, String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NestConstraints {
    /// Exact clear space between neighbouring placements, micrometres.
    pub gutter_um: u32,
    /// Clear space between placements and every page edge, micrometres.
    pub margin_um: u32,
}

impl GangSheet {
    pub fn new(page_width_um: u32, page_height_um: u32, placements: Vec<Placement>) -> Self {
        Self {
            page_width_um,
            page_height_um,
            placements,
            notes: BTreeMap::new(),
        }
    }

    pub fn set_note(&mut self, id: PlacementId, note: impl Into<String>) -> Result<(), SheetError> {
        if !self.placements.iter().any(|p| p.id == id) {
            return Err(SheetError::UnknownPlacement(id));
        }
        self.notes.insert(id, note.into());
        Ok(())
    }

    /// Freeze into a validated domain snapshot. `mirror` is the sheet mirror request.
    pub fn to_snapshot(&self, mirror: bool) -> Result<LayoutSnapshot, ValidationError> {
        LayoutSnapshot::new(LayoutInput {
            schema_version: 1,
            page_width_um: self.page_width_um,
            page_height_um: self.page_height_um,
            mirror,
            placements: self.placements.clone(),
        })
    }
}

/// Add `copies` copies of a placement in a row to its right, each separated by
/// exactly `gutter_um`. Copies share the source asset, mask, crop, size and
/// rotation but get fresh placement identities. Placements are plain values, so
/// editing a copy never changes its source. Page bounds are checked by validation.
pub fn duplicate_row(
    sheet: &mut GangSheet,
    id: PlacementId,
    copies: u32,
    gutter_um: u32,
    new_id: &mut dyn FnMut() -> PlacementId,
) -> Result<Vec<PlacementId>, SheetError> {
    let source = sheet
        .placements
        .iter()
        .find(|p| p.id == id)
        .cloned()
        .ok_or(SheetError::UnknownPlacement(id))?;
    let mut right = placement_rect(&source).right();
    let width = i64::from(placement_rect(&source).width);
    let mut created = Vec::new();
    let mut added = Vec::new();
    for _ in 0..copies {
        let x = right
            .checked_add(i64::from(gutter_um))
            .ok_or(SheetError::Overflow)?;
        right = x.checked_add(width).ok_or(SheetError::Overflow)?;
        let copy = Placement {
            id: new_id(),
            x_um: x,
            ..source.clone()
        };
        created.push(copy.id);
        added.push(copy);
    }
    sheet.placements.extend(added);
    Ok(created)
}

/// Deterministic shelf nesting. Placements keep their rotation; only positions
/// change. Order is tallest first, then widest, then identity, so input order
/// never matters. Fails atomically with the first placement that does not fit.
pub fn block_nest(sheet: &mut GangSheet, constraints: &NestConstraints) -> Result<(), SheetError> {
    let margin = i64::from(constraints.margin_um);
    let gutter = i64::from(constraints.gutter_um);
    let limit_x = i64::from(sheet.page_width_um) - margin;
    let limit_y = i64::from(sheet.page_height_um) - margin;
    let mut order: Vec<(u32, u32, PlacementId)> = sheet
        .placements
        .iter()
        .map(|p| {
            let (w, h) = rotated_extent(p.width_um, p.height_um, p.rotation);
            (w, h, p.id)
        })
        .collect();
    order.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)).then(a.2.cmp(&b.2)));

    let mut positions = BTreeMap::new();
    let (mut x, mut y, mut shelf) = (margin, margin, 0_i64);
    for (w, h, id) in order {
        let (w, h) = (i64::from(w), i64::from(h));
        if x > margin && x + w > limit_x {
            y += shelf + gutter;
            x = margin;
            shelf = 0;
        }
        if x + w > limit_x || y + h > limit_y {
            return Err(SheetError::DoesNotFit(id));
        }
        positions.insert(id, (x, y));
        x += w + gutter;
        shelf = shelf.max(h);
    }
    for p in &mut sheet.placements {
        if let Some(&(px, py)) = positions.get(&p.id) {
            p.x_um = px;
            p.y_um = py;
        }
    }
    Ok(())
}

/// The subset for a selective reprint. Identities, positions and the page are
/// kept exactly, so reprinted output lines up with the original sheet.
pub fn subset(sheet: &GangSheet, ids: &[PlacementId]) -> Result<GangSheet, SheetError> {
    if let Some(missing) = ids
        .iter()
        .find(|id| !sheet.placements.iter().any(|p| p.id == **id))
    {
        return Err(SheetError::UnknownPlacement(*missing));
    }
    let placements: Vec<Placement> = sheet
        .placements
        .iter()
        .filter(|p| ids.contains(&p.id))
        .cloned()
        .collect();
    let notes = sheet
        .notes
        .iter()
        .filter(|(id, _)| ids.contains(id))
        .map(|(id, n)| (*id, n.clone()))
        .collect();
    Ok(GangSheet {
        page_width_um: sheet.page_width_um,
        page_height_um: sheet.page_height_um,
        placements,
        notes,
    })
}
