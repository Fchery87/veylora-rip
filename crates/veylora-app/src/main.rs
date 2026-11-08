//! Veylora RIP desktop shell (Tauri 2).
//!
//! Thin adapter: it exposes typed commands that return read models. All
//! authoritative behavior stays in Rust application services (ADR 0001).
//! Presentation lives in `veylora_app_lib::screens`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use tauri::State;
use veylora_app_lib::jobs::JobStore;
use veylora_app_lib::screens::{inspect, job_actions, preflight, review};
use veylora_app_lib::state::CapabilityState;
use veylora_app_lib::view_models::{
    InspectSelection, InspectView, JobActionsView, JobDetail, JobSummary, Plane, PreflightView,
    ReviewOutcome, ReviewView,
};

/// Fixture job state shared by the commands until application services exist.
struct Jobs(Mutex<JobStore>);

impl Jobs {
    fn with<T>(&self, f: impl FnOnce(&mut JobStore) -> Result<T, String>) -> Result<T, String> {
        let mut store = self
            .0
            .lock()
            .map_err(|_| "Job store is unavailable.".to_owned())?;
        f(&mut store)
    }
}

#[tauri::command]
fn get_capabilities(jobs: State<Jobs>) -> Result<CapabilityState, String> {
    jobs.with(|s| Ok(s.capabilities()))
}

#[tauri::command]
fn list_jobs(jobs: State<Jobs>) -> Result<Vec<JobSummary>, String> {
    jobs.with(|s| Ok(s.list()))
}

#[tauri::command]
fn get_job(jobs: State<Jobs>, job_id: String) -> Result<JobDetail, String> {
    jobs.with(|s| s.get(&job_id))
}

#[tauri::command]
fn get_preflight_view(jobs: State<Jobs>, job_id: String) -> Result<PreflightView, String> {
    jobs.with(|s| {
        let caps = s.capabilities();
        Ok(preflight::present(
            &s.get(&job_id)?,
            &caps.setup.name,
            &caps.mode.id,
        ))
    })
}

#[tauri::command]
fn get_job_actions(jobs: State<Jobs>, job_id: String) -> Result<JobActionsView, String> {
    jobs.with(|s| Ok(job_actions::present(&s.capabilities(), &s.get(&job_id)?)))
}

#[tauri::command]
fn get_inspect_view(
    jobs: State<Jobs>,
    job_id: String,
    selection: InspectSelection,
) -> Result<InspectView, String> {
    jobs.with(|s| inspect::present(&s.get(&job_id)?, &s.capabilities(), &selection))
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
    jobs.with(|s| inspect::present(&s.get(&job_id)?, &s.capabilities(), &next))
}

#[tauri::command]
fn get_review_view(jobs: State<Jobs>, job_id: String) -> Result<Option<ReviewView>, String> {
    jobs.with(|s| Ok(review::present(&s.get(&job_id)?)))
}

#[tauri::command]
fn acknowledge_finding(
    jobs: State<Jobs>,
    job_id: String,
    finding_id: String,
) -> Result<JobDetail, String> {
    jobs.with(|s| s.acknowledge_finding(&job_id, &finding_id))
}

#[tauri::command]
fn render_and_hold(jobs: State<Jobs>, job_id: String) -> Result<JobDetail, String> {
    jobs.with(|s| s.render_and_hold(&job_id))
}

#[tauri::command]
fn write_to_file_sink(jobs: State<Jobs>, job_id: String) -> Result<JobDetail, String> {
    jobs.with(|s| s.write_to_file_sink(&job_id))
}

#[tauri::command]
fn release_to_printer(jobs: State<Jobs>, job_id: String) -> Result<JobDetail, String> {
    jobs.with(|s| s.release_to_printer(&job_id))
}

#[tauri::command]
fn resolve_review(
    jobs: State<Jobs>,
    attempt_id: String,
    outcome: ReviewOutcome,
) -> Result<CapabilityState, String> {
    jobs.with(|s| s.resolve_review(&attempt_id, outcome))
}

#[tauri::command]
fn set_white_policy(
    jobs: State<Jobs>,
    job_id: String,
    policy: String,
) -> Result<JobDetail, String> {
    jobs.with(|s| s.set_white_policy(&job_id, &policy))
}

#[tauri::command]
fn set_white_mask(
    jobs: State<Jobs>,
    job_id: String,
    source: String,
    invert: bool,
) -> Result<JobDetail, String> {
    jobs.with(|s| s.set_white_mask(&job_id, &source, invert))
}

#[tauri::command]
fn set_white_choke(jobs: State<Jobs>, job_id: String, choke_mm: f64) -> Result<JobDetail, String> {
    jobs.with(|s| s.set_white_choke(&job_id, choke_mm))
}

fn main() {
    tauri::Builder::default()
        .manage(Jobs(Mutex::new(JobStore::fixture())))
        .invoke_handler(tauri::generate_handler![
            get_capabilities,
            list_jobs,
            get_job,
            get_preflight_view,
            get_job_actions,
            get_inspect_view,
            toggle_inspect_plane,
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
