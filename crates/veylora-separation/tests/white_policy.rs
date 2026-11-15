use veylora_domain::{
    ids::AssetId,
    recipe::{MaskCombination, WhitePolicy, WhiteSource},
};
use veylora_separation::{
    generate_white, ClearMask, PixelPitch, Plane, SeparationError, SeparationInput, SuppliedMask,
    MAX_MORPHOLOGY_RADIUS_PX,
};

const PITCH: PixelPitch = PixelPitch {
    x_nm: 100_000,
    y_nm: 100_000,
};

fn row(values: &[f32]) -> Plane {
    Plane::from_pixels(values.len(), 1, values.to_vec()).unwrap()
}
fn input(alpha: &[f32]) -> SeparationInput {
    SeparationInput {
        alpha: row(alpha),
        tonal: Some(row(&vec![0.5; alpha.len()])),
        supplied_mask: None,
        clear_mask: None,
    }
}
fn policy(source: WhiteSource) -> WhitePolicy {
    WhitePolicy {
        source,
        combine: MaskCombination::Replace,
        density_percent: 100,
        spread_um: 0,
        response: vec![0, 65535],
    }
}
fn white(i: &SeparationInput, p: &WhitePolicy) -> Vec<f32> {
    generate_white(i, p, PITCH).unwrap().pixels
}
fn close(a: &[f32], b: &[f32]) {
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b) {
        assert!((x - y).abs() < 1e-4, "{a:?} != {b:?}");
    }
}

