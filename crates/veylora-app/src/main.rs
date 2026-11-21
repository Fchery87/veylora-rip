//! Veylora RIP desktop shell (Tauri 2).
//!
//! Thin adapter: it exposes typed commands that return read models. All
//! authoritative behavior stays in Rust application services (ADR 0001).
//! Presentation lives in `veylora_app_lib::screens`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use tauri::State;
use veylora_app_lib::artifacts::retained_for_job;
use veylora_app_lib::jobs::JobStore;
use veylora_app_lib::screens::{inspect, job_actions, preflight, review};
use veylora_app_lib::state::CapabilityState;
use veylora_app_lib::view_models::{
    InspectSelection, InspectView, JobActionsView, JobDetail, JobSummary, MaskSource, Plane,
    PreflightView, ReviewOutcome, ReviewView, WhitePolicyKind,
};

/// Fixture job state shared by the commands until application services exist.
struct Jobs(Mutex<JobStore>);

impl Jobs {
    fn with_store<T>(
        &self,
        f: impl FnOnce(&mut JobStore) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut store = self
            .0
            .lock()
            .map_err(|_| "Job store is unavailable.".to_owned())?;
        f(&mut store)
    }
}

#[tauri::command]
fn get_capabilities(jobs: State<Jobs>) -> Result<CapabilityState, String> {
    jobs.with_store(|s| Ok(s.capabilities()))
}

#[tauri::command]
fn list_jobs(jobs: State<Jobs>) -> Result<Vec<JobSummary>, String> {
    jobs.with_store(|s| Ok(s.list()))
}

#[tauri::command]
fn get_job(jobs: State<Jobs>, job_id: String) -> Result<JobDetail, String> {
    jobs.with_store(|s| s.get(&job_id))
}

#[tauri::command]
fn get_preflight_view(jobs: State<Jobs>, job_id: String) -> Result<PreflightView, String> {
    jobs.with_store(|s| Ok(preflight::present(&s.get(&job_id)?, &s.capabilities())))
}

#[tauri::command]
fn get_job_actions(jobs: State<Jobs>, job_id: String) -> Result<JobActionsView, String> {
    jobs.with_store(|s| Ok(job_actions::present(&s.capabilities(), &s.get(&job_id)?)))
}

#[tauri::command]
fn get_inspect_view(
    jobs: State<Jobs>,
    job_id: String,
    selection: Option<InspectSelection>,
) -> Result<InspectView, String> {
    let selection = selection.unwrap_or_else(inspect::default_selection);
    jobs.with_store(|s| inspect::present(&s.get(&job_id)?, &s.capabilities(), &selection))
}

#[tauri::command]
fn toggle_inspect_plane(
    jobs: State<Jobs>,
    job_id: String,
    selection: InspectSelection,
    plane: Plane,
    on: bool,
) -> Result<InspectView, String> {
    let next = inspect::toggle_plane(&selection, plane, on);
    jobs.with_store(|s| inspect::present(&s.get(&job_id)?, &s.capabilities(), &next))
}

#[tauri::command]
fn set_inspect_composite(
    jobs: State<Jobs>,
    job_id: String,
    selection: InspectSelection,
    on: bool,
) -> Result<InspectView, String> {
    let next = inspect::set_composite(&selection, on);
    jobs.with_store(|s| inspect::present(&s.get(&job_id)?, &s.capabilities(), &next))
}

#[tauri::command]
fn get_review_view(jobs: State<Jobs>, job_id: String) -> Result<Option<ReviewView>, String> {
    jobs.with_store(|s| Ok(review::present(&s.capabilities(), &s.get(&job_id)?)))
}

#[tauri::command]
fn acknowledge_finding(
    jobs: State<Jobs>,
    job_id: String,
    finding_id: String,
) -> Result<JobDetail, String> {
    jobs.with_store(|s| s.acknowledge_finding(&job_id, &finding_id))
}

