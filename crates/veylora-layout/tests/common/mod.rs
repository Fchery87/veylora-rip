#![allow(dead_code)]
use veylora_domain::{
    ids::{AssetId, PlacementId},
    layout::{LayoutInput, LayoutSnapshot, Placement, Rotation},
};

pub fn placement(x: i64, y: i64, w: u32, h: u32, rotation: Rotation) -> Placement {
    Placement {
        id: PlacementId::new(),
        asset_id: AssetId::new(),
        mask_id: None,
        x_um: x,
        y_um: y,
        width_um: w,
        height_um: h,
        rotation,
        crop_x_px: 0,
        crop_y_px: 0,
        crop_width_px: 1000,
        crop_height_px: 1000,
    }
}

pub fn layout(
    page_w: u32,
    page_h: u32,
    mirror: bool,
    placements: Vec<Placement>,
) -> LayoutSnapshot {
    LayoutSnapshot::new(LayoutInput {
        schema_version: 1,
        page_width_um: page_w,
        page_height_um: page_h,
        mirror,
        placements,
    })
    .unwrap()
}
