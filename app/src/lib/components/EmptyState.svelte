<script lang="ts">
  import type { Snippet } from "svelte";

  let { title, children }: { title: string; children?: Snippet } = $props();
</script>

<div class="empty">
  <h2>{title}</h2>
  {#if children}<p class="muted">{@render children()}</p>{/if}
</div>

<style>
  .empty {
    padding: 32px;
    display: grid;
    gap: 8px;
    text-align: center;
    justify-items: center;
  }
</style>
