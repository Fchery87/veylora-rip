// Fallback presenters for the mocked services. They mirror the Rust view
// models in `crates/veylora-app/src/screens` so the UI behaves the same
// outside the Tauri webview. Rust is the authority: `tests/golden.test.ts`
// asserts these presenters against the golden fixture the Rust tests write.

import { deriveJobCommands } from "../capability";
import { COMPOSITE_LABEL, simulationSource, stageChipText, STAGES } from "../status";
import type {
  Attempt,
  CapabilityState,
  Finding,
  FindingView,
  InspectSelection,
  InspectView,
  JobActionsView,
  JobDetail,
  Plane,
  PreflightView,
  ReviewView,
  Severity,
  StageId,
  Tone,
} from "./types";

const LOGICAL_PLANES: Plane[] = ["C", "M", "Y", "K", "W"];
const SLOT_PLANES: Plane[] = ["C", "M", "Y", "K", "W1", "W2"];
const INSPECT_STAGES = ["logical", "slots", "screened"] as const;
const NOT_RENDERED = "Render the job to inspect its retained output.";
const NO_SCREENED = "The active print mode does not support screened inspection.";
const NEEDS_RASTER = "Available when a raster exists";
const NOT_RECORDED = "Not recorded";
const NO_AUTOMATIC_FIX =
  "There is no automatic fix for this finding. Resolve the cause described above, then preflight checks again.";

// Fixture metadata values, matching the Rust presenter.
const FIXTURE_RECIPE_ID = "RCP-fixture-01";
const FIXTURE_PROFILE_HASH = "sha256:fixture-profile-0000";
const FIXTURE_SCREEN_ID = "SCR-fixture-01";
const FIXTURE_ARTIFACT_HASH = "sha256:fixture-artifact-0000";

function findingView(f: Finding): FindingView {
  const fix = f.fix.trim();
  return {
    id: f.id,
    severity: f.severity,
    title: f.title,
    cause: f.cause,
    correction: fix === "" && f.severity === "blocking" ? NO_AUTOMATIC_FIX : fix,
    placementId: f.placementId,
    acknowledged: f.acknowledged,
    acknowledgement: f.acknowledgement
      ? `Acknowledged by ${f.acknowledgement.operator} at ${f.acknowledgement.at}`
      : undefined,
    canAcknowledge: f.severity === "confirm" && !f.acknowledged,
  };
}

export function preflightView(job: JobDetail, caps: CapabilityState): PreflightView {
  const groups: { severity: Severity; title: string; tone: Tone }[] = [
    { severity: "blocking", title: "Blocking", tone: "bad" },
    { severity: "confirm", title: "Confirm", tone: "warn" },
    { severity: "info", title: "Info", tone: "info" },
  ];
  const blocking = job.findings.filter((f) => f.severity === "blocking").length;
  const toConfirm = job.findings.filter((f) => f.severity === "confirm" && !f.acknowledged).length;
  let summary: PreflightView["summary"];
  if (blocking > 0) {
    const parts = [`${blocking} blocking`];
    if (toConfirm > 0) parts.push(`${toConfirm} to confirm`);
    summary = { text: parts.join(" · "), tone: "bad" };
  } else if (toConfirm > 0) {
    summary = { text: `${toConfirm} to confirm`, tone: "warn" };
  } else {
    summary = job.rendered
      ? { text: "Passed at render", tone: "neutral" }
      : { text: "Ready to render", tone: "ok" };
  }
  return {
    checkedAgainst: `Checked just now against ${caps.setup.name}, mode ${caps.mode.id}`,
    groups: groups
      .map((g) => ({
        ...g,
        findings: job.findings.filter((f) => f.severity === g.severity).map(findingView),
      }))
      .filter((g) => g.findings.length > 0),
    summary,
    openCount: blocking + toConfirm,
  };
}

