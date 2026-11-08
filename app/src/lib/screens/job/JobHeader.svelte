<script lang="ts">
    import Button from "../../components/Button.svelte";
  import StatusBadge from "../../components/StatusBadge.svelte";
  import PreflightSummary from "../../components/PreflightSummary.svelte";
  import type { AppStore } from "../../store.svelte";
  import type {
    CapabilityState,
    JobActionsView,
    JobDetail,
    PreflightView,
  } from "../../services/types";

  let {
    store,
    job,
    caps,
    actions: commands,
    preflight,
  }: {
    store: AppStore;
    job: JobDetail;
    caps: CapabilityState;
    actions: JobActionsView;
    preflight: PreflightView;
  } = $props();

  let confirming = $state<"file-sink" | "printer" | null>(null);

  async function confirm() {
    const kind = confirming;
    confirming = null;
    if (kind === "file-sink") await store.writeToFileSink(job.id);
    if (kind === "printer") await store.releaseToPrinter(job.id);
  }
</script>

<div class="head" role="group" aria-label="Job header">
  <div class="row">
    <h1>{job.name}</h1>
    <StatusBadge state={job.state} progressPercent={job.progressPercent} />
    <span class="meta"><span class="mono">{job.id}</span> · {job.sheet} · {caps.mode.id}</span>
    <PreflightSummary summary={preflight.summary} />
    <span class="spacer"></span>

    <div class="action">
      {#if !commands.writeToFileSink.enabled && commands.writeToFileSink.reason && job.rendered}
        <span class="reason">{commands.writeToFileSink.reason}</span>
      {/if}
      <Button
        disabled={!commands.writeToFileSink.enabled}
        onclick={() => (confirming = "file-sink")}>Write to file sink</Button
      >
    </div>

    {#if !job.rendered}
      <div class="action">
        {#if commands.renderAndHold.reason}<span class="reason">{commands.renderAndHold.reason}</span>{/if}
        <Button
          variant="primary"
          disabled={!commands.renderAndHold.enabled}
          onclick={() => store.renderAndHold(job.id)}>Render and hold</Button
        >
      </div>
    {:else}
      <div class="action">
        {#if commands.releaseToPrinter.reason}<span class="reason">{commands.releaseToPrinter.reason}</span>{/if}
        <Button
          variant="primary"
          disabled={!commands.releaseToPrinter.enabled}
          onclick={() => (confirming = "printer")}>Release to printer</Button
        >
      </div>
    {/if}
  </div>

  {#if confirming === "file-sink"}
    <div class="confirm">
      <p>
        Write {job.id} to the file sink? This creates a file in the sink folder and does not print.
        <strong>Not physical output.</strong>
      </p>
      <Button variant="primary" size="sm" onclick={confirm}>Confirm write to file sink</Button>
      <Button size="sm" onclick={() => (confirming = null)}>Go back</Button>
    </div>
  {:else if confirming === "printer"}
    <div class="confirm">
      <p>Send {job.id} to ET-8550 · {caps.setup.name}? This creates a new submission attempt.</p>
      <Button variant="primary" size="sm" onclick={confirm}>Confirm release to printer</Button>
      <Button size="sm" onclick={() => (confirming = null)}>Go back</Button>
    </div>
  {/if}
  {#if store.error}<p class="error" role="status">{store.error}</p>{/if}
</div>

<style>
  .head {
    padding: 16px 24px 12px;
    background: var(--surface);
    border-bottom: 1px solid var(--line);
    display: grid;
    gap: 8px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  .meta {
    color: var(--muted);
    font-size: 12px;
  }
  .spacer {
    flex: 1;
  }
  .action {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .reason {
    font-size: 12px;
    color: var(--muted);
  }
  .confirm {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--subtle);
    border-radius: 8px;
    font-size: 13px;
  }
  .confirm p {
    flex: 1;
  }
  .error {
    color: var(--bad-fg);
    font-size: 13px;
  }
</style>
