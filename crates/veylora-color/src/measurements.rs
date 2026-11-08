use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, error::Error, fmt};
use veylora_domain::provenance::ContentHash;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchMeasurement {
    pub patch_id: String,
    pub values: Vec<f64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementRecord {
    pub chart_id: String,
    pub units: String,
    pub instrument: String,
    pub provenance: String,
    pub patches: Vec<PatchMeasurement>,
    pub holdout_patch_ids: Vec<String>,
    pub expected_patch_ids: Vec<String>,
    pub source_hash: ContentHash,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeasurementError {
    Empty,
    Malformed(String),
    DuplicatePatch(String),
    MissingPatch(String),
    LimitExceeded,
}
impl fmt::Display for MeasurementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for MeasurementError {}
pub type MeasurementSet = MeasurementRecord;
const MAX_BYTES: usize = 8 * 1024 * 1024;
const MAX_PATCHES: usize = 100_000;
#[allow(clippy::too_many_arguments)]
fn finish(
    chart_id: String,
    units: String,
    instrument: String,
    provenance: String,
    patches: Vec<PatchMeasurement>,
    holdout_patch_ids: Vec<String>,
    expected_patch_ids: Vec<String>,
    raw: &[u8],
) -> Result<MeasurementRecord, MeasurementError> {
    if patches.is_empty() {
        return Err(MeasurementError::Empty);
    }
    if patches.len() > MAX_PATCHES
        || patches.iter().any(|p| {
            p.patch_id.trim().is_empty()
                || p.values.is_empty()
                || p.values.iter().any(|v| !v.is_finite())
        })
    {
        return Err(MeasurementError::Malformed(
            "patch IDs and raw values must be present and finite".into(),
        ));
    }
    let mut ids = BTreeSet::new();
    for patch in &patches {
        if !ids.insert(&patch.patch_id) {
            return Err(MeasurementError::DuplicatePatch(patch.patch_id.clone()));
        }
    }
    for id in &holdout_patch_ids {
        if !ids.contains(id) {
            return Err(MeasurementError::MissingPatch(id.clone()));
        }
    }
    for id in &expected_patch_ids {
        if !ids.contains(id) {
            return Err(MeasurementError::MissingPatch(id.clone()));
        }
    }
    Ok(MeasurementRecord {
        chart_id,
        units,
        instrument,
        provenance,
        patches,
        holdout_patch_ids,
        expected_patch_ids,
        source_hash: ContentHash::digest(raw),
    })
}
impl MeasurementRecord {
    /// Return only fitting patches. Holdouts are retained but never fitted.
    pub fn fitting_patches(&self) -> impl Iterator<Item = &PatchMeasurement> {
        self.patches.iter().filter(|patch| {
            !self
                .holdout_patch_ids
                .iter()
                .any(|id| id == &patch.patch_id)
        })
    }
}
pub fn import_json(bytes: &[u8]) -> Result<MeasurementRecord, MeasurementError> {
    if bytes.is_empty() {
        return Err(MeasurementError::Empty);
    }
    if bytes.len() > MAX_BYTES {
        return Err(MeasurementError::LimitExceeded);
    }
    #[derive(Deserialize)]
    struct Wire {
        chart_id: String,
        units: String,
        instrument: String,
        provenance: String,
        patches: Vec<PatchMeasurement>,
        #[serde(default)]
        holdout_patch_ids: Vec<String>,
        #[serde(default)]
        expected_patch_ids: Vec<String>,
    }
    let wire: Wire =
        serde_json::from_slice(bytes).map_err(|e| MeasurementError::Malformed(e.to_string()))?;
    finish(
        wire.chart_id,
        wire.units,
        wire.instrument,
        wire.provenance,
        wire.patches,
        wire.holdout_patch_ids,
        wire.expected_patch_ids,
        bytes,
    )
}
pub fn import_csv(bytes: &[u8]) -> Result<MeasurementRecord, MeasurementError> {
    if bytes.is_empty() {
        return Err(MeasurementError::Empty);
    }
    if bytes.len() > MAX_BYTES {
        return Err(MeasurementError::LimitExceeded);
    }
    let text =
        std::str::from_utf8(bytes).map_err(|e| MeasurementError::Malformed(e.to_string()))?;
    let mut lines = text.lines();
    let header = lines.next().ok_or(MeasurementError::Empty)?;
    let columns: Vec<_> = header.split(',').map(str::trim).collect();
    if columns.len() < 2 || columns[0] != "patch_id" {
        return Err(MeasurementError::Malformed(
            "CSV header must start with patch_id".into(),
        ));
    }
    let mut patches = Vec::new();
    for line in lines {
        let fields: Vec<_> = line.split(',').map(str::trim).collect();
        if fields.len() != columns.len() {
            return Err(MeasurementError::Malformed("CSV row width mismatch".into()));
        }
        let values = fields[1..]
            .iter()
            .map(|v| {
                v.parse::<f64>()
                    .map_err(|e| MeasurementError::Malformed(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        patches.push(PatchMeasurement {
            patch_id: fields[0].into(),
            values,
        });
    }
    finish(
        "csv-import".into(),
        "unspecified".into(),
        "unspecified".into(),
        "raw CSV import".into(),
        patches,
        Vec::new(),
        Vec::new(),
        bytes,
    )
}