export function jobActionsView(job: JobDetail, caps: CapabilityState): JobActionsView {
  const c = deriveJobCommands(caps, job);
  return {
    ...c,
    inspect: job.rendered ? { enabled: true } : { enabled: false, reason: NOT_RENDERED },
  };
}

export function defaultInspectSelection(): InspectSelection {
  return { stage: "logical", planes: ["K"], composite: false };
}

const planesFor = (stage: StageId): Plane[] =>
  stage === "slots" || stage === "screened" ? SLOT_PLANES : LOGICAL_PLANES;

const PLANE_PREFIX: Partial<Record<StageId, string>> = {
  logical: "Logical plane",
  slots: "Physical slot",
};

const SHEET_MM: Record<string, [number, number]> = { A3: [297, 420], A4: [210, 297] };

function metadata(
  job: JobDetail,
  caps: CapabilityState,
  selection: InspectSelection,
): InspectView["metadata"] {
  const stage = selection.stage;
  const mm = SHEET_MM[job.sheet];
  const dimensions = mm
    ? `${mm[0]} x ${mm[1]} mm · pixel size unavailable until a raster exists`
    : "Unknown sheet size";
  const plane = selection.composite
    ? "Composite of retained planes"
    : selection.planes.length === 0
      ? "None selected"
      : `${PLANE_PREFIX[stage] ?? "Screened slot"} ${selection.planes.join(", ")}`;
  const logical = stage === "logical";
  const transform = logical
    ? "None: artwork orientation, no transform"
    : `Mode ${caps.mode.id} mirrors for film; the UI does not`;
  const orientation = logical ? "Artwork orientation" : "Film orientation, mirrored by the mode";
  const id = (label: string, value: string) => ({ label, value, kind: "identifier" as const });
  const text = (label: string, value: string) => ({ label, value, kind: "text" as const });
  return [
    id("Job", job.id),
    text("Dimensions", dimensions),
    text("Plane", plane),
    id("Mode ID", caps.mode.id),
    id("Setup ID", caps.setup.name),
    id("Recipe ID", FIXTURE_RECIPE_ID),
    id("Profile hash", FIXTURE_PROFILE_HASH),
    id("Screen ID", FIXTURE_SCREEN_ID),
    text("Transform ownership", transform),
    id("Artifact hash", FIXTURE_ARTIFACT_HASH),
    text("Orientation", orientation),
  ];
}

export function inspectView(
  job: JobDetail,
  caps: CapabilityState,
  requested: InspectSelection = defaultInspectSelection(),
): InspectView {
  if (!job.rendered) throw new Error(NOT_RENDERED);
  if (!(INSPECT_STAGES as readonly string[]).includes(requested.stage)) {
    throw new Error("Inspect shows the Logical, Slots, and Screened stages.");
  }
  if (requested.stage === "screened" && !caps.mode.screenedInspect) throw new Error(NO_SCREENED);
  // A plane the stage does not offer (logical W in the slot stages) is dropped.
  const offered = planesFor(requested.stage);
  const selection: InspectSelection = {
    stage: requested.stage,
    planes: requested.planes.filter((p) => offered.includes(p)),
    composite: requested.composite,
  };
  const source = simulationSource(caps.dataSource);
  const viewportLabel = selection.composite
    ? "Placeholder for composite of retained planes"
    : selection.planes.length === 0
      ? "Placeholder for no plane"
      : `Placeholder for plane ${selection.planes.join(", ")}`;
  return {
    selection,
    stages: INSPECT_STAGES.map((id) => {
      const blocked = id === "screened" && !caps.mode.screenedInspect;
      return {
        id,
        label: STAGES.find((s) => s.id === id)!.label,
        disabled: blocked,
        ...(blocked ? { reason: NO_SCREENED } : {}),
      };
    }),
    planes: offered.map((plane) => ({ plane, pressed: selection.planes.includes(plane) })),
    composite: { label: COMPOSITE_LABEL, pressed: selection.composite },
    chip: stageChipText(selection.stage, source, selection.composite),
    source,
    metadataNote:
      caps.dataSource === "fixture"
        ? "Fixture data. No retained artifact exists yet."
        : "No retained artifact exists yet.",
    metadata: metadata(job, caps, selection),
    panels: [
      { title: "Coverage", message: NEEDS_RASTER },
      { title: "Loupe", message: NEEDS_RASTER },
    ],
    rasterShown: false,
    viewportLabel,
    placeholder:
      "No raster is shown. Retained artifacts arrive when backend integration is unblocked.",
  };
}

