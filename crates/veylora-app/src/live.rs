//! Job list and job detail read from a real workspace. Nothing here is fixture data and nothing
//! is approximated: a value the UI model cannot represent exactly is an error, not a guess.

use veylora_device::mode::ModeStatus;
use veylora_domain::{
    errors::ErrorCode,
    ids::JobId,
    job::AttemptState,
    layout::Rotation,
    lifecycle::JobState as DomainState,
    preflight::{Finding as DomainFinding, Severity as DomainSeverity},
    recipe::WhiteSource,
};
use veylora_workflow::{
    PreflightOptions, RenderOptions, ReviewResolution, WorkflowError, Workspace,
};

use crate::state::{
    CapabilityState, DataSource, ModeCapability, ModeMaturity, OpenReview, SetupCapability,
    SinkCapability,
};
use crate::view_models::{
    Acknowledgement, Attempt, AttemptKind, AttemptOutcome, Finding, JobDetail, JobState,
    JobSummary, MaskSource, Placement, ProvenanceRow, ReviewOutcome, ReviewView, Severity,
    WhitePolicy, WhitePolicyKind,
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

/// Detail for one job. Findings come only from a preflight run, so they are empty here.
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
            report.submission_attempts.len().to_string(),
        ),
    ];
    Ok(JobDetail {
        summary: summary(ws, id)?,
        placements,
        findings: Vec::new(),
        attempts: attempts(ws, id)?,
        provenance,
        white_policy: white_policy(&snapshot)?,
        rendered: !report.renders.is_empty(),
    })
}

/// Acknowledgements the operator has made this session, per job. They are shown at once and
/// recorded for good in the manifest of the render that uses them.
#[derive(Default)]
pub struct Acknowledgements(std::sync::Mutex<std::collections::BTreeMap<String, Vec<String>>>);

impl Acknowledgements {
    pub fn for_job(&self, job_id: &str) -> Vec<String> {
        self.0
            .lock()
            .map(|m| m.get(job_id).cloned().unwrap_or_default())
            .unwrap_or_default()
    }

    pub fn add(&self, job_id: &str, finding_id: &str) -> Result<(), String> {
        let mut map = self
            .0
            .lock()
            .map_err(|_| "Acknowledgements are unavailable.".to_owned())?;
        let list = map.entry(job_id.to_owned()).or_default();
        if !list.iter().any(|f| f == finding_id) {
            list.push(finding_id.to_owned());
        }
        Ok(())
    }

    /// A render (or a new preflight policy) starts a fresh decision.
    pub fn clear(&self, job_id: &str) {
        if let Ok(mut map) = self.0.lock() {
            map.remove(job_id);
        }
    }
}

/// Capabilities for the app as a whole: those of the newest job's device, or "no setup" when the
/// workspace holds no jobs. Nothing is invented for an empty workspace.
pub fn workspace_capabilities(ws: &Workspace) -> Result<CapabilityState, String> {
    match ws.list_jobs().map_err(|e| e.to_string())?.first() {
        Some(id) => {
            let mut caps = capabilities(ws, &id.to_string())?;
            caps.open_review = open_review(ws)?;
            Ok(caps)
        }
        None => Ok(CapabilityState {
            data_source: DataSource::Live,
            setup: SetupCapability {
                exists: false,
                verified: false,
                experimental: false,
                name: String::new(),
            },
            mode: ModeCapability {
                id: String::new(),
                maturity: ModeMaturity::Experimental,
                sheet_sizes: Vec::new(),
                white_choke: false,
                white_mask_invert: false,
                white_policies: Vec::new(),
                screened_inspect: false,
            },
            sinks: SinkCapability {
                physical_printer: false,
                file_sink: true,
            },
            open_review: None,
        }),
    }
}

fn maturity(status: ModeStatus) -> ModeMaturity {
    match status {
        ModeStatus::Experimental => ModeMaturity::Experimental,
        ModeStatus::Validated => ModeMaturity::Validated,
        ModeStatus::Retired => ModeMaturity::Retired,
    }
}

fn attempt_outcome(state: AttemptState) -> AttemptOutcome {
    match state {
        AttemptState::Submitting => AttemptOutcome::Pending,
        // The transport took the data; whether it printed is not known.
        AttemptState::Submitted => AttemptOutcome::Accepted,
        AttemptState::Ambiguous => AttemptOutcome::Ambiguous,
        AttemptState::Failed => AttemptOutcome::Failed,
        AttemptState::Cancelled => AttemptOutcome::Cancelled,
        // Only an operator's confirmation is ever called finished.
        AttemptState::OperatorConfirmedFinished => AttemptOutcome::Finished,
    }
}

