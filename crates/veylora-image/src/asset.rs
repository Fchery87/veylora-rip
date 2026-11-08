use serde::{Deserialize, Serialize};
use veylora_domain::{ids::AssetId, provenance::ContentHash};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub color: ColorMetadata,
    pub dpi: Option<Resolution>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorMetadata {
    Srgb,
    EmbeddedIcc,
    UntaggedRgb,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileBlob {
    pub bytes: Vec<u8>,
    pub hash: ContentHash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlphaDescription {
    None,
    Straight,
    Premultiplied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceAsset {
    pub id: AssetId,
    pub original_hash: ContentHash,
    pub metadata: ImageMetadata,
    pub profile: Option<ProfileBlob>,
    pub alpha: AlphaDescription,
    pub pixels: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrayscaleMask {
    pub original_hash: ContentHash,
    pub width: u32,
    pub height: u32,
    pub dpi: Option<Resolution>,
    pub coverage: Vec<u8>,
}
