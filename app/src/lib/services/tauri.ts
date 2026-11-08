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
export function createTauriServices(invoke: Invoke, fallback: AppServices): AppServices {
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
