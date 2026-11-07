import type {
  AppServices,
  Attempt,
  CapabilityState,
  ChannelTestResult,
  DeviceModeModel,
  DeviceSetupModel,
  ModeMaturity,
  RecipeModel,
  DiagnosticExportRequest,
  Finding,
  JobDetail,
  JobSummary,
  Placement,
  ReviewOutcome,
} from "./types";

// Mocked application services backed by clearly labeled fixture data.
// Nothing here renders, separates, encodes, persists, or talks to Windows.

export type Scenario = "working-day" | "fresh-install";

export interface MockServices extends AppServices {
  /** Recorded command calls, for tests. */
  calls: { exportDiagnosticReport: DiagnosticExportRequest[] };
}

const placement = (id: string, name: string, over: Partial<Placement> = {}): Placement => ({
  id,
  name,
  xMm: 20,
  yMm: 30,
  widthMm: 180,
  heightMm: 240,
  clipped: false,
  ...over,
});

const finding = (id: string, over: Partial<Finding>): Finding => ({
  id,
  severity: "info",
  title: "",
  cause: "",
  fix: "",
  acknowledged: false,
  ...over,
});

function baseJob(over: Partial<JobDetail> & Pick<JobDetail, "id" | "name">): JobDetail {
  return {
    state: "draft",
    sheet: "A3",
    placementCount: 1,
    updated: "just now",
    placements: [placement("p1", "artwork.png")],
    findings: [],
    attempts: [],
    provenance: [
      { label: "Device setup", value: "Home bench rev 3" },
      { label: "Print mode", value: "PM-exp-01 (experimental)" },
      { label: "Recipe", value: "RCP-fixture-01" },
      { label: "Snapshot hash", value: "sha256:fixture-0000" },
    ],
    whitePolicy: { source: "Auto from alpha", policy: "Underbase", invertMask: false, chokeMm: 0 },
    rendered: false,
    ...over,
  };
}

function workingDayJobs(): JobDetail[] {
  const ambiguousAttempt: Attempt = {
    id: "A-0142-1",
    kind: "printer",
    at: "09:14:02",
    outcome: "ambiguous",
    note: "Spool ended before confirmation. Fixture data.",
  };
  return [
    baseJob({
      id: "J-0142",
      name: "North Ridge tee run",
      state: "ambiguous",
      rendered: true,
      updated: "12 min ago",
      attempts: [ambiguousAttempt],
    }),
    baseJob({
      id: "J-0143",
      name: "Front chest logo",
      state: "held",
      rendered: true,
      updated: "1 h ago",
    }),
    baseJob({
      id: "J-0144",
      name: "Hoodie sleeve",
      placements: [placement("p1", "sleeve.png", { xMm: -4.2, clipped: true })],
      findings: [
        finding("f1", {
          severity: "blocking",
          title: "Placement is outside the printable area",
          cause: "sleeve.png extends past the left edge of the printable box.",
          fix: "Move it left by at least 4.2 mm",
        }),
      ],
    }),
    baseJob({
      id: "J-0145",
      name: "Pocket print",
      findings: [
        finding("f2", {
          severity: "confirm",
          title: "Artwork has no color profile",
          cause: "pocket.png is untagged, so sRGB is assumed.",
          fix: "Confirm sRGB is correct, or re-export the file with a profile.",
        }),
      ],
    }),
  ];
}

function newSetup(name: string): DeviceSetupModel {
  return {
    name,
    revision: 1,
    verified: false,
    channels: [
      {
        logicalInk: "C",
        physicalSlot: "Cyan tank",
        protocolChannel: 0,
        evidence: "not-tested",
        inferred: true,
      },
      {
        logicalInk: "M",
        physicalSlot: "Magenta tank",
        protocolChannel: 1,
        evidence: "not-tested",
        inferred: true,
      },
      {
        logicalInk: "Y",
        physicalSlot: "Yellow tank",
        protocolChannel: 2,
        evidence: "not-tested",
        inferred: true,
      },
      {
        logicalInk: "K",
        physicalSlot: "Black tank",
        protocolChannel: 3,
        evidence: "not-tested",
        inferred: false,
      },
      {
        logicalInk: "W",
        physicalSlot: "W1 + W2 tanks",
        protocolChannel: 4,
        evidence: "not-tested",
        inferred: true,
      },
    ],
  };
}

function workingDaySetup(): DeviceSetupModel {
  return { ...newSetup("Home bench"), revision: 3 };
}

interface ModeFixture {
  maturity: ModeMaturity;
  sheetSizes: string[];
  whiteChoke: boolean;
  whiteMaskInvert: boolean;
  whitePolicies: string[];
  screenedInspect: boolean;
}

