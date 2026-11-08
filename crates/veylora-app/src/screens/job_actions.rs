//! Hold / inspect / release availability for a job. Every disabled command
//! carries the reason shown beside it.

use crate::state::CapabilityState;
use crate::view_models::{CommandView, JobActionsView, JobDetail, JobState, Severity};

/// The one reason physical submission is refused. The release command and the
/// release control both read it through `release_to_printer`.
pub const PHYSICAL_GATE: &str =
    "Physical submission is unavailable until the ET-8550 physical gate passes";

fn count_label(count: usize, noun: &str) -> String {
    format!("{count} {noun}{}", if count == 1 { "" } else { "s" })
}

pub fn render_and_hold(job: &JobDetail) -> CommandView {
    if job.rendered || job.summary.state != JobState::Draft {
        return CommandView::disabled("Already rendered. Duplicate as new job to edit.");
    }
    let blocking = job
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Blocking)
        .count();
    if blocking > 0 {
        return CommandView::disabled(format!(
            "Fix {} to render",
            count_label(blocking, "blocking finding")
        ));
    }
    let unconfirmed = job
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Confirm && !f.acknowledged)
        .count();
    if unconfirmed > 0 {
        return CommandView::disabled(format!(
            "Confirm {} to render",
            count_label(unconfirmed, "finding")
        ));
    }
    CommandView::enabled()
}

fn is_held(job: &JobDetail) -> bool {
    job.rendered && matches!(job.summary.state, JobState::Held | JobState::Ready)
}

pub fn release_to_printer(caps: &CapabilityState, job: &JobDetail) -> CommandView {
    if !is_held(job) {
        return CommandView::disabled("Render and hold the job first");
    }
    if !caps.sinks.physical_printer {
        return CommandView::disabled(PHYSICAL_GATE);
    }
    if let Some(open) = &caps.open_review {
        return CommandView::disabled(format!(
            "Waiting: printer blocked by review of {}",
            open.job_id
        ));
    }
    if !caps.setup.verified && !caps.setup.experimental {
        return CommandView::disabled("Verify the setup, or turn on experimental mode in Devices");
    }
    CommandView::enabled()
}

pub fn write_to_file_sink(caps: &CapabilityState, job: &JobDetail) -> CommandView {
    if !caps.sinks.file_sink {
        return CommandView::disabled("File sink is not available");
    }
    if !is_held(job) {
        return CommandView::disabled("Render and hold the job before writing to the file sink");
    }
    CommandView::enabled()
}

pub fn inspect(job: &JobDetail) -> CommandView {
    if job.rendered {
        CommandView::enabled()
    } else {
        CommandView::disabled("Render the job to inspect its retained output.")
    }
}

pub fn present(caps: &CapabilityState, job: &JobDetail) -> JobActionsView {
    JobActionsView {
        render_and_hold: render_and_hold(job),
        release_to_printer: release_to_printer(caps, job),
        write_to_file_sink: write_to_file_sink(caps, job),
        inspect: inspect(job),
    }
}
