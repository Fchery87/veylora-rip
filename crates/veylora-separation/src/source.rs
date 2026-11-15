//! Bridge from decoded source assets to coverage planes. No color conversion happens here:
//! samples keep their source encoding, and values are only scaled from 0..=255 to 0.0..=1.0.
use crate::{Plane, SeparationError};
use veylora_image::{SampleLayout, SourceAsset};

const fn channels(layout: SampleLayout) -> usize {
    match layout {
        SampleLayout::Gray8 => 1,
        SampleLayout::GrayAlpha8 => 2,
        SampleLayout::Rgb8 => 3,
        SampleLayout::Rgba8 => 4,
    }
}

fn dimensions(asset: &SourceAsset) -> Result<(usize, usize, usize), SeparationError> {
    let width = asset.metadata.width as usize;
    let height = asset.metadata.height as usize;
    let n = channels(asset.decoded.layout);
    let expected = width
        .checked_mul(height)
        .and_then(|p| p.checked_mul(n))
        .ok_or(SeparationError::MalformedSource)?;
    if width == 0 || height == 0 || asset.decoded.samples.len() != expected {
        return Err(SeparationError::MalformedSource);
    }
    Ok((width, height, n))
}

/// Validated `(width, height, rows)` for a band of source rows, with the samples of those rows.
pub(crate) fn band<'a>(
    asset: &'a SourceAsset,
    rows: &std::ops::Range<usize>,
) -> Result<(usize, &'a [u8], usize), SeparationError> {
    let (width, height, n) = dimensions(asset)?;
    if rows.start >= rows.end || rows.end > height {
        return Err(SeparationError::DimensionMismatch);
    }
    let stride = width * n;
    Ok((
        width,
        &asset.decoded.samples[rows.start * stride..rows.end * stride],
        n,
    ))
}

/// Straight alpha coverage. Sources without alpha are fully covered.
pub fn alpha_plane(asset: &SourceAsset) -> Result<Plane, SeparationError> {
    let (_, height, _) = dimensions(asset)?;
    alpha_rows(asset, 0..height)
}

/// Alpha coverage for a band of source rows, so large pages never need a whole-page plane.
pub fn alpha_rows(
    asset: &SourceAsset,
    rows: std::ops::Range<usize>,
) -> Result<Plane, SeparationError> {
    let (width, samples, n) = band(asset, &rows)?;
    let height = rows.len();
    let pixels = match asset.decoded.layout {
        SampleLayout::Gray8 | SampleLayout::Rgb8 => vec![1.0; width * height],
        SampleLayout::GrayAlpha8 | SampleLayout::Rgba8 => samples
            .chunks_exact(n)
            .map(|p| f32::from(p[n - 1]) / 255.0)
            .collect(),
    };
    Plane::from_pixels(width, height, pixels)
}

/// A grayscale supplied mask: sample value is coverage, tags are not interpreted.
/// Only Gray8 is accepted; RGB or alpha-bearing sources are never reduced implicitly.
pub fn mask_plane(asset: &SourceAsset) -> Result<Plane, SeparationError> {
    let (_, height, _) = dimensions(asset)?;
    mask_rows(asset, 0..height)
}

/// Supplied-mask coverage for a band of rows. Same rules as [`mask_plane`].
pub fn mask_rows(
    asset: &SourceAsset,
    rows: std::ops::Range<usize>,
) -> Result<Plane, SeparationError> {
    let (width, samples, _) = band(asset, &rows)?;
    if asset.decoded.layout != SampleLayout::Gray8 {
        return Err(SeparationError::UnsupportedSource);
    }
    Plane::from_pixels(
        width,
        rows.len(),
        samples.iter().map(|v| f32::from(*v) / 255.0).collect(),
    )
}
