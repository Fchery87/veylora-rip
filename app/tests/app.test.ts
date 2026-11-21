import { cleanup, render, screen, waitFor, within } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vite-plus/test";
import App from "../src/App.svelte";
import { createMockServices } from "../src/lib/services/mock";

afterEach(() => {
  cleanup();
  document.documentElement.removeAttribute("data-theme");
});

type User = ReturnType<typeof userEvent.setup>;

async function openApp(scenario: "working-day" | "fresh-install" = "working-day") {
  const user = userEvent.setup();
  const services = createMockServices(scenario);
  render(App, { services });
  await screen.findByRole("navigation", { name: "Sections" });
  return { user, services };
}

async function openJob(user: User, name: RegExp) {
  await user.click(await screen.findByRole("option", { name }));
  await screen.findByRole("heading", { level: 1, name });
}

describe("shell", () => {
  it("labels mocked services as fixture data in the status bar", async () => {
    await openApp();
    const bar = screen.getByRole("contentinfo", { name: "Device status" });
    expect(within(bar).getByText("Fixture data")).toBeInTheDocument();
    expect(within(bar).getByText(/Unverified/)).toBeInTheDocument();
    expect(within(bar).getByText(/PM-exp-01/)).toBeInTheDocument();
  });

  it("shows a non-dismissable alert banner while an attempt is ambiguous", async () => {
    await openApp();
    const banner = screen.getByRole("alert");
    expect(banner).toHaveTextContent("Printer blocked");
    expect(banner).toHaveTextContent("J-0142");
    expect(within(banner).queryByRole("button", { name: /dismiss|close/i })).toBeNull();
  });

  it("disables setup-dependent navigation with a reason on a fresh install", async () => {
    await openApp("fresh-install");
    const targets = screen.getByRole("button", { name: "Targets" });
    expect(targets).toBeDisabled();
    expect(targets).toHaveAccessibleDescription(/create a device setup/i);
    expect(screen.getByRole("button", { name: "Devices" })).toBeEnabled();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("switches the theme", async () => {
    const { user } = await openApp();
    await user.click(screen.getByRole("button", { name: "Dark theme" }));
    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
  });
});

describe("job commands from capability state", () => {
  it("keeps Render and hold disabled with the reason beside it while blocking findings exist", async () => {
    const { user } = await openApp();
    await openJob(user, /Hoodie sleeve/);
    expect(screen.getByRole("button", { name: "Render and hold" })).toBeDisabled();
    expect(screen.getByText("Fix 1 blocking finding to render")).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: /Preflight/ }));
    expect(screen.getByText(/Move it left by at least 4\.2 mm/)).toBeInTheDocument();
  });

  it("enables Render and hold once a confirm finding is acknowledged", async () => {
    const { user } = await openApp();
    await openJob(user, /Pocket print/);
    expect(screen.getByRole("button", { name: "Render and hold" })).toBeDisabled();
    await user.click(screen.getByRole("tab", { name: /Preflight/ }));
    await user.click(screen.getByRole("button", { name: "Acknowledge finding" }));
    const render = await screen.findByRole("button", { name: "Render and hold" });
    expect(render).toBeEnabled();
    await user.click(render);
    const header = screen.getByRole("group", { name: "Job header" });
    expect(await within(header).findByText("Held")).toBeInTheDocument();
  });

  it("disables Release to printer with the gate reason and never offers physical output", async () => {
    const { user } = await openApp();
    await openJob(user, /Front chest/);
    expect(screen.getByRole("button", { name: "Release to printer" })).toBeDisabled();
    expect(screen.getByText(/physical gate passes/i)).toBeInTheDocument();
  });

  it("writes a held job to the file sink and records a labeled attempt", async () => {
    const { user } = await openApp();
    await openJob(user, /Front chest/);
    await user.click(screen.getByRole("button", { name: "Write to file sink" }));
    await user.click(await screen.findByRole("button", { name: "Confirm write to file sink" }));
    await user.click(await screen.findByRole("tab", { name: /Attempts/ }));
    const attempts = screen.getByRole("list", { name: "Attempt history" });
    expect(within(attempts).getByText(/Not physical output/)).toBeInTheDocument();
    expect(within(attempts).getByText(/\.vrip/)).toBeInTheDocument();
  });
});

