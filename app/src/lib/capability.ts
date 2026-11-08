import type { CapabilityState, JobDetail } from "./services/types";

export type SectionId = "jobs" | "devices" | "recipes" | "targets" | "reports" | "settings";

export interface NavItem {
  id: SectionId;
  label: string;
  group: "top" | "bottom";
  enabled: boolean;
  /** Why the item is disabled. Always present when `enabled` is false. */
  reason?: string;
  badge?: "review" | "warn";
}

export interface CommandState {
  enabled: boolean;
  /** Shown as text beside the control, never hidden in a tooltip. */
  reason?: string;
}

export interface JobCommands {
  renderAndHold: CommandState;
  releaseToPrinter: CommandState;
  writeToFileSink: CommandState;
}

const NO_SETUP = "Create a device setup in Devices first";

export function deriveNavigation(caps: CapabilityState): NavItem[] {
  const needsSetup = !caps.setup.exists;
  const gated = (id: SectionId, label: string): NavItem => ({
    id,
    label,
    group: "top",
    enabled: !needsSetup,
    reason: needsSetup ? NO_SETUP : undefined,
  });
  return [
    {
      id: "jobs",
      label: "Jobs",
      group: "top",
      enabled: true,
      badge: caps.openReview ? "review" : undefined,
    },
    {
      id: "devices",
      label: "Devices",
      group: "top",
      enabled: true,
      badge: caps.setup.verified ? undefined : "warn",
    },
    gated("recipes", "Recipes"),
    gated("targets", "Targets"),
    { id: "reports", label: "Reports", group: "top", enabled: true },
    { id: "settings", label: "Settings", group: "bottom", enabled: true },
  ];
}

function countLabel(n: number, noun: string): string {
  return `${n} ${noun}${n === 1 ? "" : "s"}`;
}

function deriveRender(job: JobDetail): CommandState {
  if (job.rendered || job.state !== "draft") {
    return { enabled: false, reason: "Already rendered. Duplicate as new job to edit." };
  }
  const blocking = job.findings.filter((f) => f.severity === "blocking").length;
  if (blocking > 0) {
    return { enabled: false, reason: `Fix ${countLabel(blocking, "blocking finding")} to render` };
  }
  const unconfirmed = job.findings.filter(
    (f) => f.severity === "confirm" && !f.acknowledged,
  ).length;
  if (unconfirmed > 0) {
    return { enabled: false, reason: `Confirm ${countLabel(unconfirmed, "finding")} to render` };
  }
  return { enabled: true };
}

function isHeld(job: JobDetail): boolean {
  return job.rendered && (job.state === "held" || job.state === "ready");
}

function deriveRelease(caps: CapabilityState, job: JobDetail): CommandState {
  if (!isHeld(job)) {
    return { enabled: false, reason: "Render and hold the job first" };
  }
  if (!caps.sinks.physicalPrinter) {
    return {
      enabled: false,
      reason: "Physical submission is unavailable until the ET-8550 physical gate passes",
    };
  }
  if (caps.openReview) {
    return {
      enabled: false,
      reason: `Waiting: printer blocked by review of ${caps.openReview.jobId}`,
    };
  }
  if (!caps.setup.verified && !caps.setup.experimental) {
    return { enabled: false, reason: "Verify the setup, or turn on experimental mode in Devices" };
  }
  return { enabled: true };
}

function deriveFileSink(caps: CapabilityState, job: JobDetail): CommandState {
  if (!caps.sinks.fileSink) {
    return { enabled: false, reason: "File sink is not available" };
  }
  if (!isHeld(job)) {
    return { enabled: false, reason: "Render and hold the job before writing to the file sink" };
  }
  return { enabled: true };
}

export function deriveJobCommands(caps: CapabilityState, job: JobDetail): JobCommands {
  return {
    renderAndHold: deriveRender(job),
    releaseToPrinter: deriveRelease(caps, job),
    writeToFileSink: deriveFileSink(caps, job),
  };
}
