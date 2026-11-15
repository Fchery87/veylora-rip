use crate::{
    axis::{axis_weights, SourceAxis},
    ALGORITHM,
};
use veylora_domain::{
    ids::{AssetId, PlacementId},
    layout::Rotation,
    provenance::ContentHash,
};
use veylora_image::{
    AlphaDescription, ColorEncoding, DecodedImage, ImageMetadata, ProfileBlob, SampleLayout,
    SourceAsset, UntaggedPolicy,
};
use veylora_layout::{ResolvedLayout, ResolvedPlacement};
use veylora_separation::PixelPitch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposeError {
    Cancelled,
    InvalidPitch,
    EmptyPage,
    PageTooLarge,
    MissingAsset(AssetId),
    MissingMask(AssetId),
    /// Decoded samples do not match the declared size and layout.
    Malformed(AssetId),
    CropOutsideSource(PlacementId),
    /// The placement leaves the page. Content is never clipped silently.
    OutsidePage(PlacementId),
    MixedColorEncoding,
    /// A gray source whose encoding is not sRGB-like cannot be expanded to RGB exactly.
    UnsupportedColor(AssetId),
    /// Masks must be Gray8.
    UnsupportedMask(AssetId),
    MaskSizeMismatch(PlacementId),
}

pub struct ComposeInput<'a> {
    pub layout: &'a ResolvedLayout,
    pub images: &'a [&'a SourceAsset],
    pub masks: &'a [&'a SourceAsset],
    pub pitch: PixelPitch,
    pub max_page_pixels: usize,
}

#[derive(Debug)]
pub struct ComposedPage {
    pub width_px: usize,
    pub height_px: usize,
    /// Straight-alpha RGBA8 page. Its hash covers the composed samples.
    pub image: SourceAsset,
    /// Gray8 page mask when any placement carries a mask.
    pub mask: Option<SourceAsset>,
}

struct Prepared<'a> {
    p: &'a ResolvedPlacement,
    source: &'a SourceAsset,
    /// Destination pixel ranges that can receive coverage.
    cols: Vec<(usize, SourceAxis, Vec<(u32, f64)>)>,
    rows: std::ops::Range<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Image,
    Mask,
}

