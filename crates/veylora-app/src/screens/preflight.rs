//! Preflight tab presentation (design 3.8).

use crate::view_models::{
    Finding, FindingGroupView, FindingView, JobDetail, PreflightView, Severity, SummaryView, Tone,
};

const NO_AUTOMATIC_FIX: &str = "There is no automatic fix for this finding. Resolve the cause described above, then preflight checks again.";

fn finding_view(finding: &Finding) -> FindingView {
    let fix = finding.fix.trim();
    let correction = if fix.is_empty() && finding.severity == Severity::Blocking {
        NO_AUTOMATIC_FIX.to_owned()
    } else {
        fix.to_owned()
    };
    FindingView {
        id: finding.id.clone(),
        severity: finding.severity,
        title: finding.title.clone(),
        cause: finding.cause.clone(),
        correction,
        acknowledged: finding.acknowledged,
        can_acknowledge: finding.severity == Severity::Confirm && !finding.acknowledged,
    }
}

fn summary(job: &JobDetail, blocking: usize, to_confirm: usize) -> SummaryView {
    let (text, tone) = if blocking > 0 {
        let mut parts = vec![format!("{blocking} blocking")];
        if to_confirm > 0 {
            parts.push(format!("{to_confirm} to confirm"));
        }
        (parts.join(" \u{b7} "), Tone::Bad)
    } else if to_confirm > 0 {
        (format!("{to_confirm} to confirm"), Tone::Warn)
    } else if job.rendered {
        ("Passed at render".to_owned(), Tone::Neutral)
    } else {
        ("Ready to render".to_owned(), Tone::Ok)
    };
    SummaryView { text, tone }
}

/// Builds the preflight view for a job against the active setup and mode.
pub fn present(job: &JobDetail, setup_label: &str, mode_id: &str) -> PreflightView {
    let groups = [
        (Severity::Blocking, "Blocking", Tone::Bad),
        (Severity::Confirm, "Confirm", Tone::Warn),
        (Severity::Info, "Info", Tone::Info),
    ]
    .into_iter()
    .filter_map(|(severity, title, tone)| {
        let findings: Vec<FindingView> = job
            .findings
            .iter()
            .filter(|f| f.severity == severity)
            .map(finding_view)
            .collect();
        (!findings.is_empty()).then(|| FindingGroupView {
            severity,
            title: title.to_owned(),
            tone,
            findings,
        })
    })
    .collect();

    let blocking = job
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Blocking)
        .count();
    let to_confirm = job
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Confirm && !f.acknowledged)
        .count();

    PreflightView {
        checked_against: format!("Checked just now against {setup_label}, mode {mode_id}"),
        groups,
        summary: summary(job, blocking, to_confirm),
        open_count: blocking + to_confirm,
    }
}
