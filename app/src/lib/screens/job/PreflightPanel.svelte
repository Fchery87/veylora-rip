<script lang="ts">
  import FindingItem from "../../components/Finding.svelte";
  import type { AppStore } from "../../store.svelte";
  import type { JobDetail, Severity } from "../../services/types";

  let { store, job }: { store: AppStore; job: JobDetail } = $props();

  const groups: { severity: Severity; title: string; tone: "bad" | "warn" | "info" }[] = [
    { severity: "blocking", title: "Blocking", tone: "bad" },
    { severity: "confirm", title: "Confirm", tone: "warn" },
    { severity: "info", title: "Info", tone: "info" },
  ];
</script>

{#if job.findings.length === 0}
  <p class="muted">No findings. Preflight runs continuously while you edit.</p>
{:else}
  <div class="stack">
    {#each groups as group (group.severity)}
      {@const items = job.findings.filter((f) => f.severity === group.severity)}
      {#if items.length > 0}
        <section class="card" aria-label={group.title}>
          <h2>{group.title}</h2>
          {#each items as f (f.id)}
            <FindingItem finding={f} onAcknowledge={() => store.acknowledgeFinding(job.id, f.id)} />
          {/each}
        </section>
      {/if}
    {/each}
  </div>
{/if}

<style>
  .stack {
    display: grid;
    gap: 12px;
    max-width: 720px;
  }
</style>