describe("creating jobs", () => {
  const options = {
    setups: [{ hash: "a".repeat(64), id: "SETUP-1", verified: false }],
    modes: [
      { hash: "b".repeat(64), id: "MODE-1", setupId: "SETUP-1", maturity: "experimental" as const },
      {
        hash: "c".repeat(64),
        id: "MODE-OTHER",
        setupId: "SETUP-2",
        maturity: "experimental" as const,
      },
    ],
    recipes: [
      { hash: "d".repeat(64), id: "RECIPE-1", revision: 1, retired: false, validated: false },
    ],
  };

  async function openLive(
    createJob: (req: unknown) => Promise<unknown>,
    extra: Record<string, unknown> = {},
  ) {
    const user = userEvent.setup();
    const services = {
      ...createMockServices("working-day"),
      getCreationOptions: async () => options,
      createJob,
      ...extra,
    } as ReturnType<typeof createMockServices>;
    render(App, { services });
    await screen.findByRole("navigation", { name: "Sections" });
    return { user };
  }

  it("offers no New job button on fixture data", async () => {
    await openApp();
    expect(screen.queryByRole("button", { name: "New job" })).not.toBeInTheDocument();
  });

  it("sends the chosen records and sizes to Rust, then opens the new job", async () => {
    const sent: unknown[] = [];
    const created = await createMockServices("working-day").getJob("J-0142");
    const { user } = await openLive(async (req) => {
      sent.push(req);
      return created;
    });
    await user.click(screen.getByRole("button", { name: "New job" }));
    const form = await screen.findByRole("form", { name: "New job" });
    expect(within(form).getByRole("button", { name: "Create job" })).toBeDisabled();
    await user.type(within(form).getByLabelText(/Artwork 1 file/), "C:Artlogo.png");
    // Each select starts on a real choice instead of showing blank.
    expect(within(form).getByLabelText("Print mode")).toHaveValue(options.modes[0].hash);
    expect(within(form).getByLabelText("Recipe")).toHaveValue(options.recipes[0].hash);
    // Modes follow the chosen setup: another setup's mode is not offered.
    expect(within(form).queryByRole("option", { name: /MODE-OTHER/ })).not.toBeInTheDocument();
    await user.click(within(form).getByRole("button", { name: "Create job" }));
    await waitFor(() => expect(sent).toHaveLength(1));
    expect(sent[0]).toMatchObject({
      setup: options.setups[0].hash,
      mode: options.modes[0].hash,
      recipe: options.recipes[0].hash,
      placements: [{ artworkPath: "C:Artlogo.png", rotationDeg: 0 }],
      mirror: false,
    });
    await screen.findByRole("heading", { level: 1, name: created.name });
    expect(screen.queryByRole("form", { name: "New job" })).not.toBeInTheDocument();
  });

  it("fills the artwork path from the native file picker", async () => {
    const { user } = await openLive(async () => ({}), {
      pickArtwork: async () => "C:/Art/logo.png",
    });
    await user.click(screen.getByRole("button", { name: "New job" }));
    const form = await screen.findByRole("form", { name: "New job" });
    await user.click(within(form).getByRole("button", { name: "Browse…" }));
    await waitFor(() =>
      expect(within(form).getByLabelText(/Artwork 1 file/)).toHaveValue("C:/Art/logo.png"),
    );
  });

  it("keeps a typed path when the picker is cancelled", async () => {
    const { user } = await openLive(async () => ({}), { pickArtwork: async () => null });
    await user.click(screen.getByRole("button", { name: "New job" }));
    const form = await screen.findByRole("form", { name: "New job" });
    await user.type(within(form).getByLabelText(/Artwork 1 file/), "typed.png");
    await user.click(within(form).getByRole("button", { name: "Browse…" }));
    expect(within(form).getByLabelText(/Artwork 1 file/)).toHaveValue("typed.png");
  });

  it("offers no Browse button when there is no native picker", async () => {
    const { user } = await openLive(async () => ({}), { pickArtwork: undefined });
    await user.click(screen.getByRole("button", { name: "New job" }));
    const form = await screen.findByRole("form", { name: "New job" });
    expect(within(form).queryByRole("button", { name: "Browse…" })).not.toBeInTheDocument();
  });

  it("places several artworks on one sheet, each with its own position", async () => {
    const sent: { placements: Record<string, unknown>[] }[] = [];
    const created = await createMockServices("working-day").getJob("J-0142");
    const picks = ["C:/Art/one.png", "C:/Art/two.png"];
    const { user } = await openLive(
      async (req) => {
        sent.push(req as never);
        return created;
      },
      { pickArtwork: async () => picks.shift() ?? null },
    );
    await user.click(screen.getByRole("button", { name: "New job" }));
    const form = await screen.findByRole("form", { name: "New job" });
    expect(within(form).queryByRole("button", { name: /Remove artwork/ })).not.toBeInTheDocument();
    await user.click(within(form).getByRole("button", { name: "Add artwork" }));
    const one = within(within(form).getByRole("group", { name: "Artwork 1" }));
    const two = within(within(form).getByRole("group", { name: "Artwork 2" }));
    await user.click(one.getByRole("button", { name: "Browse…" }));
    await user.click(two.getByRole("button", { name: "Browse…" }));
    await waitFor(() => expect(two.getByLabelText(/file/)).toHaveValue("C:/Art/two.png"));
    expect(one.getByLabelText(/file/)).toHaveValue("C:/Art/one.png");
    // The second artwork starts beside the first, not on top of it.
    expect(Number((two.getByLabelText("X (mm)") as HTMLInputElement).value)).toBeGreaterThan(0);
    await user.click(within(form).getByRole("button", { name: "Create job" }));
    await waitFor(() => expect(sent).toHaveLength(1));
    expect(sent[0].placements.map((p) => p.artworkPath)).toEqual([
      "C:/Art/one.png",
      "C:/Art/two.png",
    ]);
  });

  it("removes an artwork and will not create with an empty one", async () => {
    const { user } = await openLive(async () => ({}));
    await user.click(screen.getByRole("button", { name: "New job" }));
    const form = await screen.findByRole("form", { name: "New job" });
    await user.click(within(form).getByRole("button", { name: "Add artwork" }));
    await user.type(within(form).getByLabelText(/Artwork 1 file/), "a.png");
    // Artwork 2 has no file yet, so the job cannot be created.
    expect(within(form).getByRole("button", { name: "Create job" })).toBeDisabled();
    await user.click(within(form).getByRole("button", { name: "Remove artwork 2" }));
    expect(within(form).queryByRole("group", { name: "Artwork 2" })).not.toBeInTheDocument();
    expect(within(form).getByRole("button", { name: "Create job" })).toBeEnabled();
  });

  it("draws the sheet at the job's real shape, not a fixture paper size", async () => {
    const created = {
      ...(await createMockServices("working-day").getJob("J-0142")),
      sheet: "1 x 0.250 mm",
      sheetMm: { w: 1, h: 0.25 },
    };
    const { user } = await openLive(async () => created, { getJob: async () => created });
    await user.click(screen.getByRole("button", { name: "New job" }));
    const form = await screen.findByRole("form", { name: "New job" });
    await user.type(within(form).getByLabelText(/Artwork 1 file/), "a.png");
    await user.click(within(form).getByRole("button", { name: "Create job" }));
    await user.click(await screen.findByRole("tab", { name: /Sheet/ }));
    const sheet = await screen.findByRole("img", { name: /Simulated 1 x 0.250 mm sheet/ });
    expect(sheet.style.aspectRatio).toBe("1 / 0.25");
  });

  it("keeps the form open and shows the reason when Rust refuses", async () => {
    const { user } = await openLive(async () => {
      throw new Error("Cannot read 'x.png': not found");
    });
    await user.click(screen.getByRole("button", { name: "New job" }));
    const form = await screen.findByRole("form", { name: "New job" });
    await user.type(within(form).getByLabelText(/Artwork 1 file/), "x.png");
    await user.click(within(form).getByRole("button", { name: "Create job" }));
    expect(await within(form).findByRole("alert")).toHaveTextContent("Cannot read 'x.png'");
    expect(screen.getByRole("form", { name: "New job" })).toBeInTheDocument();
  });

  it("goes back to the job view on Cancel without creating anything", async () => {
    const sent: unknown[] = [];
    const { user } = await openLive(async (req) => {
      sent.push(req);
      throw new Error("no");
    });
    await user.click(screen.getByRole("button", { name: "New job" }));
    await user.click(
      within(await screen.findByRole("form", { name: "New job" })).getByRole("button", {
        name: "Cancel",
      }),
    );
    expect(screen.queryByRole("form", { name: "New job" })).not.toBeInTheDocument();
    expect(sent).toEqual([]);
  });
});

