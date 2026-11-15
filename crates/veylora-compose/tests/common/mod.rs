#![allow(dead_code)]
use veylora_device::capability::MirrorOwner;
use veylora_domain::{
    ids::{AssetId, PlacementId},
    layout::{LayoutInput, LayoutSnapshot, Placement, Rotation},
    provenance::ContentHash,
};
use veylora_image::{
    AlphaDescription, ColorEncoding, DecodedImage, ImageMetadata, SampleLayout, SourceAsset,
    UntaggedPolicy,
};
use veylora_layout::{resolve_transform, ResolvedLayout};
use veylora_separation::PixelPitch;

/// 10 um pixels, so one source pixel of 10 um maps onto one page pixel.
pub const PITCH: PixelPitch = PixelPitch {
    x_nm: 10_000,
    y_nm: 10_000,
};

pub fn image(w: u32, h: u32, layout: SampleLayout, samples: Vec<u8>) -> SourceAsset {
    image_with(w, h, layout, samples, ColorEncoding::AssumedSrgb)
}
pub fn image_with(
    w: u32,
    h: u32,
    layout: SampleLayout,
    samples: Vec<u8>,
    color: ColorEncoding,
) -> SourceAsset {
    let alpha = match layout {
        SampleLayout::Gray8 | SampleLayout::Rgb8 => AlphaDescription::Opaque,
        _ => AlphaDescription::StraightCoverage,
    };
    SourceAsset {
        id: AssetId::new(),
        original_hash: ContentHash::digest(&samples),
        original_bytes: samples.clone(),
        metadata: ImageMetadata {
            width: w,
            height: h,
            color,
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

/// A placement of the whole `asset`, `w_um` x `h_um` before rotation, with its box origin at (x, y).
pub fn place(
    asset: &SourceAsset,
    x: i64,
    y: i64,
    w_um: u32,
    h_um: u32,
    rotation: Rotation,
) -> Placement {
    Placement {
        id: PlacementId::new(),
        asset_id: asset.id,
        mask_id: None,
        x_um: x,
        y_um: y,
        width_um: w_um,
        height_um: h_um,
        rotation,
        crop_x_px: 0,
        crop_y_px: 0,
        crop_width_px: asset.metadata.width,
        crop_height_px: asset.metadata.height,
    }
}

pub fn resolved(page: (u32, u32), mirror: bool, placements: Vec<Placement>) -> ResolvedLayout {
    let l = LayoutSnapshot::new(LayoutInput {
        schema_version: 1,
        page_width_um: page.0,
        page_height_um: page.1,
        mirror,
        placements,
    })
    .unwrap();
    resolve_transform(
        &l,
        if mirror {
            MirrorOwner::Rip
        } else {
            MirrorOwner::None
        },
    )
    .unwrap()
}

pub fn px(page: &SourceAsset, x: usize, y: usize) -> [u8; 4] {
    assert_eq!(page.decoded.layout, SampleLayout::Rgba8);
    let w = page.metadata.width as usize;
    let i = (y * w + x) * 4;
    page.decoded.samples[i..i + 4].try_into().unwrap()
}
