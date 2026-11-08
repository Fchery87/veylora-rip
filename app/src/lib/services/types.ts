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
    /** Mask inversion needs device-mode support. */
    whiteMaskInvert: boolean;
    whitePolicies: string[];
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
  /** The placement this finding points at, if any. */
  placementId?: string;
  acknowledged: boolean;
  acknowledgement?: { operator: string; at: string };
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
export type AttemptOutcome =
  | "pending"
  | "ambiguous"
  | "finished"
  | "reprint"
  | "printed-badly"
  | "kept-in-review"
  /** A reprint attempt created by a review choice. Never sent until released. */
  | "held";

export interface Attempt {
  id: string;
  kind: AttemptKind;
  at: string;
  outcome: AttemptOutcome;
  note: string;
  /** Present for file-sink attempts. */
  sinkPath?: string;
  spoolJobId?: string;
  bytesWritten?: number;
  bytesIntended?: number;
  artifactHash?: string;
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
  whitePolicy: {
    /** Mask source. */
    source: string;
    policy: string;
    invertMask: boolean;
    chokeMm: number;
  };
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
  selected: boolean;
}

export interface DeviceModeModel {
  id: string;
  maturity: ModeMaturity;
  selected: boolean;
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

export const WHITE_MASK_SOURCES = ["Auto from alpha", "Whole placement"] as const;

export interface AppServices {
  getCapabilities(): Promise<CapabilityState>;
  listJobs(): Promise<JobSummary[]>;
  getJob(jobId: string): Promise<JobDetail>;
  getPreflightView(jobId: string): Promise<PreflightView>;
  getJobActions(jobId: string): Promise<JobActionsView>;
  /** Rejects until the job is rendered, or for a stage the mode cannot inspect. */
  getInspectView(jobId: string, selection?: InspectSelection): Promise<InspectView>;
  /** Selecting a plane shows it alone and keeps the stage. */
  toggleInspectPlane(
    jobId: string,
    selection: InspectSelection,
    plane: Plane,
    on: boolean,
  ): Promise<InspectView>;
  /** The composite option replaces the single plane. */
  setInspectComposite(
    jobId: string,
    selection: InspectSelection,
    on: boolean,
  ): Promise<InspectView>;
  /** Null when the job has no attempt waiting for review. */
  getReviewView(jobId: string): Promise<ReviewView | null>;
  /** Null until a device setup has been created. */
  getDeviceSetup(): Promise<DeviceSetupModel | null>;
  /** Creates revision 1 of a setup. Every channel starts as not tested. */
  createDeviceSetup(name: string): Promise<DeviceSetupModel>;
  /** Records bench evidence for one logical ink. Verified once every channel matched. */
  recordChannelTest(logicalInk: string, result: ChannelTestResult): Promise<DeviceSetupModel>;
  listRecipes(): Promise<RecipeModel[]>;
  listDeviceModes(): Promise<DeviceModeModel[]>;
  /** Changing the mode updates capabilities, and the selected recipe if it no longer fits. */
  selectDeviceMode(modeId: string): Promise<CapabilityState>;
  /** Rejects when the recipe belongs to a different device mode. */
  selectRecipe(recipeId: string): Promise<RecipeModel[]>;
  listTargets(): Promise<TargetModel[]>;
  listArtworkCandidates(): Promise<ArtworkCandidate[]>;
  /** Draft jobs only; rendered jobs are frozen snapshots. */
  setWhitePolicy(jobId: string, policy: string): Promise<JobDetail>;
  setWhiteMask(jobId: string, mask: { source: string; invert: boolean }): Promise<JobDetail>;
  /** Rejects when the active device mode does not support choke. */
  setWhiteChoke(jobId: string, chokeMm: number): Promise<JobDetail>;
  renderAndHold(jobId: string): Promise<JobDetail>;
  acknowledgeFinding(jobId: string, findingId: string): Promise<JobDetail>;
  writeToFileSink(jobId: string): Promise<JobDetail>;
  /** Physical submission. Rejects until the ET-8550 physical gate passes. */
  releaseToPrinter(jobId: string): Promise<JobDetail>;
  resolveReview(attemptId: string, outcome: ReviewOutcome): Promise<CapabilityState>;
  exportDiagnosticReport(request: DiagnosticExportRequest): Promise<DiagnosticExportResult>;
}

// ---- Presentation view models ----
// Built by Rust (`crates/veylora-app/src/screens`). The UI renders them as given
// and does not re-derive gating, labels, or corrections.

export type Tone = "ok" | "info" | "warn" | "bad" | "review" | "neutral";

export interface FindingView {
  id: string;
  severity: Severity;
  title: string;
  cause: string;
  /** Never empty for a blocking finding. */
  correction: string;
  placementId?: string;
  acknowledged: boolean;
  /** "Acknowledged by <operator> at <time>". */
  acknowledgement?: string;
  canAcknowledge: boolean;
}

export interface PreflightView {
  checkedAgainst: string;
  groups: { severity: Severity; title: string; tone: Tone; findings: FindingView[] }[];
  summary: { text: string; tone: Tone };
  /** Blocking plus unacknowledged confirm findings. */
  openCount: number;
}

export interface CommandView {
  enabled: boolean;
  /** Shown as text beside the control. */
  reason?: string;
}

export interface JobActionsView {
  renderAndHold: CommandView;
  releaseToPrinter: CommandView;
  writeToFileSink: CommandView;
  inspect: CommandView;
}

export type StageId = "garment" | "film" | "logical" | "slots" | "screened";
/** W is the logical white plane; W1 and W2 are the physical white slots. */
export type Plane = "C" | "M" | "Y" | "K" | "W" | "W1" | "W2";
export type PreviewSource = "simulation" | "retained" | "fixture";

export interface InspectSelection {
  stage: StageId;
  planes: Plane[];
  /** Composite simulation of all retained planes. Replaces the plane choice. */
  composite: boolean;
}

export interface InspectView {
  selection: InspectSelection;
  stages: { id: StageId; label: string; disabled: boolean; reason?: string }[];
  planes: { plane: Plane; pressed: boolean }[];
  composite: { label: string; pressed: boolean };
  /** Always says simulation or retained, and always "Not physical output". */
  chip: string;
  source: PreviewSource;
  /** Says whether the metadata values are fixture data. */
  metadataNote: string;
  metadata: { label: string; value: string; kind: "text" | "identifier" }[];
  /** Coverage and loupe stubs shown until a raster exists. */
  panels: { title: string; message: string }[];
  rasterShown: boolean;
  viewportLabel: string;
  placeholder: string;
}

export interface ReviewView {
  headline: string;
  body: string;
  attempt: { id: string; source: DataSource; sourceLabel: string };
  /** Bytes written / intended, spool job ID, last signal, source. */
  facts: { label: string; value: string }[];
  whatWasSent: {
    artifactHash: string;
    artifactSize: string;
    snapshotHash: string;
    placements: string[];
  };
  choices: { outcome: ReviewOutcome; label: string; confirmLabel: string; effect: string }[];
  /** True while the attempt is unresolved. Nothing may be sent to the printer. */
  printerBlocked: boolean;
}
