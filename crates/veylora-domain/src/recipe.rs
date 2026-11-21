//! Retained recipe policy values, never evidence of physical printer acceptance.
use crate::{
    errors::ValidationError,
    ids::{AssetId, RecipeId},
    provenance::ContentHash,
};
use serde::{Deserialize, Serialize};

/// Current recipe schema. Version 2 added `clear`; version 1 records are not readable.
pub const RECIPE_SCHEMA_VERSION: u32 = 2;

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
    /// Carries [`RecipeValidation`]: pressed samples and a named sign-off.
    Validated,
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
/// Fewest pressed samples, and fewest separate sessions, behind a validated recipe. The product
/// requirements call a recipe validated only when repeated pressed samples meet agreed tolerances
/// across sessions; these are the floor for "repeated" and "across sessions". The tolerances
/// themselves are the operator's, stated in the evidence, not decided here.
pub const MIN_VALIDATION_SAMPLES: usize = 3;
pub const MIN_VALIDATION_SESSIONS: usize = 2;

/// One pressed sample, measured against the stated tolerances.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PressedSample {
    /// The printing session it came from, as the operator names it (usually a date and a part of
    /// the day). Samples from one session do not show the recipe holds across sessions.
    pub session: String,
    pub label: String,
    pub within_tolerance: bool,
    pub note: String,
}
/// What was physically observed, and under which conditions. It names the recipe revision it
/// validates and the setup and mode it was measured with, so it cannot be reused for others.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeEvidence {
    /// The experimental revision this one was made from, and its content hash.
    pub validated_revision: u32,
    pub validated_hash: ContentHash,
    pub setup_hash: ContentHash,
    pub mode_hash: ContentHash,
    /// Who pressed and measured the samples.
    pub operator: String,
    /// The tolerances the samples were measured against, as the operator agreed them.
    pub tolerances: String,
    pub samples: Vec<PressedSample>,
}
/// The evidence plus the named person who accepted it. Both are required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeValidation {
    pub evidence: RecipeEvidence,
    pub signed_off_by: String,
    /// UTC time of the sign-off, supplied by the operator.
    pub signed_off_at: String,
    pub sign_off_note: String,
}
impl RecipeValidation {
    fn validate(&self) -> Result<(), ValidationError> {
        let bad = |message: &str| Err(ValidationError::invalid("recipe.validation", message));
        let e = &self.evidence;
        let blank = |v: &str| v.trim().is_empty();
        if blank(&e.operator) || blank(&e.tolerances) {
            return bad(
                "Name who measured the samples and the tolerances they were measured against.",
            );
        }
        if blank(&self.signed_off_by) || blank(&self.signed_off_at) || blank(&self.sign_off_note) {
            return bad("A sign-off needs a name, a time and a note about what was reviewed.");
        }
        if e.validated_revision == 0 {
            return bad("Name the revision that was validated.");
        }
        if e.samples.len() < MIN_VALIDATION_SAMPLES {
            return bad("Validation needs repeated pressed samples.");
        }
        if e.samples
            .iter()
            .any(|s| blank(&s.session) || blank(&s.label))
        {
            return bad("Name the session and label of every sample.");
        }
        let sessions: std::collections::BTreeSet<&str> =
            e.samples.iter().map(|s| s.session.trim()).collect();
        if sessions.len() < MIN_VALIDATION_SESSIONS {
            return bad("Validation needs samples from more than one session.");
        }
        if e.samples.iter().any(|s| !s.within_tolerance) {
            return bad("A sample outside tolerance means the recipe is not validated.");
        }
        Ok(())
    }
}

/// Most knockout rules one recipe may carry, so mask building stays bounded.
pub const MAX_KNOCKOUT_RULES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnockoutTarget {
    /// Source-encoded black, for black and near-black knockout.
    Black,
    /// A selected source color in the source's own 8-bit encoding. Garment color is separate.
    Color { rgb: [u8; 3] },
}
/// Clears source pixels whose color is within `tolerance_permille` of the target, ramping back to
/// fully retained over `transition_permille`. Distance is the largest per-channel difference as a
/// fraction of 255, in the source encoding with no color management. Integers keep hashes exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnockoutPolicy {
    pub target: KnockoutTarget,
    pub tolerance_permille: u16,
    /// Zero is a hard clip.
    pub transition_permille: u16,
}
/// Deterministic square-grid perforation in physical units, anchored to absolute page pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerforationPolicy {
    pub diameter_um: u32,
    /// Center-to-center distance; must exceed the diameter so holes never overlap.
    pub spacing_um: u32,
    pub phase_x_px: i32,
    pub phase_y_px: i32,
    /// Grayscale mask of areas where holes may be punched. None makes the whole page eligible.
    pub eligible_mask_id: Option<AssetId>,
    /// Keeps holes this far from the edge of the eligible area. Requires `eligible_mask_id`.
    pub edge_protection_um: u32,
}
/// Rules for the single final clear mask. An empty policy is a solid transfer; a perforation
/// makes a different recipe revision with a different content hash.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClearPolicy {
    pub knockouts: Vec<KnockoutPolicy>,
    pub perforation: Option<PerforationPolicy>,
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
    pub clear: ClearPolicy,
    pub finish: FinishPolicy,
    /// Present exactly when `status` is `Validated`. Absent from the serialized record otherwise,
    /// so recipes that predate validation keep their content hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<RecipeValidation>,
}
impl ClearPolicy {
    fn validate(&self) -> Result<(), ValidationError> {
        let bad = |message: &str| Err(ValidationError::invalid("recipe.clear", message));
        if self.knockouts.len() > MAX_KNOCKOUT_RULES {
            return bad("Too many knockout rules for one recipe.");
        }
        if self
            .knockouts
            .iter()
            .any(|k| k.tolerance_permille > 1000 || k.transition_permille > 1000)
        {
            return bad("Knockout tolerance and transition are per-mille values up to 1000.");
        }
        if let Some(p) = &self.perforation {
            if p.diameter_um == 0 || p.spacing_um <= p.diameter_um {
                return bad("Perforation spacing must exceed a positive hole diameter.");
            }
            if p.edge_protection_um > 0 && p.eligible_mask_id.is_none() {
                return bad("Edge protection needs an eligible-area mask.");
            }
        }
        Ok(())
    }
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
        if input.schema_version != RECIPE_SCHEMA_VERSION
            || input.revision == 0
            || input.created_at_ms == 0
        {
            return Err(ValidationError::invalid(
                "recipe.metadata",
                "Use the current recipe schema, a positive revision and a UTC creation timestamp.",
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
        input.clear.validate()?;
        match (&input.status, &input.validation) {
            (RecipeStatus::Validated, Some(v)) => v.validate()?,
            (RecipeStatus::Validated, None) => {
                return Err(ValidationError::invalid(
                    "recipe.validation",
                    "A validated recipe carries its physical evidence and sign-off.",
                ))
            }
            (_, Some(_)) => {
                return Err(ValidationError::invalid(
                    "recipe.validation",
                    "Only a validated recipe carries validation evidence.",
                ))
            }
            (_, None) => {}
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
