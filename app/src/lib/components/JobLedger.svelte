<script lang="ts">
  import type { JobSummary } from "../services/types";
  import { isReviewState } from "../status";
  import EmptyState from "./EmptyState.svelte";
  import StatusBadge from "./StatusBadge.svelte";

  interface Props {
    jobs: JobSummary[];
    selectedId: string | null;
    onSelect: (id: string) => void;
  }

  let { jobs, selectedId, onSelect }: Props = $props();

  // Ambiguous and needs-review jobs are always pinned to the top.
  const pinned = $derived(jobs.filter((j) => isReviewState(j.state)));
  const rest = $derived(jobs.filter((j) => !isReviewState(j.state)));
  const ordered = $derived([...pinned, ...rest]);

  function onKeydown(event: KeyboardEvent, index: number) {
    const move = event.key === "ArrowDown" ? 1 : event.key === "ArrowUp" ? -1 : 0;
    if (move) {
      event.preventDefault();
      const next = document.querySelectorAll<HTMLElement>("[data-ledger-row]")[index + move];
      next?.focus();
    } else if (event.key === "Enter") {
      onSelect(ordered[index].id);
    }
  }
</script>

<aside class="ledger" aria-label="Job ledger">
  <header>
    <h2>Jobs</h2>
  </header>
  {#if jobs.length === 0}
    <EmptyState title="No jobs yet">Import artwork to start a sheet.</EmptyState>
  {:else}
    <div role="listbox" aria-label="Jobs">
      {#each ordered as job, index (job.id)}
        {#if index === 0 && pinned.length > 0}<p class="group-title">Needs review</p>{/if}
        {#if index === pinned.length && pinned.length > 0 && rest.length > 0}
          <p class="group-title">Other jobs</p>
        {/if}
        <div
          role="option"
          aria-selected={selectedId === job.id}
          tabindex={selectedId === job.id || (selectedId === null && index === 0) ? 0 : -1}
          data-ledger-row
          class="row"
          class:selected={selectedId === job.id}
          onclick={() => onSelect(job.id)}
          onkeydown={(e) => onKeydown(e, index)}
        >
          <span class="line">
            <span class="name">{job.name}</span>
            <StatusBadge state={job.state} progressPercent={job.progressPercent} />
          </span>
          <span class="meta"><span class="mono">{job.id}</span> · {job.sheet} · {job.placementCount} placement{job.placementCount === 1 ? "" : "s"} · {job.updated}</span>
          {#if isReviewState(job.state)}
            <span class="attention">Printer result unknown. Review before printing anything else.</span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</aside>

<style>
  .ledger {
    width: 320px;
    background: var(--surface);
    border-right: 1px solid var(--line);
    overflow: auto;
  }
  header {
    padding: 16px;
  }
  .group-title {
    padding: 8px 16px 4px;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
  }
  .row {
    display: grid;
    gap: 2px;
    margin: 2px 8px;
    padding: 10px 12px;
    border-radius: 8px;
    cursor: pointer;
  }
  .row:hover {
    background: var(--subtle);
  }
  .row.selected {
    background: var(--accent-soft);
  }
  .line {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    align-items: center;
  }
  .name {
    font-size: 13.5px;
    font-weight: 500;
  }
  .meta {
    font-size: 12px;
    color: var(--muted);
  }
  .attention {
    font-size: 12px;
    color: var(--review-fg);
  }
</style>
