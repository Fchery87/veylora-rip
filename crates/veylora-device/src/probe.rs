//! Protocol investigation metadata only. No decoder, encoder or transport.

use std::collections::BTreeMap;

/// Check operator-declared one-variable comparison metadata, not physical truth.
/// Settings must describe the same conditions except the named changed variable.
pub fn validate_comparison(
    baseline: &BTreeMap<String, String>,
    candidate: &BTreeMap<String, String>,
    changed_variable: &str,
) -> Result<(), &'static str> {
    if baseline.is_empty() || baseline.len() > 64 || baseline.keys().ne(candidate.keys()) {
        return Err("comparison must use the same 1..64 setting names");
    }
    let changed: Vec<_> = baseline
        .iter()
        .filter(|(key, value)| candidate.get(*key) != Some(*value))
        .map(|(key, _)| key.as_str())
        .collect();
    if changed != [changed_variable] {
        return Err("comparison must change exactly the declared one variable");
    }
    for (name, value) in baseline.iter().chain(candidate) {
        if name.trim().is_empty()
            || value.trim().is_empty()
            || name.len() > 128
            || value.len() > 4096
        {
            return Err("invalid comparison setting");
        }
    }
    Ok(())
}
