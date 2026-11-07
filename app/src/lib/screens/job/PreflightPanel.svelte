<script lang="ts">
  import Button from "../../components/Button.svelte";
  import Tag from "../../components/Tag.svelte";
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
            <article>
              <div class="line">
                <Tag tone={group.tone}>{group.title}</Tag>
                <strong>{f.title}</strong>
              </div>
              <p>{f.cause}</p>
              <p class="fix"><span class="muted">Fix:</span> {f.fix}</p>
              {#if f.severity === "confirm"}
                {#if f.acknowledged}
                  <Tag tone="ok">Acknowledged</Tag>
                {:else}
                  <Button size="sm" aria-label="Acknowledge finding" onclick={() => store.acknowledgeFinding(job.id, f.id)}>
                    Acknowledge
                  </Button>
                {/if}
              {/if}
            </article>
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
  article {
    display: grid;
    gap: 6px;
    padding: 12px 0;
    border-top: 1px solid var(--line);
    justify-items: start;
  }
  .line {
    display: flex;
    gap: 8px;
    align-items: center;
  }
</style>
