use flate2::{write::ZlibEncoder, Compression};
use std::io::Write;
use veylora_image::{decode_png, ColorEncoding, DecodeLimits, ImageError};

#[test]
fn configurable_untagged_policy_rejects_without_overriding_explicit_tags() {
    use veylora_image::{decode_png_with_policy, UntaggedPolicy};
    let b = include_bytes!("../../../fixtures/images/hidden-rgb.png");
    assert_eq!(
        decode_png_with_policy(b, DecodeLimits::default(), UntaggedPolicy::Reject).unwrap_err(),
        ImageError::UntaggedColor
    );
    let a = decode_png_with_policy(b, DecodeLimits::default(), UntaggedPolicy::AssumeSrgb).unwrap();
    assert_eq!(a.metadata.untagged_policy, UntaggedPolicy::AssumeSrgb);
    let tagged = include_bytes!("../../../fixtures/images/srgb.png");
    let a =
        decode_png_with_policy(tagged, DecodeLimits::default(), UntaggedPolicy::Reject).unwrap();
    assert_eq!(
        a.metadata.color,
        ColorEncoding::Srgb {
            rendering_intent: 0
        }
    );
    assert_eq!(a.metadata.untagged_policy, UntaggedPolicy::Reject);
}

fn chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&(data.len() as u32).to_be_bytes());
    b.extend_from_slice(kind);
    b.extend_from_slice(data);
    b.extend_from_slice(&crc32fast::hash(&b[4..]).to_be_bytes());
    b
}
fn tagged(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let b = include_bytes!("../../../fixtures/images/hidden-rgb.png");
    [b[..33].to_vec(), chunk(kind, data), b[33..].to_vec()].concat()
}
fn compressed(b: &[u8]) -> Vec<u8> {
    let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
    z.write_all(b).unwrap();
    z.finish().unwrap()
}
fn iccp(profile: &[u8]) -> Vec<u8> {
    [b"synthetic\0\0".to_vec(), compressed(profile)].concat()
}
fn profile() -> Vec<u8> {
    let mut p = vec![0; 132];
    p[..4].copy_from_slice(&132u32.to_be_bytes());
    p[8] = 4;
    p[12..16].copy_from_slice(b"mntr");
    p[16..20].copy_from_slice(b"RGB ");
    p[20..24].copy_from_slice(b"XYZ ");
    p[36..40].copy_from_slice(b"acsp");
    p
}
#[test]
fn preserves_hidden_rgb_and_straight_alpha() {
    let bytes = include_bytes!("../../../fixtures/images/hidden-rgb.png");
    let asset = decode_png(bytes, DecodeLimits::default()).unwrap();
    assert_eq!(asset.decoded.samples, [255, 0, 127, 0, 255, 255, 255, 255]);
    assert_eq!(asset.original_bytes, bytes);
    assert_eq!(
        asset.original_hash,
        veylora_domain::provenance::ContentHash::digest(bytes)
    );
    assert!(!asset.decoded.converted_to_working_space);
    assert_eq!(asset.metadata.color, ColorEncoding::AssumedSrgb);
    assert!(!asset.metadata.warnings.is_empty());
}
#[test]
fn preserves_partial_alpha() {
    let a = decode_png(
        include_bytes!("../../../fixtures/images/alpha-ramp.png"),
        DecodeLimits::default(),
    )
    .unwrap();
    assert_eq!(
        a.decoded
            .samples
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| p[3])
            .collect::<Vec<_>>(),
        [0, 64, 128, 255]
    );
}
#[test]
fn rejects_animation_hdr_and_conflicts() {
    for (b, e) in [
        (
            &include_bytes!("../../../fixtures/images/unsupported-apng.png")[..],
            ImageError::Animation,
        ),
        (
            &include_bytes!("../../../fixtures/images/unsupported-hdr.png")[..],
            ImageError::Hdr,
        ),
        (
            &include_bytes!("../../../fixtures/images/conflicting-color-tags.png")[..],
            ImageError::ColorConflict,
        ),
    ] {
        assert_eq!(decode_png(b, DecodeLimits::default()).unwrap_err(), e);
    }
}
#[test]
fn retains_non_square_resolution_without_resampling() {
    let a = decode_png(
        include_bytes!("../../../fixtures/images/non-square-ppi.png"),
        DecodeLimits::default(),
    )
    .unwrap();
    let p = a.metadata.physical.unwrap();
    assert_eq!(
        (p.x_pixels_per_unit, p.y_pixels_per_unit, p.unit_is_meter),
        (3780, 7560, true)
    );
    let mm = a.metadata.physical_size_mm().unwrap();
    assert!((mm.0 - 2000.0 / 3780.0).abs() < 1e-12);
    assert!((mm.1 - 1000.0 / 7560.0).abs() < 1e-12);
}
#[test]
fn enforces_each_budget_before_raster_decode() {
    let b = include_bytes!("../../../fixtures/images/hidden-rgb.png");
    for l in [
        DecodeLimits {
            max_file_bytes: 10,
            ..Default::default()
        },
        DecodeLimits {
            max_dimension: 1,
            ..Default::default()
        },
        DecodeLimits {
            max_pixels: 1,
            ..Default::default()
        },
        DecodeLimits {
            max_decoded_bytes: 7,
            ..Default::default()
        },
        DecodeLimits {
            max_metadata_bytes: 24,
            ..Default::default()
        },
    ] {
        assert_eq!(decode_png(b, l).unwrap_err(), ImageError::ResourceLimit);
    }
}
#[test]
fn checks_all_truncations_crc_and_trailing_data() {
    let b = include_bytes!("../../../fixtures/images/hidden-rgb.png");
    for n in 0..b.len() {
        assert!(decode_png(&b[..n], DecodeLimits::default()).is_err(), "{n}");
    }
    let mut bad = b.to_vec();
    bad[45] ^= 1;
    assert_eq!(
        decode_png(&bad, DecodeLimits::default()).unwrap_err(),
        ImageError::Malformed
    );
    let mut trailing = b.to_vec();
    trailing.push(0);
    assert_eq!(
        decode_png(&trailing, DecodeLimits::default()).unwrap_err(),
        ImageError::Malformed
    );
}
#[test]
fn bounds_profile_inflation_and_rejects_malformed_profiles() {
    let bomb = tagged(b"iCCP", &iccp(&vec![0; 100_000]));
    assert_eq!(
        decode_png(
            &bomb,
            DecodeLimits {
                max_profile_bytes: 256,
                ..Default::default()
            }
        )
        .unwrap_err(),
        ImageError::ResourceLimit
    );
    assert_eq!(
        decode_png(
            &tagged(b"iCCP", &iccp(b"not an ICC")),
            DecodeLimits::default()
        )
        .unwrap_err(),
        ImageError::MalformedProfile
    );
    let mut bad = profile();
    bad[128..132].copy_from_slice(&1u32.to_be_bytes());
    assert_eq!(
        decode_png(&tagged(b"iCCP", &iccp(&bad)), DecodeLimits::default()).unwrap_err(),
        ImageError::MalformedProfile
    );
}
#[test]
fn preserves_profile_and_explicit_color_precedence() {
    let p = profile();
    let a = decode_png(&tagged(b"iCCP", &iccp(&p)), DecodeLimits::default()).unwrap();
    assert_eq!(a.profile.unwrap().bytes, p);
    assert_eq!(a.metadata.color, ColorEncoding::Icc);
    let a = decode_png(
        include_bytes!("../../../fixtures/images/srgb.png"),
        DecodeLimits::default(),
    )
    .unwrap();
    assert_eq!(
        a.metadata.color,
        ColorEncoding::Srgb {
            rendering_intent: 0
        }
    );
    let a = decode_png(&tagged(b"cICP", &[1, 13, 0, 1]), DecodeLimits::default()).unwrap();
    assert_eq!(a.metadata.color, ColorEncoding::CicpSrgb);
}
#[test]
fn rejects_compressed_text_without_inflating_it() {
    let b = tagged(
        b"zTXt",
        &[b"comment\0\0".to_vec(), compressed(&vec![0; 100_000])].concat(),
    );
    assert_eq!(
        decode_png(&b, DecodeLimits::default()).unwrap_err(),
        ImageError::UnsupportedMetadata
    );
}
#[test]
fn rejects_unsupported_depth_and_huge_dimensions_before_bad_idat() {
    let b = include_bytes!("../../../fixtures/images/hidden-rgb.png");
    let mut ihdr = b[16..29].to_vec();
    ihdr[..4].copy_from_slice(&u32::MAX.to_be_bytes());
    let png = [
        b[..8].to_vec(),
        chunk(b"IHDR", &ihdr),
        chunk(b"IDAT", b"invalid"),
        chunk(b"IEND", &[]),
    ]
    .concat();
    assert_eq!(
        decode_png(&png, DecodeLimits::default()).unwrap_err(),
        ImageError::ResourceLimit
    );
    ihdr[..4].copy_from_slice(&2u32.to_be_bytes());
    ihdr[8] = 16;
    let png = [b[..8].to_vec(), chunk(b"IHDR", &ihdr), b[33..].to_vec()].concat();
    assert_eq!(
        decode_png(&png, DecodeLimits::default()).unwrap_err(),
        ImageError::UnsupportedFormat
    );
}

