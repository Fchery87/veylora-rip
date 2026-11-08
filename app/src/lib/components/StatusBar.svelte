<script lang="ts">
  import type { CapabilityState } from "../services/types";
  import Tag from "./Tag.svelte";

  let { caps }: { caps: CapabilityState } = $props();

  const setupText = $derived(
    caps.setup.experimental ? "Experimental mode" : caps.setup.verified ? "Verified" : "Unverified",
  );
  const setupTone = $derived(caps.setup.verified && !caps.setup.experimental ? "ok" : "warn");
</script>

<footer class="bar" aria-label="Device status">
  <span>ET-8550 · {caps.setup.exists ? caps.setup.name : "No device setup"}</span>
  <Tag tone={setupTone}>{setupText}</Tag>
  <span><span class="mono">{caps.mode.id}</span> {caps.mode.maturity}</span>
  <span>Sender: {caps.openReview ? "Blocked: review required" : "Idle"}</span>
  <span class="spacer"></span>
  {#if caps.dataSource === "fixture"}<Tag tone="warn">Fixture data</Tag>{/if}
</footer>

<style>
  .bar {
    height: 28px;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 0 16px;
    background: var(--surface);
    border-top: 1px solid var(--line);
    font-size: 12px;
    color: var(--text-2);
  }
  .spacer {
    flex: 1;
  }
</style>
