<script lang="ts">
  import FindingItem from "../../components/Finding.svelte";
  import type { AppStore } from "../../store.svelte";
  import type { JobDetail, PreflightView } from "../../services/types";

  let { store, job, view }: { store: AppStore; job: JobDetail; view: PreflightView } = $props();
</script>

<p class="muted checked">{view.checkedAgainst}</p>
{#if view.groups.length === 0}
  <p class="muted">No findings. Preflight runs continuously while you edit.</p>
{:else}
  <div class="stack">
    {#each view.groups as group (group.severity)}
      <section class="card" aria-label={group.title}>
        <h2>{group.title}</h2>
        {#each group.findings as f (f.id)}
          <FindingItem finding={f} onAcknowledge={() => store.acknowledgeFinding(job.id, f.id)} />
        {/each}
      </section>
    {/each}
  </div>
{/if}

<style>
  .stack {
    display: grid;
    gap: 12px;
    max-width: 720px;
  }
  .checked {
    margin: 0 0 12px;
  }
</style>