describe("retained raster", () => {
  async function openRetained(rasterFiles = ["logical-w.f32"]) {
    const user = userEvent.setup();
    const base = createMockServices("working-day");
    const calls: string[] = [];
    const services = {
      ...base,
      getInspectView: async (id: string, selection?: unknown) => ({
        ...(await base.getInspectView(id, selection as never)),
        source: "retained" as const,
        rasterShown: true,
        rasterFiles,
        panels: [],
        placeholder: "Drawn from the retained bytes after checking them against the manifest.",
      }),
      getPlaneImage: async (_j: string, file: string) => {
        calls.push(`image ${file}`);
        return {
          dataUrl: "data:image/png;base64,AAAA",
          width: 4,
          height: 1,
          scale: 2,
          sourceWidth: 8,
          sourceHeight: 2,
        };
      },
      getPlaneWindow: async (_j: string, file: string, x: number, y: number) => {
        calls.push(`window ${file} ${x},${y}`);
        return { x, y, width: 2, height: 1, values: [0.25, 1] };
      },
    } as ReturnType<typeof createMockServices>;
    render(App, { services });
    await screen.findByRole("navigation", { name: "Sections" });
    await openJob(user, /Front chest/);
    await user.click(screen.getByRole("tab", { name: "Inspect" }));
    return { user, calls };
  }

  it("draws the retained plane with its reduction and no placeholder stubs", async () => {
    const { calls } = await openRetained();
    const img = await screen.findByRole("img", { name: "Retained plane logical-w.f32" });
    expect(img).toHaveAttribute("src", "data:image/png;base64,AAAA");
    expect(screen.getByText(/shown reduced 2:1/)).toBeInTheDocument();
    expect(calls).toEqual(["image logical-w.f32"]);
    expect(screen.queryByText("Available when a raster exists")).not.toBeInTheDocument();
  });

  it("reads exact values for the clicked pixel into the loupe", async () => {
    const { user, calls } = await openRetained();
    await screen.findByRole("img", { name: "Retained plane logical-w.f32" });
    await user.click(screen.getByRole("button", { name: "Inspect values in logical-w.f32" }));
    const loupe = await screen.findByRole("region", { name: "Loupe" });
    expect(within(loupe).getByText("0.250")).toBeInTheDocument();
    expect(within(loupe).getByText("1.000")).toBeInTheDocument();
    expect(calls.some((c) => c.startsWith("window logical-w.f32"))).toBe(true);
  });
});