fn with_idat(data: &[u8]) -> Vec<u8> {
    let b = include_bytes!("../../../fixtures/images/hidden-rgb.png");
    [b[..33].to_vec(), chunk(b"IDAT", data), chunk(b"IEND", &[])].concat()
}

#[test]
fn bounds_idat_bombs_and_validates_zlib_end() {
    assert_eq!(
        decode_png(
            &with_idat(&compressed(&vec![0; 100_000])),
            DecodeLimits::default()
        )
        .unwrap_err(),
        ImageError::ResourceLimit
    );
    let raw = [0, 255, 0, 127, 0, 255, 255, 255, 255];
    let z = compressed(&raw);
    for n in 0..z.len() {
        assert!(decode_png(&with_idat(&z[..n]), DecodeLimits::default()).is_err());
    }
    let mut trailing = z.clone();
    trailing.push(0);
    assert_eq!(
        decode_png(&with_idat(&trailing), DecodeLimits::default()).unwrap_err(),
        ImageError::Malformed
    );
    let mut bad_adler = z;
    let last = bad_adler.len() - 1;
    bad_adler[last] ^= 1;
    assert_eq!(
        decode_png(&with_idat(&bad_adler), DecodeLimits::default()).unwrap_err(),
        ImageError::Malformed
    );
}

