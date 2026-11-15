#![allow(dead_code)]
use veylora_domain::{ids::AssetId, provenance::ContentHash};
use veylora_image::{
    AlphaDescription, ColorEncoding, DecodedImage, ImageMetadata, SampleLayout, SourceAsset,
    UntaggedPolicy,
};

pub fn asset(width: u32, height: u32, layout: SampleLayout, samples: Vec<u8>) -> SourceAsset {
    let alpha = match layout {
        SampleLayout::Gray8 | SampleLayout::Rgb8 => AlphaDescription::Opaque,
        _ => AlphaDescription::StraightCoverage,
    };
    SourceAsset {
        id: AssetId::new(),
        original_hash: ContentHash::digest(&samples),
        original_bytes: samples.clone(),
        metadata: ImageMetadata {
            width,
            height,
            color: ColorEncoding::AssumedSrgb,
            physical: None,
            warnings: vec![],
            untagged_policy: UntaggedPolicy::AssumeSrgb,
            decoder_version: "test",
            file_type: "image/png",
        },
        profile: None,
        alpha,
        decoded: DecodedImage {
            samples,
            layout,
            alpha,
            converted_to_working_space: false,
        },
    }
}
pub fn rgb(width: u32, pixels: &[[u8; 3]]) -> SourceAsset {
    asset(
        width,
        pixels.len() as u32 / width,
        SampleLayout::Rgb8,
        pixels.concat(),
    )
}
