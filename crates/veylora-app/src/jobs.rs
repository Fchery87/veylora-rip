//! Fixture job store. Labeled fixture data until the application-service
//! crates exist; it mirrors the working-day scenario in
//! `app/src/lib/services/mock.ts`.

use crate::screens::{job_actions, review};
use crate::state::{CapabilityState, OpenReview};
use crate::view_models::{
    Acknowledgement, Attempt, AttemptKind, AttemptOutcome, Finding, JobDetail, JobState,
    JobSummary, MaskSource, Placement, ProvenanceRow, ReviewOutcome, Severity, WhitePolicy,
    WhitePolicyKind,
};

fn placement(id: &str, name: &str, x_mm: f64, clipped: bool) -> Placement {
    Placement {
        id: id.to_owned(),
        name: name.to_owned(),
        x_mm,
        y_mm: 30.0,
        width_mm: 180.0,
        height_mm: 240.0,
        clipped,
    }
}

/// Operator recorded on acknowledgements until real operator identity exists.
const FIXTURE_OPERATOR: &str = "Fixture operator";

fn finding(id: &str, severity: Severity, title: &str, cause: &str, fix: &str) -> Finding {
    Finding {
        id: id.to_owned(),
        severity,
        title: title.to_owned(),
        cause: cause.to_owned(),
        fix: fix.to_owned(),
        placement_id: Some("p1".to_owned()),
        acknowledged: false,
        acknowledgement: None,
    }
}

/// UTC wall-clock time of day, `HH:MM:SS UTC`.
fn time_of_day_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
        % 86_400;
    format!(
        "{:02}:{:02}:{:02} UTC",
        secs / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}

fn base_job(id: &str, name: &str) -> JobDetail {
    let row = |label: &str, value: &str| ProvenanceRow {
        label: label.to_owned(),
        value: value.to_owned(),
    };
    JobDetail {
        summary: JobSummary {
            id: id.to_owned(),
            name: name.to_owned(),
            state: JobState::Draft,
            sheet: "A3".to_owned(),
            placement_count: 1,
            updated: "just now".to_owned(),
            progress_percent: None,
        },
        placements: vec![placement("p1", "artwork.png", 20.0, false)],
        findings: Vec::new(),
        attempts: Vec::new(),
        provenance: vec![
            row("Device setup", "Home bench rev 3"),
            row("Print mode", "PM-exp-01 (experimental)"),
            row("Recipe", "RCP-fixture-01"),
            row("Snapshot hash", "sha256:fixture-0000"),
        ],
        white_policy: WhitePolicy {
            source: MaskSource::AutoFromAlpha,
            policy: WhitePolicyKind::Underbase,
            invert_mask: false,
            choke_mm: 0.0,
        },
        rendered: false,
    }
}

pub struct JobStore {
    jobs: Vec<JobDetail>,
}

impl JobStore {
    /// The working-day fixture scenario.
    pub fn fixture() -> Self {
        let mut ambiguous = base_job("J-0142", "North Ridge tee run");
        ambiguous.summary.state = JobState::Ambiguous;
        ambiguous.summary.updated = "12 min ago".to_owned();
        ambiguous.rendered = true;
        ambiguous.attempts = vec![Attempt {
            id: "A-0142-1".to_owned(),
            kind: AttemptKind::Printer,
            at: "09:14:02".to_owned(),
            outcome: AttemptOutcome::Ambiguous,
            note: "Spool ended before confirmation. Fixture data.".to_owned(),
            sink_path: None,
            spool_job_id: Some("SPOOL-0142".to_owned()),
            bytes_written: Some(1_048_576),
            bytes_intended: Some(4_194_304),
            artifact_hash: Some("sha256:fixture-artifact-0142".to_owned()),
        }];

        let mut held = base_job("J-0143", "Front chest logo");
        held.summary.state = JobState::Held;
        held.summary.updated = "1 h ago".to_owned();
        held.rendered = true;

        let mut hoodie = base_job("J-0144", "Hoodie sleeve");
        hoodie.placements = vec![placement("p1", "sleeve.png", -4.2, true)];
        hoodie.findings = vec![finding(
            "f1",
            Severity::Blocking,
            "Placement is outside the printable area",
            "sleeve.png extends past the left edge of the printable box.",
            "Move it left by at least 4.2 mm",
        )];

        let mut pocket = base_job("J-0145", "Pocket print");
        pocket.findings = vec![finding(
            "f2",
            Severity::Confirm,
            "Artwork has no color profile",
            "pocket.png is untagged, so sRGB is assumed.",
            "Confirm sRGB is correct, or re-export the file with a profile.",
        )];

        Self {
            jobs: vec![ambiguous, held, hoodie, pocket],
        }
    }

