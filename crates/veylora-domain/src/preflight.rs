//! Shared validation contracts, independent of application orchestration.

use crate::errors::ErrorCode;
use serde::{Deserialize, Serialize};

/// The action a finding requires, not an ordered scale of importance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Must be corrected. Operator confirmation cannot bypass this finding.
    Blocking,
    /// Requires an explicit operator decision before proceeding.
    ConfirmationRequired,
    /// Context only, requiring neither correction nor confirmation.
    Informational,
}

/// A validation observation. The validator chooses severity for its context.
/// This record does not grant overrides or authorize printer submission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub code: ErrorCode,
    pub severity: Severity,
    /// Human guidance, optionally more specific than the code's default message.
    pub message: String,
    /// Identifies the affected asset, field, or other domain subject when known.
    pub subject: Option<String>,
}

/// Pure domain validation. An empty collection means no findings were reported.
pub trait Validate {
    fn validate(&self) -> Vec<Finding>;
}

impl Severity {
    pub const fn is_blocking(self) -> bool {
        matches!(self, Self::Blocking)
    }

    pub const fn requires_confirmation(self) -> bool {
        matches!(self, Self::ConfirmationRequired)
    }
}
