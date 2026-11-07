import { describe, expect, it } from "vite-plus/test";
import { deriveJobCommands, deriveNavigation } from "../src/lib/capability";
import type { CapabilityState, JobDetail } from "../src/lib/services/types";

function caps(over: Partial<CapabilityState> = {}): CapabilityState {
  return {
    dataSource: "fixture",
    setup: { exists: true, verified: false, experimental: false, name: "Home bench rev 3" },
    mode: {
      id: "PM-exp-01",
      maturity: "experimental",
      sheetSizes: ["A3"],
      whiteChoke: false,
      screenedInspect: true,
    },
    sinks: { physicalPrinter: false, fileSink: true },
    openReview: null,
    ...over,
  };
}

function job(over: Partial<JobDetail> = {}): JobDetail {
  return {
    id: "J-1",
    name: "Test job",
    state: "draft",
    sheet: "A3",
    placementCount: 1,
    updated: "now",
    placements: [],
    findings: [],
    attempts: [],
    provenance: [],
    whitePolicy: { source: "Auto", policy: "Underbase", chokeMm: 0 },
    rendered: false,
    ...over,
  };
}

const finding = (severity: "blocking" | "confirm", acknowledged = false) => ({
  id: "f",
  severity,
  title: "t",
  cause: "c",
  fix: "f",
  acknowledged,
});

describe("deriveNavigation", () => {
  it("disables setup-dependent sections with a reason when no setup exists", () => {
    const nav = deriveNavigation(
      caps({ setup: { exists: false, verified: false, experimental: false, name: "" } }),
    );
    const targets = nav.find((i) => i.id === "targets")!;
    expect(targets.enabled).toBe(false);
    expect(targets.reason).toMatch(/create a device setup/i);
    expect(nav.find((i) => i.id === "jobs")!.enabled).toBe(true);
    expect(nav.find((i) => i.id === "devices")!.enabled).toBe(true);
  });

  it("badges Devices while unverified and Jobs while a review is open", () => {
    const nav = deriveNavigation(
      caps({ openReview: { jobId: "J-1", attemptId: "A-1", jobName: "x" } }),
    );
    expect(nav.find((i) => i.id === "devices")!.badge).toBe("warn");
    expect(nav.find((i) => i.id === "jobs")!.badge).toBe("review");
  });
});

describe("deriveJobCommands", () => {
  it("enables Render and hold for a clean draft", () => {
    expect(deriveJobCommands(caps(), job()).renderAndHold.enabled).toBe(true);
  });

  it("disables Render and hold with a count while blocking findings exist", () => {
    const c = deriveJobCommands(
      caps(),
      job({ findings: [finding("blocking"), finding("blocking")] }),
    );
    expect(c.renderAndHold.enabled).toBe(false);
    expect(c.renderAndHold.reason).toBe("Fix 2 blocking findings to render");
  });

  it("requires confirm-level findings to be acknowledged before rendering", () => {
    const c = deriveJobCommands(caps(), job({ findings: [finding("confirm")] }));
    expect(c.renderAndHold.enabled).toBe(false);
    expect(c.renderAndHold.reason).toBe("Confirm 1 finding to render");
    const ok = deriveJobCommands(caps(), job({ findings: [finding("confirm", true)] }));
    expect(ok.renderAndHold.enabled).toBe(true);
  });

  it("never enables Release to printer while physical submission is gated", () => {
    const c = deriveJobCommands(caps(), job({ state: "held", rendered: true }));
    expect(c.releaseToPrinter.enabled).toBe(false);
    expect(c.releaseToPrinter.reason).toMatch(/physical.*gate/i);
  });

  it("names the blocking review when release would otherwise be possible", () => {
    const c = deriveJobCommands(
      caps({
        sinks: { physicalPrinter: true, fileSink: true },
        setup: { exists: true, verified: true, experimental: false, name: "s" },
        openReview: { jobId: "J-0142", attemptId: "A-1", jobName: "North Ridge tee run" },
      }),
      job({ state: "held", rendered: true }),
    );
    expect(c.releaseToPrinter.enabled).toBe(false);
    expect(c.releaseToPrinter.reason).toBe("Waiting: printer blocked by review of J-0142");
  });

  it("requires a verified setup or experimental mode to release", () => {
    const c = deriveJobCommands(
      caps({ sinks: { physicalPrinter: true, fileSink: true } }),
      job({ state: "held", rendered: true }),
    );
    expect(c.releaseToPrinter.reason).toBe(
      "Verify the setup, or turn on experimental mode in Devices",
    );
  });

  it("allows the file sink for a held job even while a printer review is open", () => {
    const c = deriveJobCommands(
      caps({ openReview: { jobId: "J-0142", attemptId: "A-1", jobName: "x" } }),
      job({ state: "held", rendered: true }),
    );
    expect(c.writeToFileSink.enabled).toBe(true);
  });

  it("disables the file sink before the job has rendered", () => {
    const c = deriveJobCommands(caps(), job());
    expect(c.writeToFileSink.enabled).toBe(false);
    expect(c.writeToFileSink.reason).toMatch(/render/i);
  });
});
