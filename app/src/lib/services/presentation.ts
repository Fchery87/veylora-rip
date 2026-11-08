// Fallback presenters for the mocked services. They mirror the Rust view
// models in `crates/veylora-app/src/screens` so the UI behaves the same
// outside the Tauri webview. Rust is the authority; keep these in step with it.

import { deriveJobCommands } from "../capability";
import { simulationSource, stageChipText, STAGES } from "../status";
import type {
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
  Tone,
} from "./types";

const PLANES: Plane[] = ["C", "M", "Y", "K", "W"];
const INSPECT_STAGES = ["logical", "slots", "screened"] as const;
const NOT_RENDERED = "Render the job to inspect its retained output.";
const NO_SCREENED = "The active print mode does not support screened inspection.";
const NO_AUTOMATIC_FIX =
  "There is no automatic fix for this finding. Resolve the cause described above, then preflight checks again.";

function findingView(f: Finding): FindingView {
  const fix = f.fix.trim();
  return {
    id: f.id,
    severity: f.severity,
    title: f.title,
    cause: f.cause,
    correction: fix === "" && f.severity === "blocking" ? NO_AUTOMATIC_FIX : fix,
    acknowledged: f.acknowledged,
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

export function inspectView(
  job: JobDetail,
  caps: CapabilityState,
  selection: InspectSelection,
): InspectView {
  if (!job.rendered) throw new Error(NOT_RENDERED);
  if (!(INSPECT_STAGES as readonly string[]).includes(selection.stage)) {
    throw new Error("Inspect shows the Logical, Slots, and Screened stages.");
  }
  if (selection.stage === "screened" && !caps.mode.screenedInspect) throw new Error(NO_SCREENED);
  const source = simulationSource(caps.dataSource);
  return {
    selection: { stage: selection.stage, planes: [...selection.planes] },
    stages: INSPECT_STAGES.map((id) => {
      const blocked = id === "screened" && !caps.mode.screenedInspect;
      return {
        id,
        label: STAGES.find((s) => s.id === id)!.label,
        disabled: blocked,
        ...(blocked ? { reason: NO_SCREENED } : {}),
      };
    }),
    planes: PLANES.map((plane) => ({ plane, pressed: selection.planes.includes(plane) })),
    chip: stageChipText(selection.stage, source),
    source,
    metadata: [
      { label: "Job", value: job.id },
      { label: "Mode", value: caps.mode.id },
      {
        label: "Orientation",
        value:
          selection.stage === "logical"
            ? "Artwork orientation"
            : "Film orientation, mirrored by the mode",
      },
    ],
    rasterShown: false,
    placeholder:
      "No raster is shown. Retained artifacts arrive when backend integration is unblocked.",
  };
}

export function toggleInspectSelection(
  selection: InspectSelection,
  plane: Plane,
  on: boolean,
): InspectSelection {
  const rest = selection.planes.filter((p) => p !== plane);
  const planes = (on ? [...rest, plane] : rest).sort(
    (a, b) => PLANES.indexOf(a) - PLANES.indexOf(b),
  );
  return { stage: "logical", planes };
}

export function reviewView(job: JobDetail): ReviewView | null {
  const attempt = job.attempts.find(
    (a) => a.outcome === "ambiguous" || a.outcome === "kept-in-review",
  );
  if (!attempt) return null;
  return {
    headline: "Check the printer before printing anything else",
    body: `Attempt ${attempt.id} ended without a confirmed result. It may or may not have printed. Nothing can be sent to this printer until you record what happened.`,
    attempt: { id: attempt.id, lastSignal: attempt.at, source: "Fixture data" },
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
