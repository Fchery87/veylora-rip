<script lang="ts">
  import { onMount } from "svelte";
  import Tag from "../components/Tag.svelte";
  import type { AppStore } from "../store.svelte";
  import type { RecipeModel } from "../services/types";

  let { store }: { store: AppStore } = $props();

  let recipes = $state<RecipeModel[]>([]);
  onMount(async () => {
    recipes = await store.services.listRecipes();
  });

  const maturity = { experimental: "Experimental", validated: "Validated", retired: "Retired" } as const;
  const tone = { experimental: "warn", validated: "ok", retired: "neutral" } as const;
</script>

<div class="page">
  <h1>Recipes</h1>
  <section class="card">
    <h2>Device mode</h2>
    {#if store.caps}
      <p>
        <span class="mono">{store.caps.mode.id}</span>
        <Tag tone={tone[store.caps.mode.maturity]}>{maturity[store.caps.mode.maturity]}</Tag>
        <span class="muted">Sheet sizes: {store.caps.mode.sheetSizes.join(", ")}</span>
      </p>
    {/if}
  </section>
  <section class="card">
    <h2>Recipes</h2>
    <ul aria-label="Recipes">
      {#each recipes as r (r.id)}
        <li>
          <span>{r.name}</span>
          <span class="mono">{r.id}</span>
          <Tag tone={tone[r.state]}>{maturity[r.state]}</Tag>
        </li>
      {/each}
    </ul>
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
</style>
