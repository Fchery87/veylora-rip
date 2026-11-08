use serde::Serialize;
use veylora_domain::provenance::ContentHash;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ChartMetadata {
    pub chart_id: String,
    pub version: u32,
    pub patch_ids: Vec<String>,
    pub provenance: String,
    pub content_hash: ContentHash,
}

pub fn export_chart_metadata(
    chart_id: impl Into<String>,
    patch_ids: Vec<String>,
    provenance: impl Into<String>,
) -> ChartMetadata {
    let chart_id = chart_id.into();
    let provenance = provenance.into();
    let bytes = serde_json::to_vec(&(chart_id.as_str(), 1u32, &patch_ids, provenance.as_str()))
        .expect("chart metadata serializes");
    ChartMetadata {
        chart_id,
        version: 1,
        patch_ids,
        provenance,
        content_hash: ContentHash::digest(&bytes),
    }
}

pub fn export_chart_json(metadata: &ChartMetadata) -> String {
    serde_json::to_string_pretty(metadata).expect("chart metadata serializes")
}
