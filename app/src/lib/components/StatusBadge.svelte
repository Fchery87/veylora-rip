<script lang="ts">
  import { stateBadge } from "../status";
  import type { JobState } from "../services/types";

  let { state, progressPercent }: { state: JobState; progressPercent?: number } = $props();
  const badge = $derived(stateBadge(state, progressPercent));
</script>

<span class="badge {badge.tone}"><span class="dot" aria-hidden="true"></span>{badge.text}</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 22px;
    padding: 0 10px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }
  .ok { color: var(--ok-fg); background: var(--ok-bg); }
  .info { color: var(--info-fg); background: var(--info-bg); }
  .warn { color: var(--warn-fg); background: var(--warn-bg); }
  .bad { color: var(--bad-fg); background: var(--bad-bg); }
  .review { color: var(--review-fg); background: var(--review-bg); }
  .neutral { color: var(--neutral-fg); background: var(--neutral-bg); }
</style>
