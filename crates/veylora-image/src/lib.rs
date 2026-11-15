//! Bounded PNG import, independent of rendering and printer transport.
mod asset;
mod container;
mod inflate;
mod limits;
mod png;
pub use asset::*;
pub use limits::DecodeLimits;
pub use png::{decode_png, decode_png_with_policy};

/// Stable machine-readable import failures. No decoder messages leak into this contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageError {
    ResourceLimit,
    Malformed,
    MalformedProfile,
    UnsupportedFormat,
    UnsupportedMetadata,
    UnsupportedColor,
    ColorConflict,
    UntaggedColor,
    Animation,
    Hdr,
}
impl ImageError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ResourceLimit => "image.resource_limit",
            Self::Malformed => "image.malformed_png",
            Self::MalformedProfile => "image.malformed_profile",
            Self::UnsupportedFormat => "image.unsupported_png_format",
            Self::UnsupportedMetadata => "image.unsupported_metadata",
            Self::UnsupportedColor => "image.unsupported_color",
            Self::ColorConflict => "image.conflicting_color_tags",
            Self::UntaggedColor => "image.untagged_color_rejected",
            Self::Animation => "image.apng_unsupported",
            Self::Hdr => "image.hdr_unsupported",
        }
    }
}
impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}
impl std::error::Error for ImageError {}