const MODES: Record<string, ModeFixture> = {
  "PM-exp-01": {
    maturity: "experimental",
    sheetSizes: ["A3", "A4"],
    whiteChoke: false,
    whiteMaskInvert: false,
    whitePolicies: ["Underbase", "No white"],
    screenedInspect: true,
  },
  "PM-exp-02": {
    maturity: "experimental",
    sheetSizes: ["A4"],
    whiteChoke: true,
    whiteMaskInvert: false,
    whitePolicies: ["Underbase", "Highlight", "No white"],
    screenedInspect: false,
  },
};

function capabilities(
  jobs: JobDetail[],
  setup: DeviceSetupModel | null,
  modeId: string,
): CapabilityState {
  const open = jobs.find((j) =>
    j.attempts.some((a) => a.outcome === "ambiguous" || a.outcome === "kept-in-review"),
  );
  const attempt = open?.attempts.find(
    (a) => a.outcome === "ambiguous" || a.outcome === "kept-in-review",
  );
  return {
    dataSource: "fixture",
    setup: {
      exists: setup !== null,
      verified: setup?.verified ?? false,
      experimental: false,
      name: setup ? `${setup.name} rev ${setup.revision}` : "",
    },
    mode: { id: modeId, ...MODES[modeId] },
    sinks: { physicalPrinter: false, fileSink: true },
    openReview:
      open && attempt ? { jobId: open.id, attemptId: attempt.id, jobName: open.name } : null,
  };
}

const recipes: Omit<RecipeModel, "selected">[] = [
  {
    id: "RCP-fixture-01",
    name: "Fixture transfer recipe",
    state: "experimental",
    modeId: "PM-exp-01",
  },
  {
    id: "RCP-fixture-03",
    name: "Alternate fixture recipe",
    state: "experimental",
    modeId: "PM-exp-01",
  },
  {
    id: "RCP-fixture-02",
    name: "Second fixture recipe",
    state: "experimental",
    modeId: "PM-exp-02",
  },
];

const deviceModes = (selected: string): DeviceModeModel[] =>
  Object.entries(MODES).map(([id, m]) => ({
    id,
    maturity: m.maturity,
    selected: id === selected,
  }));

function assertDraft(job: JobDetail) {
  if (job.rendered) throw new Error("Snapshot frozen. Duplicate the job to edit.");
}

const summary = (j: JobDetail): JobSummary => ({
  id: j.id,
  name: j.name,
  state: j.state,
  sheet: j.sheet,
  placementCount: j.placementCount,
  updated: j.updated,
});

