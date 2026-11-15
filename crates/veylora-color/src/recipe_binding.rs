//! Checks that a recipe's limits stay inside the calibration record it was built from.
//!
//! Unit contract (from the calibration record): `channel_max` values are fractions where `1.0`
//! is 100% of one channel, and `combined_wet_load_max` is on the same scale summed over channels
//! (`4.0` is 400%). The recipe stores percentages. A record that names no limit for a channel
//! imposes none; nothing is assumed in its place. Curve shape is not compared: the record's
//! curve units are not pinned down enough to check a recipe's samples against it.
use crate::calibration::{CalibrationRecord, RecipeStatus};
use veylora_domain::{
    errors::ErrorCode,
    preflight::{Finding, Severity},
    recipe::PrintRecipe,
};

const CHANNELS: [&str; 4] = ["C", "M", "Y", "K"];

fn blocking(code: ErrorCode, subject: &str, message: String) -> Finding {
    Finding {
        code,
        severity: Severity::Blocking,
        message,
        subject: Some(subject.to_string()),
    }
}

/// Percent for a fractional limit, rounded to the nearest whole percent.
fn percent(fraction: f64) -> f64 {
    (fraction * 100.0).round()
}

pub fn check_recipe_calibration(recipe: &PrintRecipe, record: &CalibrationRecord) -> Vec<Finding> {
    let mut findings = Vec::new();
    if record.content_hash != record.compute_hash() {
        findings.push(blocking(
            ErrorCode::IncompatibleRecipe,
            "calibration.integrity",
            "The calibration record does not match its content hash.".into(),
        ));
    }
    if record.status == RecipeStatus::Superseded {
        findings.push(blocking(
            ErrorCode::IncompatibleRecipe,
            "calibration.status",
            "The calibration record is superseded and cannot back a new recipe.".into(),
        ));
    }
    let policy = &recipe.inputs().calibration;
    for (i, name) in CHANNELS.iter().enumerate() {
        let limit = record
            .limits
            .channel_max
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| *v);
        if let Some(max) = limit {
            if f64::from(policy.channel_limits_percent[i]) > percent(max) {
                findings.push(blocking(
                    ErrorCode::CoverageLimitExceeded,
                    &format!("recipe.calibration.channel_limits.{name}"),
                    format!(
                        "Recipe allows {}% {name}, above the calibrated {}%.",
                        policy.channel_limits_percent[i],
                        percent(max)
                    ),
                ));
            }
        }
    }
    if let Some(max) = record.limits.combined_wet_load_max {
        if f64::from(policy.total_coverage_percent) > percent(max) {
            findings.push(blocking(
                ErrorCode::CoverageLimitExceeded,
                "recipe.calibration.total_coverage",
                format!(
                    "Recipe allows {}% total coverage, above the calibrated {}%.",
                    policy.total_coverage_percent,
                    percent(max)
                ),
            ));
        }
    }
    findings
}
