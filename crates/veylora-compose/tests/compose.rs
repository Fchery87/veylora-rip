mod common;
use common::*;
use veylora_compose::{compose_page, ComposeError, ComposeInput};
use veylora_domain::layout::Rotation;
use veylora_image::{ColorEncoding, SampleLayout};

fn run(
    layout: &veylora_layout::ResolvedLayout,
    images: &[&veylora_image::SourceAsset],
) -> Result<veylora_compose::ComposedPage, ComposeError> {
    compose_page(
        &ComposeInput {
            layout,
            images,
            masks: &[],
            pitch: PITCH,
            max_page_pixels: 1 << 20,
        },
        &|| false,
    )
}

fn rgba_ramp(w: u32, h: u32) -> Vec<u8> {
    let mut v = Vec::new();
    for y in 0..h {
        for x in 0..w {
            v.extend_from_slice(&[
                (x * 37 + 5) as u8,
                (y * 53 + 11) as u8,
                (x * y * 7) as u8,
                255 - (x + y) as u8 * 9,
            ]);
        }
    }
    v
}

#[test]
fn identity_placement_is_bit_exact_including_partial_alpha() {
    let a = image(5, 4, SampleLayout::Rgba8, rgba_ramp(5, 4));
    let l = resolved(
        (50, 40),
        false,
        vec![place(&a, 0, 0, 50, 40, Rotation::Deg0)],
    );
    let page = run(&l, &[&a]).unwrap();
    assert_eq!((page.width_px, page.height_px), (5, 4));
    // 5 px at 10 um pitch is 50 um.
    for y in 0..4 {
        for x in 0..5 {
            let i = (y * 5 + x) * 4;
            let src: [u8; 4] = a.decoded.samples[i..i + 4].try_into().unwrap();
            let got = px(&page.image, x, y);
            if src[3] == 0 {
                assert_eq!(got[3], 0);
            } else {
                assert_eq!(got, src, "pixel {x},{y}");
            }
        }
    }
}

#[test]
fn opaque_rgb_and_srgb_gray_expand_exactly() {
    let rgb = image(2, 1, SampleLayout::Rgb8, vec![1, 2, 3, 4, 5, 6]);
    let l = resolved(
        (20, 10),
        false,
        vec![place(&rgb, 0, 0, 20, 10, Rotation::Deg0)],
    );
    let page = run(&l, &[&rgb]).unwrap();
    assert_eq!(px(&page.image, 1, 0), [4, 5, 6, 255]);
    let gray = image(2, 1, SampleLayout::GrayAlpha8, vec![9, 255, 7, 128]);
    let l = resolved(
        (20, 10),
        false,
        vec![place(&gray, 0, 0, 20, 10, Rotation::Deg0)],
    );
    let page = run(&l, &[&gray]).unwrap();
    assert_eq!(px(&page.image, 0, 0), [9, 9, 9, 255]);
    assert_eq!(px(&page.image, 1, 0), [7, 7, 7, 128]);
}

#[test]
fn every_rotation_and_mirror_matches_the_layout_transform() {
    // Each source pixel is 10 um and each page pixel is 10 um, so the page pixel that holds a
    // source pixel's centre (per ResolvedPlacement::map_source_um) must carry that pixel.
    let (w, h) = (4u32, 3u32);
    let mut samples = Vec::new();
    for y in 0..h {
        for x in 0..w {
            samples.extend_from_slice(&[
                (x * 40 + 10) as u8,
                (y * 70 + 20) as u8,
                (x + y * 4) as u8 * 11,
                255,
            ]);
        }
    }
    let a = image(w, h, SampleLayout::Rgba8, samples);
    for rotation in [
        Rotation::Deg0,
        Rotation::Deg90,
        Rotation::Deg180,
        Rotation::Deg270,
    ] {
        for mirror in [false, true] {
            let l = resolved(
                (100, 100),
                mirror,
                vec![place(&a, 20, 30, 40, 30, rotation)],
            );
            let page = run(&l, &[&a]).unwrap();
            let p = &l.placements[0];
            for sy in 0..h {
                for sx in 0..w {
                    let (cx, cy) = p.map_source_um(sx * 10 + 5, sy * 10 + 5);
                    let got = px(&page.image, (cx / 10) as usize, (cy / 10) as usize);
                    let i = ((sy * w + sx) * 4) as usize;
                    let want: [u8; 4] = a.decoded.samples[i..i + 4].try_into().unwrap();
                    assert_eq!(got, want, "{rotation:?} mirror={mirror} src=({sx},{sy})");
                }
            }
            // Nothing is drawn outside the placement box.
            assert_eq!(px(&page.image, 0, 0)[3], 0);
        }
    }
}

