<script lang="ts">
  import { onMount } from "svelte";
  import { deriveJobTabs } from "../jobTabs";
  import Tabs from "../components/Tabs.svelte";
  import JobLedger from "../components/JobLedger.svelte";
  import EmptyState from "../components/EmptyState.svelte";
  import type { AppStore } from "../store.svelte";
  import type { ArtworkCandidate } from "../services/types";
  import AttemptsPanel from "./job/AttemptsPanel.svelte";
  import ImportPanel from "./job/ImportPanel.svelte";
  import InspectPanel from "./job/InspectPanel.svelte";
  import JobHeader from "./job/JobHeader.svelte";
  import PreflightPanel from "./job/PreflightPanel.svelte";
  import ProvenancePanel from "./job/ProvenancePanel.svelte";
  import ReviewPanel from "./job/ReviewPanel.svelte";
  import SheetPanel from "./job/SheetPanel.svelte";
  import type { JobTabId } from "../jobTabs";

  let { store }: { store: AppStore } = $props();

  let candidates = $state<ArtworkCandidate[]>([]);
  onMount(async () => {
    candidates = await store.queries.listArtworkCandidates();
  });

  const job = $derived(store.job);
  const caps = $derived(store.caps);
  const tabs = $derived(job ? deriveJobTabs(job, store.views?.preflight.openCount) : []);
</script>

<div class="jobs">
  <JobLedger jobs={store.jobs} selectedId={job?.id ?? null} onSelect={(id) => store.selectJob(id)} />
  <section class="detail" aria-label="Job detail">
    {#if job && caps && store.views}
      {@const views = store.views}
      <JobHeader {store} {job} {caps} actions={views.actions} preflight={views.preflight} />
      <Tabs {tabs} value={store.tab} onValueChange={(v) => (store.tab = v as JobTabId)}>
        {#snippet panel(id)}
          {#if id === "review"}<ReviewPanel {store} view={views.review} />
          {:else if id === "import"}<ImportPanel {candidates} />
          {:else if id === "sheet"}<SheetPanel {store} {job} {caps} />
          {:else if id === "inspect"}<InspectPanel {store} {job} />
          {:else if id === "preflight"}<PreflightPanel {store} {job} view={views.preflight} />
          {:else if id === "attempts"}<AttemptsPanel attempts={job.attempts} freshIds={store.freshAttemptIds} />
          {:else if id === "provenance"}<ProvenancePanel {job} />
          {/if}
        {/snippet}
      </Tabs>
    {:else}
      <EmptyState title="Select a job">Choose a job from the ledger to see its sheet, findings, and attempts.</EmptyState>
    {/if}
  </section>
</div>

<style>
  .jobs {
    display: flex;
    min-height: 0;
    flex: 1;
  }
  .detail {
    flex: 1;
    overflow: auto;
    min-width: 0;
  }
</style>
