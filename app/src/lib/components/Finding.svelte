<script lang="ts">
  import type { Finding } from "../services/types";
  import Button from "./Button.svelte";
  import Tag from "./Tag.svelte";

  let { finding, onAcknowledge }: { finding: Finding; onAcknowledge: () => void } = $props();

  const tones = { blocking: "bad", confirm: "warn", info: "info" } as const;
  const labels = { blocking: "Blocking", confirm: "Confirm", info: "Info" } as const;
</script>

<article>
  <div class="line">
    <Tag tone={tones[finding.severity]}>{labels[finding.severity]}</Tag>
    <strong>{finding.title}</strong>
  </div>
  <p>{finding.cause}</p>
  <p class="fix"><span class="muted">Fix:</span> {finding.fix}</p>
  {#if finding.severity === "confirm"}
    {#if finding.acknowledged}
      <Tag tone="ok">Acknowledged</Tag>
    {:else}
      <Button size="sm" aria-label="Acknowledge finding" onclick={onAcknowledge}>Acknowledge</Button>
    {/if}
  {/if}
</article>

<style>
  article {
    display: grid;
    gap: 8px;
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
