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
    <dl class="facts" aria-label="Facts">
      {#each view.facts as fact (fact.label)}
        <div><dt>{fact.label}</dt><dd class="mono">{fact.value}</dd></div>
      {/each}
    </dl>
    <section class="sent" aria-label="What was sent">
      <h3>What was sent</h3>
      <dl class="facts">
        <div><dt>Artifact hash</dt><dd class="mono">{view.whatWasSent.artifactHash}</dd></div>
        <div><dt>Artifact size</dt><dd class="mono">{view.whatWasSent.artifactSize}</dd></div>
        <div><dt>Snapshot hash</dt><dd class="mono">{view.whatWasSent.snapshotHash}</dd></div>
      </dl>
      <ul>
        {#each view.whatWasSent.placements as placement (placement)}<li>{placement}</li>{/each}
      </ul>
    </section>
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
    flex-wrap: wrap;
    gap: 8px 24px;
    margin: 0;
  }
  .sent {
    display: grid;
    gap: 8px;
  }
  .sent h3 {
    margin: 0;
    font-size: 13px;
  }
  .sent ul {
    margin: 0;
    padding-left: 16px;
    font-size: 13px;
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
