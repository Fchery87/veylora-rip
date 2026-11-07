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
    expect(chip).toHaveTextContent("Simulation");
    expect(chip).toHaveTextContent("Fixture data");
    expect(chip).toHaveTextContent("Not physical output");
  });

  it("disables white choke with a stated reason when the mode does not support it", async () => {
    const { user } = await openApp();
    await openJob(user, /Front chest/);
    await user.click(screen.getByRole("tab", { name: "Sheet" }));
    const choke = screen.getByRole("spinbutton", { name: /White choke/ });
    expect(choke).toBeDisabled();
    expect(screen.getByText("Not supported by mode PM-exp-01")).toBeInTheDocument();
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