    pub fn list(&self) -> Vec<JobSummary> {
        self.jobs.iter().map(|j| j.summary.clone()).collect()
    }

    pub fn get(&self, id: &str) -> Result<JobDetail, String> {
        self.jobs
            .iter()
            .find(|j| j.summary.id == id)
            .cloned()
            .ok_or_else(|| format!("Unknown job {id}"))
    }
}

/// Shown if the gate ever opens before a transport exists.
const NO_PHYSICAL_TRANSPORT: &str = "Physical submission has no transport yet.";

impl JobStore {
    fn job_mut(&mut self, id: &str) -> Result<&mut JobDetail, String> {
        self.jobs
            .iter_mut()
            .find(|j| j.summary.id == id)
            .ok_or_else(|| format!("Unknown job {id}"))
    }

    /// Capability state for the current jobs. An unresolved attempt opens a review.
    pub fn capabilities(&self) -> CapabilityState {
        let mut caps = CapabilityState::fixture();
        caps.open_review = self.jobs.iter().find_map(|job| {
            review::open_attempt(job).map(|attempt| OpenReview {
                job_id: job.summary.id.clone(),
                attempt_id: attempt.id.clone(),
                job_name: job.summary.name.clone(),
            })
        });
        caps
    }

    pub fn acknowledge_finding(
        &mut self,
        job_id: &str,
        finding_id: &str,
    ) -> Result<JobDetail, String> {
        let job = self.job_mut(job_id)?;
        let finding = job
            .findings
            .iter_mut()
            .find(|f| f.id == finding_id)
            .ok_or_else(|| format!("Unknown finding {finding_id}"))?;
        if finding.severity != Severity::Confirm {
            return Err("Only confirm-level findings can be acknowledged.".to_owned());
        }
        finding.acknowledged = true;
        finding.acknowledgement = Some(Acknowledgement {
            operator: FIXTURE_OPERATOR.to_owned(),
            at: time_of_day_utc(),
        });
        Ok(job.clone())
    }

    pub fn render_and_hold(&mut self, job_id: &str) -> Result<JobDetail, String> {
        let job = self.job_mut(job_id)?;
        let gate = job_actions::render_and_hold(job);
        if !gate.enabled {
            return Err(gate.reason.unwrap_or_default());
        }
        job.summary.state = JobState::Held;
        job.summary.updated = "just now".to_owned();
        job.rendered = true;
        Ok(job.clone())
    }

    pub fn write_to_file_sink(&mut self, job_id: &str) -> Result<JobDetail, String> {
        let caps = self.capabilities();
        let job = self.job_mut(job_id)?;
        let gate = job_actions::write_to_file_sink(&caps, job);
        if !gate.enabled {
            return Err(gate.reason.unwrap_or_default());
        }
        let sink_number = job
            .attempts
            .iter()
            .filter(|a| a.kind == AttemptKind::FileSink)
            .count()
            + 1;
        let number = job_id.trim_start_matches("J-");
        job.attempts.push(Attempt {
            id: format!("A-{number}-f{sink_number}"),
            kind: AttemptKind::FileSink,
            at: "now".to_owned(),
            outcome: AttemptOutcome::Finished,
            note: "Written to the file sink. Not physical output.".to_owned(),
            sink_path: Some(format!(r"C:\Veylora\sink\{job_id}-{sink_number}.vrip")),
            spool_job_id: None,
            bytes_written: None,
            bytes_intended: None,
            artifact_hash: None,
        });
        job.summary.state = JobState::Held;
        Ok(job.clone())
    }

    /// Physical submission. Refused for the same reason the release control
    /// shows (`job_actions::release_to_printer`); it never creates an attempt.
    pub fn release_to_printer(&mut self, job_id: &str) -> Result<JobDetail, String> {
        let caps = self.capabilities();
        let job = self.job_mut(job_id)?;
        let gate = job_actions::release_to_printer(&caps, job);
        if !gate.enabled {
            return Err(gate.reason.unwrap_or_default());
        }
        Err(NO_PHYSICAL_TRANSPORT.to_owned())
    }