fn find<'a>(list: &'a [&'a SourceAsset], id: AssetId) -> Option<&'a SourceAsset> {
    list.iter().copied().find(|a| a.id == id)
}

fn page_px(um: u32, pitch_nm: u32) -> usize {
    let (n, p) = (u64::from(um) * 1000, u64::from(pitch_nm));
    ((n * 2 + p) / (p * 2)) as usize
}

fn check_samples(a: &SourceAsset) -> Result<(), ComposeError> {
    let n = match a.decoded.layout {
        SampleLayout::Gray8 => 1,
        SampleLayout::GrayAlpha8 => 2,
        SampleLayout::Rgb8 => 3,
        SampleLayout::Rgba8 => 4,
    };
    let want = (a.metadata.width as usize)
        .checked_mul(a.metadata.height as usize)
        .and_then(|p| p.checked_mul(n));
    if want != Some(a.decoded.samples.len()) || a.metadata.width == 0 || a.metadata.height == 0 {
        return Err(ComposeError::Malformed(a.id));
    }
    Ok(())
}

/// Premultiplied RGBA in 0..=1 for one source pixel.
fn fetch(source: &SourceAsset, x: u32, y: u32, kind: Kind) -> [f64; 4] {
    let w = source.metadata.width as usize;
    let at = y as usize * w + x as usize;
    let s = &source.decoded.samples;
    let c = |v: u8| f64::from(v) / 255.0;
    if kind == Kind::Mask {
        return [c(s[at]), 0.0, 0.0, 1.0];
    }
    let (r, g, b, a) = match source.decoded.layout {
        SampleLayout::Gray8 => (c(s[at]), c(s[at]), c(s[at]), 1.0),
        SampleLayout::GrayAlpha8 => (c(s[at * 2]), c(s[at * 2]), c(s[at * 2]), c(s[at * 2 + 1])),
        SampleLayout::Rgb8 => (c(s[at * 3]), c(s[at * 3 + 1]), c(s[at * 3 + 2]), 1.0),
        SampleLayout::Rgba8 => (
            c(s[at * 4]),
            c(s[at * 4 + 1]),
            c(s[at * 4 + 2]),
            c(s[at * 4 + 3]),
        ),
    };
    [r * a, g * a, b * a, a]
}

/// The source axis and interval (nanometres) that a destination interval along the page's x or y
/// axis reads. `a..b` is local to the placement's final bounding box.
fn source_interval(
    p: &ResolvedPlacement,
    horizontal: bool,
    a: i128,
    b: i128,
) -> (SourceAxis, i128, i128) {
    let (w, h) = (
        i128::from(p.source_width_um) * 1000,
        i128::from(p.source_height_um) * 1000,
    );
    let (bw, _) = match p.rotation {
        Rotation::Deg0 | Rotation::Deg180 => (w, h),
        Rotation::Deg90 | Rotation::Deg270 => (h, w),
    };
    // Undo the sheet mirror, which flips the final box about its own vertical axis.
    let (a, b) = if horizontal && p.mirrored {
        (bw - b, bw - a)
    } else {
        (a, b)
    };
    match (p.rotation, horizontal) {
        (Rotation::Deg0, true) => (SourceAxis::X, a, b),
        (Rotation::Deg0, false) => (SourceAxis::Y, a, b),
        (Rotation::Deg90, true) => (SourceAxis::Y, h - b, h - a),
        (Rotation::Deg90, false) => (SourceAxis::X, a, b),
        (Rotation::Deg180, true) => (SourceAxis::X, w - b, w - a),
        (Rotation::Deg180, false) => (SourceAxis::Y, h - b, h - a),
        (Rotation::Deg270, true) => (SourceAxis::Y, a, b),
        (Rotation::Deg270, false) => (SourceAxis::X, w - b, w - a),
    }
}

fn weights(
    p: &ResolvedPlacement,
    axis: SourceAxis,
    lo: i128,
    hi: i128,
    pixel_nm: u32,
) -> Vec<(u32, f64)> {
    match axis {
        SourceAxis::X => axis_weights(
            lo,
            hi,
            i128::from(p.source_width_um) * 1000,
            p.crop_width_px,
            i128::from(pixel_nm),
        ),
        SourceAxis::Y => axis_weights(
            lo,
            hi,
            i128::from(p.source_height_um) * 1000,
            p.crop_height_px,
            i128::from(pixel_nm),
        ),
    }
}

fn prepare<'a>(
    p: &'a ResolvedPlacement,
    source: &'a SourceAsset,
    pitch: PixelPitch,
    size: (usize, usize),
) -> Prepared<'a> {
    let (px_nm, py_nm) = (i128::from(pitch.x_nm), i128::from(pitch.y_nm));
    let (ox, oy) = (i128::from(p.rect.x) * 1000, i128::from(p.rect.y) * 1000);
    let (bw, bh) = (
        i128::from(p.rect.width) * 1000,
        i128::from(p.rect.height) * 1000,
    );
    let range = |origin: i128, extent: i128, pitch: i128, limit: usize| {
        let first = (origin / pitch).max(0) as usize;
        let last = (((origin + extent + pitch - 1) / pitch).max(0) as usize).min(limit);
        first..last.max(first)
    };
    let cols = range(ox, bw, px_nm, size.0)
        .filter_map(|px| {
            let (a, b) = (
                (px as i128 * px_nm - ox).max(0),
                ((px as i128 + 1) * px_nm - ox).min(bw),
            );
            if a >= b {
                return None;
            }
            let (axis, lo, hi) = source_interval(p, true, a, b);
            let w = weights(p, axis, lo, hi, pitch.x_nm);
            (!w.is_empty()).then_some((px, axis, w))
        })
        .collect();
    Prepared {
        p,
        source,
        cols,
        rows: range(oy, bh, py_nm, size.1),
    }
}

