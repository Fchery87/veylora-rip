use serde::{Deserialize, Serialize};
use veylora_domain::provenance::{ContentHash, Provenance};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelCurve {
    pub channel: String,
    pub points: Vec<(f64, f64)>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InkLimits {
    pub channel_max: Vec<(String, f64)>,
    pub combined_wet_load_max: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WhiteCalibration {
    pub slots: Vec<(String, f64)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementProvenance {
    pub chart_id: String,
    pub instrument: String,
    pub units: String,
    pub source_hash: ContentHash,
    pub provenance: Provenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeStatus {
    Experimental,
    Validated,
    Superseded,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationRecord {
    pub schema: String,
    pub version: u32,
    pub channels: Vec<ChannelCurve>,
    pub limits: InkLimits,
    pub white_slots: WhiteCalibration,
    pub measurement: MeasurementProvenance,
    pub status: RecipeStatus,
    pub content_hash: ContentHash,
}

impl CalibrationRecord {
    pub fn new(
        channels: Vec<ChannelCurve>,
        limits: InkLimits,
        white_slots: WhiteCalibration,
        measurement: MeasurementProvenance,
    ) -> Result<Self, String> {
        validate_finite(&channels, &limits, &white_slots)?;
        let mut record = Self {
            schema: "veylora.calibration".into(),
            version: 1,
            channels,
            limits,
            white_slots,
            measurement,
            status: RecipeStatus::Experimental,
            content_hash: ContentHash::digest(b"placeholder"),
        };
        record.content_hash = record.compute_hash();
        Ok(record)
    }
    pub fn compute_hash(&self) -> ContentHash {
        let bytes = serde_json::to_vec(&(
            self.schema.as_str(),
            self.version,
            &self.channels,
            &self.limits,
            &self.white_slots,
            &self.measurement,
        ))
        .expect("validated calibration serializes");
        ContentHash::digest(&bytes)
    }
    pub fn is_unvalidated(&self) -> bool {
        self.status != RecipeStatus::Validated
    }
}
fn validate_finite(
    channels: &[ChannelCurve],
    limits: &InkLimits,
    white: &WhiteCalibration,
) -> Result<(), String> {
    for curve in channels {
        if curve.channel.trim().is_empty()
            || curve.points.is_empty()
            || curve
                .points
                .iter()
                .any(|(a, b)| !a.is_finite() || !b.is_finite())
        {
            return Err("channel curve contains missing or non-finite values".into());
        }
    }
    if limits.channel_max.iter().any(|(_, v)| !v.is_finite())
        || limits.combined_wet_load_max.is_some_and(|v| !v.is_finite())
        || white.slots.iter().any(|(_, v)| !v.is_finite())
    {
        return Err("limits contain non-finite values".into());
    }
    Ok(())
}
