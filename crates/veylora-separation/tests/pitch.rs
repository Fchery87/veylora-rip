mod common;
use common::asset;
use veylora_domain::recipe::{ClearPolicy, PerforationPolicy};
use veylora_image::SampleLayout;
use veylora_separation::{build_clear_mask, MaskInput, PixelPitch, SeparationError};

#[test]
fn pitch_follows_real_printer_resolutions() {
    assert_eq!(
        PixelPitch::from_dpi(1440, 720),
        Some(PixelPitch {
            x_nm: 17_639,
            y_nm: 35_278
        })
    );
    assert_eq!(
        PixelPitch::from_dpi(600, 600),
        Some(PixelPitch {
            x_nm: 42_333,
            y_nm: 42_333
        })
    );
    assert_eq!(PixelPitch::from_dpi(0, 600), None);
    assert_eq!(PixelPitch::from_dpi(600, 0), None);
}
#[test]
fn radius_converts_micrometres_with_half_up_rounding_and_limits() {
    let p = PixelPitch::from_dpi(1440, 1440).unwrap();
    assert_eq!(p.radius_px(0), Ok(0));
    assert_eq!(p.radius_px(8), Ok(0)); // 0.45 px
    assert_eq!(p.radius_px(9), Ok(1)); // 0.51 px
    assert_eq!(p.radius_px(100), Ok(6)); // 5.67 px
    assert_eq!(p.radius_px(1_000_000), Err(SeparationError::InvalidPolicy));
    let skew = PixelPitch {
        x_nm: 17_639,
        y_nm: 35_278,
    };
    assert_eq!(skew.radius_px(100), Err(SeparationError::AnisotropicPitch));
}
#[test]
fn perforation_stays_on_the_physical_grid_across_a_whole_inch_at_1440_dpi() {
    // 1440 px at 1440 dpi is one inch (25 400 um). Holes every 1000 um must land on 1000 um
    // multiples; whole-micrometre pitch (18 um) would put the 25th hole about 0.5 mm off.
    let width = 1440u32;
    let a = asset(width, 1, SampleLayout::Gray8, vec![128; width as usize]);
    let policy = ClearPolicy {
        knockouts: vec![],
        perforation: Some(PerforationPolicy {
            diameter_um: 300,
            spacing_um: 1000,
            phase_x_px: 0,
            phase_y_px: 0,
            eligible_mask_id: None,
            edge_protection_um: 0,
        }),
    };
    let mask = build_clear_mask(
        &MaskInput {
            source: &a,
            rows: None,
            eligible_mask: None,
            origin_px: (0, 0),
            pitch: PixelPitch::from_dpi(1440, 1440).unwrap(),
        },
        &policy,
    )
    .unwrap();
    let row = &mask.retained().pixels;
    let hole = |i: usize| row[i] == 0.0;
    // Hole at 25 000 um is centered on pixel 25000 / 17.6389 = 1417.3.
    assert!(hole(1417) && hole(1410) && hole(1424));
    assert!(!hole(1400) && !hole(1435));
    // Count of separate holes across the inch: grid points at 0, 1000, ..., 25000.
    let starts = (0..row.len())
        .filter(|i| hole(*i) && (*i == 0 || !hole(i - 1)))
        .count();
    assert_eq!(starts, 26);
}