describe("retained composite", () => {
  it("draws the composite with its caption and offers no loupe, since it is not one plane", async () => {
    const user = userEvent.setup();
    const base = createMockServices("working-day");
    const calls: string[] = [];
    const services = {
      ...base,
      getInspectView: async (id: string, selection?: unknown) => ({
        ...(await base.getInspectView(id, selection as never)),
        source: "retained" as const,
        rasterShown: true,
        rasterFiles: ["composite"],
        panels: [],
        placeholder: "Composite of the retained logical planes. Not color managed.",
      }),
      getPlaneImage: async (_j: string, file: string) => {
        calls.push(`image ${file}`);
        return {
          dataUrl: "data:image/png;base64,AAAA",
          width: 4,
          height: 1,
          scale: 1,
          sourceWidth: 4,
          sourceHeight: 1,
        };
      },
    } as ReturnType<typeof createMockServices>;
    render(App, { services });
    await screen.findByRole("navigation", { name: "Sections" });
    await openJob(user, /Front chest/);
    await user.click(screen.getByRole("tab", { name: "Inspect" }));
    const img = await screen.findByRole("img", {
      name: "Composite simulation of the retained logical planes",
    });
    expect(img).toHaveAttribute("src", "data:image/png;base64,AAAA");
    expect(calls).toEqual(["image composite"]);
    expect(screen.getByText(/Not color managed/)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Inspect values in/ })).not.toBeInTheDocument();
  });
});

describe("white edits that make a new job", () => {
  it("opens the new job when an edit returns a different one", async () => {
    const user = userEvent.setup();
    const base = createMockServices("working-day");
    const jobs = await base.listJobs();
    const other = jobs.find((j) => /Front chest/.test(j.name))!;
    const calls: string[] = [];
    const services = {
      ...base,
      getCapabilities: async () => {
        const caps = await base.getCapabilities();
        return { ...caps, mode: { ...caps.mode, whiteChoke: true } };
      },
      setWhiteChoke: async (id: string, mm: number) => {
        calls.push(`choke ${id} ${mm}`);
        return base.getJob(other.id);
      },
    } as ReturnType<typeof createMockServices>;
    render(App, { services });
    await screen.findByRole("navigation", { name: "Sections" });
    await openJob(user, /Pocket print/);
    await user.click(screen.getByRole("tab", { name: "Sheet" }));
    const input = await screen.findByLabelText("White choke (mm)");
    await user.clear(input);
    await user.type(input, "0.3");
    await user.tab();
    expect(calls).toHaveLength(1);
    expect(await screen.findByRole("heading", { level: 1, name: other.name })).toBeInTheDocument();
    expect(screen.getByText(/New job created with the changed white settings/)).toBeInTheDocument();
  });
});

describe("refused white settings", () => {
  it("puts a dropdown back on what the job has when the change is refused", async () => {
    const user = userEvent.setup();
    const base = createMockServices("working-day");
    const services = {
      ...base,
      setWhiteMask: async () => {
        throw new Error("A whole placement mask is not something a recipe can state yet.");
      },
    } as ReturnType<typeof createMockServices>;
    render(App, { services });
    await screen.findByRole("navigation", { name: "Sections" });
    await openJob(user, /Pocket print/);
    await user.click(screen.getByRole("tab", { name: "Sheet" }));
    const select = (await screen.findByLabelText("Mask source")) as HTMLSelectElement;
    const stored = select.value;
    await user.selectOptions(select, "Whole placement");
    expect(await screen.findByText(/not something a recipe can state/)).toBeInTheDocument();
    expect(select.value).toBe(stored);
  });
});

