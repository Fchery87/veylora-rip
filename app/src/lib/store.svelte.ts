import { defaultTab, deriveJobTabs, type JobTabId } from "./jobTabs";
import type { SectionId } from "./capability";
import type {
  AppServices,
  CapabilityState,
  JobDetail,
  JobSummary,
  ReviewOutcome,
} from "./services/types";

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
  error = $state("");

  readonly #services: AppServices;

  constructor(services: AppServices) {
    this.#services = services;
  }

  get services(): AppServices {
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
    this.job = job;
    const tabs = deriveJobTabs(job);
    const current = tabs.find((t) => t.id === this.tab);
    if (!current || current.disabled) this.tab = defaultTab(job);
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

  private async run(action: () => Promise<unknown>, message: string) {
    try {
      this.error = "";
      await action();
      await this.refresh();
      this.announcement = message;
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    }
  }

  renderAndHold(jobId: string) {
    return this.run(() => this.#services.renderAndHold(jobId), "Job rendered and held");
  }

  acknowledgeFinding(jobId: string, findingId: string) {
    return this.run(
      () => this.#services.acknowledgeFinding(jobId, findingId),
      "Finding acknowledged",
    );
  }

  writeToFileSink(jobId: string) {
    return this.run(async () => {
      await this.#services.writeToFileSink(jobId);
      this.tab = "attempts";
    }, "Written to file sink. Not physical output.");
  }

  releaseToPrinter(jobId: string) {
    return this.run(() => this.#services.releaseToPrinter(jobId), "Job released to printer");
  }

  resolveReview(attemptId: string, outcome: ReviewOutcome) {
    return this.run(async () => {
      await this.#services.resolveReview(attemptId, outcome);
      this.tab = "attempts";
    }, "Review outcome recorded");
  }
}
