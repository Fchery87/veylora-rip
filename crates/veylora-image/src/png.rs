use crate::{
    container::{u32be, Chunks},
    inflate::inflate,
    *,
};
use std::io::Cursor;
use veylora_domain::{ids::AssetId, provenance::ContentHash};

/// Import the explicitly supported SDR PNG subset with source-encoded samples.
/// Limits apply independently to input retention, output samples, metadata, ICC, and decoder scratch.
pub fn decode_png(bytes: &[u8], limits: DecodeLimits) -> Result<SourceAsset, ImageError> {
    decode_png_with_policy(bytes, limits, UntaggedPolicy::AssumeSrgb)
}

/// Like [`decode_png`], with an explicit policy for sources that carry no color tag.
/// The policy never overrides an explicit tag. `Reject` fails before any raster allocation.
pub fn decode_png_with_policy(
    bytes: &[u8],
    limits: DecodeLimits,
    untagged: UntaggedPolicy,
) -> Result<SourceAsset, ImageError> {
    let h = container::preflight(bytes, limits)?;
    let (metadata, profile) = metadata(bytes, &h, limits, untagged)?;
    // Validate exact zlib length/checksum/end before the decoder can allocate output.
    let size = inflate(
        Chunks::new(bytes).filter_map(|(k, d)| (k == b"IDAT").then_some(d)),
        h.filtered,
        |_| Ok(()),
    )?;
    if size != h.filtered {
        return Err(ImageError::Malformed);
    }
    let mut decoder = ::png::Decoder::new_with_limits(
        Cursor::new(bytes),
        ::png::Limits {
            bytes: limits.max_decoder_bytes,
        },
    );
    decoder.set_ignore_text_chunk(true);
    decoder.set_ignore_iccp_chunk(true);
    let mut reader = decoder.read_info().map_err(decoder_error)?;
    if reader.output_buffer_size() != Some(h.decoded) {
        return Err(ImageError::Malformed);
    }
    let mut samples = allocate(h.decoded)?;
    let out = reader.next_frame(&mut samples).map_err(decoder_error)?;
    if out.buffer_size() != h.decoded {
        return Err(ImageError::Malformed);
    }
    reader.finish().map_err(decoder_error)?;
    let (layout, alpha) = match h.color {
        0 => (SampleLayout::Gray8, AlphaDescription::Opaque),
        2 => (SampleLayout::Rgb8, AlphaDescription::Opaque),
        4 => (SampleLayout::GrayAlpha8, AlphaDescription::StraightCoverage),
        6 => (SampleLayout::Rgba8, AlphaDescription::StraightCoverage),
        _ => unreachable!("preflight validates color type"),
    };
    let mut original_bytes = allocate(bytes.len())?;
    original_bytes.copy_from_slice(bytes);
    Ok(SourceAsset {
        id: AssetId::new(),
        original_hash: ContentHash::digest(bytes),
        original_bytes,
        metadata,
        profile,
        alpha,
        decoded: DecodedImage {
            samples,
            layout,
            alpha,
            converted_to_working_space: false,
        },
    })
}
fn allocate(n: usize) -> Result<Vec<u8>, ImageError> {
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| ImageError::ResourceLimit)?;
    v.resize(n, 0);
    Ok(v)
}
fn decoder_error(e: ::png::DecodingError) -> ImageError {
    match e {
        ::png::DecodingError::LimitsExceeded => ImageError::ResourceLimit,
        _ => ImageError::Malformed,
    }
}
fn metadata(
    bytes: &[u8],
    h: &container::Header,
    limits: DecodeLimits,
    untagged: UntaggedPolicy,
) -> Result<(ImageMetadata, Option<ProfileBlob>), ImageError> {
    let mut gamma = None;
    let mut chromaticities = None;
    let mut srgb = None;
    let mut cicp = false;
    let mut profile = None;
    let mut physical = None;
    for (k, d) in Chunks::new(bytes) {
        match k {
            b"gAMA" => gamma = Some(u32be(d)),
            b"cHRM" => chromaticities = Some(std::array::from_fn(|i| u32be(&d[i * 4..]))),
            b"sRGB" => srgb = Some(d[0]),
            b"cICP" => cicp = true,
            b"iCCP" => profile = Some(read_profile(d, limits.max_profile_bytes, h.color)?),
            b"pHYs" => {
                physical = Some(PhysicalResolution {
                    x_pixels_per_unit: u32be(d),
                    y_pixels_per_unit: u32be(&d[4..]),
                    unit_is_meter: d[8] == 1,
                })
            }
            _ => {}
        }
    }
    if srgb.is_some() && profile.is_some() {
        return Err(ImageError::ColorConflict);
    }
    // sRGB and cICP's supported sRGB tuple prescribe these legacy values.
    if (srgb.is_some() || cicp)
        && (gamma.is_some_and(|g| g != 45455)
            || chromaticities
                .is_some_and(|c| c != [31270, 32900, 64000, 33000, 30000, 60000, 15000, 6000]))
    {
        return Err(ImageError::ColorConflict);
    }
    let mut warnings = Vec::new();
    let color = if cicp {
        if profile.is_some() || srgb.is_some() || gamma.is_some() || chromaticities.is_some() {
            warnings.push(ImportWarning::LowerPriorityColorMetadata);
        }
        ColorEncoding::CicpSrgb
    } else if profile.is_some() {
        if gamma.is_some() || chromaticities.is_some() {
            warnings.push(ImportWarning::LowerPriorityColorMetadata);
        }
        ColorEncoding::Icc
    } else if let Some(rendering_intent) = srgb {
        ColorEncoding::Srgb { rendering_intent }
    } else if gamma.is_some() || chromaticities.is_some() {
        ColorEncoding::GammaChromaticity {
            gamma,
            chromaticities,
        }
    } else {
        if untagged == UntaggedPolicy::Reject {
            return Err(ImageError::UntaggedColor);
        }
        warnings.push(ImportWarning::AssumedSrgb);
        ColorEncoding::AssumedSrgb
    };
    Ok((
        ImageMetadata {
            width: h.width,
            height: h.height,
            color,
            physical,
            warnings,
            untagged_policy: untagged,
            decoder_version: "veylora-png-policy-1/png-0.18.1",
            file_type: "image/png",
        },
        profile,
    ))
}

