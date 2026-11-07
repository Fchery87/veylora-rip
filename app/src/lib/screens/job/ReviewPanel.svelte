<script lang="ts">
  import Button from "../../components/Button.svelte";
  import type { AppStore } from "../../store.svelte";
  import type { JobDetail, ReviewOutcome } from "../../services/types";

  let { store, job }: { store: AppStore; job: JobDetail } = $props();

  const attempt = $derived(
    job.attempts.find((a) => a.outcome === "ambiguous" || a.outcome === "kept-in-review"),
  );

  interface Choice {
    outcome: ReviewOutcome;
    label: string;
    confirmLabel: string;
    effect: string;
  }

  const choices: Choice[] = [
    {
      outcome: "finished",
      label: "It printed correctly",
      confirmLabel: "Mark finished",
      effect: "The attempt will be recorded as finished by the operator. No new attempt is created.",
    },
    {
      outcome: "reprint",
      label: "Nothing printed, reprint",
      confirmLabel: "Create reprint attempt",
      effect: "The attempt will be recorded as not printed and a reprint attempt will be created held.",
    },
    {
      outcome: "printed-badly",
      label: "Printed badly, reprint",
      confirmLabel: "Create reprint attempt",
      effect: "The attempt will be recorded as printed badly and a reprint attempt will be created held.",
    },
    {
      outcome: "decide-later",
      label: "Decide later",
      confirmLabel: "Keep in review",
      effect: "The attempt stays in review. The printer remains blocked until it is resolved.",
    },
  ];

  let pending = $state<Choice | null>(null);

  async function confirm() {
    if (!pending || !attempt) return;
    const outcome = pending.outcome;
    pending = null;
    await store.resolveReview(attempt.id, outcome);
  }
</script>

<section class="card review">
  <h2>Check the printer before printing anything else</h2>
  <p>
    {#if attempt}
      Attempt <span class="mono">{attempt.id}</span> ended without a confirmed result. It may or may
      not have printed. Nothing can be sent to this printer until you record what happened.
    {:else}
      No attempt is waiting for review.
    {/if}
  </p>
  {#if attempt}
    <dl class="facts">
      <div><dt>Last signal</dt><dd class="mono">{attempt.at}</dd></div>
      <div><dt>Source</dt><dd>Fixture data</dd></div>
    </dl>
    <div class="choices">
      {#each choices as choice (choice.outcome)}
        <Button onclick={() => (pending = choice)}>{choice.label}</Button>
      {/each}
    </div>
    {#if pending}
      <div class="confirm">
        <p>{pending.effect}</p>
        <Button variant="primary" onclick={confirm}>{pending.confirmLabel}</Button>
        <Button onclick={() => (pending = null)}>Go back</Button>
      </div>
    {/if}
  {/if}
</section>

<style>
  .review {
    border-color: var(--review-fg);
    display: grid;
    gap: 12px;
    max-width: 720px;
  }
  .facts {
    display: flex;
    gap: 24px;
    margin: 0;
  }
  dt {
    font-size: 12px;
    color: var(--muted);
  }
  dd {
    margin: 0;
  }
  .choices {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .confirm {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 10px 12px;
    background: var(--review-bg);
    color: var(--review-fg);
    border-radius: 8px;
  }
  .confirm p {
    flex: 1;
  }
</style>
