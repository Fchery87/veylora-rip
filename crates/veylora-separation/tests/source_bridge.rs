mod common;
use common::asset;
use veylora_image::SampleLayout;
use veylora_separation::{alpha_plane, mask_plane, SeparationError, SuppliedMask};

#[test]
fn alpha_comes_from_the_alpha_channel_and_hidden_rgb_is_ignored() {
    let a = asset(
        2,
        1,
        SampleLayout::Rgba8,
        vec![255, 0, 255, 0, 1, 2, 3, 128],
    );
    let plane = alpha_plane(&a).unwrap();
    assert_eq!(plane.pixels, vec![0.0, 128.0 / 255.0]);
    let ga = asset(2, 1, SampleLayout::GrayAlpha8, vec![9, 255, 9, 0]);
    assert_eq!(alpha_plane(&ga).unwrap().pixels, vec![1.0, 0.0]);
}
#[test]
fn sources_without_alpha_are_fully_covered() {
    let rgb = asset(2, 1, SampleLayout::Rgb8, vec![0, 0, 0, 9, 9, 9]);
    assert_eq!(alpha_plane(&rgb).unwrap().pixels, vec![1.0, 1.0]);
    let gray = asset(1, 2, SampleLayout::Gray8, vec![0, 200]);
    assert_eq!(alpha_plane(&gray).unwrap().pixels, vec![1.0, 1.0]);
}
#[test]
fn only_gray8_can_be_a_supplied_mask_and_values_are_coverage() {
    let g = asset(3, 1, SampleLayout::Gray8, vec![0, 51, 255]);
    assert_eq!(mask_plane(&g).unwrap().pixels, vec![0.0, 0.2, 1.0]);
    let id = g.id;
    let supplied = SuppliedMask::from_asset(&g).unwrap();
    assert_eq!(supplied.id, id);
    for (layout, n) in [
        (SampleLayout::Rgb8, 3),
        (SampleLayout::Rgba8, 4),
        (SampleLayout::GrayAlpha8, 2),
    ] {
        let a = asset(1, 1, layout, vec![0; n]);
        assert_eq!(mask_plane(&a), Err(SeparationError::UnsupportedSource));
    }
}
#[test]
fn inconsistent_sample_counts_are_rejected() {
    let bad = asset(2, 2, SampleLayout::Gray8, vec![0, 1, 2]);
    assert_eq!(alpha_plane(&bad), Err(SeparationError::MalformedSource));
    assert_eq!(mask_plane(&bad), Err(SeparationError::MalformedSource));
    let empty = asset(0, 0, SampleLayout::Gray8, vec![]);
    assert_eq!(alpha_plane(&empty), Err(SeparationError::MalformedSource));
}

#[test]
fn row_bands_equal_slices_of_the_whole_plane_and_reject_bad_ranges() {
    let a = asset(
        2,
        4,
        SampleLayout::GrayAlpha8,
        vec![1, 10, 2, 20, 3, 30, 4, 40, 5, 50, 6, 60, 7, 70, 8, 80],
    );
    let whole = veylora_separation::alpha_plane(&a).unwrap();
    let band = veylora_separation::alpha_rows(&a, 1..3).unwrap();
    assert_eq!((band.width, band.height), (2, 2));
    assert_eq!(band.pixels, whole.pixels[2..6].to_vec());
    let g = asset(3, 3, SampleLayout::Gray8, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
    let whole = mask_plane(&g).unwrap();
    assert_eq!(
        veylora_separation::mask_rows(&g, 2..3).unwrap().pixels,
        whole.pixels[6..9].to_vec()
    );
    for bad in [0..0, 2..2, 1..5, 4..5] {
        assert_eq!(
            veylora_separation::alpha_rows(&a, bad.clone()),
            Err(SeparationError::DimensionMismatch),
            "{bad:?}"
        );
    }
}