fn attempts(ws: &Workspace, id: JobId) -> Result<Vec<Attempt>, String> {
    Ok(ws
        .job_attempts(id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|a| Attempt {
            id: a.attempt_id.to_string(),
            kind: AttemptKind::Transport,
            at: a.recorded_at,
            outcome: attempt_outcome(a.state),
            note: a.note,
            sink_path: None,
            spool_job_id: None,
            bytes_written: Some(a.bytes_written),
            bytes_intended: None,
            artifact_hash: None,
        })
        .collect())
}

/// The choices a live review can honour. The workflow records "it printed" and "nothing printed"
/// (which cancels the job; a new print is a new job). It cannot record "printed badly", so that is
/// not offered rather than promised.
fn live_choices() -> Vec<crate::view_models::ReviewChoiceView> {
    let choice =
        |outcome, label: &str, confirm: &str, effect: &str| crate::view_models::ReviewChoiceView {
            outcome,
            label: label.to_owned(),
            confirm_label: confirm.to_owned(),
            effect: effect.to_owned(),
        };
    vec![
        choice(
            ReviewOutcome::Finished,
            "It printed correctly",
            "Mark finished",
            "The attempt will be recorded as finished by the operator. No new attempt is created.",
        ),
        choice(
            ReviewOutcome::Reprint,
            "Nothing printed",
            "Cancel this job",
            "The job will be recorded as not printed and cancelled. Printing again is a new job.",
        ),
        choice(
            ReviewOutcome::DecideLater,
            "Decide later",
            "Keep in review",
            "The attempt stays in review. The printer remains blocked until it is resolved.",
        ),
    ]
}

/// The attempt waiting for an operator outcome, if the job has one.
pub fn review_view(ws: &Workspace, job_id: &str) -> Result<Option<ReviewView>, String> {
    let id = parse_id(job_id)?;
    let review = ws.review(id).map_err(|e| e.to_string())?;
    if !review.requires_review {
        return Ok(None);
    }
    let detail = job_detail(ws, job_id)?;
    let caps = capabilities(ws, job_id)?;
    Ok(
        crate::screens::review::present(&caps, &detail).map(|mut v| {
            v.choices = live_choices();
            v
        }),
    )
}

/// Records the operator's decision for the attempt, wherever its job is.
pub fn resolve_review(
    ws: &Workspace,
    attempt_id: &str,
    outcome: ReviewOutcome,
) -> Result<CapabilityState, String> {
    let mut found = None;
    for id in ws.list_jobs().map_err(|e| e.to_string())? {
        if ws
            .job_attempts(id)
            .map_err(|e| e.to_string())?
            .iter()
            .any(|a| a.attempt_id.to_string() == attempt_id)
        {
            found = Some(id);
            break;
        }
    }
    let id = found.ok_or_else(|| format!("No attempt '{attempt_id}' exists in this workspace."))?;
    let resolution = match outcome {
        ReviewOutcome::Finished => Some(ReviewResolution::ConfirmedFinished),
        ReviewOutcome::Reprint => Some(ReviewResolution::NotPrinted),
        ReviewOutcome::DecideLater => None,
        ReviewOutcome::PrintedBadly => {
            return Err("Recording 'printed badly' is not available for live jobs yet.".to_owned())
        }
    };
    if let Some(resolution) = resolution {
        ws.resolve_review(
            id,
            resolution,
            LOCAL_OPERATOR,
            "Decision recorded from the app's review card; no further notes were entered.",
        )
        .map_err(|e| e.to_string())?;
    }
    workspace_capabilities(ws)
}

/// The first job that still needs an operator decision, as the app-wide capability state shows it.
fn open_review(ws: &Workspace) -> Result<Option<OpenReview>, String> {
    for id in ws.list_jobs().map_err(|e| e.to_string())? {
        let review = ws.review(id).map_err(|e| e.to_string())?;
        if review.requires_review {
            if let Some(a) = review.attempts.iter().find(|a| a.requires_review) {
                return Ok(Some(OpenReview {
                    job_id: id.to_string(),
                    attempt_id: a.attempt_id.clone(),
                    job_name: summary(ws, id)?.name,
                }));
            }
        }
    }
    Ok(None)
}

/// Capabilities of the device a job was created against. What the stored records do not say is
/// reported conservatively (controls off, no sheet list) rather than guessed: sheet sizes are the
/// job's own layout, and white choke, mask inversion and policy choices are not read from the mode yet.
pub fn capabilities(ws: &Workspace, job_id: &str) -> Result<CapabilityState, String> {
    let device = ws
        .job_device(parse_id(job_id)?)
        .map_err(|e| e.to_string())?;
    Ok(CapabilityState {
        data_source: DataSource::Live,
        setup: SetupCapability {
            exists: true,
            verified: device.setup_physically_verified,
            experimental: false,
            name: device.setup_id,
        },
        mode: ModeCapability {
            id: device.mode_id,
            maturity: maturity(device.mode_status),
            sheet_sizes: vec![format!(
                "{} x {} mm",
                mm_text(device.printable_um.0 as u32),
                mm_text(device.printable_um.1 as u32)
            )],
            // The renderer applies a recipe's white spread, so choke is supported; recipes have
            // no mask inversion.
            white_choke: true,
            white_mask_invert: false,
            white_policies: if device.white_capable {
                vec![WhitePolicyKind::Underbase, WhitePolicyKind::NoWhite]
            } else {
                vec![WhitePolicyKind::NoWhite]
            },
            // Every render retains screened planes.
            screened_inspect: true,
        },
        sinks: SinkCapability {
            // No physical submission exists until the physical gate passes.
            physical_printer: false,
            file_sink: true,
        },
        open_review: None,
    })
}