#[test]
fn rejects_invalid_chromaticities() {
    assert_eq!(
        decode_png(&tagged(b"cHRM", &[0; 32]), DecodeLimits::default()).unwrap_err(),
        ImageError::Malformed
    );
}

#[test]
fn rejects_malformed_plain_metadata() {
    for (kind, data) in [
        (b"tEXt", &b"no separator"[..]),
        (b"tIME", &b"short"[..]),
        (b"bKGD", &b"short"[..]),
    ] {
        assert_eq!(
            decode_png(&tagged(kind, data), DecodeLimits::default()).unwrap_err(),
            ImageError::Malformed
        );
    }
}

#[test]
fn decodes_rgb_and_grayscale_without_changing_samples() {
    for (color, samples) in [
        (::png::ColorType::Rgb, vec![255, 255, 255]),
        (::png::ColorType::Grayscale, vec![127]),
        (::png::ColorType::GrayscaleAlpha, vec![99, 64]),
    ] {
        let mut b = Vec::new();
        {
            let mut encoder = ::png::Encoder::new(&mut b, 1, 1);
            encoder.set_color(color);
            encoder.set_depth(::png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&samples)
                .unwrap();
        }
        assert_eq!(
            decode_png(&b, DecodeLimits::default())
                .unwrap()
                .decoded
                .samples,
            samples
        );
    }
}

#[test]
fn rejects_duplicate_tags_and_nonconsecutive_idat() {
    let b = include_bytes!("../../../fixtures/images/hidden-rgb.png");
    let dup = [
        b[..33].to_vec(),
        chunk(b"sRGB", &[0]),
        chunk(b"sRGB", &[0]),
        b[33..].to_vec(),
    ]
    .concat();
    assert_eq!(
        decode_png(&dup, DecodeLimits::default()).unwrap_err(),
        ImageError::Malformed
    );
    let gap = [
        b[..b.len() - 12].to_vec(),
        chunk(b"tEXt", b"note\0synthetic"),
        chunk(b"IDAT", &[]),
        chunk(b"IEND", &[]),
    ]
    .concat();
    assert_eq!(
        decode_png(&gap, DecodeLimits::default()).unwrap_err(),
        ImageError::Malformed
    );
}

#[test]
fn honors_precedence_while_retaining_overridden_profile() {
    let b = tagged(b"iCCP", &iccp(&profile()));
    let with_cicp = [
        b[..33].to_vec(),
        chunk(b"cICP", &[1, 13, 0, 1]),
        b[33..].to_vec(),
    ]
    .concat();
    let a = decode_png(&with_cicp, DecodeLimits::default()).unwrap();
    assert_eq!(a.metadata.color, ColorEncoding::CicpSrgb);
    assert!(a.profile.is_some());
    assert_eq!(
        a.metadata.warnings,
        [veylora_image::ImportWarning::LowerPriorityColorMetadata]
    );
    let conflict = [b[..33].to_vec(), chunk(b"sRGB", &[0]), b[33..].to_vec()].concat();
    assert_eq!(
        decode_png(&conflict, DecodeLimits::default()).unwrap_err(),
        ImageError::ColorConflict
    );
}

#[test]
fn decodes_split_idat_and_rejects_too_small_decoder_budget() {
    let b = include_bytes!("../../../fixtures/images/hidden-rgb.png");
    let z = compressed(&[0, 255, 0, 127, 0, 255, 255, 255, 255]);
    for split in 0..=z.len() {
        let split_png = [
            b[..33].to_vec(),
            chunk(b"IDAT", &z[..split]),
            chunk(b"IDAT", &z[split..]),
            chunk(b"IEND", &[]),
        ]
        .concat();
        assert!(
            decode_png(&split_png, DecodeLimits::default()).is_ok(),
            "split {split}"
        );
    }
    assert_eq!(
        decode_png(
            b,
            DecodeLimits {
                max_decoder_bytes: 0,
                ..Default::default()
            }
        )
        .unwrap_err(),
        ImageError::ResourceLimit
    );
}

#[test]
fn malformed_chunk_payloads_never_panic() {
    // Deterministic, bounded smoke fuzzing of CRC-valid payloads and framing.
    let mut seed = 123456789u32;
    for kind in [
        b"IHDR", b"iCCP", b"cHRM", b"sRGB", b"cICP", b"pHYs", b"tEXt", b"tIME", b"bKGD", b"IDAT",
    ] {
        for len in 0..128 {
            let data: Vec<_> = (0..len)
                .map(|_| {
                    seed ^= seed << 13;
                    seed ^= seed >> 17;
                    seed ^= seed << 5;
                    seed as u8
                })
                .collect();
            let _ = decode_png(&tagged(kind, &data), DecodeLimits::default());
        }
    }
}
