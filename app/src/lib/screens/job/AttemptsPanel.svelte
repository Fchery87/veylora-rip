<script lang="ts">
  import type { Attempt } from "../../services/types";
  import AttemptTimeline from "./AttemptTimeline.svelte";

  let { attempts, freshIds = [] }: { attempts: Attempt[]; freshIds?: string[] } = $props();
</script>

{#if attempts.length === 0}
  <p class="muted">
    No attempts yet. Render and hold the job, inspect it, then release it to the printer.
  </p>
{:else}
  <AttemptTimeline {attempts} {freshIds} />
  <p class="muted caption">
    Append-only. Reprints add a new attempt; nothing here is ever overwritten.
  </p>
{/if}

<style>
  .caption {
    font-size: 12px;
  }
</style>
