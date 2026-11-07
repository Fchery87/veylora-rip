<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    tone = "warn",
    children,
  }: { tone?: "warn" | "bad" | "review"; children: Snippet } = $props();
</script>

<div class="callout {tone}">{@render children()}</div>

<style>
  .callout {
    padding: 12px 12px;
    border-radius: 8px;
    border-left: 3px solid currentColor;
    font-size: 13px;
  }
  .warn { color: var(--warn-fg); background: var(--warn-bg); }
  .bad { color: var(--bad-fg); background: var(--bad-bg); }
  .review { color: var(--review-fg); background: var(--review-bg); }
</style>