/** Selecting a plane shows it alone and keeps the stage. Deselecting leaves none. */
export function toggleInspectSelection(
  selection: InspectSelection,
  plane: Plane,
  on: boolean,
): InspectSelection {
  const planes = on ? [plane] : selection.planes.filter((p) => p !== plane);
  return { stage: selection.stage, planes, composite: false };
}

export function setInspectComposite(selection: InspectSelection, on: boolean): InspectSelection {
  return { stage: selection.stage, planes: [], composite: on };
}

const recorded = (value: string | number | undefined) =>
  value === undefined ? NOT_RECORDED : String(value);

function reviewFacts(caps: CapabilityState, attempt: Attempt): ReviewView["facts"] {
  const bytes =
    attempt.bytesWritten !== undefined && attempt.bytesIntended !== undefined
      ? `${attempt.bytesWritten} / ${attempt.bytesIntended}`
      : NOT_RECORDED;
  return [
    { label: "Bytes written / intended", value: bytes },
    { label: "Spool job ID", value: recorded(attempt.spoolJobId) },
    { label: "Last signal", value: attempt.at },
    { label: "Source", value: sourceLabel(caps) },
  ];
}

const sourceLabel = (caps: CapabilityState) =>
  caps.dataSource === "fixture" ? "Fixture data" : "Live data";

export function reviewView(job: JobDetail, caps: CapabilityState): ReviewView | null {
  const attempt = job.attempts.find(
    (a) => a.outcome === "ambiguous" || a.outcome === "kept-in-review",
  );
  if (!attempt) return null;
  return {
    headline: "Check the printer before printing anything else",
    body: `Attempt ${attempt.id} ended without a confirmed result. It may or may not have printed. Nothing can be sent to this printer until you record what happened.`,
    attempt: { id: attempt.id, source: caps.dataSource, sourceLabel: sourceLabel(caps) },
    facts: reviewFacts(caps, attempt),
    whatWasSent: {
      artifactHash: recorded(attempt.artifactHash),
      artifactSize:
        attempt.bytesIntended === undefined ? NOT_RECORDED : `${attempt.bytesIntended} bytes`,
      snapshotHash:
        job.provenance.find((row) => row.label === "Snapshot hash")?.value ?? NOT_RECORDED,
      placements: job.placements.map(
        (p) => `${p.name} · ${p.xMm}, ${p.yMm} mm · ${p.widthMm} x ${p.heightMm} mm`,
      ),
    },
    printerBlocked: true,
    choices: [
      {
        outcome: "finished",
        label: "It printed correctly",
        confirmLabel: "Mark finished",
        effect:
          "The attempt will be recorded as finished by the operator. No new attempt is created.",
      },
      {
        outcome: "reprint",
        label: "Nothing printed, reprint",
        confirmLabel: "Create reprint attempt",
        effect:
          "The attempt will be recorded as not printed and a reprint attempt will be created held.",
      },
      {
        outcome: "printed-badly",
        label: "Printed badly, reprint",
        confirmLabel: "Create reprint attempt",
        effect:
          "The attempt will be recorded as printed badly and a reprint attempt will be created held.",
      },
      {
        outcome: "decide-later",
        label: "Decide later",
        confirmLabel: "Keep in review",
        effect: "The attempt stays in review. The printer remains blocked until it is resolved.",
      },
    ],
  };
}
