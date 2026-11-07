<script lang="ts">
  import { onMount } from "svelte";
  import Button from "../components/Button.svelte";
  import Callout from "../components/Callout.svelte";
  import Tag from "../components/Tag.svelte";
  import type { AppStore } from "../store.svelte";
  import type { ChannelRow, ChannelTestResult, DeviceSetupModel } from "../services/types";

  let { store }: { store: AppStore } = $props();

  let setup = $state<DeviceSetupModel | null>(null);
  let newName = $state("");
  let testInk = $state("C");
  let testResult = $state<ChannelTestResult>("matched");

  onMount(async () => {
    setup = await store.services.getDeviceSetup();
  });

  const evidence: Record<ChannelRow["evidence"], { text: string; tone: "warn" | "ok" | "bad" }> = {
    "not-tested": { text: "Not tested", tone: "warn" },
    matched: { text: "Matched", tone: "ok" },
    mismatch: { text: "Mismatch", tone: "bad" },
  };
  const untested = $derived(setup?.channels.filter((c) => c.evidence !== "matched").length ?? 0);

  async function create() {
    const name = newName.trim();
    if (!name) return;
    const created = await store.command(
      () => store.services.createDeviceSetup(name),
      "Device setup created",
    );
    if (created) setup = created;
  }

  async function record() {
    const updated = await store.command(
      () => store.services.recordChannelTest(testInk, testResult),
      "Channel test result recorded",
    );
    if (updated) setup = updated;
  }
</script>

<div class="page">
  <h1>Devices</h1>
  {#if setup}
    <section class="card">
      <div class="head">
        <h2>{setup.name} · rev {setup.revision}</h2>
        <Tag tone={setup.verified ? "ok" : "warn"}>{setup.verified ? "Verified" : "Unverified"}</Tag>
      </div>
      {#if !setup.verified}
        <Callout tone="warn">
          Channel identity is unverified. {untested} channel{untested === 1 ? "" : "s"} still need a
          matched print record. Inferred values are labeled.
        </Callout>
      {/if}
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
        <label>
          Channel
          <select bind:value={testInk}>
            {#each setup.channels as row (row.logicalInk)}
              <option value={row.logicalInk}>{row.logicalInk}</option>
            {/each}
          </select>
        </label>
        <label>
          Result
          <select bind:value={testResult}>
            <option value="matched">matched</option>
            <option value="mismatch">mismatch</option>
          </select>
        </label>
        <Button onclick={record}>Record test result</Button>
      </div>
      <div class="actions">
        <Button disabled>Generate channel identity target</Button>
        <span class="muted reason">
          Target generation arrives with the backend. Test results above are fixture data.
        </span>
      </div>
    </section>
  {:else}
    <section class="card">
      <h2>No device setup yet</h2>
      <p class="muted">Create a setup to start channel verification. Targets and recipes unlock after this.</p>
      <div class="actions">
        <label>
          Setup name
          <input type="text" bind:value={newName} />
        </label>
        <Button variant="primary" disabled={!newName.trim()} onclick={create}>Create setup</Button>
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
  label {
    display: grid;
    gap: 4px;
    font-size: 12px;
    color: var(--muted);
  }
  .reason {
    font-size: 12px;
  }
</style>
