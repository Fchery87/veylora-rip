use png::Encoder;
use veylora_image::{
    decode_png, decode_png_with_policy, AlphaDescription, ColorMetadata, DecodeLimits, ImageError,
    UntaggedPolicy,
};

fn png(color: png::ColorType, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut encoder = Encoder::new(&mut out, 2, 1);
    encoder.set_color(color);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(data).unwrap();
    drop(writer);
    out
}

#[test]
fn retains_rgb_under_transparent_alpha_and_alpha_channel() {
    let asset = decode_png(
        &png(png::ColorType::Rgba, &[255, 0, 255, 0, 1, 2, 3, 128]),
        DecodeLimits::default(),
    )
    .unwrap();
    assert_eq!(asset.alpha, AlphaDescription::Straight);
    assert_eq!(asset.pixels, [255, 0, 255, 0, 1, 2, 3, 128]);
}

#[test]
fn default_untagged_policy_is_visible_and_configurable() {
    let bytes = png(png::ColorType::Rgb, &[1, 2, 3, 4, 5, 6]);
    assert_eq!(
        decode_png(&bytes, DecodeLimits::default())
            .unwrap()
            .metadata
            .color,
        ColorMetadata::UntaggedRgb
    );
    assert!(matches!(
        decode_png_with_policy(&bytes, DecodeLimits::default(), UntaggedPolicy::Reject),
        Err(ImageError::UntaggedColor)
    ));
}

#[test]
fn dimensions_are_rejected_before_pixel_allocation() {
    let bytes = png(png::ColorType::Rgb, &[1, 2, 3, 4, 5, 6]);
    let limits = DecodeLimits {
        max_pixels: 1,
        ..DecodeLimits::default()
    };
    assert!(matches!(
        decode_png(&bytes, limits),
        Err(ImageError::Dimensions)
    ));
}

#[test]
fn truncation_and_crc_fail_closed() {
    let bytes = png(png::ColorType::Rgb, &[1, 2, 3, 4, 5, 6]);
    assert!(decode_png(&bytes[..bytes.len() / 2], DecodeLimits::default()).is_err());
    let mut corrupt = bytes;
    let mut offset = 8;
    while offset + 12 <= corrupt.len() {
        let length = u32::from_be_bytes(corrupt[offset..offset + 4].try_into().unwrap()) as usize;
        if &corrupt[offset + 4..offset + 8] == b"IDAT" {
            corrupt[offset + 8] ^= 1;
            break;
        }
        offset += 12 + length;
    }
    assert!(decode_png(&corrupt, DecodeLimits::default()).is_err());
}
