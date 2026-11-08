import type { DataSource, JobState, PreviewSource, StageId, Tone } from "./services/types";

export type { PreviewSource, StageId, Tone };

const STATE_BADGES: Record<JobState, { text: string; tone: Tone }> = {
  draft: { text: "Draft", tone: "neutral" },
  "preflight-failed": { text: "Preflight failed", tone: "bad" },
  rendering: { text: "Rendering", tone: "neutral" },
  ready: { text: "Ready", tone: "info" },
  held: { text: "Held", tone: "info" },
  submitting: { text: "Sending", tone: "info" },
  submitted: { text: "Submitted", tone: "info" },
  ambiguous: { text: "Ambiguous", tone: "review" },
  "needs-review": { text: "Needs review", tone: "review" },
  cancelled: { text: "Cancelled", tone: "neutral" },
  failed: { text: "Failed", tone: "bad" },
  // Only an operator-confirmed outcome is ever called finished.
  "operator-confirmed-finished": { text: "Finished", tone: "ok" },
};

export function stateBadge(
  state: JobState,
  progressPercent?: number,
): { text: string; tone: Tone } {
  const base = STATE_BADGES[state];
  const withProgress =
    progressPercent !== undefined && (state === "rendering" || state === "submitting");
  return withProgress ? { ...base, text: `${base.text} ${progressPercent}%` } : base;
}

export function isReviewState(state: JobState): boolean {
  return state === "ambiguous" || state === "needs-review";
}

/** Source for a simulated preview, given what the app is currently running on. */
export function simulationSource(dataSource: DataSource): PreviewSource {
  return dataSource === "fixture" ? "fixture" : "simulation";
}

export const STAGES: { id: StageId; label: string; sublabel: string; chip: string }[] = [
  {
    id: "garment",
    label: "Garment",
    sublabel: "Approximate",
    chip: "Garment simulation · approximate, not a color proof",
  },
  { id: "film", label: "Film", sublabel: "Mirrored", chip: "Film view · mirrored for transfer" },
  {
    id: "logical",
    label: "Logical",
    sublabel: "CMYK + W",
    chip: "Logical planes: C M Y K W · continuous tone",
  },
  {
    id: "slots",
    label: "Slots",
    sublabel: "W1 + W2",
    chip: "Physical-slot raster · W split to W1 and W2",
  },
  { id: "screened", label: "Screened", sublabel: "Device truth", chip: "Screened device raster" },
];

export const COMPOSITE_LABEL = "Composite simulation of retained planes";

export function stageChipText(stage: StageId, source: PreviewSource, composite = false): string {
  const def = STAGES.find((s) => s.id === stage)!;
  const parts = [def.chip];
  if (composite) parts.push(COMPOSITE_LABEL);
  parts.push(source === "retained" ? "retained artifact" : "simulation");
  if (source === "fixture") parts.push("Fixture data");
  // Not in the design.md 6.3 vocabulary alone: the plan requires every preview to say so.
  parts.push("Not physical output");
  return parts.join(" · ");
}
