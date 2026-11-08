//! Retained physical geometry. The layout owns mirroring for every plane.
use crate::{
    errors::ValidationError,
    ids::{AssetId, PlacementId},
    provenance::ContentHash,
    recipe::record_hash,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

/// A source crop is a rectangle in source pixels. `crop_x_px` and `crop_y_px`
/// identify its top-left corner, while the width and height retain its complete
/// extent. `width_um` and `height_um` are the scaled, pre-rotation crop size.
/// Rotation is clockwise about the crop's top-left anchor. Thus a quarter-turn
/// swaps the bounding-box dimensions and preserves that anchor before the
/// placement translation. Sheet mirroring is applied once, after placements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    pub id: PlacementId,
    pub asset_id: AssetId,
    pub mask_id: Option<AssetId>,
    pub x_um: i64,
    pub y_um: i64,
    pub width_um: u32,
    pub height_um: u32,
    pub rotation: Rotation,
    pub crop_x_px: u32,
    pub crop_y_px: u32,
    pub crop_width_px: u32,
    pub crop_height_px: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutInput {
    pub schema_version: u32,
    pub page_width_um: u32,
    pub page_height_um: u32,
    /// Horizontal mirror of the complete sheet, applied once after placement.
    pub mirror: bool,
    pub placements: Vec<Placement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "LayoutInput", into = "LayoutInput")]
pub struct LayoutSnapshot(LayoutInput);

impl LayoutSnapshot {
    pub fn new(input: LayoutInput) -> Result<Self, ValidationError> {
        if input.schema_version != 1 || input.page_width_um == 0 || input.page_height_um == 0 {
            return Err(ValidationError::invalid(
                "layout",
                "Use schema 1 and positive page dimensions.",
            ));
        }
        if input.placements.is_empty() {
            return Err(ValidationError::invalid(
                "layout.placements",
                "Retain at least one placement.",
            ));
        }
        let mut ids = std::collections::BTreeSet::new();
        for placement in &input.placements {
            if !ids.insert(placement.id)
                || placement.width_um == 0
                || placement.height_um == 0
                || placement.crop_width_px == 0
                || placement.crop_height_px == 0
            {
                return Err(ValidationError::invalid(
                    "layout.placements",
                    "Use unique placement identities and positive dimensions.",
                ));
            }
            if placement
                .crop_x_px
                .checked_add(placement.crop_width_px)
                .is_none()
                || placement
                    .crop_y_px
                    .checked_add(placement.crop_height_px)
                    .is_none()
            {
                return Err(ValidationError::invalid(
                    "layout.placements",
                    "Crop coordinates must not overflow.",
                ));
            }
        }
        Ok(Self(input))
    }
    pub fn inputs(&self) -> &LayoutInput {
        &self.0
    }
    pub fn placements(&self) -> &[Placement] {
        &self.0.placements
    }
    pub fn content_hash(&self) -> ContentHash {
        record_hash(b"veylora-layout-snapshot-v1\0", &self.0)
    }
}
impl TryFrom<LayoutInput> for LayoutSnapshot {
    type Error = ValidationError;
    fn try_from(input: LayoutInput) -> Result<Self, Self::Error> {
        Self::new(input)
    }
}
impl From<LayoutSnapshot> for LayoutInput {
    fn from(layout: LayoutSnapshot) -> Self {
        layout.0
    }
}
