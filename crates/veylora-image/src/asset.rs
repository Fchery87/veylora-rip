//! Source bytes and samples. No color conversion or ink generation occurs here.
use veylora_domain::{ids::AssetId, provenance::ContentHash};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorEncoding {
    CicpSrgb,
    Icc,
    Srgb {
        rendering_intent: u8,
    },
    GammaChromaticity {
        gamma: Option<u32>,
        chromaticities: Option<[u32; 8]>,
    },
    AssumedSrgb,
}
/// What to do with a source that has no color tag. Explicit tags are never overridden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UntaggedPolicy {
    AssumeSrgb,
    Reject,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleLayout {
    Rgb8,
    Rgba8,
    Gray8,
    GrayAlpha8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaDescription {
    Opaque,
    StraightCoverage,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportWarning {
    AssumedSrgb,
    LowerPriorityColorMetadata,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalResolution {
    pub x_pixels_per_unit: u32,
    pub y_pixels_per_unit: u32,
    pub unit_is_meter: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub color: ColorEncoding,
    pub physical: Option<PhysicalResolution>,
    pub warnings: Vec<ImportWarning>,
    /// Policy that was in force when this asset was imported.
    pub untagged_policy: UntaggedPolicy,
    pub decoder_version: &'static str,
    pub file_type: &'static str,
}
impl ImageMetadata {
    /// Proposal only, never a placement or resampling instruction. Unknown units have no size.
    pub fn physical_size_mm(&self) -> Option<(f64, f64)> {
        let p = self.physical?;
        (p.unit_is_meter && p.x_pixels_per_unit != 0 && p.y_pixels_per_unit != 0).then(|| {
            (
                f64::from(self.width) * 1000.0 / f64::from(p.x_pixels_per_unit),
                f64::from(self.height) * 1000.0 / f64::from(p.y_pixels_per_unit),
            )
        })
    }
}
#[derive(Debug)]
pub struct ProfileBlob {
    pub bytes: Vec<u8>,
    pub hash: ContentHash,
}
#[derive(Debug)]
pub struct DecodedImage {
    /// Source-encoded 8-bit samples in row-major order, never premultiplied.
    pub samples: Vec<u8>,
    pub layout: SampleLayout,
    pub alpha: AlphaDescription,
    pub converted_to_working_space: bool,
}
#[derive(Debug)]
pub struct SourceAsset {
    pub id: AssetId,
    pub original_hash: ContentHash,
    pub original_bytes: Vec<u8>,
    pub metadata: ImageMetadata,
    pub profile: Option<ProfileBlob>,
    pub alpha: AlphaDescription,
    pub decoded: DecodedImage,
}
