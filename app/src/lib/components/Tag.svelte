<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Tone } from "../status";

  let { tone = "neutral", children }: { tone?: Tone; children: Snippet } = $props();
</script>

<span class="tag {tone}">{@render children()}</span>

<style>
  .tag {
    display: inline-flex;
    align-items: center;
    height: 22px;
    padding: 0 10px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
  }
  .ok { color: var(--ok-fg); background: var(--ok-bg); }
  .info { color: var(--info-fg); background: var(--info-bg); }
  .warn { color: var(--warn-fg); background: var(--warn-bg); }
  .bad { color: var(--bad-fg); background: var(--bad-bg); }
  .review { color: var(--review-fg); background: var(--review-bg); }
  .neutral { color: var(--neutral-fg); background: var(--neutral-bg); }
</style>
