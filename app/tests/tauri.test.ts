import { describe, expect, it } from "vite-plus/test";
import type { InspectSelection } from "../src/lib/services/types";
import { createMockServices } from "../src/lib/services/mock";
import { createTauriServices, isTauriRuntime } from "../src/lib/services/tauri";

type Call = { command: string; args?: Record<string, unknown> };

function connected() {
  const calls: Call[] = [];
  const fallback = createMockServices("working-day");
  const services = createTauriServices(
    async <T>(command: string, args?: Record<string, unknown>) => {
      calls.push({ command, args });
      return { command } as T;
    },
    fallback,
  );
  return { calls, services, fallback };
}

describe("tauri adapter", () => {
  it("routes capabilities, jobs and presentation views to Rust commands", async () => {
    const { calls, services } = connected();
    await services.getCapabilities();
    await services.listJobs();
    await services.getJob("J-1");
    await services.getPreflightView("J-1");
    await services.getJobActions("J-1");
    await services.getReviewView("J-1");
    expect(calls).toEqual([
      { command: "get_capabilities", args: undefined },
      { command: "list_jobs", args: undefined },
      { command: "get_job", args: { jobId: "J-1" } },
      { command: "get_preflight_view", args: { jobId: "J-1" } },
      { command: "get_job_actions", args: { jobId: "J-1" } },
      { command: "get_review_view", args: { jobId: "J-1" } },
    ]);
  });

  it("sends inspect selections and plane toggles to Rust instead of deciding locally", async () => {
    const { calls, services } = connected();
    const selection: InspectSelection = { stage: "logical", planes: ["K"], composite: false };
    await services.getInspectView("J-1", selection);
    await services.toggleInspectPlane("J-1", selection, "C", true);
    await services.setInspectComposite("J-1", selection, true);
    await services.getInspectView("J-1");
    expect(calls).toEqual([
      { command: "get_inspect_view", args: { jobId: "J-1", selection } },
      { command: "toggle_inspect_plane", args: { jobId: "J-1", selection, plane: "C", on: true } },
      { command: "set_inspect_composite", args: { jobId: "J-1", selection, on: true } },
      { command: "get_inspect_view", args: { jobId: "J-1", selection: undefined } },
    ]);
  });

  it("routes job commands, including review resolution and release, to Rust", async () => {
    const { calls, services } = connected();
    await services.acknowledgeFinding("J-1", "f1");
    await services.renderAndHold("J-1");
    await services.writeToFileSink("J-1");
    await services.releaseToPrinter("J-1");
    await services.resolveReview("A-1", "decide-later");
    await services.setWhitePolicy("J-1", "No white");
    await services.setWhiteMask("J-1", { source: "Whole placement", invert: false });
    await services.setWhiteChoke("J-1", 0.5);
    expect(calls.map((c) => c.command)).toEqual([
      "acknowledge_finding",
      "render_and_hold",
      "write_to_file_sink",
      "release_to_printer",
      "resolve_review",
      "set_white_policy",
      "set_white_mask",
      "set_white_choke",
    ]);
    expect(calls[4].args).toEqual({ attemptId: "A-1", outcome: "decide-later" });
    expect(calls[6].args).toEqual({ jobId: "J-1", source: "Whole placement", invert: false });
    expect(calls[7].args).toEqual({ jobId: "J-1", chokeMm: 0.5 });
  });

  it("leaves device setup, recipes and diagnostics on the fallback", async () => {
    const { calls, services } = connected();
    expect((await services.listRecipes()).length).toBeGreaterThan(0);
    expect(calls).toEqual([]);
  });

  it("detects the Tauri runtime only when its internals are present", () => {
    expect(isTauriRuntime({})).toBe(false);
    expect(isTauriRuntime({ __TAURI_INTERNALS__: {} })).toBe(true);
  });
});