fn composite(
    items: &[Prepared<'_>],
    kind: Kind,
    pitch: PixelPitch,
    size: (usize, usize),
    cancelled: &dyn Fn() -> bool,
) -> Result<Vec<u8>, ComposeError> {
    let (w, h) = size;
    let channels = if kind == Kind::Image { 4 } else { 1 };
    let mut out = Vec::with_capacity(w * h * channels);
    let mut row = vec![[0.0f64; 4]; w];
    for py in 0..h {
        if cancelled() {
            return Err(ComposeError::Cancelled);
        }
        row.fill([0.0; 4]);
        for it in items {
            if !it.rows.contains(&py) {
                continue;
            }
            let p = it.p;
            let (oy, bh) = (
                i128::from(p.rect.y) * 1000,
                i128::from(p.rect.height) * 1000,
            );
            let py_nm = i128::from(pitch.y_nm);
            let (a, b) = (
                (py as i128 * py_nm - oy).max(0),
                ((py as i128 + 1) * py_nm - oy).min(bh),
            );
            if a >= b {
                continue;
            }
            let (row_axis, lo, hi) = source_interval(p, false, a, b);
            let row_w = weights(p, row_axis, lo, hi, pitch.y_nm);
            if row_w.is_empty() {
                continue;
            }
            for (px, col_axis, col_w) in &it.cols {
                let mut c = [0.0f64; 4];
                for &(ci, wc) in col_w {
                    for &(ri, wr) in &row_w {
                        let (sx, sy) = match col_axis {
                            SourceAxis::X => (ci, ri),
                            SourceAxis::Y => (ri, ci),
                        };
                        let s = fetch(it.source, p.crop_x_px + sx, p.crop_y_px + sy, kind);
                        let weight = wc * wr;
                        for k in 0..4 {
                            c[k] += s[k] * weight;
                        }
                    }
                }
                let below = row[*px];
                let keep = 1.0 - c[3];
                for k in 0..4 {
                    row[*px][k] = c[k] + below[k] * keep;
                }
            }
        }
        let q = |v: f64| (v * 255.0 + 0.5).floor().clamp(0.0, 255.0) as u8;
        for px in &row {
            if kind == Kind::Mask {
                out.push(q(px[0]));
            } else if px[3] <= 0.0 {
                out.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let a = q(px[3]);
                if a == 0 {
                    out.extend_from_slice(&[0, 0, 0, 0]);
                } else {
                    out.extend_from_slice(&[
                        q(px[0] / px[3]),
                        q(px[1] / px[3]),
                        q(px[2] / px[3]),
                        a,
                    ]);
                }
            }
        }
    }
    Ok(out)
}

fn synthetic(
    width: usize,
    height: usize,
    layout: SampleLayout,
    samples: Vec<u8>,
    color: ColorEncoding,
    profile: Option<&ProfileBlob>,
    policy: UntaggedPolicy,
) -> SourceAsset {
    let hash = ContentHash::digest(&samples);
    let hex = hash.to_string();
    let id = format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
    .parse()
    .expect("32 hex digits form a UUID");
    let alpha = if layout == SampleLayout::Rgba8 {
        AlphaDescription::StraightCoverage
    } else {
        AlphaDescription::Opaque
    };
    SourceAsset {
        id,
        original_hash: hash,
        original_bytes: Vec::new(),
        metadata: ImageMetadata {
            width: width as u32,
            height: height as u32,
            color,
            physical: None,
            warnings: Vec::new(),
            untagged_policy: policy,
            decoder_version: ALGORITHM,
            file_type: "composed-page",
        },
        profile: profile.map(|p| ProfileBlob {
            bytes: p.bytes.clone(),
            hash: p.hash,
        }),
        alpha,
        decoded: DecodedImage {
            samples,
            layout,
            alpha,
            converted_to_working_space: false,
        },
    }
}

/// Composes every placement into one page raster. `cancelled` is polled once per page row.
pub fn compose_page(
    input: &ComposeInput<'_>,
    cancelled: &dyn Fn() -> bool,
) -> Result<ComposedPage, ComposeError> {
    if cancelled() {
        return Err(ComposeError::Cancelled);
    }
    let pitch = input.pitch;
    if pitch.x_nm == 0 || pitch.y_nm == 0 {
        return Err(ComposeError::InvalidPitch);
    }
    let layout = input.layout;
    let size = (
        page_px(layout.page_width_um, pitch.x_nm),
        page_px(layout.page_height_um, pitch.y_nm),
    );
    if size.0 == 0 || size.1 == 0 {
        return Err(ComposeError::EmptyPage);
    }
    if size
        .0
        .checked_mul(size.1)
        .is_none_or(|n| n > input.max_page_pixels)
    {
        return Err(ComposeError::PageTooLarge);
    }

    let mut images = Vec::new();
    let mut masks = Vec::new();
    let mut shared: Option<&SourceAsset> = None;
    for p in &layout.placements {
        let source =
            find(input.images, p.asset_id).ok_or(ComposeError::MissingAsset(p.asset_id))?;
        check_samples(source)?;
        let crop_fits = u64::from(p.crop_x_px) + u64::from(p.crop_width_px)
            <= u64::from(source.metadata.width)
            && u64::from(p.crop_y_px) + u64::from(p.crop_height_px)
                <= u64::from(source.metadata.height);
        if !crop_fits {
            return Err(ComposeError::CropOutsideSource(p.id));
        }
        let inside = p.rect.x >= 0
            && p.rect.y >= 0
            && p.rect.right() <= i64::from(layout.page_width_um)
            && p.rect.bottom() <= i64::from(layout.page_height_um);
        if !inside {
            return Err(ComposeError::OutsidePage(p.id));
        }
        let first = *shared.get_or_insert(source);
        let same = source.metadata.color == first.metadata.color
            && source.profile.as_ref().map(|b| b.hash) == first.profile.as_ref().map(|b| b.hash);
        if !same {
            return Err(ComposeError::MixedColorEncoding);
        }
        let gray = matches!(
            source.decoded.layout,
            SampleLayout::Gray8 | SampleLayout::GrayAlpha8
        );
        let srgb_like = matches!(
            source.metadata.color,
            ColorEncoding::CicpSrgb | ColorEncoding::Srgb { .. } | ColorEncoding::AssumedSrgb
        );
        if gray && !srgb_like {
            return Err(ComposeError::UnsupportedColor(source.id));
        }
        images.push(prepare(p, source, pitch, size));

        if let Some(mask_id) = p.mask_id {
            let mask = find(input.masks, mask_id).ok_or(ComposeError::MissingMask(mask_id))?;
            check_samples(mask)?;
            if mask.decoded.layout != SampleLayout::Gray8 {
                return Err(ComposeError::UnsupportedMask(mask_id));
            }
            if (mask.metadata.width, mask.metadata.height)
                != (source.metadata.width, source.metadata.height)
            {
                return Err(ComposeError::MaskSizeMismatch(p.id));
            }
            masks.push(prepare(p, mask, pitch, size));
        }
    }
    let first = shared.ok_or(ComposeError::EmptyPage)?;
    let samples = composite(&images, Kind::Image, pitch, size, cancelled)?;
    let image = synthetic(
        size.0,
        size.1,
        SampleLayout::Rgba8,
        samples,
        first.metadata.color.clone(),
        first.profile.as_ref(),
        first.metadata.untagged_policy,
    );
    let mask = if masks.is_empty() {
        None
    } else {
        let samples = composite(&masks, Kind::Mask, pitch, size, cancelled)?;
        Some(synthetic(
            size.0,
            size.1,
            SampleLayout::Gray8,
            samples,
            ColorEncoding::AssumedSrgb,
            None,
            UntaggedPolicy::AssumeSrgb,
        ))
    };
    Ok(ComposedPage {
        width_px: size.0,
        height_px: size.1,
        image,
        mask,
    })
}