#[test]
fn alpha_and_tonal_sources_follow_identity_response() {
    let i = input(&[0.0, 0.25, 1.0]);
    close(&white(&i, &policy(WhiteSource::Alpha)), &[0.0, 0.25, 1.0]);
    // Tonal white is still clear where alpha is fully transparent.
    close(&white(&i, &policy(WhiteSource::Tonal)), &[0.0, 0.5, 0.5]);
}
#[test]
fn disabled_source_is_zero_even_with_a_supplied_mask() {
    let mut i = input(&[1.0, 1.0]);
    i.supplied_mask = Some(SuppliedMask {
        id: AssetId::new(),
        plane: row(&[1.0, 1.0]),
    });
    close(&white(&i, &policy(WhiteSource::Disabled)), &[0.0, 0.0]);
}
#[test]
fn supplied_source_requires_the_referenced_mask() {
    let id = AssetId::new();
    let p = policy(WhiteSource::Supplied { mask_id: id });
    let mut i = input(&[0.0, 0.0]);
    assert_eq!(
        generate_white(&i, &p, PITCH),
        Err(SeparationError::MissingSuppliedMask)
    );
    i.supplied_mask = Some(SuppliedMask {
        id: AssetId::new(),
        plane: row(&[1.0, 0.5]),
    });
    assert_eq!(
        generate_white(&i, &p, PITCH),
        Err(SeparationError::SuppliedMaskMismatch)
    );
    i.supplied_mask = Some(SuppliedMask {
        id,
        plane: row(&[1.0, 0.5]),
    });
    // Explicit white artwork stays eligible where alpha is zero.
    close(&white(&i, &p), &[1.0, 0.5]);
}
#[test]
fn replace_union_and_intersection_combine_a_supplied_mask_with_alpha() {
    let mut i = input(&[1.0, 0.25, 1.0, 0.0]);
    i.supplied_mask = Some(SuppliedMask {
        id: AssetId::new(),
        plane: row(&[0.5, 0.75, 0.0, 1.0]),
    });
    let mut p = policy(WhiteSource::Alpha);
    p.combine = MaskCombination::Replace;
    close(&white(&i, &p), &[0.5, 0.75, 0.0, 1.0]);
    p.combine = MaskCombination::Union;
    close(&white(&i, &p), &[1.0, 0.75, 1.0, 1.0]);
    p.combine = MaskCombination::Intersection;
    close(&white(&i, &p), &[0.5, 0.25, 0.0, 0.0]);
}
#[test]
fn response_curve_and_density_shape_coverage() {
    let i = input(&[0.25, 0.5, 1.0]);
    let mut p = policy(WhiteSource::Alpha);
    p.response = vec![65535, 0];
    close(&white(&i, &p), &[0.75, 0.5, 0.0]);
    p.response = vec![0, 0, 65535];
    close(&white(&i, &p), &[0.0, 0.0, 1.0]);
    p.response = vec![0, 65535];
    p.density_percent = 50;
    close(&white(&i, &p), &[0.125, 0.25, 0.5]);
}
#[test]
fn a_response_that_lifts_zero_never_inks_transparent_pixels() {
    let i = input(&[0.0, 1.0]);
    let mut p = policy(WhiteSource::Alpha);
    p.response = vec![65535, 65535];
    close(&white(&i, &p), &[0.0, 1.0]);
}
#[test]
fn spread_and_choke_use_physical_units_with_sign() {
    let mut i2 = input(&[1.0; 5]);
    i2.tonal = Some(row(&[0.0, 0.0, 1.0, 0.0, 0.0]));
    let mut p = policy(WhiteSource::Tonal);
    p.spread_um = 100;
    close(&white(&i2, &p), &[0.0, 1.0, 1.0, 1.0, 0.0]);
    p.spread_um = 40; // rounds down to zero pixels at 100 um pitch
    close(&white(&i2, &p), &[0.0, 0.0, 1.0, 0.0, 0.0]);
    p.spread_um = 50; // rounds half up to one pixel
    close(&white(&i2, &p), &[0.0, 1.0, 1.0, 1.0, 0.0]);
    p.spread_um = -100;
    close(&white(&i2, &p), &[0.0, 0.0, 0.0, 0.0, 0.0]);
}
#[test]
fn unsafe_geometry_is_rejected_not_approximated() {
    let i = input(&[1.0, 1.0]);
    let mut p = policy(WhiteSource::Alpha);
    p.spread_um = (MAX_MORPHOLOGY_RADIUS_PX as i32 + 1) * 100;
    assert_eq!(
        generate_white(&i, &p, PITCH),
        Err(SeparationError::InvalidPolicy)
    );
    p.spread_um = i32::MIN;
    assert_eq!(
        generate_white(&i, &p, PITCH),
        Err(SeparationError::InvalidPolicy)
    );
    p.spread_um = 100;
    let skew = PixelPitch {
        x_nm: 100_000,
        y_nm: 200_000,
    };
    assert_eq!(
        generate_white(&i, &p, skew),
        Err(SeparationError::AnisotropicPitch)
    );
    let zero = PixelPitch { x_nm: 0, y_nm: 0 };
    assert_eq!(
        generate_white(&i, &p, zero),
        Err(SeparationError::InvalidPolicy)
    );
}
#[test]
fn invalid_recipe_values_and_inputs_are_errors_not_panics() {
    let i = input(&[1.0, 1.0]);
    let mut p = policy(WhiteSource::Alpha);
    p.response = vec![0];
    assert_eq!(
        generate_white(&i, &p, PITCH),
        Err(SeparationError::InvalidPolicy)
    );
    p.response = vec![0, 65535];
    p.density_percent = 101;
    assert_eq!(
        generate_white(&i, &p, PITCH),
        Err(SeparationError::InvalidPolicy)
    );
    p.density_percent = 100;
    let mut bad = input(&[1.0, 1.0]);
    bad.supplied_mask = Some(SuppliedMask {
        id: AssetId::new(),
        plane: Plane::new(3, 1, 1.0),
    });
    assert_eq!(
        generate_white(&bad, &p, PITCH),
        Err(SeparationError::DimensionMismatch)
    );
    let mut nan = Plane::new(2, 1, 1.0);
    nan.pixels[0] = f32::NAN;
    bad.supplied_mask = Some(SuppliedMask {
        id: AssetId::new(),
        plane: nan,
    });
    assert_eq!(
        generate_white(&bad, &p, PITCH),
        Err(SeparationError::InvalidPlane)
    );
}
#[test]
fn clear_mask_is_final_so_spread_cannot_reintroduce_white() {
    let mut i = input(&[0.0, 1.0, 0.0]);
    i.clear_mask = Some(ClearMask::from_plane(row(&[0.0, 1.0, 0.0])).unwrap());
    let mut p = policy(WhiteSource::Alpha);
    p.spread_um = 100;
    close(&white(&i, &p), &[0.0, 1.0, 0.0]);
    i.clear_mask = Some(ClearMask::open(4, 1));
    assert_eq!(
        generate_white(&i, &p, PITCH),
        Err(SeparationError::DimensionMismatch)
    );
}

#[test]
fn tonal_source_without_a_tonal_plane_is_an_error() {
    let mut i = input(&[1.0, 1.0]);
    i.tonal = None;
    assert_eq!(
        generate_white(&i, &policy(WhiteSource::Tonal), PITCH),
        Err(SeparationError::MissingTonal)
    );
    // Other sources do not need it.
    assert!(generate_white(&i, &policy(WhiteSource::Alpha), PITCH).is_ok());
}
