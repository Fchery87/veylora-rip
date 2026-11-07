<script lang="ts">
  import { onMount } from "svelte";
  import Button from "../components/Button.svelte";
  import Callout from "../components/Callout.svelte";
  import Tag from "../components/Tag.svelte";
  import type { AppStore } from "../store.svelte";
  import type { ChannelRow, DeviceSetupModel } from "../services/types";

  let { store }: { store: AppStore } = $props();

  let setup = $state<DeviceSetupModel | null>(null);
  onMount(async () => {
    setup = await store.services.getDeviceSetup();
  });

  const evidence: Record<ChannelRow["evidence"], { text: string; tone: "warn" | "ok" | "bad" }> = {
    "not-tested": { text: "Not tested", tone: "warn" },
    matched: { text: "Matched", tone: "ok" },
    mismatch: { text: "Mismatch", tone: "bad" },
  };
  const untested = $derived(setup?.channels.filter((c) => c.evidence !== "matched").length ?? 0);
</script>

<div class="page">
  <h1>Devices</h1>
  {#if setup}
    <section class="card">
      <div class="head">
        <h2>{setup.name} · rev {setup.revision}</h2>
        <Tag tone={setup.verified ? "ok" : "warn"}>{setup.verified ? "Verified" : "Unverified"}</Tag>
      </div>
      <Callout tone="warn">
        Channel identity is unverified. {untested} channel{untested === 1 ? "" : "s"} still need a
        matched print record. Inferred values are labeled.
      </Callout>
      <table aria-label="Channel map">
        <thead>
          <tr><th>Logical ink</th><th>Physical slot</th><th>Protocol channel</th><th>Evidence</th></tr>
        </thead>
        <tbody>
          {#each setup.channels as row (row.logicalInk)}
            <tr>
              <td>{row.logicalInk}</td>
              <td>{row.physicalSlot}{row.inferred ? " (inferred)" : ""}</td>
              <td class="mono">{row.protocolChannel}</td>
              <td><Tag tone={evidence[row.evidence].tone}>{evidence[row.evidence].text}</Tag></td>
            </tr>
          {/each}
        </tbody>
      </table>
      <div class="actions">
        <Button disabled>Generate channel identity target</Button>
        <Button disabled>Record test result</Button>
        <span class="muted reason">Available when backend integration is connected. Fixture data.</span>
      </div>
    </section>
  {/if}
</div>

<style>
  .card {
    display: grid;
    gap: 16px;
    max-width: 840px;
  }
  .head {
    display: flex;
    gap: 12px;
    align-items: center;
  }
  table {
    border-collapse: collapse;
    width: 100%;
    font-size: 13px;
  }
  th,
  td {
    text-align: left;
    padding: 8px;
    border-bottom: 1px solid var(--line);
  }
  th {
    color: var(--muted);
    font-weight: 500;
  }
  .actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .reason {
    font-size: 12px;
  }
</style>