fn finding_id(f: &DomainFinding) -> String {
    format!("{}:{}", f.code, f.subject.as_deref().unwrap_or("-"))
}

fn humanize(code: ErrorCode) -> String {
    let text = code.as_str().replace('_', " ");
    let mut chars = text.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

fn now_utc() -> String {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64);
    utc_text(ms)
}

/// Operator recorded on acknowledgements: operator identity does not exist yet, so the record names
/// the workstation session instead of inventing a person.
const LOCAL_OPERATOR: &str = "Operator at this workstation";

fn view_finding(f: &DomainFinding, acknowledged: &[String], at: &str) -> Finding {
    let id = finding_id(f);
    let severity = match f.severity {
        DomainSeverity::Blocking => Severity::Blocking,
        DomainSeverity::ConfirmationRequired => Severity::Confirm,
        DomainSeverity::Informational => Severity::Info,
    };
    let is_acknowledged = severity == Severity::Confirm && acknowledged.contains(&id);
    Finding {
        id,
        severity,
        title: humanize(f.code),
        cause: f.message.clone(),
        fix: f.code.message().to_owned(),
        placement_id: None,
        acknowledged: is_acknowledged,
        acknowledgement: is_acknowledged.then(|| Acknowledgement {
            operator: LOCAL_OPERATOR.to_owned(),
            at: at.to_owned(),
        }),
    }
}

/// Acknowledgement ids must name a confirm-level finding of the current preflight.
fn check_acknowledgements(findings: &[DomainFinding], ids: &[String]) -> Result<(), String> {
    for id in ids {
        match findings.iter().find(|f| &finding_id(f) == id) {
            None => return Err(format!("'{id}' is not a finding of this job's preflight.")),
            Some(f) if f.severity != DomainSeverity::ConfirmationRequired => {
                return Err(format!(
                    "'{id}' cannot be acknowledged: only confirmation-required findings can be."
                ))
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// Runs preflight (which records its outcome in the job's lifecycle) and returns the job with real
/// findings. `acknowledged` ids are marked acknowledged; they are not stored until a render.
pub fn run_preflight(
    ws: &Workspace,
    job_id: &str,
    acknowledged: &[String],
) -> Result<JobDetail, String> {
    let id = parse_id(job_id)?;
    let report = ws
        .preflight(id, &PreflightOptions::default())
        .map_err(|e| e.to_string())?;
    check_acknowledgements(&report.findings, acknowledged)?;
    let at = now_utc();
    let mut detail = job_detail(ws, job_id)?;
    detail.findings = report
        .findings
        .iter()
        .map(|f| view_finding(f, acknowledged, &at))
        .collect();
    Ok(detail)
}

/// Renders the job with the acknowledged confirmations, which the render records in its manifest.
/// Blocking findings, or confirmations left unacknowledged, refuse the render and name themselves.
pub fn render_job(
    ws: &Workspace,
    job_id: &str,
    acknowledged: &[String],
) -> Result<JobDetail, String> {
    let id = parse_id(job_id)?;
    let report = ws
        .preflight(id, &PreflightOptions::default())
        .map_err(|e| e.to_string())?;
    check_acknowledgements(&report.findings, acknowledged)?;
    let options = RenderOptions {
        acknowledged: report
            .findings
            .iter()
            .filter(|f| acknowledged.contains(&finding_id(f)))
            .map(|f| (f.code, f.subject.clone()))
            .collect(),
        ..RenderOptions::default()
    };
    match ws.render(id, &options) {
        Ok(_) => {}
        Err(WorkflowError::PreflightBlocked(open)) => {
            let names: Vec<String> = open.iter().map(|f| humanize(f.code)).collect();
            return Err(format!("Render refused: {}.", names.join(", ")));
        }
        Err(e) => return Err(e.to_string()),
    }
    let mut detail = job_detail(ws, job_id)?;
    let at = now_utc();
    detail.findings = report
        .findings
        .iter()
        .map(|f| view_finding(f, acknowledged, &at))
        .collect();
    Ok(detail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mode_status_maps_to_its_own_maturity() {
        assert_eq!(
            maturity(ModeStatus::Experimental),
            ModeMaturity::Experimental
        );
        assert_eq!(maturity(ModeStatus::Validated), ModeMaturity::Validated);
        assert_eq!(maturity(ModeStatus::Retired), ModeMaturity::Retired);
    }

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
