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

/// Acknowledgements made this session, kept in Rust so the UI holds no decision state.
struct Acks(veylora_app_lib::live::Acknowledgements);

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
fn live_available(live: State<LiveWorkspace>) -> bool {
    live.0.is_some()
}

#[tauri::command]
fn live_get_capabilities(live: State<LiveWorkspace>) -> Result<CapabilityState, String> {
    veylora_app_lib::live::workspace_capabilities(&live.open()?)
}

#[tauri::command]
fn live_get_preflight_view(
    live: State<LiveWorkspace>,
    acks: State<Acks>,
    job_id: String,
) -> Result<PreflightView, String> {
    let ws = live.open()?;
    let job = veylora_app_lib::live::run_preflight(&ws, &job_id, &acks.0.for_job(&job_id))?;
    let caps = veylora_app_lib::live::capabilities(&ws, &job_id)?;
    Ok(preflight::present(&job, &caps))
}

#[tauri::command]
fn live_get_job_actions(
    live: State<LiveWorkspace>,
    acks: State<Acks>,
    job_id: String,
) -> Result<JobActionsView, String> {
    let ws = live.open()?;
    let job = veylora_app_lib::live::run_preflight(&ws, &job_id, &acks.0.for_job(&job_id))?;
    let caps = veylora_app_lib::live::capabilities(&ws, &job_id)?;
    Ok(job_actions::present(&caps, &job))
}

#[tauri::command]
fn live_acknowledge_finding(
    live: State<LiveWorkspace>,
    acks: State<Acks>,
    job_id: String,
    finding_id: String,
) -> Result<JobDetail, String> {
    let ws = live.open()?;
    // Validate against the current preflight before remembering anything.
    veylora_app_lib::live::run_preflight(&ws, &job_id, std::slice::from_ref(&finding_id))?;
    acks.0.add(&job_id, &finding_id)?;
    veylora_app_lib::live::run_preflight(&ws, &job_id, &acks.0.for_job(&job_id))
}

#[tauri::command]
fn live_get_review_view(
    live: State<LiveWorkspace>,
    job_id: String,
) -> Result<Option<ReviewView>, String> {
    veylora_app_lib::live::review_view(&live.open()?, &job_id)
}

#[tauri::command]
fn live_resolve_review(
    live: State<LiveWorkspace>,
    attempt_id: String,
    outcome: ReviewOutcome,
) -> Result<CapabilityState, String> {
    veylora_app_lib::live::resolve_review(&live.open()?, &attempt_id, outcome)
}

#[tauri::command]
fn live_write_to_file_sink(
    live: State<LiveWorkspace>,
    job_id: String,
) -> Result<JobDetail, String> {
    veylora_app_lib::live::write_to_file_sink(&live.open()?, &job_id)
}

#[tauri::command]
fn live_creation_options(
    live: State<LiveWorkspace>,
) -> Result<veylora_app_lib::live::CreationOptions, String> {
    veylora_app_lib::live::creation_options(&live.open()?)
}

#[tauri::command]
fn live_create_job(
    live: State<LiveWorkspace>,
    request: veylora_app_lib::live::NewJobRequest,
) -> Result<JobDetail, String> {
    veylora_app_lib::live::create_job(&live.open()?, &request)
}

#[tauri::command]
fn live_plane_image(
    live: State<LiveWorkspace>,
    job_id: String,
    file: String,
    max_px: usize,
) -> Result<veylora_app_lib::live::PlaneImage, String> {
    veylora_app_lib::live::plane_image(&live.open()?, &job_id, &file, max_px)
}

#[tauri::command]
fn live_plane_window(
    live: State<LiveWorkspace>,
    job_id: String,
    file: String,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> Result<veylora_app_lib::live::PlaneValues, String> {
    veylora_app_lib::live::plane_window(&live.open()?, &job_id, &file, x, y, width, height)
}

#[tauri::command]
fn live_render(
    live: State<LiveWorkspace>,
    acks: State<Acks>,
    job_id: String,
) -> Result<JobDetail, String> {
    let detail =
        veylora_app_lib::live::render_job(&live.open()?, &job_id, &acks.0.for_job(&job_id))?;
    acks.0.clear(&job_id);
    Ok(detail)
}

#[tauri::command]
fn live_toggle_inspect_plane(
    live: State<LiveWorkspace>,
    job_id: String,
    selection: InspectSelection,
    plane: Plane,
    on: bool,
) -> Result<InspectView, String> {
    let next = inspect::toggle_plane(&selection, plane, on);
    live_inspect(&live, &job_id, &next)
}

#[tauri::command]
fn live_set_inspect_composite(
    live: State<LiveWorkspace>,
    job_id: String,
    selection: InspectSelection,
    on: bool,
) -> Result<InspectView, String> {
    let next = inspect::set_composite(&selection, on);
    live_inspect(&live, &job_id, &next)
}

fn live_inspect(
    live: &LiveWorkspace,
    job_id: &str,
    selection: &InspectSelection,
) -> Result<InspectView, String> {
    let retained = retained_for_job(&live.open()?, job_id)?
        .ok_or("This job has no completed render. Render it to inspect its retained output.")?;
    inspect::present_retained(&retained, selection)
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

/// The operating system's file dialog, limited to PNG artwork. `None` means cancelled. The dialog
/// blocks, so this is async (a blocking dialog on the main thread would freeze the window).
#[tauri::command]
async fn live_pick_artwork(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let Some(chosen) = app
        .dialog()
        .file()
        .set_title("Choose artwork")
        .add_filter("PNG image", &["png"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let path = chosen
        .into_path()
        .map_err(|e| format!("The chosen file is not a local path: {e}"))?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Jobs(Mutex::new(JobStore::fixture())))
        .manage(Acks(veylora_app_lib::live::Acknowledgements::default()))
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
            live_available,
            live_get_capabilities,
            live_get_preflight_view,
            live_get_job_actions,
            live_acknowledge_finding,
            live_render,
            live_plane_image,
            live_plane_window,
            live_creation_options,
            live_create_job,
            live_pick_artwork,
            live_write_to_file_sink,
            live_get_review_view,
            live_resolve_review,
            live_toggle_inspect_plane,
            live_set_inspect_composite,
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