#[test]
fn downscale_is_an_area_average() {
    // 2x2 source of 20 um pixels shown at 20 um total: each page pixel is half of a source
    // pixel row/column... use 4x1 source squeezed to 2 page pixels: pairs average.
    let a = image(
        4,
        1,
        SampleLayout::Rgb8,
        vec![0, 0, 0, 100, 100, 100, 200, 200, 200, 254, 254, 254],
    );
    let l = resolved(
        (20, 10),
        false,
        vec![place(&a, 0, 0, 20, 10, Rotation::Deg0)],
    );
    let page = run(&l, &[&a]).unwrap();
    assert_eq!(px(&page.image, 0, 0), [50, 50, 50, 255]);
    assert_eq!(px(&page.image, 1, 0), [227, 227, 227, 255]);
}

#[test]
fn upscale_replicates_source_pixels() {
    let a = image(2, 1, SampleLayout::Rgb8, vec![10, 20, 30, 40, 50, 60]);
    let l = resolved(
        (40, 10),
        false,
        vec![place(&a, 0, 0, 40, 10, Rotation::Deg0)],
    );
    let page = run(&l, &[&a]).unwrap();
    assert_eq!(px(&page.image, 0, 0), [10, 20, 30, 255]);
    assert_eq!(px(&page.image, 1, 0), [10, 20, 30, 255]);
    assert_eq!(px(&page.image, 2, 0), [40, 50, 60, 255]);
    assert_eq!(px(&page.image, 3, 0), [40, 50, 60, 255]);
}

#[test]
fn hidden_color_under_transparency_never_bleeds_into_neighbours() {
    // Opaque red beside fully transparent saturated green, averaged into one pixel.
    let a = image(
        2,
        1,
        SampleLayout::Rgba8,
        vec![255, 0, 0, 255, 0, 255, 0, 0],
    );
    let l = resolved(
        (10, 10),
        false,
        vec![place(&a, 0, 0, 10, 10, Rotation::Deg0)],
    );
    let page = run(&l, &[&a]).unwrap();
    assert_eq!(px(&page.image, 0, 0), [255, 0, 0, 128]);
}

#[test]
fn partial_pixel_coverage_at_a_fractional_edge_becomes_alpha() {
    // Placement starts 5 um into a 10 um pixel, so that pixel is half covered.
    let a = image(1, 1, SampleLayout::Rgb8, vec![200, 100, 50]);
    let l = resolved(
        (30, 10),
        false,
        vec![place(&a, 5, 0, 10, 10, Rotation::Deg0)],
    );
    let page = run(&l, &[&a]).unwrap();
    assert_eq!(px(&page.image, 0, 0), [200, 100, 50, 128]);
    assert_eq!(px(&page.image, 1, 0), [200, 100, 50, 128]);
    assert_eq!(px(&page.image, 2, 0)[3], 0);
}

