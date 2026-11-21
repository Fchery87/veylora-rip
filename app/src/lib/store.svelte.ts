import { defaultTab, deriveJobTabs, type JobTab, type JobTabId } from "./jobTabs";
import type { SectionId } from "./capability";
import type {
  AppServices,
  CapabilityState,
  CreationOptions,
  JobActionsView,
  JobDetail,
  JobSummary,
  NewJobRequest,
  PreflightView,
  ReviewOutcome,
  ReviewView,
} from "./services/types";

/** Rust-built view models for the open job. Rendered as given. */
export interface JobViews {
  preflight: PreflightView;
  actions: JobActionsView;
  review: ReviewView | null;
}

export type QueryServices = Pick<
  AppServices,
  | "getDeviceSetup"
  | "listRecipes"
  | "listDeviceModes"
  | "listTargets"
  | "listArtworkCandidates"
  | "getInspectView"
  | "getPlaneImage"
  | "getPlaneWindow"
  | "toggleInspectPlane"
  | "setInspectComposite"
>;

export type Theme = "light" | "dark";

/** Presentation state only. Every authoritative change goes through AppServices. */
export class AppStore {
  caps = $state<CapabilityState | null>(null);
  jobs = $state<JobSummary[]>([]);
  job = $state<JobDetail | null>(null);
  views = $state<JobViews | null>(null);
  section = $state<SectionId>("jobs");
  tab = $state<JobTabId>("sheet");
  /** Placement highlighted on the Sheet tab, e.g. after clicking a finding. */
  selectedPlacementId = $state<string | null>(null);
  theme = $state<Theme>("light");
  announcement = $state("");
  /** Attempt ids added by the last command. Cleared after the 500 ms highlight (design 5.6). */
  freshAttemptIds = $state<string[]>([]);
  error = $state("");
  /** Stored setups, modes and recipes a job can be built from. Null when jobs cannot be created here. */
  creationOptions = $state<CreationOptions | null>(null);
  /** The New job form is open. */
  creating = $state(false);

  readonly #services: AppServices;

  constructor(services: AppServices) {
    this.#services = services;
  }

  /** Tabs for the open job. The preflight view's open count is the only badge source. */
  get tabs(): JobTab[] {
    return this.job && this.views ? deriveJobTabs(this.job, this.views.preflight.openCount) : [];
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
    const [caps, jobs, creationOptions] = await Promise.all([
      this.#services.getCapabilities(),
      this.#services.listJobs(),
      this.#services.getCreationOptions(),
    ]);
    this.caps = caps;
    this.jobs = jobs;
    this.creationOptions = creationOptions;
  }

  async refresh() {
    await this.load();
    if (this.job) await this.applyJob(await this.#services.getJob(this.job.id));
  }

  private async loadViews(jobId: string): Promise<JobViews> {
    const [preflight, actions, review] = await Promise.all([
      this.#services.getPreflightView(jobId),
      this.#services.getJobActions(jobId),
      this.#services.getReviewView(jobId),
    ]);
    return { preflight, actions, review };
  }

  /** Keep the active tab valid when the job's state changes which tabs exist. */
  private async applyJob(job: JobDetail) {
    this.markFreshAttempts(this.job, job);
    this.views = await this.loadViews(job.id);
    this.job = job;
    const current = this.tabs.find((t) => t.id === this.tab);
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
    this.views = await this.loadViews(id);
    this.job = job;
    this.selectedPlacementId = null;
    this.tab = defaultTab(job);
  }

  selectPlacement(placementId: string) {
    this.selectedPlacementId = placementId;
  }

  /** Select a placement and show it on the Sheet tab. */
  showPlacement(placementId: string) {
    this.selectPlacement(placementId);
    this.tab = "sheet";
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

  /** Opens the form. Nothing is created until it is submitted. */
  startCreating() {
    this.error = "";
    this.creating = true;
  }

  cancelCreating() {
    this.error = "";
    this.creating = false;
  }

  /** Whether the shell can open a native file dialog (live workspace only). */
  get canPickArtwork() {
    return this.#services.pickArtwork !== undefined;
  }

  /** The path chosen in the native dialog, or null when cancelled or it failed (reason in `error`). */
  async pickArtwork(): Promise<string | null> {
    try {
      this.error = "";
      return (await this.#services.pickArtwork?.()) ?? null;
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      return null;
    }
  }

  /** Creates the job in Rust, then opens it. A refusal keeps the form open and shows the reason. */
  async createJob(request: NewJobRequest) {
    try {
      this.error = "";
      const job = await this.#services.createJob(request);
      await this.load();
      this.creating = false;
      await this.selectJob(job.id);
      this.announcement = "Job created";
      return job;
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
      const before = this.job?.attempts.length ?? 0;
      const job = await s.writeToFileSink(jobId);
      // A sink write that sent nothing is not an attempt; its folder is listed under Provenance.
      this.tab = job.attempts.length > before ? "attempts" : "provenance";
    }, "Written to file sink. Not physical output.");
  }

  /**
   * A white change on a real job returns a different job (the stored one is never altered), so the
   * view moves to it. A fixture edit returns the same job and stays put.
   */
  private async editWhite(action: (s: AppServices) => Promise<JobDetail>, message: string) {
    try {
      this.error = "";
      const before = this.job?.id;
      const job = await action(this.#services);
      if (job.id !== before) {
        await this.load();
        await this.selectJob(job.id);
        this.tab = "sheet";
        this.announcement = "New job created with the changed white settings";
      } else {
        await this.refresh();
        this.announcement = message;
      }
      return job;
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      return undefined;
    }
  }

  setWhitePolicy(jobId: string, policy: string) {
    return this.editWhite((s) => s.setWhitePolicy(jobId, policy), "White policy changed");
  }

  setWhiteMask(jobId: string, mask: { source: string; invert: boolean }) {
    return this.editWhite((s) => s.setWhiteMask(jobId, mask), "White mask changed");
  }

  setWhiteChoke(jobId: string, chokeMm: number) {
    return this.editWhite((s) => s.setWhiteChoke(jobId, chokeMm), "White choke changed");
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
