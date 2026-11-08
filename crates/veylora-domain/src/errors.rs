//! Stable machine identifiers and human guidance for domain findings.

use serde::{Deserialize, Serialize};

/// The domain area responsible for a finding, independent of its severity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    Input,
    Color,
    Device,
    Layout,
    Separation,
    Resource,
    Submission,
}

/// Stable codes shared by headless consumers. Messages are not identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    MissingAsset,
    UnsupportedInput,
    UnsupportedProfile,
    UnverifiedDeviceSetup,
    IncompatibleRecipe,
    UnsupportedControl,
    Clipping,
    CoverageLimitExceeded,
    InsufficientDiskSpace,
    AmbiguousSubmission,
}

impl ErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingAsset => "missing_asset",
            Self::UnsupportedInput => "unsupported_input",
            Self::UnsupportedProfile => "unsupported_profile",
            Self::UnverifiedDeviceSetup => "unverified_device_setup",
            Self::IncompatibleRecipe => "incompatible_recipe",
            Self::UnsupportedControl => "unsupported_control",
            Self::Clipping => "clipping",
            Self::CoverageLimitExceeded => "coverage_limit_exceeded",
            Self::InsufficientDiskSpace => "insufficient_disk_space",
            Self::AmbiguousSubmission => "ambiguous_submission",
        }
    }

    pub const fn category(self) -> ErrorCategory {
        match self {
            Self::MissingAsset | Self::UnsupportedInput => ErrorCategory::Input,
            Self::UnsupportedProfile => ErrorCategory::Color,
            Self::UnverifiedDeviceSetup | Self::IncompatibleRecipe | Self::UnsupportedControl => {
                ErrorCategory::Device
            }
            Self::Clipping => ErrorCategory::Layout,
            Self::CoverageLimitExceeded => ErrorCategory::Separation,
            Self::InsufficientDiskSpace => ErrorCategory::Resource,
            Self::AmbiguousSubmission => ErrorCategory::Submission,
        }
    }

    pub const fn message(self) -> &'static str {
        match self {
            Self::MissingAsset => "Restore the missing source asset.",
            Self::UnsupportedInput => "Use a supported input format and metadata policy.",
            Self::UnsupportedProfile => "Select a supported color profile.",
            Self::UnverifiedDeviceSetup => "Verify the device setup with physical channel evidence.",
            Self::IncompatibleRecipe => "Select a recipe compatible with the device setup and print mode.",
            Self::UnsupportedControl => "Remove the control that the backend cannot honor.",
            Self::Clipping => "Move or resize artwork to fit the printable bounds.",
            Self::CoverageLimitExceeded => "Reduce coverage to the validated ink and wet-load limits.",
            Self::InsufficientDiskSpace => "Free disk space before rendering or retaining artifacts.",
            Self::AmbiguousSubmission => "Review the physical output before creating another submission attempt. Do not retry automatically.",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