#[test]
fn later_placements_composite_over_earlier_ones() {
    let bottom = image(1, 1, SampleLayout::Rgb8, vec![255, 0, 0]);
    let top = image(1, 1, SampleLayout::Rgba8, vec![0, 0, 255, 128]);
    let l = resolved(
        (10, 10),
        false,
        vec![
            place(&bottom, 0, 0, 10, 10, Rotation::Deg0),
            place(&top, 0, 0, 10, 10, Rotation::Deg0),
        ],
    );
    let page = run(&l, &[&bottom, &top]).unwrap();
    // 128/255 blue over opaque red.
    let got = px(&page.image, 0, 0);
    assert_eq!(got[3], 255);
    assert_eq!(got, [127, 0, 128, 255]);
    // Reversing the order reverses the result.
    let l = resolved(
        (10, 10),
        false,
        vec![
            place(&top, 0, 0, 10, 10, Rotation::Deg0),
            place(&bottom, 0, 0, 10, 10, Rotation::Deg0),
        ],
    );
    assert_eq!(
        px(&run(&l, &[&bottom, &top]).unwrap().image, 0, 0),
        [255, 0, 0, 255]
    );
}

#[test]
fn crop_selects_a_sub_rectangle_of_the_source() {
    let a = image(3, 1, SampleLayout::Rgb8, vec![1, 1, 1, 2, 2, 2, 3, 3, 3]);
    let mut p = place(&a, 0, 0, 10, 10, Rotation::Deg0);
    p.crop_x_px = 1;
    p.crop_width_px = 1;
    let l = resolved((10, 10), false, vec![p]);
    assert_eq!(px(&run(&l, &[&a]).unwrap().image, 0, 0), [2, 2, 2, 255]);
}

#[test]
fn composition_is_deterministic_and_hash_names_the_pixels() {
    let a = image(5, 4, SampleLayout::Rgba8, rgba_ramp(5, 4));
    let l = resolved(
        (50, 50),
        true,
        vec![place(&a, 0, 0, 50, 40, Rotation::Deg90)],
    );
    let one = run(&l, &[&a]).unwrap();
    let two = run(&l, &[&a]).unwrap();
    assert_eq!(one.image.decoded.samples, two.image.decoded.samples);
    assert_eq!(one.image.original_hash, two.image.original_hash);
    assert_eq!(one.image.id, two.image.id);
}

#[test]
fn refuses_instead_of_approximating() {
    let a = image(2, 1, SampleLayout::Rgb8, vec![0; 6]);
    // Clipped placement.
    let l = resolved(
        (20, 10),
        false,
        vec![place(&a, 10, 0, 20, 10, Rotation::Deg0)],
    );
    assert!(matches!(run(&l, &[&a]), Err(ComposeError::OutsidePage(_))));
    // Missing asset.
    let l = resolved(
        (20, 10),
        false,
        vec![place(&a, 0, 0, 20, 10, Rotation::Deg0)],
    );
    assert!(matches!(run(&l, &[]), Err(ComposeError::MissingAsset(_))));
    // Crop outside the source.
    let mut p = place(&a, 0, 0, 20, 10, Rotation::Deg0);
    p.crop_width_px = 3;
    let l = resolved((20, 10), false, vec![p]);
    assert!(matches!(
        run(&l, &[&a]),
        Err(ComposeError::CropOutsideSource(_))
    ));
    // Mixed color encodings.
    let b = image_with(
        1,
        1,
        SampleLayout::Rgb8,
        vec![0; 3],
        ColorEncoding::CicpSrgb,
    );
    let l = resolved(
        (30, 10),
        false,
        vec![
            place(&a, 0, 0, 20, 10, Rotation::Deg0),
            place(&b, 20, 0, 10, 10, Rotation::Deg0),
        ],
    );
    assert!(matches!(
        run(&l, &[&a, &b]),
        Err(ComposeError::MixedColorEncoding)
    ));
    // Gray cannot be expanded when its encoding is not sRGB-like.
    let g = image_with(
        1,
        1,
        SampleLayout::Gray8,
        vec![5],
        ColorEncoding::GammaChromaticity {
            gamma: Some(45455),
            chromaticities: None,
        },
    );
    let l = resolved(
        (10, 10),
        false,
        vec![place(&g, 0, 0, 10, 10, Rotation::Deg0)],
    );
    assert!(matches!(
        run(&l, &[&g]),
        Err(ComposeError::UnsupportedColor(_))
    ));
}

