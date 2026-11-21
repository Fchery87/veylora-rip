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

  it("offers no job creation on fixture jobs", async () => {
    const { calls, services } = connected();
    expect(await services.getCreationOptions()).toBeNull();
    await expect(services.createJob({} as never)).rejects.toThrow(/live workspace/);
    expect(calls).toEqual([]);
  });

  it("leaves device setup, recipes and diagnostics on the fallback", async () => {
    const { calls, services } = connected();
    expect((await services.listRecipes()).length).toBeGreaterThan(0);
    expect(calls).toEqual([]);
  });

  describe("with a live workspace", () => {
    function live() {
      const calls: Call[] = [];
      const services = createTauriServices(
        async <T>(command: string, args?: Record<string, unknown>) => {
          calls.push({ command, args });
          return { command } as T;
        },
        createMockServices("working-day"),
        true,
      );
      return { calls, services };
    }

    it("routes jobs, views and actions to the live commands", async () => {
      const { calls, services } = live();
      await services.getCapabilities();
      await services.listJobs();
      await services.getJob("J-1");
      await services.getPreflightView("J-1");
      await services.getJobActions("J-1");
      await services.acknowledgeFinding("J-1", "f1");
      await services.renderAndHold("J-1");
      await services.writeToFileSink("J-1");
      expect(calls).toEqual([
        { command: "live_get_capabilities", args: undefined },
        { command: "live_list_jobs", args: undefined },
        { command: "live_get_job", args: { jobId: "J-1" } },
        { command: "live_get_preflight_view", args: { jobId: "J-1" } },
        { command: "live_get_job_actions", args: { jobId: "J-1" } },
        { command: "live_acknowledge_finding", args: { jobId: "J-1", findingId: "f1" } },
        { command: "live_render", args: { jobId: "J-1" } },
        { command: "live_write_to_file_sink", args: { jobId: "J-1" } },
      ]);
    });

    it("lists creation choices and creates jobs through Rust", async () => {
      const { calls, services } = live();
      const request = {
        setup: "s",
        mode: "m",
        recipe: "r",
        pageWidthMm: 1,
        pageHeightMm: 1,
        placements: [
          {
            artworkPath: "a.png",
            xMm: 0,
            yMm: 0,
            widthMm: 1,
            heightMm: 1,
            rotationDeg: 0 as const,
          },
        ],
        mirror: false,
      };
      await services.getCreationOptions();
      await services.createJob(request);
      expect(calls).toEqual([
        { command: "live_creation_options", args: undefined },
        { command: "live_create_job", args: { request } },
      ]);
    });

    it("opens the native picker only on a live workspace", async () => {
      const { calls, services } = live();
      await services.pickArtwork?.();
      expect(calls).toEqual([{ command: "live_pick_artwork", args: undefined }]);
    });

    it("draws retained planes through Rust", async () => {
      const { calls, services } = live();
      await services.getPlaneImage("J-1", "logical-w.f32", 512);
      await services.getPlaneWindow("J-1", "logical-w.f32", 1, 2, 7, 7);
      expect(calls).toEqual([
        { command: "live_plane_image", args: { jobId: "J-1", file: "logical-w.f32", maxPx: 512 } },
        {
          command: "live_plane_window",
          args: { jobId: "J-1", file: "logical-w.f32", x: 1, y: 2, width: 7, height: 7 },
        },
      ]);
    });

    it("inspects retained artifacts, never the fixture inspector", async () => {
      const { calls, services } = live();
      const selection: InspectSelection = { stage: "logical", planes: ["K"], composite: false };
      await services.getInspectView("J-1", selection);
      await services.toggleInspectPlane("J-1", selection, "C", true);
      await services.setInspectComposite("J-1", selection, true);
      expect(calls.map((c) => c.command)).toEqual([
        "get_retained_inspect_view",
        "live_toggle_inspect_plane",
        "live_set_inspect_composite",
      ]);
    });

    it("reads review and records its resolution through Rust", async () => {
      const { calls, services } = live();
      await services.getReviewView("J-1");
      await services.resolveReview("A-1", "finished");
      expect(calls).toEqual([
        { command: "live_get_review_view", args: { jobId: "J-1" } },
        { command: "live_resolve_review", args: { attemptId: "A-1", outcome: "finished" } },
      ]);
    });

    it("rejects what the live backend cannot do instead of touching fixture state", async () => {
      const { calls, services } = live();
      for (const attempt of [
        () => services.releaseToPrinter("J-1"),
        () => services.setWhitePolicy("J-1", "No white"),
        () => services.setWhiteMask("J-1", { source: "Whole placement", invert: false }),
        () => services.setWhiteChoke("J-1", 0.5),
      ]) {
        await expect(attempt()).rejects.toThrow(/not available for live jobs/);
      }
      expect(calls).toEqual([]);
    });
  });

  it("detects the Tauri runtime only when its internals are present", () => {
    expect(isTauriRuntime({})).toBe(false);
    expect(isTauriRuntime({ __TAURI_INTERNALS__: {} })).toBe(true);
  });
});
