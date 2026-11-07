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

export function deriveJobTabs(job: JobDetail): JobTab[] {
  const tabs: JobTab[] = [];
  if (isReviewState(job.state)) tabs.push({ id: "review", label: "Review" });
  const open = job.findings.filter((f) => f.severity !== "info" && !f.acknowledged).length;
  tabs.push(
    { id: "import", label: "Import" },
    { id: "sheet", label: "Sheet" },
    {
      id: "inspect",
      label: "Inspect",
      disabled: !job.rendered,
      reason: "Render the job to inspect its retained output.",
    },
    { id: "preflight", label: "Preflight", count: open || undefined },
    { id: "attempts", label: "Attempts", count: job.attempts.length || undefined },
    { id: "provenance", label: "Provenance" },
  );
  return tabs;
}

export function defaultTab(job: JobDetail): JobTabId {
  return isReviewState(job.state) ? "review" : "sheet";
}