describe("inspect loading", () => {
  async function openInspect(overrides: Record<string, unknown>) {
    const user = userEvent.setup();
    const base = createMockServices("working-day");
    render(App, { services: { ...base, ...overrides } as ReturnType<typeof createMockServices> });
    await screen.findByRole("navigation", { name: "Sections" });
    await openJob(user, /Front chest/);
    await user.click(screen.getByRole("tab", { name: "Inspect" }));
    await screen.findByRole("group", { name: "Planes" });
    return { user, base };
  }

  it("says it is loading while a view is pending instead of claiming nothing is drawn", async () => {
    let release: (v: unknown) => void = () => {};
    const base = createMockServices("working-day");
    const { user } = await openInspect({
      setInspectComposite: (id: string, sel: never, on: boolean) =>
        new Promise((resolve) => {
          release = () => resolve(base.setInspectComposite(id, sel, on));
        }),
    });
    await user.click(screen.getByRole("button", { name: "Composite" }));
    expect(await screen.findByRole("status", { name: "Loading inspect view" })).toBeInTheDocument();
    expect(screen.queryByText(/Nothing is drawn/)).not.toBeInTheDocument();
    release(undefined);
    await waitFor(() =>
      expect(
        screen.queryByRole("status", { name: "Loading inspect view" }),
      ).not.toBeInTheDocument(),
    );
  });

  it("says it is loading on the first open too, before any view exists", async () => {
    const user = userEvent.setup();
    const base = createMockServices("working-day");
    let release: () => void = () => {};
    render(App, {
      services: {
        ...base,
        getInspectView: (id: string, sel?: never) =>
          new Promise((resolve) => {
            release = () => resolve(base.getInspectView(id, sel));
          }),
      } as ReturnType<typeof createMockServices>,
    });
    await screen.findByRole("navigation", { name: "Sections" });
    await openJob(user, /Front chest/);
    await user.click(screen.getByRole("tab", { name: "Inspect" }));
    expect(await screen.findByRole("status", { name: "Loading inspect view" })).toBeInTheDocument();
    release();
    await screen.findByRole("group", { name: "Planes" });
  });

  it("keeps the newest answer when an older request finishes last", async () => {
    const base = createMockServices("working-day");
    const finish: Array<() => void> = [];
    const { user } = await openInspect({
      toggleInspectPlane: (id: string, sel: never, plane: never, on: boolean) =>
        new Promise((resolve) => {
          finish.push(() => resolve(base.toggleInspectPlane(id, sel, plane, on)));
        }),
    });
    await user.click(screen.getByRole("button", { name: "Plane C" }));
    await user.click(screen.getByRole("button", { name: "Plane M" }));
    finish[1]();
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Plane M" })).toHaveAttribute(
        "aria-pressed",
        "true",
      ),
    );
    finish[0]();
    await new Promise((r) => setTimeout(r, 20));
    expect(screen.getByRole("button", { name: "Plane M" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByRole("button", { name: "Plane C" })).toHaveAttribute(
      "aria-pressed",
      "false",
    );
  });
});

describe("composite and stages", () => {
  it("leaves the composite when another stage is chosen, since it is built from logical planes", async () => {
    const user = userEvent.setup();
    const base = createMockServices("working-day");
    const selections: { composite?: boolean; stage?: string }[] = [];
    const services = {
      ...base,
      getInspectView: async (id: string, selection?: { composite?: boolean; stage?: string }) => {
        if (selection) selections.push(selection);
        return {
          ...(await base.getInspectView(id, selection as never)),
          selection: { stage: "logical", planes: [], composite: true },
        };
      },
    } as ReturnType<typeof createMockServices>;
    render(App, { services });
    await screen.findByRole("navigation", { name: "Sections" });
    await openJob(user, /Front chest/);
    await user.click(screen.getByRole("tab", { name: "Inspect" }));
    await user.click(await screen.findByRole("radio", { name: "Slots" }));
    expect(selections.at(-1)).toMatchObject({ stage: "slots", composite: false });
  });
});

