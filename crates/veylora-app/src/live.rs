//! Job list and job detail read from a real workspace. Nothing here is fixture data and nothing
//! is approximated: a value the UI model cannot represent exactly is an error, not a guess.

use veylora_domain::{
    ids::JobId, layout::Rotation, lifecycle::JobState as DomainState, recipe::WhiteSource,
};
use veylora_workflow::Workspace;

use crate::view_models::{
    JobDetail, JobState, JobSummary, MaskSource, Placement, ProvenanceRow, WhitePolicy,
    WhitePolicyKind,
};

fn parse_id(job_id: &str) -> Result<JobId, String> {
    job_id
        .parse()
        .map_err(|_| format!("'{job_id}' is not a job id."))
}

fn state(s: DomainState) -> JobState {
    match s {
        DomainState::Draft => JobState::Draft,
        DomainState::PreflightFailed => JobState::PreflightFailed,
        DomainState::Held => JobState::Held,
        DomainState::Rendering => JobState::Rendering,
        DomainState::Ready => JobState::Ready,
        DomainState::Submitting => JobState::Submitting,
        DomainState::Submitted => JobState::Submitted,
        DomainState::Ambiguous => JobState::Ambiguous,
        DomainState::Review => JobState::NeedsReview,
        DomainState::Cancelled => JobState::Cancelled,
        DomainState::Failed => JobState::Failed,
        DomainState::OperatorConfirmedFinished => JobState::OperatorConfirmedFinished,
    }
}

fn mm(um: impl Into<f64>) -> f64 {
    um.into() / 1000.0
}

/// Whole millimetres without a trailing ".0", otherwise up to three decimals.
fn mm_text(um: u32) -> String {
    if um.is_multiple_of(1000) {
        (um / 1000).to_string()
    } else {
        format!("{:.3}", f64::from(um) / 1000.0)
    }
}

/// `YYYY-MM-DD HH:MM UTC` from Unix milliseconds (proleptic Gregorian, no leap seconds).
fn utc_text(ms: u64) -> String {
    let secs = ms / 1000;
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02} UTC",
        rem / 3600,
        rem % 3600 / 60
    )
}

fn summary(ws: &Workspace, id: JobId) -> Result<JobSummary, String> {
    let snapshot = ws.job_snapshot(id).map_err(|e| e.to_string())?;
    let inputs = snapshot.inputs();
    let layout = inputs.layout.inputs();
    let current = ws.state(id).map_err(|e| e.to_string())?;
    let updated = ws
        .lifecycle_events(id)
        .map_err(|e| e.to_string())?
        .last()
        .map_or_else(|| utc_text(inputs.created_at_ms), |r| r.recorded_at.clone());
    Ok(JobSummary {
        id: id.to_string(),
        // Jobs have no operator-given name yet; the id prefix is the honest label.
        name: format!("Job {}", &id.to_string()[..8]),
        state: state(current),
        sheet: format!(
            "{} x {} mm",
            mm_text(layout.page_width_um),
            mm_text(layout.page_height_um)
        ),
        placement_count: layout.placements.len(),
        updated,
        progress_percent: None,
    })
}

/// Every stored job, newest first. A job whose records cannot be read is an error for the whole
/// list rather than a silent omission.
pub fn job_summaries(ws: &Workspace) -> Result<Vec<JobSummary>, String> {
    ws.list_jobs()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|id| summary(ws, id))
        .collect()
}

fn white_policy(snapshot: &veylora_domain::job::JobSnapshot) -> Result<WhitePolicy, String> {
    let white = &snapshot.inputs().recipe.inputs().white;
    let (source, policy) = match &white.source {
        WhiteSource::Alpha => (MaskSource::AutoFromAlpha, WhitePolicyKind::Underbase),
        // The source is irrelevant when white is off; the UI model still needs one value.
        WhiteSource::Disabled => (MaskSource::AutoFromAlpha, WhitePolicyKind::NoWhite),
        other => {
            return Err(format!(
                "This job's recipe uses the white source {other:?}, which is not represented in the app yet."
            ))
        }
    };
    if white.spread_um > 0 {
        return Err(format!(
            "This job's recipe expands white by {} um, which is not represented in the app yet.",
            white.spread_um
        ));
    }
    Ok(WhitePolicy {
        source,
        policy,
        invert_mask: false,
        choke_mm: mm(white.spread_um.unsigned_abs()),
    })
}

/// Detail for one job. Findings and attempts are not itemized here: findings come from a preflight
/// run, and attempt outcomes need the operator-review vocabulary mapped without approximation.
pub fn job_detail(ws: &Workspace, job_id: &str) -> Result<JobDetail, String> {
    let id = parse_id(job_id)?;
    let snapshot = ws.job_snapshot(id).map_err(|e| e.to_string())?;
    let report = ws.report(id).map_err(|e| e.to_string())?;
    let layout = snapshot.inputs().layout.inputs();
    let placements = layout
        .placements
        .iter()
        .map(|p| {
            let (w, h) = match p.rotation {
                Rotation::Deg90 | Rotation::Deg270 => (p.height_um, p.width_um),
                _ => (p.width_um, p.height_um),
            };
            let clipped = p.x_um < 0
                || p.y_um < 0
                || p.x_um + i64::from(w) > i64::from(layout.page_width_um)
                || p.y_um + i64::from(h) > i64::from(layout.page_height_um);
            Placement {
                id: p.id.to_string(),
                name: format!("Asset {}", &p.asset_id.to_string()[..8]),
                x_mm: mm(p.x_um as f64),
                y_mm: mm(p.y_um as f64),
                width_mm: mm(p.width_um),
                height_mm: mm(p.height_um),
                clipped,
            }
        })
        .collect();
    let row = |label: &str, value: String| ProvenanceRow {
        label: label.to_owned(),
        value,
    };
    let recipe = &snapshot.inputs().recipe.inputs();
    let provenance = vec![
        row("Device setup", report.setup.to_string()),
        row("Print mode", report.mode.to_string()),
        row(
            "Recipe",
            format!("{} rev {} ({})", recipe.id, recipe.revision, report.recipe),
        ),
        row("Snapshot hash", report.snapshot_hash.to_string()),
        row("Layout hash", report.layout_hash.to_string()),
        row("Created", utc_text(report.created_at_ms)),
        row(
            "Submission attempts",
            format!(
                "{} recorded (not itemized here yet)",
                report.submission_attempts.len()
            ),
        ),
    ];
    Ok(JobDetail {
        summary: summary(ws, id)?,
        placements,
        findings: Vec::new(),
        attempts: Vec::new(),
        provenance,
        white_policy: white_policy(&snapshot)?,
        rendered: !report.renders.is_empty(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_lifecycle_state_maps_to_its_own_view_state() {
        let all = [
            (DomainState::Draft, JobState::Draft),
            (DomainState::PreflightFailed, JobState::PreflightFailed),
            (DomainState::Held, JobState::Held),
            (DomainState::Rendering, JobState::Rendering),
            (DomainState::Ready, JobState::Ready),
            (DomainState::Submitting, JobState::Submitting),
            (DomainState::Submitted, JobState::Submitted),
            (DomainState::Ambiguous, JobState::Ambiguous),
            (DomainState::Review, JobState::NeedsReview),
            (DomainState::Cancelled, JobState::Cancelled),
            (DomainState::Failed, JobState::Failed),
            (
                DomainState::OperatorConfirmedFinished,
                JobState::OperatorConfirmedFinished,
            ),
        ];
        for (domain, view) in all {
            assert_eq!(state(domain), view);
        }
    }
}
