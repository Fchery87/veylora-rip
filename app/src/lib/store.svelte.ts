import { defaultTab, deriveJobTabs, type JobTabId } from "./jobTabs";
import type { SectionId } from "./capability";
import type {
  AppServices,
  CapabilityState,
  JobDetail,
  JobSummary,
  ReviewOutcome,
} from "./services/types";

export type QueryServices = Pick<
  AppServices,
  "getDeviceSetup" | "listRecipes" | "listDeviceModes" | "listTargets" | "listArtworkCandidates"
>;

export type Theme = "light" | "dark";

/** Presentation state only. Every authoritative change goes through AppServices. */
export class AppStore {
  caps = $state<CapabilityState | null>(null);
  jobs = $state<JobSummary[]>([]);
  job = $state<JobDetail | null>(null);
  section = $state<SectionId>("jobs");
  tab = $state<JobTabId>("sheet");
  theme = $state<Theme>("light");
  announcement = $state("");
  /** Attempt ids added by the last command. Cleared after the 500 ms highlight (design 5.6). */
  freshAttemptIds = $state<string[]>([]);
  error = $state("");

  readonly #services: AppServices;

  constructor(services: AppServices) {
    this.#services = services;
  }

  /** Read-only queries for screens. Commands go through `command`. */
  get queries(): QueryServices {
    return this.#services;
  }

  setTheme(theme: Theme) {
    this.theme = theme;
    document.documentElement.setAttribute("data-theme", theme);
  }

  async load() {
    const [caps, jobs] = await Promise.all([
      this.#services.getCapabilities(),
      this.#services.listJobs(),
    ]);
    this.caps = caps;
    this.jobs = jobs;
  }

  async refresh() {
    await this.load();
    if (this.job) await this.applyJob(await this.#services.getJob(this.job.id));
  }

  /** Keep the active tab valid when the job's state changes which tabs exist. */
  private async applyJob(job: JobDetail) {
    this.markFreshAttempts(this.job, job);
    this.job = job;
    const tabs = deriveJobTabs(job);
    const current = tabs.find((t) => t.id === this.tab);
    if (!current || current.disabled) this.tab = defaultTab(job);
  }

  private markFreshAttempts(previous: JobDetail | null, next: JobDetail) {
    if (!previous || previous.id !== next.id) return;
    const known = new Set(previous.attempts.map((a) => a.id));
    const added = next.attempts.filter((a) => !known.has(a.id)).map((a) => a.id);
    if (added.length === 0) return;
    this.freshAttemptIds = added;
    setTimeout(() => {
      this.freshAttemptIds = this.freshAttemptIds.filter((id) => !added.includes(id));
    }, 500);
  }

  async selectJob(id: string) {
    this.section = "jobs";
    const job = await this.#services.getJob(id);
    this.job = job;
    this.tab = defaultTab(job);
  }

  async openReview() {
    if (!this.caps?.openReview) return;
    await this.selectJob(this.caps.openReview.jobId);
    this.tab = "review";
  }

  /** Run a service command, refresh capability and job state, and announce the result. */
  async command<T>(
    action: (services: AppServices) => Promise<T>,
    message: string,
  ): Promise<T | undefined> {
    try {
      this.error = "";
      const result = await action(this.#services);
      await this.refresh();
      this.announcement = message;
      return result;
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      return undefined;
    }
  }

  renderAndHold(jobId: string) {
    return this.command((s) => s.renderAndHold(jobId), "Job rendered and held");
  }

  acknowledgeFinding(jobId: string, findingId: string) {
    return this.command((s) => s.acknowledgeFinding(jobId, findingId), "Finding acknowledged");
  }

  writeToFileSink(jobId: string) {
    return this.command(async (s) => {
      await s.writeToFileSink(jobId);
      this.tab = "attempts";
    }, "Written to file sink. Not physical output.");
  }

  setWhitePolicy(jobId: string, policy: string) {
    return this.command((s) => s.setWhitePolicy(jobId, policy), "White policy changed");
  }

  setWhiteMask(jobId: string, mask: { source: string; invert: boolean }) {
    return this.command((s) => s.setWhiteMask(jobId, mask), "White mask changed");
  }

  setWhiteChoke(jobId: string, chokeMm: number) {
    return this.command((s) => s.setWhiteChoke(jobId, chokeMm), "White choke changed");
  }

  releaseToPrinter(jobId: string) {
    return this.command((s) => s.releaseToPrinter(jobId), "Job released to printer");
  }

  resolveReview(attemptId: string, outcome: ReviewOutcome) {
    return this.command(async (s) => {
      await s.resolveReview(attemptId, outcome);
      this.tab = "attempts";
    }, "Review outcome recorded");
  }
}
