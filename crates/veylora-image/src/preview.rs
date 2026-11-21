//! Encoding of 8-bit gray preview images. Used to show retained planes; not a source import path.

/// Encodes `width * height` 8-bit gray samples (row-major) as a PNG. The output is deterministic.
pub fn encode_gray8_png(width: u32, height: u32, pixels: &[u8]) -> Result<Vec<u8>, String> {
    if width == 0 || height == 0 {
        return Err("A preview image needs a width and height.".to_owned());
    }
    if pixels.len() as u64 != u64::from(width) * u64::from(height) {
        return Err(format!(
            "A {width} x {height} preview needs {} samples, not {}.",
            u64::from(width) * u64::from(height),
            pixels.len()
        ));
    }
    let mut out = Vec::new();
    {
        let mut encoder = ::png::Encoder::new(&mut out, width, height);
        encoder.set_color(::png::ColorType::Grayscale);
        encoder.set_depth(::png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(pixels).map_err(|e| e.to_string())?;
    }
    Ok(out)
}
