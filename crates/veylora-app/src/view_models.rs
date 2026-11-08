//! Job read models and presentation view models.
//!
//! These mirror `app/src/lib/services/types.ts`. Presenters in `screens/`
//! build the view models; the frontend renders them as given.

use serde::{Deserialize, Serialize};

use crate::state::DataSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Tone {
    Ok,
    Info,
    Warn,
    Bad,
    Review,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JobState {
    Draft,
    PreflightFailed,
    Rendering,
    Ready,
    Held,
    Submitting,
    Submitted,
    Ambiguous,
    NeedsReview,
    Cancelled,
    Failed,
    OperatorConfirmedFinished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Blocking,
    Confirm,
    Info,
}

/// Who acknowledged a confirm-level finding, and when.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Acknowledgement {
    pub operator: String,
    pub at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub id: String,
    pub severity: Severity,
    pub title: String,
    pub cause: String,
    /// Exact correction. Every blocking finding must show one.
    pub fix: String,
    /// The placement this finding points at, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement_id: Option<String>,
    pub acknowledged: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acknowledgement: Option<Acknowledgement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    pub id: String,
    pub name: String,
    pub x_mm: f64,
    pub y_mm: f64,
    pub width_mm: f64,
    pub height_mm: f64,
    pub clipped: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttemptKind {
    FileSink,
    Printer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttemptOutcome {
    Pending,
    Ambiguous,
    Finished,
    Reprint,
    KeptInReview,
}

impl AttemptOutcome {
    /// An attempt that may or may not have printed and has no operator outcome yet.
    pub fn is_open_review(self) -> bool {
        matches!(self, Self::Ambiguous | Self::KeptInReview)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attempt {
    pub id: String,
    pub kind: AttemptKind,
    pub at: String,
    pub outcome: AttemptOutcome,
    pub note: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sink_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobSummary {
    pub id: String,
    pub name: String,
    pub state: JobState,
    pub sheet: String,
    pub placement_count: usize,
    pub updated: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceRow {
    pub label: String,
    pub value: String,
}

/// A white policy a print mode can offer. Wire names are the display names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhitePolicyKind {
    Underbase,
    #[serde(rename = "No white")]
    NoWhite,
}

/// Where the white mask comes from. Wire names are the display names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaskSource {
    #[serde(rename = "Auto from alpha")]
    AutoFromAlpha,
    #[serde(rename = "Whole placement")]
    WholePlacement,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhitePolicy {
    pub source: MaskSource,
    pub policy: WhitePolicyKind,
    pub invert_mask: bool,
    pub choke_mm: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobDetail {
    #[serde(flatten)]
    pub summary: JobSummary,
    pub placements: Vec<Placement>,
    pub findings: Vec<Finding>,
    pub attempts: Vec<Attempt>,
    pub provenance: Vec<ProvenanceRow>,
    pub white_policy: WhitePolicy,
    pub rendered: bool,
}

// ---- Preflight presentation ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindingView {
    pub id: String,
    pub severity: Severity,
    pub title: String,
    pub cause: String,
    /// Never empty for a blocking finding.
    pub correction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement_id: Option<String>,
    pub acknowledged: bool,
    /// "Acknowledged by <operator> at <time>", once acknowledged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acknowledgement: Option<String>,
    pub can_acknowledge: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindingGroupView {
    pub severity: Severity,
    pub title: String,
    pub tone: Tone,
    pub findings: Vec<FindingView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SummaryView {
    pub text: String,
    pub tone: Tone,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightView {
    pub checked_against: String,
    pub groups: Vec<FindingGroupView>,
    pub summary: SummaryView,
    /// Blocking plus unacknowledged confirm findings. Shown as the tab count.
    pub open_count: usize,
}

// ---- Inspect presentation ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StageId {
    Garment,
    Film,
    Logical,
    Slots,
    Screened,
}

/// A logical plane. Not a physical slot or protocol channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Plane {
    C,
    M,
    Y,
    K,
    W,
}

impl Plane {
    pub const ALL: [Plane; 5] = [Plane::C, Plane::M, Plane::Y, Plane::K, Plane::W];
}

/// Where a preview came from. `Fixture` is a simulation produced from fixture data.
/// `Retained` is reserved for real retained artifacts, which no backend emits yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PreviewSource {
    Simulation,
    Retained,
    Fixture,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectSelection {
    pub stage: StageId,
    pub planes: Vec<Plane>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StageOption {
    pub id: StageId,
    pub label: String,
    pub disabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlaneOption {
    pub plane: Plane,
    pub pressed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetaRow {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectView {
    pub selection: InspectSelection,
    pub stages: Vec<StageOption>,
    pub planes: Vec<PlaneOption>,
    /// Always says simulation or retained, and always "Not physical output".
    pub chip: String,
    pub source: PreviewSource,
    pub metadata: Vec<MetaRow>,
    /// False until retained artifacts exist; the UI shows `placeholder` instead.
    pub raster_shown: bool,
    pub placeholder: String,
}

// ---- Job actions and review presentation ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandView {
    pub enabled: bool,
    /// Shown as text beside the control, never hidden in a tooltip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl CommandView {
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            reason: None,
        }
    }

    pub fn disabled(reason: impl Into<String>) -> Self {
        Self {
            enabled: false,
            reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobActionsView {
    pub render_and_hold: CommandView,
    pub release_to_printer: CommandView,
    pub write_to_file_sink: CommandView,
    pub inspect: CommandView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewOutcome {
    Finished,
    Reprint,
    PrintedBadly,
    DecideLater,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewChoiceView {
    pub outcome: ReviewOutcome,
    pub label: String,
    pub confirm_label: String,
    /// What recording this outcome will do, shown before the operator confirms.
    pub effect: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewAttemptView {
    pub id: String,
    pub last_signal: String,
    pub source: DataSource,
    /// Operator-facing label for `source`.
    pub source_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewView {
    pub headline: String,
    pub body: String,
    pub attempt: ReviewAttemptView,
    pub choices: Vec<ReviewChoiceView>,
    /// True while this attempt is unresolved. Nothing may be sent to the printer.
    pub printer_blocked: bool,
}
