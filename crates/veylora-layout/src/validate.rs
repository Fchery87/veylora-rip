//! Layout findings: clipping, safe margins, effective resolution, media size.

use crate::{geometry::Rect, transforms::ResolvedLayout};
use veylora_domain::{
    errors::ErrorCode,
    preflight::{Finding, Severity},
};

const UM_PER_INCH: u64 = 25_400;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutPolicy {
    /// Minimum acceptable effective resolution on the worse axis.
    pub min_effective_ppi: u32,
    /// Safe margin on every page edge, micrometres.
    pub margin_um: u32,
    /// Page sizes the device setup supports. Empty means no media check.
    pub supported_page_sizes_um: Vec<(u32, u32)>,
}

fn finding(code: ErrorCode, severity: Severity, subject: String, message: String) -> Finding {
    Finding {
        code,
        severity,
        message,
        subject: Some(subject),
    }
}

/// Validate final page-space geometry. Placement order is preserved so output is
/// deterministic. Clipping blocks; margin and resolution findings need a decision.
pub fn validate_layout(layout: &ResolvedLayout, policy: &LayoutPolicy) -> Vec<Finding> {
    let mut out = Vec::new();
    let page = Rect {
        x: 0,
        y: 0,
        width: layout.page_width_um,
        height: layout.page_height_um,
    };
    let page_size = (layout.page_width_um, layout.page_height_um);
    if !policy.supported_page_sizes_um.is_empty()
        && !policy.supported_page_sizes_um.contains(&page_size)
    {
        out.push(finding(
            ErrorCode::UnsupportedMediaSize,
            Severity::Blocking,
            "layout.page".into(),
            format!(
                "Page {} x {} um is not a supported media size.",
                page_size.0, page_size.1
            ),
        ));
    }
    let safe = Rect {
        x: i64::from(policy.margin_um),
        y: i64::from(policy.margin_um),
        width: layout
            .page_width_um
            .saturating_sub(policy.margin_um.saturating_mul(2)),
        height: layout
            .page_height_um
            .saturating_sub(policy.margin_um.saturating_mul(2)),
    };
    for p in &layout.placements {
        let subject = p.id.to_string();
        if !page.contains(&p.rect) {
            out.push(finding(
                ErrorCode::Clipping,
                Severity::Blocking,
                subject.clone(),
                "Placement extends beyond the page.".into(),
            ));
        } else if !safe.contains(&p.rect) {
            out.push(finding(
                ErrorCode::UnsafeMargin,
                Severity::ConfirmationRequired,
                subject.clone(),
                "Placement is inside the safe margin.".into(),
            ));
        }
        // ppi = crop_px * 25_400 / size_um. Compare cross-multiplied to avoid rounding.
        let min = u64::from(policy.min_effective_ppi);
        let low = |px: u32, um: u32| u64::from(px) * UM_PER_INCH < min * u64::from(um);
        if low(p.crop_width_px, p.source_width_um) || low(p.crop_height_px, p.source_height_um) {
            out.push(finding(
                ErrorCode::LowEffectiveResolution,
                Severity::ConfirmationRequired,
                subject,
                format!(
                    "Effective resolution is below {} ppi.",
                    policy.min_effective_ppi
                ),
            ));
        }
    }
    out
}
