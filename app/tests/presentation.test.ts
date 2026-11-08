import { describe, expect, it } from "vite-plus/test";
import { createMockServices } from "../src/lib/services/mock";

describe("fallback presentation views", () => {
  it("gives every blocking finding a visible correction", async () => {
    const services = createMockServices("working-day");
    let blocking = 0;
    for (const job of await services.listJobs()) {
      const view = await services.getPreflightView(job.id);
      for (const finding of view.groups.flatMap((g) => g.findings)) {
        if (finding.severity === "blocking") {
          blocking += 1;
          expect(finding.correction.trim()).not.toBe("");
        }
      }
    }
    expect(blocking).toBeGreaterThan(0);
  });

  it("never labels an inspect preview as final or physical output", async () => {
    const services = createMockServices("working-day");
    for (const stage of ["logical", "slots", "screened"] as const) {
      const view = await services.getInspectView("J-0143", {
        stage,
        planes: ["W"],
        composite: false,
      });
      expect(view.chip).toMatch(/simulation/);
      expect(view.chip).toMatch(/Fixture data/);
      expect(view.chip).toMatch(/Not physical output$/);
      expect(view.rasterShown).toBe(false);
    }
  });

  it("keeps the stage when a plane is toggled, shows one plane, and refuses unrendered jobs", async () => {
    const services = createMockServices("working-day");
    const view = await services.toggleInspectPlane(
      "J-0143",
      { stage: "slots", planes: ["K"], composite: false },
      "C",
      true,
    );
    expect(view.selection).toEqual({ stage: "slots", planes: ["C"], composite: false });
    await expect(
      services.getInspectView("J-0145", { stage: "logical", planes: [], composite: false }),
    ).rejects.toThrow(/Render the job/);
  });

  it("opens inspect on the default selection chosen by the presenter", async () => {
    const services = createMockServices("working-day");
    const view = await services.getInspectView("J-0143");
    expect(view.selection).toEqual({ stage: "logical", planes: ["K"], composite: false });
  });

  it("never gives a blocking fixture finding the generic fallback correction", async () => {
    const services = createMockServices("working-day");
    for (const job of await services.listJobs()) {
      const view = await services.getPreflightView(job.id);
      for (const finding of view.groups.flatMap((g) => g.findings)) {
        if (finding.severity === "blocking") {
          expect(finding.correction).not.toMatch(/no automatic fix/);
        }
      }
    }
  });
});
