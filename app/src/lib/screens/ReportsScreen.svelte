<script lang="ts">
  import Button from "../components/Button.svelte";
  import type { AppStore } from "../store.svelte";
  import type { DiagnosticExportResult } from "../services/types";

  let { store }: { store: AppStore } = $props();

  let redact = $state(true);
  let omitArtwork = $state(true);
  let result = $state<DiagnosticExportResult | null>(null);
  let error = $state("");

  async function exportReport() {
    try {
      error = "";
      result = await store.services.exportDiagnosticReport({ redact, omitArtwork });
      store.announcement = "Diagnostic report exported";
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<div class="page">
  <h1>Reports</h1>
  <section class="card">
    <h2>Diagnostic report</h2>
    <p class="muted">
      Exports a support bundle through the application service. Fixture data: the bundle contains no
      real printer evidence.
    </p>
    <label><input type="checkbox" bind:checked={redact} /> Redact file paths and names</label>
    <label><input type="checkbox" bind:checked={omitArtwork} /> Omit artwork</label>
    <div>
      <Button variant="primary" onclick={exportReport}>Export diagnostic report</Button>
    </div>
    {#if result}
      <div role="status">
        <p>Exported to <span class="mono">{result.path}</span></p>
        <ul aria-label="Included files">
          {#each result.files as f (f)}<li class="mono">{f}</li>{/each}
        </ul>
      </div>
    {/if}
    {#if error}<p class="error" role="status">{error}</p>{/if}
  </section>
</div>

<style>
  .card {
    display: grid;
    gap: 12px;
    max-width: 640px;
  }
  label {
    display: flex;
    gap: 8px;
    align-items: center;
    font-size: 13px;
  }
  .error {
    color: var(--bad-fg);
  }
</style>
