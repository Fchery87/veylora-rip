<script lang="ts">
  import type { ReviewOutcome, ReviewView } from "../services/types";
  import Button from "./Button.svelte";

  let {
    view,
    onResolve,
  }: { view: ReviewView | null; onResolve: (attemptId: string, outcome: ReviewOutcome) => void } =
    $props();

  let pending = $state<ReviewView["choices"][number] | null>(null);

  async function confirm() {
    if (!pending || !view) return;
    const outcome = pending.outcome;
    pending = null;
    onResolve(view.attempt.id, outcome);
  }
</script>

<section class="card review">
  <h2>{view?.headline ?? "Check the printer before printing anything else"}</h2>
  <p>{view?.body ?? "No attempt is waiting for review."}</p>
  {#if view}
    <dl class="facts">
      <div><dt>Last signal</dt><dd class="mono">{view.attempt.lastSignal}</dd></div>
      <div><dt>Source</dt><dd>{view.attempt.source}</dd></div>
    </dl>
    <div class="choices">
      {#each view.choices as choice (choice.outcome)}
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
    padding: 12px 12px;
    background: var(--review-bg);
    color: var(--review-fg);
    border-radius: 8px;
  }
  .confirm p {
    flex: 1;
  }
</style>
