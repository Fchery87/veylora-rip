use veylora_domain::{
    ids::{AssetId, PlacementId},
    layout::{LayoutInput, LayoutSnapshot, Placement, Rotation},
    recipe::*,
};

#[test]
fn layout_retains_geometry_and_validates_deserialization() {
    let placement = Placement {
        id: PlacementId::new(),
        asset_id: AssetId::new(),
        mask_id: None,
        x_um: 100,
        y_um: 200,
        width_um: 10000,
        height_um: 20000,
        rotation: Rotation::Deg90,
        crop_x_px: 2,
        crop_y_px: 3,
        crop_width_px: 40,
        crop_height_px: 80,
    };
    let data = LayoutInput {
        schema_version: 1,
        page_width_um: 300000,
        page_height_um: 400000,
        mirror: true,
        placements: vec![placement.clone()],
    };
    let layout = LayoutSnapshot::new(data.clone()).unwrap();
    assert_eq!(layout.inputs(), &data);
    let restored: LayoutSnapshot =
        serde_json::from_str(&serde_json::to_string(&layout).unwrap()).unwrap();
    assert_eq!(layout, restored);
    let mut invalid = data.clone();
    invalid.placements.push(placement);
    assert!(LayoutSnapshot::new(invalid.clone()).is_err());
    assert!(
        serde_json::from_value::<LayoutSnapshot>(serde_json::to_value(invalid).unwrap()).is_err()
    );
    let mut invalid = data;
    invalid.placements[0].width_um = 0;
    assert!(LayoutSnapshot::new(invalid).is_err());
}

#[test]
fn crop_extent_is_retained_and_overflow_is_rejected() {
    let mut data = LayoutInput {
        schema_version: 1,
        page_width_um: 300_000,
        page_height_um: 400_000,
        mirror: false,
        placements: vec![Placement {
            id: PlacementId::new(),
            asset_id: AssetId::new(),
            mask_id: None,
            x_um: 0,
            y_um: 0,
            width_um: 100,
            height_um: 200,
            rotation: Rotation::Deg180,
            crop_x_px: 10,
            crop_y_px: 20,
            crop_width_px: 30,
            crop_height_px: 40,
        }],
    };
    let layout = LayoutSnapshot::new(data.clone()).unwrap();
    assert_eq!(layout.placements()[0].crop_width_px, 30);
    assert_eq!(layout.placements()[0].crop_height_px, 40);
    data.placements[0].crop_x_px = u32::MAX;
    assert!(LayoutSnapshot::new(data).is_err());
}

#[test]
fn unknown_policy_variants_are_rejected() {
    assert!(serde_json::from_str::<RenderingIntent>("\"guess\"").is_err());
    assert!(serde_json::from_str::<MaskCombination>("\"guess\"").is_err());
    assert!(serde_json::from_str::<ScreenAlgorithm>("\"guess\"").is_err());
    assert!(serde_json::from_str::<RecipeStatus>("\"verified\"").is_err());
}
