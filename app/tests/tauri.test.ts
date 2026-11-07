import { describe, expect, it } from "vite-plus/test";
import { createMockServices } from "../src/lib/services/mock";
import { createTauriServices, isTauriRuntime } from "../src/lib/services/tauri";

describe("tauri adapter", () => {
  it("reads capabilities from the get_capabilities command and delegates the rest to the fallback", async () => {
    const calls: string[] = [];
    const fallback = createMockServices("working-day");
    const caps = await fallback.getCapabilities();
    const services = createTauriServices(async <T>(command: string) => {
      calls.push(command);
      return { ...caps, dataSource: "live" } as T;
    }, fallback);
    expect((await services.getCapabilities()).dataSource).toBe("live");
    expect(calls).toEqual(["get_capabilities"]);
    expect((await services.listJobs()).length).toBeGreaterThan(0);
    expect(calls).toEqual(["get_capabilities"]);
  });

  it("detects the Tauri runtime only when its internals are present", () => {
    expect(isTauriRuntime({})).toBe(false);
    expect(isTauriRuntime({ __TAURI_INTERNALS__: {} })).toBe(true);
  });
});
