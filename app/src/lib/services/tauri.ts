import type { AppServices, CapabilityState } from "./types";

export type Invoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

/** True inside the Tauri webview. Browsers and tests never have the internals object. */
export function isTauriRuntime(host: object = globalThis): boolean {
  return "__TAURI_INTERNALS__" in host;
}

/**
 * Thin Tauri adapter. Capabilities, jobs, the preflight/inspect/review/action
 * view models and the job commands go to Rust. Device setup, recipes, targets,
 * artwork and diagnostics stay on the fallback (mocked fixture data) until the
 * matching Rust commands exist. It owns no behavior of its own (ADR 0001).
 */
export function createTauriServices(
  invoke: Invoke,
  fallback: AppServices,
  live = false,
): AppServices {
  if (live) return createLiveServices(invoke, fallback);
  return {
    ...fallback,
    getCapabilities: () => invoke<CapabilityState>("get_capabilities"),
    listJobs: () => invoke("list_jobs"),
    getJob: (jobId) => invoke("get_job", { jobId }),
    getPreflightView: (jobId) => invoke("get_preflight_view", { jobId }),
    getJobActions: (jobId) => invoke("get_job_actions", { jobId }),
    getInspectView: (jobId, selection) => invoke("get_inspect_view", { jobId, selection }),
    toggleInspectPlane: (jobId, selection, plane, on) =>
      invoke("toggle_inspect_plane", { jobId, selection, plane, on }),
    setInspectComposite: (jobId, selection, on) =>
      invoke("set_inspect_composite", { jobId, selection, on }),
    getReviewView: (jobId) => invoke("get_review_view", { jobId }),
    acknowledgeFinding: (jobId, findingId) => invoke("acknowledge_finding", { jobId, findingId }),
    renderAndHold: (jobId) => invoke("render_and_hold", { jobId }),
    writeToFileSink: (jobId) => invoke("write_to_file_sink", { jobId }),
    releaseToPrinter: (jobId) => invoke("release_to_printer", { jobId }),
    resolveReview: (attemptId, outcome) => invoke("resolve_review", { attemptId, outcome }),
    setWhitePolicy: (jobId, policy) => invoke("set_white_policy", { jobId, policy }),
    setWhiteMask: (jobId, mask) =>
      invoke("set_white_mask", { jobId, source: mask.source, invert: mask.invert }),
    setWhiteChoke: (jobId, chokeMm) => invoke("set_white_choke", { jobId, chokeMm }),
  };
}

const NOT_LIVE = (what: string) =>
  Promise.reject(new Error(`${what} is not available for live jobs yet.`));

/**
 * Real jobs from the workspace named by VEYLORA_WORKSPACE. Every job view comes from a live Rust
 * command; there is no fallback to fixture jobs, so what is shown is what is stored. Device setup,
 * recipes, targets, artwork and diagnostics still come from the fallback (they are not job data).
 */
function createLiveServices(invoke: Invoke, fallback: AppServices): AppServices {
  return {
    ...fallback,
    getCapabilities: () => invoke<CapabilityState>("live_get_capabilities"),
    listJobs: () => invoke("live_list_jobs"),
    getJob: (jobId) => invoke("live_get_job", { jobId }),
    getPreflightView: (jobId) => invoke("live_get_preflight_view", { jobId }),
    getJobActions: (jobId) => invoke("live_get_job_actions", { jobId }),
    getInspectView: (jobId, selection) => invoke("get_retained_inspect_view", { jobId, selection }),
    toggleInspectPlane: (jobId, selection, plane, on) =>
      invoke("live_toggle_inspect_plane", { jobId, selection, plane, on }),
    setInspectComposite: (jobId, selection, on) =>
      invoke("live_set_inspect_composite", { jobId, selection, on }),
    // Attempt outcomes are not itemized for live jobs yet, so no review is offered.
    getReviewView: async () => null,
    acknowledgeFinding: (jobId, findingId) =>
      invoke("live_acknowledge_finding", { jobId, findingId }),
    renderAndHold: (jobId) => invoke("live_render", { jobId }),
    writeToFileSink: () => NOT_LIVE("Writing to the file sink"),
    releaseToPrinter: () => NOT_LIVE("Release to the printer"),
    resolveReview: () => NOT_LIVE("Resolving a review"),
    setWhitePolicy: () => NOT_LIVE("Changing the white policy"),
    setWhiteMask: () => NOT_LIVE("Changing the white mask"),
    setWhiteChoke: () => NOT_LIVE("Changing the white choke"),
  };
}