fn read_profile(d: &[u8], limit: u64, color: u8) -> Result<ProfileBlob, ImageError> {
    use ImageError::*;
    let name_end = d.iter().position(|b| *b == 0).ok_or(MalformedProfile)?;
    if !(1..=79).contains(&name_end) || d.get(name_end + 1) != Some(&0) {
        return Err(MalformedProfile);
    }
    let name = &d[..name_end];
    if name.iter().any(|b| !matches!(b,32..=126|161..=255))
        || name[0] == 32
        || name[name_end - 1] == 32
        || name.windows(2).any(|w| w == b"  ")
    {
        return Err(MalformedProfile);
    }
    let compressed = &d[name_end + 2..];
    let map = |e| {
        if e == ResourceLimit {
            e
        } else {
            MalformedProfile
        }
    };
    // First count/validate using fixed scratch, then allocate exactly the proven bounded size.
    let n = inflate([compressed], limit, |_| Ok(())).map_err(map)?;
    let mut p = allocate(usize::try_from(n).map_err(|_| ResourceLimit)?)?;
    let mut offset = 0;
    inflate([compressed], limit, |b| {
        p[offset..offset + b.len()].copy_from_slice(b);
        offset += b.len();
        Ok(())
    })
    .map_err(map)?;
    if p.len() < 132
        || u32be(&p) as usize != p.len()
        || &p[36..40] != b"acsp"
        || !matches!(p[8], 2 | 4)
        || !matches!(&p[20..24], b"XYZ " | b"Lab ")
    {
        return Err(MalformedProfile);
    }
    let expected = if matches!(color, 0 | 4) {
        b"GRAY"
    } else {
        b"RGB "
    };
    if &p[16..20] != expected
        || !matches!(&p[12..16], b"scnr" | b"mntr" | b"prtr" | b"spac")
        || u32be(&p[64..]) > 3
    {
        return Err(MalformedProfile);
    }
    let count = u32be(&p[128..]) as usize;
    let table_end = count
        .checked_mul(12)
        .and_then(|n| n.checked_add(132))
        .ok_or(MalformedProfile)?;
    if table_end > p.len() {
        return Err(MalformedProfile);
    }
    for entry in p[132..table_end].as_chunks::<12>().0 {
        let start = u32be(&entry[4..]) as usize;
        let len = u32be(&entry[8..]) as usize;
        if start < table_end
            || !start.is_multiple_of(4)
            || len < 8
            || start.checked_add(len).is_none_or(|end| end > p.len())
        {
            return Err(MalformedProfile);
        }
    }
    Ok(ProfileBlob {
        hash: ContentHash::digest(&p),
        bytes: p,
    })
}
