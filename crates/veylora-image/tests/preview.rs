//! Gray preview PNGs for showing retained planes. Output only; nothing here is a source import.
use veylora_image::encode_gray8_png;

fn decode(bytes: &[u8]) -> (u32, u32, Vec<u8>) {
    let mut reader = ::png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf).unwrap();
    assert_eq!(info.color_type, ::png::ColorType::Grayscale);
    assert_eq!(info.bit_depth, ::png::BitDepth::Eight);
    buf.truncate(info.buffer_size());
    (info.width, info.height, buf)
}

#[test]
fn pixels_round_trip_exactly() {
    let pixels: Vec<u8> = (0..24).map(|i| (i * 10) as u8).collect();
    let png = encode_gray8_png(6, 4, &pixels).unwrap();
    assert_eq!(decode(&png), (6, 4, pixels));
}

#[test]
fn encoding_is_deterministic() {
    let pixels = vec![0u8, 255, 17, 90];
    assert_eq!(
        encode_gray8_png(2, 2, &pixels).unwrap(),
        encode_gray8_png(2, 2, &pixels).unwrap()
    );
}

#[test]
fn wrong_lengths_and_empty_images_are_refused() {
    assert!(encode_gray8_png(2, 2, &[0, 1, 2]).is_err());
    assert!(encode_gray8_png(0, 2, &[]).is_err());
    assert!(encode_gray8_png(2, 0, &[]).is_err());
}

#[test]
fn rgb_pixels_round_trip_exactly() {
    let pixels: Vec<u8> = (0..36).map(|i| (i * 7) as u8).collect();
    let png = veylora_image::encode_rgb8_png(4, 3, &pixels).unwrap();
    let mut reader = ::png::Decoder::new(std::io::Cursor::new(&png))
        .read_info()
        .unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf).unwrap();
    assert_eq!(info.color_type, ::png::ColorType::Rgb);
    assert_eq!((info.width, info.height), (4, 3));
    assert_eq!(&buf[..info.buffer_size()], &pixels[..]);
    assert_eq!(png, veylora_image::encode_rgb8_png(4, 3, &pixels).unwrap());
}

#[test]
fn rgb_wrong_lengths_and_empty_images_are_refused() {
    assert!(veylora_image::encode_rgb8_png(2, 2, &[0; 11]).is_err());
    assert!(veylora_image::encode_rgb8_png(0, 2, &[]).is_err());
}
