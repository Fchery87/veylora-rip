<script lang="ts">
  import Button from "../../components/Button.svelte";
  import type { AppStore } from "../../store.svelte";
  import type { CreationOptions, NewJobRequest } from "../../services/types";

  let { store, options }: { store: AppStore; options: CreationOptions } = $props();

  // Choices come from what is stored. Modes belong to one setup, so they follow the setup.
  // svelte-ignore state_referenced_locally
  let setup = $state(options.setups[0]?.hash ?? "");
  const setupId = $derived(options.setups.find((s) => s.hash === setup)?.id);
  const modes = $derived(options.modes.filter((m) => m.setupId === setupId));
  let modeChoice = $state("");
  let recipeChoice = $state("");
  const recipes = $derived(options.recipes.filter((r) => !r.retired));
  // A select bound to a value with no matching option shows blank, so keep each choice valid.
  $effect(() => {
    if (!modes.some((m) => m.hash === modeChoice)) modeChoice = modes[0]?.hash ?? "";
  });
  $effect(() => {
    if (!recipes.some((r) => r.hash === recipeChoice)) recipeChoice = recipes[0]?.hash ?? "";
  });
  const mode = $derived(modeChoice);
  const recipe = $derived(recipeChoice);

  let artworkPath = $state("");
  let pageWidthMm = $state(297);
  let pageHeightMm = $state(420);
  let xMm = $state(0);
  let yMm = $state(0);
  let widthMm = $state(100);
  let heightMm = $state(100);
  let rotationDeg = $state<0 | 90 | 180 | 270>(0);
  let mirror = $state(false);
  let busy = $state(false);

  const missing = $derived(
    options.setups.length === 0 || options.modes.length === 0 || options.recipes.length === 0,
  );
  const canSubmit = $derived(!busy && !missing && artworkPath.trim() !== "" && !!setup && !!mode && !!recipe);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    const request: NewJobRequest = {
      artworkPath: artworkPath.trim(),
      setup,
      mode,
      recipe,
      pageWidthMm,
      pageHeightMm,
      xMm,
      yMm,
      widthMm,
      heightMm,
      rotationDeg,
      mirror,
    };
    await store.createJob(request);
    busy = false;
  }

  async function browse() {
    const chosen = await store.pickArtwork();
    if (chosen) artworkPath = chosen;
  }
</script>

<form class="new" onsubmit={submit} aria-label="New job">
  <h2>New job</h2>
  <p class="muted">
    Place one artwork on a sheet. The job is a draft: nothing is rendered or sent until you
    preflight and render it.
  </p>

  {#if missing}
    <p class="warn" role="status">
      This workspace has no {options.setups.length === 0
        ? "device setup"
        : options.modes.length === 0
          ? "print mode"
          : "recipe"} yet. Import one first (for example with the command line), then come back.
    </p>
  {/if}

  <div class="pick">
    <label>
      Artwork file (PNG path)
      <input type="text" bind:value={artworkPath} placeholder="C:\Art\logo.png" spellcheck="false" />
    </label>
    {#if store.canPickArtwork}
      <Button onclick={browse} disabled={busy}>Browse…</Button>
    {/if}
  </div>

  <div class="grid">
    <label>
      Device setup
      <select bind:value={setup}>
        {#each options.setups as s (s.hash)}
          <option value={s.hash}>{s.id}{s.verified ? "" : " (unverified)"}</option>
        {/each}
      </select>
    </label>
    <label>
      Print mode
      <select bind:value={modeChoice}>
        {#each modes as m (m.hash)}
          <option value={m.hash}>{m.id} ({m.maturity})</option>
        {/each}
      </select>
    </label>
    <label>
      Recipe
      <select bind:value={recipeChoice}>
        {#each recipes as r (r.hash)}
          <option value={r.hash}>{r.id} rev {r.revision}</option>
        {/each}
      </select>
    </label>
  </div>

  <fieldset>
    <legend>Sheet (mm)</legend>
    <label>Width <input type="number" step="any" min="0" bind:value={pageWidthMm} /></label>
    <label>Height <input type="number" step="any" min="0" bind:value={pageHeightMm} /></label>
  </fieldset>
  <fieldset>
    <legend>Artwork placement (mm)</legend>
    <label>X <input type="number" step="any" bind:value={xMm} /></label>
    <label>Y <input type="number" step="any" bind:value={yMm} /></label>
    <label>Width <input type="number" step="any" min="0" bind:value={widthMm} /></label>
    <label>Height <input type="number" step="any" min="0" bind:value={heightMm} /></label>
    <label>
      Rotation
      <select bind:value={rotationDeg}>
        {#each [0, 90, 180, 270] as r (r)}<option value={r}>{r}°</option>{/each}
      </select>
    </label>
    <label class="check"><input type="checkbox" bind:checked={mirror} /> Mirror the sheet</label>
  </fieldset>

  {#if store.error}<p class="error" role="alert">{store.error}</p>{/if}

  <div class="actions">
    <Button variant="primary" type="submit" disabled={!canSubmit}>Create job</Button>
    <Button onclick={() => store.cancelCreating()}>Cancel</Button>
  </div>
</form>

<style>
  .new {
    display: grid;
    gap: 14px;
    max-width: 640px;
    padding: 20px 24px;
  }
  .pick {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
    align-items: end;
  }
  label {
    display: grid;
    gap: 4px;
    font-size: 13px;
  }
  .check {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 12px;
  }
  fieldset {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 12px;
    border: 1px solid var(--line);
    border-radius: 6px;
    padding: 10px 12px;
  }
  input[type="text"],
  input[type="number"],
  select {
    width: 100%;
    min-width: 0;
    box-sizing: border-box;
    font: inherit;
    padding: 6px 8px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--surface);
    color: inherit;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  .muted {
    color: var(--muted);
    margin: 0;
  }
  .error {
    color: var(--bad, #b42318);
    margin: 0;
  }
  .warn {
    margin: 0;
  }
</style>