#[tauri::command]
fn render_and_hold(jobs: State<Jobs>, job_id: String) -> Result<JobDetail, String> {
    jobs.with_store(|s| s.render_and_hold(&job_id))
}

#[tauri::command]
fn write_to_file_sink(jobs: State<Jobs>, job_id: String) -> Result<JobDetail, String> {
    jobs.with_store(|s| s.write_to_file_sink(&job_id))
}

#[tauri::command]
fn release_to_printer(jobs: State<Jobs>, job_id: String) -> Result<JobDetail, String> {
    jobs.with_store(|s| s.release_to_printer(&job_id))
}

#[tauri::command]
fn resolve_review(
    jobs: State<Jobs>,
    attempt_id: String,
    outcome: ReviewOutcome,
) -> Result<CapabilityState, String> {
    jobs.with_store(|s| s.resolve_review(&attempt_id, outcome))
}

#[tauri::command]
fn set_white_policy(
    jobs: State<Jobs>,
    job_id: String,
    policy: WhitePolicyKind,
) -> Result<JobDetail, String> {
    jobs.with_store(|s| s.set_white_policy(&job_id, policy))
}

#[tauri::command]
fn set_white_mask(
    jobs: State<Jobs>,
    job_id: String,
    source: MaskSource,
    invert: bool,
) -> Result<JobDetail, String> {
    jobs.with_store(|s| s.set_white_mask(&job_id, source, invert))
}

#[tauri::command]
fn set_white_choke(jobs: State<Jobs>, job_id: String, choke_mm: f64) -> Result<JobDetail, String> {
    jobs.with_store(|s| s.set_white_choke(&job_id, choke_mm))
}

/// Workspace folder of real jobs, from `VEYLORA_WORKSPACE`. Without it, only fixture data exists
/// and the retained-artifact command says so instead of inventing values.
struct LiveWorkspace(Option<std::path::PathBuf>);

impl LiveWorkspace {
    fn open(&self) -> Result<veylora_workflow::Workspace, String> {
        let root = self.0.as_ref().ok_or(
            "No workspace is configured (set VEYLORA_WORKSPACE); only fixture jobs exist.",
        )?;
        veylora_workflow::Workspace::open(root).map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn live_list_jobs(live: State<LiveWorkspace>) -> Result<Vec<JobSummary>, String> {
    veylora_app_lib::live::job_summaries(&live.open()?)
}

#[tauri::command]
fn live_get_job(live: State<LiveWorkspace>, job_id: String) -> Result<JobDetail, String> {
    veylora_app_lib::live::job_detail(&live.open()?, &job_id)
}

#[tauri::command]
fn get_retained_inspect_view(
    live: State<LiveWorkspace>,
    job_id: String,
    selection: Option<InspectSelection>,
) -> Result<InspectView, String> {
    let retained = retained_for_job(&live.open()?, &job_id)?
        .ok_or("This job has no completed render. Render it to inspect its retained output.")?;
    inspect::present_retained(
        &retained,
        &selection.unwrap_or_else(inspect::default_selection),
    )
}

fn main() {
    tauri::Builder::default()
        .manage(Jobs(Mutex::new(JobStore::fixture())))
        .manage(LiveWorkspace(
            std::env::var_os("VEYLORA_WORKSPACE").map(std::path::PathBuf::from),
        ))
        .invoke_handler(tauri::generate_handler![
            get_capabilities,
            list_jobs,
            get_job,
            get_preflight_view,
            get_job_actions,
            get_inspect_view,
            get_retained_inspect_view,
            live_list_jobs,
            live_get_job,
            toggle_inspect_plane,
            set_inspect_composite,
            get_review_view,
            acknowledge_finding,
            render_and_hold,
            write_to_file_sink,
            release_to_printer,
            resolve_review,
            set_white_policy,
            set_white_mask,
            set_white_choke,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Veylora RIP");
}
