//! Read models the desktop shell exposes to the frontend.
//!
//! These mirror `app/src/lib/services/types.ts`. They are presentation read
//! models only: the shell owns no rendering, separation, encoding,
//! persistence, or Windows transport logic. Until the application-service
//! crates exist, the values below are labeled fixture data.

use serde::Serialize;

// `Live` and the non-`Experimental` maturities are part of the contract the
// frontend already handles; no backend emits them until the services exist.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DataSource {
    Fixture,
    Live,
}

impl DataSource {
    /// How a value from this source is labeled to the operator.
    pub fn label(self) -> &'static str {
        match self {
            Self::Fixture => "Fixture data",
            Self::Live => "Live data",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModeMaturity {
    Experimental,
    Validated,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupCapability {
    pub exists: bool,
    pub verified: bool,
    pub experimental: bool,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeCapability {
    pub id: String,
    pub maturity: ModeMaturity,
    pub sheet_sizes: Vec<String>,
    pub white_choke: bool,
    pub white_mask_invert: bool,
    pub white_policies: Vec<String>,
    pub screened_inspect: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SinkCapability {
    /// Physical ET-8550 submission. False until the physical gate passes.
    pub physical_printer: bool,
    /// Writes the printer-ready artifact to a file. Not physical output.
    pub file_sink: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenReview {
    pub job_id: String,
    pub attempt_id: String,
    pub job_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityState {
    pub data_source: DataSource,
    pub setup: SetupCapability,
    pub mode: ModeCapability,
    pub sinks: SinkCapability,
    pub open_review: Option<OpenReview>,
}

impl CapabilityState {
    /// Fixture capability state used until backend integration is unblocked.
    pub fn fixture() -> Self {
        Self {
            data_source: DataSource::Fixture,
            setup: SetupCapability {
                exists: true,
                verified: false,
                experimental: false,
                name: "Home bench rev 3".to_owned(),
            },
            mode: ModeCapability {
                id: "PM-exp-01".to_owned(),
                maturity: ModeMaturity::Experimental,
                sheet_sizes: vec!["A3".to_owned(), "A4".to_owned()],
                white_choke: false,
                white_mask_invert: false,
                white_policies: vec!["Underbase".to_owned(), "No white".to_owned()],
                screened_inspect: true,
            },
            sinks: SinkCapability {
                physical_printer: false,
                file_sink: true,
            },
            open_review: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_never_claims_physical_submission() {
        let caps = CapabilityState::fixture();
        assert!(!caps.sinks.physical_printer);
        assert_eq!(caps.data_source, DataSource::Fixture);
    }

    #[test]
    fn serializes_with_the_frontend_field_names() {
        let json = serde_json::to_value(CapabilityState::fixture()).unwrap();
        assert_eq!(json["dataSource"], "fixture");
        assert_eq!(json["sinks"]["physicalPrinter"], false);
        assert_eq!(json["mode"]["sheetSizes"][0], "A3");
        assert_eq!(json["mode"]["whiteMaskInvert"], false);
        assert_eq!(json["mode"]["whitePolicies"][0], "Underbase");
        assert!(json["openReview"].is_null());
    }
}