#[test]
fn limits_and_cancellation_are_enforced() {
    let a = image(2, 1, SampleLayout::Rgb8, vec![0; 6]);
    let l = resolved(
        (20, 10),
        false,
        vec![place(&a, 0, 0, 20, 10, Rotation::Deg0)],
    );
    let small = compose_page(
        &ComposeInput {
            layout: &l,
            images: &[&a],
            masks: &[],
            pitch: PITCH,
            max_page_pixels: 1,
        },
        &|| false,
    );
    assert!(matches!(small, Err(ComposeError::PageTooLarge)));
    let cancelled = compose_page(
        &ComposeInput {
            layout: &l,
            images: &[&a],
            masks: &[],
            pitch: PITCH,
            max_page_pixels: 100,
        },
        &|| true,
    );
    assert!(matches!(cancelled, Err(ComposeError::Cancelled)));
}

#[test]
fn masks_follow_their_placement_transform() {
    let a = image(2, 1, SampleLayout::Rgb8, vec![0; 6]);
    let m = image(2, 1, SampleLayout::Gray8, vec![255, 100]);
    let mut p = place(&a, 10, 0, 20, 10, Rotation::Deg0);
    p.mask_id = Some(m.id);
    let l = resolved((40, 10), true, vec![p]);
    let page = compose_page(
        &ComposeInput {
            layout: &l,
            images: &[&a],
            masks: &[&m],
            pitch: PITCH,
            max_page_pixels: 100,
        },
        &|| false,
    )
    .unwrap();
    let mask = page.mask.unwrap();
    assert_eq!(mask.decoded.layout, SampleLayout::Gray8);
    // Mirrored on a 40 um page: placement occupies 10..30 um, mirrored stays 10..30 reversed.
    assert_eq!(mask.decoded.samples, vec![0, 100, 255, 0]);
}

#[test]
fn mask_must_match_its_source_dimensions() {
    let a = image(2, 1, SampleLayout::Rgb8, vec![0; 6]);
    let m = image(3, 1, SampleLayout::Gray8, vec![0; 3]);
    let mut p = place(&a, 0, 0, 20, 10, Rotation::Deg0);
    p.mask_id = Some(m.id);
    let l = resolved((20, 10), false, vec![p]);
    let r = compose_page(
        &ComposeInput {
            layout: &l,
            images: &[&a],
            masks: &[&m],
            pitch: PITCH,
            max_page_pixels: 100,
        },
        &|| false,
    );
    assert!(matches!(r, Err(ComposeError::MaskSizeMismatch(_))));
}

#[test]
fn icc_profiles_must_match_and_travel_with_the_page() {
    use veylora_domain::provenance::ContentHash;
    use veylora_image::ProfileBlob;
    let profile = |bytes: &[u8]| ProfileBlob {
        bytes: bytes.to_vec(),
        hash: ContentHash::digest(bytes),
    };
    let mut a = image_with(1, 1, SampleLayout::Rgb8, vec![1, 2, 3], ColorEncoding::Icc);
    let mut b = image_with(1, 1, SampleLayout::Rgb8, vec![4, 5, 6], ColorEncoding::Icc);
    a.profile = Some(profile(b"profile-one"));
    b.profile = Some(profile(b"profile-two"));
    let l = resolved(
        (20, 10),
        false,
        vec![
            place(&a, 0, 0, 10, 10, Rotation::Deg0),
            place(&b, 10, 0, 10, 10, Rotation::Deg0),
        ],
    );
    assert!(matches!(
        run(&l, &[&a, &b]),
        Err(ComposeError::MixedColorEncoding)
    ));
    // The same profile composes, and the page keeps it for the later color engine.
    b.profile = Some(profile(b"profile-one"));
    let page = run(&l, &[&a, &b]).unwrap();
    assert_eq!(page.image.profile.as_ref().unwrap().bytes, b"profile-one");
    assert_eq!(page.image.metadata.color, ColorEncoding::Icc);
}
