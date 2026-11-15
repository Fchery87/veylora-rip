//! Retained recipe policy values, never evidence of physical printer acceptance.
use crate::{
    errors::ValidationError,
    ids::{AssetId, RecipeId},
    provenance::ContentHash,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionedReference {
    pub version: u32,
    pub content_hash: ContentHash,
}
impl VersionedReference {
    pub(crate) fn validate(&self, subject: &str) -> Result<(), ValidationError> {
        if self.version == 0 {
            return Err(ValidationError::invalid(
                subject,
                "Reference versions must be positive.",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WhiteSource {
    Alpha,
    Tonal,
    Supplied { mask_id: AssetId },
    Disabled,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaskCombination {
    Replace,
    Union,
    Intersection,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderingIntent {
    Perceptual,
    RelativeColorimetric,
    AbsoluteColorimetric,
    Saturation,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UntaggedRgbPolicy {
    Reject,
    AssumeSrgb,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreenAlgorithm {
    OrderedThreshold,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeStatus {
    Experimental,
    Retired,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeelCondition {
    Hot,
    Warm,
    Cold,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WhitePolicy {
    pub source: WhiteSource,
    pub combine: MaskCombination,
    pub density_percent: u8,
    /// Negative contracts and positive expands coverage, in micrometres.
    pub spread_um: i32,
    /// Piecewise-linear samples from input coverage 0 to 65535, inclusive.
    pub response: Vec<u16>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationPolicy {
    pub revision: u32,
    /// Logical C, M, Y, K response samples, uniformly spaced over 0..65535.
    pub process_curves: [Vec<u16>; 4],
    pub channel_limits_percent: [u8; 4],
    pub total_coverage_percent: u16,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorPolicy {
    /// None explicitly selects an unprofiled diagnostic workflow, not a guessed ICC.
    pub profile: Option<VersionedReference>,
    pub rendering_intent: RenderingIntent,
    pub black_point_compensation: bool,
    pub untagged_rgb_policy: UntaggedRgbPolicy,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScreenPolicy {
    pub algorithm: ScreenAlgorithm,
    pub revision: u32,
    pub seed: u64,
    /// Page-coordinate phase in raster pixels.
    pub phase_x: u32,
    pub phase_y: u32,
    pub cell_width: u16,
    pub cell_height: u16,
    /// Row-major thresholds in normalized coverage units. Binary software screen.
    pub thresholds: Vec<u16>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinishPolicy {
    pub ink_id: String,
    pub film_id: String,
    pub powder_id: String,
    pub garment: String,
    pub cure_temperature_c: u16,
    pub cure_seconds: u32,
    pub press_temperature_c: u16,
    pub press_seconds: u32,
    pub press_pressure_pa: u32,
    pub peel: PeelCondition,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrintRecipeInput {
    pub schema_version: u32,
    pub id: RecipeId,
    pub revision: u32,
    /// UTC Unix milliseconds supplied by the creator, not generated during hashing.
    pub created_at_ms: u64,
    pub status: RecipeStatus,
    pub white: WhitePolicy,
    pub calibration: CalibrationPolicy,
    pub color: ColorPolicy,
    pub screen: ScreenPolicy,
    pub finish: FinishPolicy,
}
/// ```compile_fail
/// use veylora_domain::recipe::PrintRecipe;
/// fn mutate(recipe: &mut PrintRecipe) { recipe.inputs().revision = 2; }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "PrintRecipeInput", into = "PrintRecipeInput")]
pub struct PrintRecipe(PrintRecipeInput);
impl PrintRecipe {
    pub fn new(input: PrintRecipeInput) -> Result<Self, ValidationError> {
        if input.schema_version != 1 || input.revision == 0 || input.created_at_ms == 0 {
            return Err(ValidationError::invalid(
                "recipe.metadata",
                "Use schema 1, a positive revision and a UTC creation timestamp.",
            ));
        }
        if input.white.density_percent > 100
            || input.calibration.total_coverage_percent > 400
            || input
                .calibration
                .channel_limits_percent
                .iter()
                .any(|v| *v > 100)
        {
            return Err(ValidationError::invalid(
                "recipe.limits",
                "Coverage limits exceed their normalized domains.",
            ));
        }
        if input.white.response.len() < 2
            || input.calibration.process_curves.iter().any(|c| c.len() < 2)
        {
            return Err(ValidationError::invalid(
                "recipe.curves",
                "Retain at least two response samples per curve.",
            ));
        }
        if input.calibration.revision == 0 || input.screen.revision == 0 {
            return Err(ValidationError::invalid(
                "recipe.revision",
                "Policy revisions must be positive.",
            ));
        }
        if let Some(profile) = &input.color.profile {
            profile.validate("recipe.color.profile")?;
        }
        let cells = usize::from(input.screen.cell_width) * usize::from(input.screen.cell_height);
        if cells == 0 || cells != input.screen.thresholds.len() {
            return Err(ValidationError::invalid(
                "recipe.screen",
                "Retain exactly one threshold per nonempty screen cell.",
            ));
        }
        for value in [
            &input.finish.ink_id,
            &input.finish.film_id,
            &input.finish.powder_id,
            &input.finish.garment,
        ] {
            if value.trim().is_empty() {
                return Err(ValidationError::invalid(
                    "recipe.finish",
                    "Declare materials and garment explicitly.",
                ));
            }
        }
        if input.finish.cure_temperature_c == 0
            || input.finish.cure_seconds == 0
            || input.finish.press_temperature_c == 0
            || input.finish.press_seconds == 0
            || input.finish.press_pressure_pa == 0
        {
            return Err(ValidationError::invalid(
                "recipe.finish",
                "Declare positive finishing conditions supplied by the operator.",
            ));
        }
        Ok(Self(input))
    }
    pub fn inputs(&self) -> &PrintRecipeInput {
        &self.0
    }
    pub fn content_hash(&self) -> ContentHash {
        record_hash(b"veylora-print-recipe-v1\0", &self.0)
    }
}
impl TryFrom<PrintRecipeInput> for PrintRecipe {
    type Error = ValidationError;
    fn try_from(input: PrintRecipeInput) -> Result<Self, Self::Error> {
        Self::new(input)
    }
}
impl From<PrintRecipe> for PrintRecipeInput {
    fn from(recipe: PrintRecipe) -> Self {
        recipe.0
    }
}
/// Stable domain-tagged compact JSON, in struct-field order. No maps or floats.
pub(crate) fn record_hash(tag: &[u8], record: &impl Serialize) -> ContentHash {
    let mut bytes = tag.to_vec();
    serde_json::to_writer(&mut bytes, record).expect("domain records serialize to JSON");
    ContentHash::digest(&bytes)
}