export function createMockServices(scenario: Scenario = "working-day"): MockServices {
  let jobs: JobDetail[] = scenario === "working-day" ? workingDayJobs() : [];
  const calls: MockServices["calls"] = { exportDiagnosticReport: [] };
  let sinkCounter = 0;
  let modeId = "PM-exp-01";
  let recipeId = "RCP-fixture-01";
  let setup: DeviceSetupModel | null = scenario === "working-day" ? workingDaySetup() : null;

  const find = (id: string): JobDetail => {
    const job = jobs.find((j) => j.id === id);
    if (!job) throw new Error(`Unknown job ${id}`);
    return job;
  };
  const update = (id: string, change: (job: JobDetail) => JobDetail): JobDetail => {
    const next = change(find(id));
    jobs = jobs.map((j) => (j.id === id ? next : j));
    return next;
  };

  return {
    calls,
    getCapabilities: async () => capabilities(jobs, setup, modeId),
    listJobs: async () => jobs.map(summary),
    getJob: async (id) => find(id),
    getDeviceSetup: async () => setup,
    createDeviceSetup: async (name) => {
      setup = newSetup(name);
      return setup;
    },
    recordChannelTest: async (logicalInk: string, result: ChannelTestResult) => {
      if (!setup) throw new Error("Create a device setup first.");
      const channels = setup.channels.map((c) =>
        c.logicalInk === logicalInk ? { ...c, evidence: result } : c,
      );
      if (!channels.some((c) => c.logicalInk === logicalInk)) {
        throw new Error(`Unknown channel ${logicalInk}`);
      }
      setup = { ...setup, channels, verified: channels.every((c) => c.evidence === "matched") };
      return setup;
    },
    listRecipes: async () => recipes.map((r) => ({ ...r, selected: r.id === recipeId })),
    listDeviceModes: async () => deviceModes(modeId),
    selectDeviceMode: async (id) => {
      if (!MODES[id]) throw new Error(`Unknown device mode ${id}`);
      modeId = id;
      const current = recipes.find((r) => r.id === recipeId);
      if (current && current.modeId !== id) {
        recipeId = recipes.find((r) => r.modeId === id)?.id ?? "";
      }
      return capabilities(jobs, setup, modeId);
    },
    selectRecipe: async (id) => {
      const recipe = recipes.find((r) => r.id === id);
      if (!recipe) throw new Error(`Unknown recipe ${id}`);
      if (recipe.modeId !== modeId) {
        throw new Error(`Recipe ${id} requires mode ${recipe.modeId}`);
      }
      recipeId = id;
      return recipes.map((r) => ({ ...r, selected: r.id === recipeId }));
    },
    listTargets: async () => [
      { id: "T-channel", name: "Channel identity", description: "One patch per protocol channel." },
      {
        id: "T-geometry",
        name: "Geometry and registration",
        description: "Rulers, corner marks, and registration crosses.",
      },
      {
        id: "T-white",
        name: "White ramps",
        description: "White coverage ramps for underbase checks.",
      },
    ],
    listArtworkCandidates: async () => [
      {
        filename: "pocket.png",
        pixels: "1200 x 1200",
        colorInterpretation: "Untagged",
        status: "warning",
        message: "Untagged, assumed sRGB",
      },
      {
        filename: "animated.apng",
        pixels: "800 x 800",
        colorInterpretation: "sRGB",
        status: "rejected",
        message: "APNG is not supported. Export a static PNG.",
      },
    ],
    setWhitePolicy: async (id, policy) => {
      assertDraft(find(id));
      if (!MODES[modeId].whitePolicies.includes(policy)) {
        throw new Error(`White policy ${policy} is not supported by mode ${modeId}`);
      }
      return update(id, (j) => ({ ...j, whitePolicy: { ...j.whitePolicy, policy } }));
    },
    setWhiteMask: async (id, mask) => {
      assertDraft(find(id));
      if (mask.invert && !MODES[modeId].whiteMaskInvert) {
        throw new Error(`Mask inversion is not supported by mode ${modeId}`);
      }
      return update(id, (j) => ({
        ...j,
        whitePolicy: { ...j.whitePolicy, source: mask.source, invertMask: mask.invert },
      }));
    },
    setWhiteChoke: async (id, chokeMm) => {
      assertDraft(find(id));
      if (!MODES[modeId].whiteChoke) {
        throw new Error(`White choke is not supported by mode ${modeId}`);
      }
      return update(id, (j) => ({ ...j, whitePolicy: { ...j.whitePolicy, chokeMm } }));
    },
    renderAndHold: async (id) =>
      update(id, (j) => ({ ...j, state: "held", rendered: true, updated: "just now" })),
    acknowledgeFinding: async (id, findingId) =>
      update(id, (j) => ({
        ...j,
        findings: j.findings.map((f) => (f.id === findingId ? { ...f, acknowledged: true } : f)),
      })),
    writeToFileSink: async (id) => {
      sinkCounter += 1;
      const attempt: Attempt = {
        id: `A-${id.slice(2)}-${sinkCounter}`,
        kind: "file-sink",
        at: "now",
        outcome: "finished",
        note: "Written to the file sink. Not physical output.",
        sinkPath: `C:\\Veylora\\sink\\${id}-${sinkCounter}.vrip`,
      };
      return update(id, (j) => ({ ...j, state: "held", attempts: [...j.attempts, attempt] }));
    },
    releaseToPrinter: async () => {
      throw new Error("Physical submission is unavailable until the ET-8550 physical gate passes.");
    },
    resolveReview: async (attemptId: string, outcome: ReviewOutcome) => {
      const owner = jobs.find((j) => j.attempts.some((a) => a.id === attemptId));
      if (owner) {
        const map = {
          finished: { attempt: "finished", state: "operator-confirmed-finished" },
          reprint: { attempt: "reprint", state: "held" },
          "printed-badly": { attempt: "reprint", state: "held" },
          "decide-later": { attempt: "kept-in-review", state: "needs-review" },
        } as const;
        const m = map[outcome];
        update(owner.id, (j) => ({
          ...j,
          state: m.state,
          attempts: j.attempts.map((a) => (a.id === attemptId ? { ...a, outcome: m.attempt } : a)),
        }));
      }
      return capabilities(jobs, setup, modeId);
    },
    exportDiagnosticReport: async (request) => {
      calls.exportDiagnosticReport.push(request);
      return {
        path: "C:\\Veylora\\reports\\diagnostics-fixture.zip",
        files: [
          "report.json",
          "versions.txt",
          ...(request.omitArtwork ? [] : ["artwork-thumbnails/"]),
        ],
      };
    },
  };
}
