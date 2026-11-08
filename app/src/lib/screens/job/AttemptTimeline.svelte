<script lang="ts">
  import type { Attempt } from "../../services/types";

  let { attempts, freshIds = [] }: { attempts: Attempt[]; freshIds?: string[] } = $props();
</script>

<ol class="timeline" aria-label="Attempt history">
  {#each attempts as a (a.id)}
    <li class={a.outcome} data-fresh={freshIds.includes(a.id) ? "true" : undefined}>
      <span class="mono">{a.at}</span>
      <span class="mono">{a.id}</span>
      <span>{a.kind === "file-sink" ? "File sink" : "Printer"} · {a.outcome}</span>
      <span class="note">{a.note}</span>
      {#if a.sinkPath}<span class="mono path">{a.sinkPath}</span>{/if}
    </li>
  {/each}
</ol>

<style>
  .timeline {
    list-style: none;
    margin: 0 0 12px;
    padding: 0;
    display: grid;
    gap: 8px;
    max-width: 840px;
  }
  li {
    display: grid;
    grid-template-columns: 90px 110px 160px 1fr;
    gap: 12px;
    padding: 12px;
    background: var(--surface);
    border: 1px solid var(--line);
    border-left-width: 3px;
    border-radius: 8px;
    font-size: 13px;
  }
  li[data-fresh="true"] {
    background: var(--accent-soft);
  }
  li.ambiguous,
  li.kept-in-review,
  li.held {
    border-left-color: var(--review-fg);
  }
  li.finished {
    border-left-color: var(--ok-fg);
  }
  .path {
    grid-column: 4;
    color: var(--muted);
  }
  @media (prefers-reduced-motion: no-preference) {
    li {
      transition: background-color 200ms ease-out;
    }
  }
</style>