describe("review flow", () => {
  it("opens the review tab from the banner and clears the banner after recording an outcome", async () => {
    const { user } = await openApp();
    await user.click(screen.getByRole("button", { name: "Review now" }));
    await screen.findByText("Check the printer before printing anything else");
    await user.click(screen.getByRole("button", { name: "It printed correctly" }));
    expect(screen.getByText(/will be recorded/i)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Mark finished" }));
    const header = screen.getByRole("group", { name: "Job header" });
    expect(await within(header).findByText("Finished")).toBeInTheDocument();
    expect(screen.queryByRole("alert")).toBeNull();
  });
});

describe("labels and unsupported controls", () => {
  it("labels sheet previews as Simulation, Fixture data and Not physical output", async () => {
    const { user } = await openApp();
    await openJob(user, /Front chest/);
    await user.click(screen.getByRole("tab", { name: "Sheet" }));
    const chip = screen.getByTestId("stage-chip");
    expect(chip).toHaveTextContent(
      "Garment simulation · approximate, not a color proof · simulation · Fixture data · Not physical output",
    );
  });

  it("labels inspect previews with the same simulation, fixture and non-physical vocabulary", async () => {
    const { user } = await openApp();
    await openJob(user, /Front chest/);
    await user.click(screen.getByRole("tab", { name: "Inspect" }));
    expect(screen.getByTestId("stage-chip")).toHaveTextContent(
      "Logical planes: C M Y K W · continuous tone · simulation · Fixture data · Not physical output",
    );
    await user.click(screen.getByRole("radio", { name: "Slots" }));
    expect(screen.getByTestId("stage-chip")).toHaveTextContent(
      "Physical-slot raster · W split to W1 and W2 · simulation · Fixture data · Not physical output",
    );
  });

  it("disables white choke with a stated reason when the mode does not support it", async () => {
    const { user } = await openApp();
    await openJob(user, /Front chest/);
    await user.click(screen.getByRole("tab", { name: "Sheet" }));
    const choke = screen.getByRole("spinbutton", { name: /White choke/ });
    expect(choke).toBeDisabled();
    expect(choke).toHaveAccessibleDescription("Not supported by mode PM-exp-01");
  });

  it("marks experimental maturity on recipes", async () => {
    const { user } = await openApp();
    await user.click(screen.getByRole("button", { name: "Recipes" }));
    expect((await screen.findAllByText("Experimental")).length).toBeGreaterThan(0);
  });
});

describe("other screens", () => {
  it("shows the channel map with untested evidence and inferred values", async () => {
    const { user } = await openApp();
    await user.click(screen.getByRole("button", { name: "Devices" }));
    const table = await screen.findByRole("table", { name: "Channel map" });
    expect(within(table).getAllByText("Not tested").length).toBeGreaterThan(0);
    expect(within(table).getAllByText(/\(inferred\)/).length).toBeGreaterThan(0);
  });

  it("warns about artwork metadata and rejections before placement", async () => {
    const { user } = await openApp();
    await openJob(user, /Pocket print/);
    await user.click(screen.getByRole("tab", { name: "Import" }));
    expect(screen.getByText("Untagged, assumed sRGB")).toBeInTheDocument();
    expect(screen.getByText(/APNG is not supported/)).toBeInTheDocument();
  });

  it("exports a diagnostic report through the application service", async () => {
    const { user, services } = await openApp();
    await user.click(screen.getByRole("button", { name: "Reports" }));
    await user.click(await screen.findByRole("button", { name: "Export diagnostic report" }));
    expect(await screen.findByText(/diagnostics-.*\.zip/)).toBeInTheDocument();
    expect(services.calls.exportDiagnosticReport).toHaveLength(1);
  });
});

describe("setup and channel verification", () => {
  it("creates a device setup from an empty install and unlocks gated navigation", async () => {
    const { user } = await openApp("fresh-install");
    expect(screen.getByRole("button", { name: "Targets" })).toBeDisabled();
    await user.click(screen.getByRole("button", { name: "Devices" }));
    await user.type(await screen.findByRole("textbox", { name: "Setup name" }), "Garage bench");
    await user.click(screen.getByRole("button", { name: "Create setup" }));
    expect(
      await screen.findByRole("heading", { name: /Garage bench · rev 1/ }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Targets" })).toBeEnabled();
  });

  it("records a channel test result and marks the setup verified once every channel matched", async () => {
    const { user } = await openApp();
    await user.click(screen.getByRole("button", { name: "Devices" }));
    const table = await screen.findByRole("table", { name: "Channel map" });
    const inks = ["C", "M", "Y", "K", "W"];
    for (const ink of inks) {
      await user.selectOptions(screen.getByRole("combobox", { name: "Channel" }), ink);
      await user.selectOptions(screen.getByRole("combobox", { name: "Result" }), "matched");
      await user.click(screen.getByRole("button", { name: "Record test result" }));
      await within(table).findAllByText("Matched");
    }
    expect(within(table).getAllByText("Matched")).toHaveLength(5);
    expect(await within(screen.getByRole("main")).findByText("Verified")).toBeInTheDocument();
  });

  it("shows a mismatch as bad evidence and keeps the setup unverified", async () => {
    const { user } = await openApp();
    await user.click(screen.getByRole("button", { name: "Devices" }));
    const table = await screen.findByRole("table", { name: "Channel map" });
    await user.selectOptions(screen.getByRole("combobox", { name: "Channel" }), "Y");
    await user.selectOptions(screen.getByRole("combobox", { name: "Result" }), "mismatch");
    await user.click(screen.getByRole("button", { name: "Record test result" }));
    expect(await within(table).findByText("Mismatch")).toBeInTheDocument();
    expect(within(screen.getByRole("main")).getByText("Unverified")).toBeInTheDocument();
  });
});

describe("device mode and recipe selection", () => {
  it("switches the device mode and updates the status bar", async () => {
    const { user } = await openApp();
    await user.click(screen.getByRole("button", { name: "Recipes" }));
    await user.selectOptions(
      await screen.findByRole("combobox", { name: "Device mode" }),
      "PM-exp-02",
    );
    const bar = screen.getByRole("contentinfo", { name: "Device status" });
    expect(await within(bar).findByText(/PM-exp-02/)).toBeInTheDocument();
  });

  it("selects a recipe in the active mode and gives a reason for recipes of other modes", async () => {
    const { user } = await openApp();
    await user.click(screen.getByRole("button", { name: "Recipes" }));
    const list = await screen.findByRole("list", { name: "Recipes" });
    const other = within(list).getByRole("button", { name: /Use Second/ });
    expect(other).toBeDisabled();
    expect(other).toHaveAccessibleDescription(/requires mode PM-exp-02/i);
    await user.click(within(list).getByRole("button", { name: /Use Alternate/ }));
    await waitFor(() =>
      expect(within(list).getByRole("button", { name: /Use Fixture transfer/ })).toBeEnabled(),
    );
    expect(within(list).getAllByText("Selected")).toHaveLength(1);
  });
});

describe("white policy and mask controls", () => {
  async function openSheet(name: RegExp) {
    const ctx = await openApp();
    await openJob(ctx.user, name);
    await ctx.user.click(screen.getByRole("tab", { name: "Sheet" }));
    return ctx;
  }

  it("changes the white policy and mask source on a draft job", async () => {
    const { user } = await openSheet(/Pocket print/);
    await user.selectOptions(screen.getByRole("combobox", { name: "White policy" }), "No white");
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Mask source" }),
      "Whole placement",
    );
    await waitFor(() =>
      expect(screen.getByRole("combobox", { name: "White policy" })).toHaveValue("No white"),
    );
    expect(screen.getByRole("combobox", { name: "Mask source" })).toHaveValue("Whole placement");
  });

  it("shows unsupported mask inversion disabled with the mode reason", async () => {
    await openSheet(/Pocket print/);
    const invert = screen.getByRole("checkbox", { name: "Invert mask" });
    expect(invert).toBeDisabled();
    expect(invert).not.toBeChecked();
    expect(invert).toHaveAccessibleDescription("Not supported by mode PM-exp-01");
  });

  it("does not show an editable-looking value for unsupported white choke", async () => {
    await openSheet(/Pocket print/);
    const choke = screen.getByRole("spinbutton", { name: /White choke/ });
    expect(choke).toBeDisabled();
    expect(choke).toHaveValue(null);
  });

  it("freezes policy and mask controls once the job has rendered", async () => {
    await openSheet(/Front chest/);
    expect(screen.getByRole("combobox", { name: "White policy" })).toBeDisabled();
    expect(screen.getByRole("combobox", { name: "Mask source" })).toBeDisabled();
  });

  it("enables white choke in a mode that supports it", async () => {
    const { user } = await openApp();
    await user.click(screen.getByRole("button", { name: "Recipes" }));
    await user.selectOptions(
      await screen.findByRole("combobox", { name: "Device mode" }),
      "PM-exp-02",
    );
    await user.click(screen.getByRole("button", { name: "Jobs" }));
    await openJob(user, /Pocket print/);
    await user.click(screen.getByRole("tab", { name: "Sheet" }));
    const choke = screen.getByRole("spinbutton", { name: /White choke/ });
    expect(choke).toBeEnabled();
    await user.clear(choke);
    await user.type(choke, "0.3");
    await user.tab();
    await user.click(screen.getByRole("tab", { name: "Provenance" }));
    await user.click(screen.getByRole("tab", { name: "Sheet" }));
    expect(screen.getByRole("spinbutton", { name: /White choke/ })).toHaveValue(0.3);
  });
});

describe("attempt timeline", () => {
  it("highlights a new attempt briefly and never the entries that were already there", async () => {
    const { user } = await openApp();
    await openJob(user, /Front chest/);
    await user.click(screen.getByRole("button", { name: "Write to file sink" }));
    await user.click(await screen.findByRole("button", { name: "Confirm write to file sink" }));
    const list = await screen.findByRole("list", { name: "Attempt history" });
    const entry = await within(list).findByRole("listitem");
    expect(entry).toHaveAttribute("data-fresh", "true");
    await waitFor(() => expect(entry).not.toHaveAttribute("data-fresh"), { timeout: 2000 });
  });

  it("does not highlight attempts that exist when the tab opens", async () => {
    const { user } = await openApp();
    await openJob(user, /North Ridge/);
    await user.click(screen.getByRole("tab", { name: /Attempts/ }));
    const list = await screen.findByRole("list", { name: "Attempt history" });
    expect(within(list).getByRole("listitem")).not.toHaveAttribute("data-fresh");
  });
});

describe("review fixes for task 8.3", () => {
  it("keeps the Preflight badge in step with the open count after acknowledging", async () => {
    const { user } = await openApp();
    await openJob(user, /Pocket print/);
    expect(screen.getByRole("tab", { name: /Preflight/ })).toHaveTextContent("1");
    await user.click(screen.getByRole("tab", { name: /Preflight/ }));
    await user.click(screen.getByRole("button", { name: "Acknowledge finding" }));
    await screen.findByText(/Acknowledged by Fixture operator/);
    expect(screen.getByRole("tab", { name: /Preflight/ })).not.toHaveTextContent("1");
  });

  it("selects the placement on the Sheet tab when a finding is clicked", async () => {
    const { user } = await openApp();
    await openJob(user, /Hoodie sleeve/);
    await user.click(screen.getByRole("tab", { name: /Preflight/ }));
    await user.click(screen.getByRole("button", { name: "Show on sheet" }));
    expect(screen.getByRole("tab", { name: "Sheet" })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("button", { name: /sleeve\.png/ })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
  });

  it("shows one plane at a time, a composite option, metadata and raster stubs in Inspect", async () => {
    const { user } = await openApp();
    await openJob(user, /Front chest/);
    await user.click(screen.getByRole("tab", { name: "Inspect" }));
    await screen.findByRole("button", { name: "Plane K" });
    await user.click(screen.getByRole("button", { name: "Plane C" }));
    expect(screen.getByRole("button", { name: "Plane C" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByRole("button", { name: "Plane K" })).toHaveAttribute(
      "aria-pressed",
      "false",
    );
    await user.click(screen.getByRole("radio", { name: "Slots" }));
    expect(await screen.findByRole("button", { name: "Plane W1" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Composite" }));
    expect(screen.getByTestId("stage-chip")).toHaveTextContent(
      "Composite simulation of retained planes",
    );
    expect(screen.getByText("Artifact hash")).toBeInTheDocument();
    expect(screen.getAllByText("Available when a raster exists")).toHaveLength(2);
  });

  it("shows review facts, what was sent, and creates a held reprint attempt", async () => {
    const { user } = await openApp();
    await user.click(screen.getByRole("button", { name: "Review now" }));
    await screen.findByText("Check the printer before printing anything else");
    expect(screen.getByText("SPOOL-0142")).toBeInTheDocument();
    expect(screen.getByText("sha256:fixture-artifact-0142")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Nothing printed, reprint" }));
    await user.click(screen.getByRole("button", { name: "Create reprint attempt" }));
    await screen.findByLabelText("Attempt history");
    expect(screen.getByText(/Printer · held/)).toBeInTheDocument();
  });
});

describe("lists that the backend changes underneath", () => {
  it("shows the state a preflight recorded while the job was being opened", async () => {
    const base = createMockServices("working-day");
    const first = (await base.listJobs())[0];
    // Loading the preflight view is what records the change, as it does in the live workspace.
    let recorded = false;
    const as = <T extends { state: string }>(job: T): T => ({
      ...job,
      state: recorded ? "held" : "draft",
    });
    const services = {
      ...base,
      listJobs: async () => (await base.listJobs()).map((j) => (j.id === first.id ? as(j) : j)),
      getJob: async (id: string) => {
        const job = await base.getJob(id);
        return id === first.id ? as(job) : job;
      },
      getPreflightView: async (id: string) => {
        if (id === first.id) recorded = true;
        return base.getPreflightView(id);
      },
    } as ReturnType<typeof createMockServices>;
    const user = userEvent.setup();
    render(App, { services });
    await screen.findByRole("navigation", { name: "Sections" });
    const row = () => screen.getByRole("option", { name: new RegExp(first.name) });
    expect(within(row()).getByText("Draft")).toBeInTheDocument();
    await user.click(row());
    await screen.findByRole("heading", { level: 1, name: new RegExp(first.name) });
    // Neither the open job nor the list still says Draft.
    await waitFor(() => expect(within(row()).getByText("Held")).toBeInTheDocument());
    expect(within(row()).queryByText("Draft")).not.toBeInTheDocument();
    const header = screen.getByRole("heading", { level: 1, name: new RegExp(first.name) });
    expect(header.closest("header, section, div")?.textContent).not.toContain("Draft");
  });

  it("offers a recipe that was created after the app started when the New job form opens", async () => {
    const recipe = (revision: number) => ({
      hash: String(revision).repeat(64),
      id: "RECIPE-1",
      revision,
      retired: false,
      validated: revision === 2,
    });
    let stored = [recipe(1)];
    const services = {
      ...createMockServices("working-day"),
      getCreationOptions: async () => ({
        setups: [{ hash: "a".repeat(64), id: "SETUP-1", verified: true }],
        modes: [
          {
            hash: "b".repeat(64),
            id: "MODE-1",
            setupId: "SETUP-1",
            maturity: "validated" as const,
          },
        ],
        recipes: stored,
      }),
    } as ReturnType<typeof createMockServices>;
    const user = userEvent.setup();
    render(App, { services });
    await screen.findByRole("navigation", { name: "Sections" });
    stored = [recipe(1), recipe(2)]; // made from the CLI while the app was open
    await user.click(screen.getByRole("button", { name: "New job" }));
    const form = await screen.findByRole("form", { name: "New job" });
    expect(
      await within(form).findByRole("option", { name: /rev 2 \(validated\)/ }),
    ).toBeInTheDocument();
  });
});
