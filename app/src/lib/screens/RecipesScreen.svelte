<script lang="ts">
  import { onMount } from "svelte";
  import Button from "../components/Button.svelte";
  import Tag from "../components/Tag.svelte";
  import type { AppStore } from "../store.svelte";
  import type { DeviceModeModel, RecipeModel } from "../services/types";

  let { store }: { store: AppStore } = $props();

  let recipes = $state<RecipeModel[]>([]);
  let modes = $state<DeviceModeModel[]>([]);

  async function load() {
    [recipes, modes] = await Promise.all([
      store.services.listRecipes(),
      store.services.listDeviceModes(),
    ]);
  }
  onMount(load);

  const maturity = {
    experimental: "Experimental",
    validated: "Validated",
    retired: "Retired",
  } as const;
  const tone = { experimental: "warn", validated: "ok", retired: "neutral" } as const;

  async function chooseMode(id: string) {
    await store.command(() => store.services.selectDeviceMode(id), "Device mode changed");
    await load();
  }

  async function chooseRecipe(id: string) {
    await store.command(() => store.services.selectRecipe(id), "Recipe selected");
    await load();
  }
</script>

<div class="page">
  <h1>Recipes</h1>
  <section class="card">
    <h2>Device mode</h2>
    {#if store.caps}
      <p>
        <label>
          <span class="sr-only">Device mode</span>
          <select
            aria-label="Device mode"
            value={store.caps.mode.id}
            onchange={(e) => chooseMode(e.currentTarget.value)}
          >
            {#each modes as m (m.id)}
              <option value={m.id}>{m.id} ({maturity[m.maturity].toLowerCase()})</option>
            {/each}
          </select>
        </label>
        <Tag tone={tone[store.caps.mode.maturity]}>{maturity[store.caps.mode.maturity]}</Tag>
        <span class="muted">Sheet sizes: {store.caps.mode.sheetSizes.join(", ")}</span>
      </p>
    {/if}
  </section>
  <section class="card">
    <h2>Recipes</h2>
    <ul aria-label="Recipes">
      {#each recipes as r (r.id)}
        {@const fits = r.modeId === store.caps?.mode.id}
        <li>
          <span>{r.name}</span>
          <span class="mono">{r.id}</span>
          <Tag tone={tone[r.state]}>{maturity[r.state]}</Tag>
          {#if r.selected}
            <Tag tone="info">Selected</Tag>
          {:else}
            <Button
              size="sm"
              aria-label="Use {r.name}"
              aria-describedby={fits ? undefined : `why-${r.id}`}
              disabled={!fits}
              onclick={() => chooseRecipe(r.id)}>Use recipe</Button
            >
            {#if !fits}<span id="why-{r.id}" class="muted reason">Requires mode {r.modeId}</span>{/if}
          {/if}
        </li>
      {/each}
    </ul>
    {#if store.error}<p class="error" role="status">{store.error}</p>{/if}
  </section>
</div>

<style>
  ul {
    list-style: none;
    margin: 8px 0 0;
    padding: 0;
    display: grid;
    gap: 8px;
  }
  li {
    display: flex;
    gap: 12px;
    align-items: center;
  }
  .reason {
    font-size: 12px;
  }
  .error {
    color: var(--bad-fg);
    font-size: 13px;
  }
</style>
