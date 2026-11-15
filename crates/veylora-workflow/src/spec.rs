//! The operator-authored job description. Assets and masks are named here and resolved by the
//! content hash of their imported bytes, so a spec never depends on where a file lives.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use veylora_domain::{ids::JobId, layout::Rotation, provenance::ContentHash};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobSpec {
    pub schema_version: u32,
    /// Generated when absent. Supply one to make creation reproducible.
    pub job_id: Option<JobId>,
    /// UTC Unix milliseconds, retained in the snapshot.
    pub created_at_ms: u64,
    /// Content hashes of the imported setup, mode and recipe records.
    pub setup: ContentHash,
    pub mode: ContentHash,
    pub recipe: ContentHash,
    pub assets: BTreeMap<String, ContentHash>,
    #[serde(default)]
    pub masks: BTreeMap<String, ContentHash>,
    pub layout: LayoutSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutSpec {
    pub page_width_um: u32,
    pub page_height_um: u32,
    /// Horizontal mirror of the whole sheet. Honoured only when the RIP owns the mirror.
    pub mirror: bool,
    pub placements: Vec<PlacementSpec>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlacementSpec {
    pub asset: String,
    #[serde(default)]
    pub mask: Option<String>,
    pub x_um: i64,
    pub y_um: i64,
    /// Scaled size of the crop before rotation.
    pub width_um: u32,
    pub height_um: u32,
    pub rotation: Rotation,
    /// Source pixels to use. The whole image when absent.
    #[serde(default)]
    pub crop: Option<CropSpec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CropSpec {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
