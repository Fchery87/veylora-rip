// Typed application-service boundary. The UI consumes these read models and
// sends commands by stable domain ID. It never touches image, separation,
// encoder, persistence, or Windows APIs (ADR 0001).

export type DataSource = "fixture" | "live";
export type ModeMaturity = "experimental" | "validated" | "retired";

export type JobState =
  | "draft"
  | "preflight-failed"
  | "rendering"
  | "ready"
  | "held"
  | "submitting"
  | "submitted"
  | "ambiguous"
  | "needs-review"
  | "cancelled"
  | "failed"
  | "operator-confirmed-finished";

export interface CapabilityState {
  dataSource: DataSource;
  setup: { exists: boolean; verified: boolean; experimental: boolean; name: string };
  mode: {
    id: string;
    maturity: ModeMaturity;
    sheetSizes: string[];
    whiteChoke: boolean;
    screenedInspect: boolean;
  };
  sinks: {
    /** Physical ET-8550 submission. False until the physical gate passes. */
    physicalPrinter: boolean;
    /** Writes the printer-ready artifact to a file. Not physical output. */
    fileSink: boolean;
  };
  /** Set while any submission attempt is ambiguous or kept in review. */
  openReview: { jobId: string; attemptId: string; jobName: string } | null;
}

export type Severity = "blocking" | "confirm" | "info";

export interface Finding {
  id: string;
  severity: Severity;
  title: string;
  cause: string;
  /** Exact correction. Every blocking finding must have one. */
  fix: string;
  acknowledged: boolean;
}

export interface Placement {
  id: string;
  name: string;
  xMm: number;
  yMm: number;
  widthMm: number;
  heightMm: number;
  clipped: boolean;
}

export type AttemptKind = "file-sink" | "printer";
export type AttemptOutcome = "pending" | "ambiguous" | "finished" | "reprint" | "kept-in-review";

export interface Attempt {
  id: string;
  kind: AttemptKind;
  at: string;
  outcome: AttemptOutcome;
  note: string;
  /** Present for file-sink attempts. */
  sinkPath?: string;
}

export interface JobSummary {
  id: string;
  name: string;
  state: JobState;
  sheet: string;
  placementCount: number;
  updated: string;
  progressPercent?: number;
}

export interface JobDetail extends JobSummary {
  placements: Placement[];
  findings: Finding[];
  attempts: Attempt[];
  provenance: { label: string; value: string }[];
  whitePolicy: { source: string; policy: string; chokeMm: number };
  rendered: boolean;
}

export interface ChannelRow {
  logicalInk: string;
  physicalSlot: string;
  protocolChannel: number;
  evidence: "not-tested" | "matched" | "mismatch";
  inferred: boolean;
}

export interface DeviceSetupModel {
  name: string;
  revision: number;
  verified: boolean;
  channels: ChannelRow[];
}

export interface RecipeModel {
  id: string;
  name: string;
  state: ModeMaturity;
  modeId: string;
}

export interface TargetModel {
  id: string;
  name: string;
  description: string;
}

export interface ArtworkCandidate {
  filename: string;
  pixels: string;
  colorInterpretation: string;
  status: "ok" | "warning" | "rejected";
  message: string;
}

export interface DiagnosticExportRequest {
  redact: boolean;
  omitArtwork: boolean;
}

export interface DiagnosticExportResult {
  path: string;
  files: string[];
}

export type ReviewOutcome = "finished" | "reprint" | "printed-badly" | "decide-later";

export type ChannelTestResult = "matched" | "mismatch";

export interface AppServices {
  getCapabilities(): Promise<CapabilityState>;
  listJobs(): Promise<JobSummary[]>;
  getJob(jobId: string): Promise<JobDetail>;
  /** Null until a device setup has been created. */
  getDeviceSetup(): Promise<DeviceSetupModel | null>;
  /** Creates revision 1 of a setup. Every channel starts as not tested. */
  createDeviceSetup(name: string): Promise<DeviceSetupModel>;
  /** Records bench evidence for one logical ink. Verified once every channel matched. */
  recordChannelTest(logicalInk: string, result: ChannelTestResult): Promise<DeviceSetupModel>;
  listRecipes(): Promise<RecipeModel[]>;
  listTargets(): Promise<TargetModel[]>;
  listArtworkCandidates(): Promise<ArtworkCandidate[]>;
  renderAndHold(jobId: string): Promise<JobDetail>;
  acknowledgeFinding(jobId: string, findingId: string): Promise<JobDetail>;
  writeToFileSink(jobId: string): Promise<JobDetail>;
  /** Physical submission. Rejects until the ET-8550 physical gate passes. */
  releaseToPrinter(jobId: string): Promise<JobDetail>;
  resolveReview(attemptId: string, outcome: ReviewOutcome): Promise<CapabilityState>;
  exportDiagnosticReport(request: DiagnosticExportRequest): Promise<DiagnosticExportResult>;
}
