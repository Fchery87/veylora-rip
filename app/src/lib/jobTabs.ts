import type { JobDetail } from "./services/types";
import { isReviewState } from "./status";

export type JobTabId =
  | "review"
  | "import"
  | "sheet"
  | "inspect"
  | "preflight"
  | "attempts"
  | "provenance";

export interface JobTab {
  id: JobTabId;
  label: string;
  count?: number;
  disabled?: boolean;
  /** Reason shown when the tab is disabled. */
  reason?: string;
}

/** `openFindings` is the open count from the preflight view, the one source for the badge. */
export function deriveJobTabs(job: JobDetail, openFindings: number): JobTab[] {
  const tabs: JobTab[] = [];
  if (isReviewState(job.state)) tabs.push({ id: "review", label: "Review" });
  tabs.push(
    { id: "import", label: "Import" },
    { id: "sheet", label: "Sheet" },
    {
      id: "inspect",
      label: "Inspect",
      disabled: !job.rendered,
      reason: "Render the job to inspect its retained output.",
    },
    { id: "preflight", label: "Preflight", count: openFindings || undefined },
    { id: "attempts", label: "Attempts", count: job.attempts.length || undefined },
    { id: "provenance", label: "Provenance" },
  );
  return tabs;
}

export function defaultTab(job: JobDetail): JobTabId {
  return isReviewState(job.state) ? "review" : "sheet";
}
