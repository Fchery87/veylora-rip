use crate::{
    AlphaDescription, ColorMetadata, DecodeLimits, GrayscaleMask, ImageMetadata, ProfileBlob,
    Resolution, SourceAsset,
};
use thiserror::Error;
use veylora_domain::{ids::AssetId, provenance::ContentHash};

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("invalid PNG: {0}")]
    Decode(String),
    #[error("image dimensions exceed configured limits")]
    Dimensions,
    #[error("PNG profile exceeds configured limit")]
    ProfileTooLarge,
    #[error("unsupported PNG color type or bit depth")]
    UnsupportedColor,
    #[error("animated PNG is unsupported")]
    Animated,
    #[error("HDR PNG signaling is unsupported")]
    Hdr,
    #[error("conflicting PNG color metadata")]
    ConflictingMetadata,
    #[error("malformed lower-priority PNG metadata")]
    MalformedMetadata,
    #[error("untagged RGB is not allowed by the selected policy")]
    UntaggedColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UntaggedPolicy {
    AssumeSrgb,
    Reject,
}

pub fn decode_png(bytes: &[u8], limits: DecodeLimits) -> Result<SourceAsset, ImageError> {
    decode_png_with_policy(bytes, limits, UntaggedPolicy::AssumeSrgb)
}

pub fn decode_png_with_policy(
    bytes: &[u8],
    limits: DecodeLimits,
    policy: UntaggedPolicy,
) -> Result<SourceAsset, ImageError> {
    if bytes.len() < 8 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(ImageError::Decode("bad signature".into()));
    }
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let decoded_bound = limits
        .max_pixels
        .checked_mul(8)
        .and_then(|value| value.checked_add(limits.max_profile_bytes))
        .and_then(|value| value.checked_add(1024 * 1024))
        .ok_or(ImageError::Dimensions)?;
    decoder.set_limits(png::Limits {
        bytes: usize::try_from(decoded_bound).map_err(|_| ImageError::Dimensions)?,
    });
    let mut reader = decoder
        .read_info()
        .map_err(|e| ImageError::Decode(e.to_string()))?;
    let info = reader.info();
    let (width, height, color_type, bit_depth, animated, hdr, srgb, profile, dpi) = (
        info.width,
        info.height,
        info.color_type,
        info.bit_depth,
        info.is_animated(),
        info.coding_independent_code_points.is_some()
            || info.mastering_display_color_volume.is_some()
            || info.content_light_level.is_some(),
        info.srgb.is_some(),
        info.icc_profile.as_deref().map(|p| p.to_vec()),
        info.pixel_dims.as_ref().map(|d| Resolution {
            x: d.xppu,
            y: d.yppu,
        }),
    );
    if width > limits.max_dimension
        || height > limits.max_dimension
        || u64::from(width) * u64::from(height) > limits.max_pixels
    {
        return Err(ImageError::Dimensions);
    }
    if animated {
        return Err(ImageError::Animated);
    }
    if hdr {
        return Err(ImageError::Hdr);
    }
    if color_type != png::ColorType::Rgb && color_type != png::ColorType::Rgba {
        return Err(ImageError::UnsupportedColor);
    }
    if bit_depth != png::BitDepth::Eight {
        return Err(ImageError::UnsupportedColor);
    }
    if profile
        .as_ref()
        .is_some_and(|p| p.len() as u64 > limits.max_profile_bytes)
    {
        return Err(ImageError::ProfileTooLarge);
    }
    if profile.is_some() && srgb {
        return Err(ImageError::ConflictingMetadata);
    }
    let color = if let Some(profile) = profile.as_ref() {
        if profile.len() < 132 {
            return Err(ImageError::MalformedMetadata);
        }
        ColorMetadata::EmbeddedIcc
    } else if srgb {
        ColorMetadata::Srgb
    } else {
        if policy == UntaggedPolicy::Reject {
            return Err(ImageError::UntaggedColor);
        }
        ColorMetadata::UntaggedRgb
    };
    let mut pixels = vec![0; reader.output_buffer_size().ok_or(ImageError::Dimensions)?];
    reader
        .next_frame(&mut pixels)
        .map_err(|e| ImageError::Decode(e.to_string()))?;
    Ok(SourceAsset {
        id: AssetId::new(),
        original_hash: ContentHash::digest(bytes),
        metadata: ImageMetadata {
            width,
            height,
            color,
            dpi,
        },
        profile: profile.map(|bytes| ProfileBlob {
            hash: ContentHash::digest(&bytes),
            bytes,
        }),
        alpha: if color_type == png::ColorType::Rgba {
            AlphaDescription::Straight
        } else {
            AlphaDescription::None
        },
        pixels,
    })
}

pub fn decode_grayscale_mask(
    bytes: &[u8],
    limits: DecodeLimits,
) -> Result<GrayscaleMask, ImageError> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_limits(png::Limits {
        bytes: usize::try_from(
            limits
                .max_pixels
                .checked_add(limits.max_profile_bytes)
                .and_then(|value| value.checked_add(1024 * 1024))
                .ok_or(ImageError::Dimensions)?,
        )
        .map_err(|_| ImageError::Dimensions)?,
    });
    let mut reader = decoder
        .read_info()
        .map_err(|error| ImageError::Decode(error.to_string()))?;
    let info = reader.info();
    let (width, height, color_type, bit_depth, dpi) = (
        info.width,
        info.height,
        info.color_type,
        info.bit_depth,
        info.pixel_dims.as_ref().map(|dimensions| Resolution {
            x: dimensions.xppu,
            y: dimensions.yppu,
        }),
    );
    if width > limits.max_dimension
        || height > limits.max_dimension
        || u64::from(width) * u64::from(height) > limits.max_pixels
    {
        return Err(ImageError::Dimensions);
    }
    if color_type != png::ColorType::Grayscale || bit_depth != png::BitDepth::Eight {
        return Err(ImageError::UnsupportedColor);
    }
    let mut coverage = vec![0; reader.output_buffer_size().ok_or(ImageError::Dimensions)?];
    reader
        .next_frame(&mut coverage)
        .map_err(|error| ImageError::Decode(error.to_string()))?;
    Ok(GrayscaleMask {
        original_hash: ContentHash::digest(bytes),
        width,
        height,
        dpi,
        coverage,
    })
}