    /// Records the operator's outcome for an attempt in review. It never
    /// retries or sends, and a recorded outcome is final. A reprint choice
    /// appends one new attempt, held: it needs an explicit release.
    pub fn resolve_review(
        &mut self,
        attempt_id: &str,
        outcome: ReviewOutcome,
    ) -> Result<CapabilityState, String> {
        let job = self
            .jobs
            .iter_mut()
            .find(|j| j.attempts.iter().any(|a| a.id == attempt_id))
            .ok_or_else(|| format!("Unknown attempt {attempt_id}"))?;
        let (attempt_outcome, state) = match outcome {
            ReviewOutcome::Finished => (
                AttemptOutcome::Finished,
                JobState::OperatorConfirmedFinished,
            ),
            ReviewOutcome::Reprint => (AttemptOutcome::Reprint, JobState::Held),
            ReviewOutcome::PrintedBadly => (AttemptOutcome::PrintedBadly, JobState::Held),
            ReviewOutcome::DecideLater => (AttemptOutcome::KeptInReview, JobState::NeedsReview),
        };
        let attempt = job
            .attempts
            .iter_mut()
            .find(|a| a.id == attempt_id)
            .expect("attempt located above");
        if !attempt.outcome.is_open_review() {
            return Err(format!("Attempt {attempt_id} is not waiting for review."));
        }
        attempt.outcome = attempt_outcome;
        job.summary.state = state;
        if matches!(
            outcome,
            ReviewOutcome::Reprint | ReviewOutcome::PrintedBadly
        ) {
            let printer_attempts = job
                .attempts
                .iter()
                .filter(|a| a.kind == AttemptKind::Printer)
                .count();
            let number = job.summary.id.trim_start_matches("J-").to_owned();
            job.attempts.push(Attempt {
                id: format!("A-{number}-{}", printer_attempts + 1),
                kind: AttemptKind::Printer,
                at: "now".to_owned(),
                outcome: AttemptOutcome::Held,
                note: "Reprint attempt created held. It has not been sent and is never retried automatically; release it explicitly.".to_owned(),
                sink_path: None,
                spool_job_id: None,
                bytes_written: None,
                bytes_intended: None,
                artifact_hash: None,
            });
        }
        Ok(self.capabilities())
    }
}

impl JobStore {
    fn draft_mut(&mut self, job_id: &str) -> Result<&mut JobDetail, String> {
        let job = self.job_mut(job_id)?;
        if job.rendered {
            return Err("Snapshot frozen. Duplicate the job to edit.".to_owned());
        }
        Ok(job)
    }

    pub fn set_white_policy(
        &mut self,
        job_id: &str,
        policy: WhitePolicyKind,
    ) -> Result<JobDetail, String> {
        let caps = self.capabilities();
        let job = self.draft_mut(job_id)?;
        if !caps.mode.white_policies.contains(&policy) {
            return Err(format!(
                "White policy {policy:?} is not supported by mode {}",
                caps.mode.id
            ));
        }
        job.white_policy.policy = policy;
        Ok(job.clone())
    }

    pub fn set_white_mask(
        &mut self,
        job_id: &str,
        source: MaskSource,
        invert: bool,
    ) -> Result<JobDetail, String> {
        let caps = self.capabilities();
        let job = self.draft_mut(job_id)?;
        if invert && !caps.mode.white_mask_invert {
            return Err(format!(
                "Mask inversion is not supported by mode {}",
                caps.mode.id
            ));
        }
        job.white_policy.source = source;
        job.white_policy.invert_mask = invert;
        Ok(job.clone())
    }

    pub fn set_white_choke(&mut self, job_id: &str, choke_mm: f64) -> Result<JobDetail, String> {
        let caps = self.capabilities();
        let job = self.draft_mut(job_id)?;
        if !caps.mode.white_choke {
            return Err(format!(
                "White choke is not supported by mode {}",
                caps.mode.id
            ));
        }
        job.white_policy.choke_mm = choke_mm;
        Ok(job.clone())
    }
}
