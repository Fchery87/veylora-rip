<script lang="ts">
  import { onMount } from "svelte";
  import Button from "../components/Button.svelte";
  import type { AppStore } from "../store.svelte";
  import type { TargetModel } from "../services/types";

  let { store }: { store: AppStore } = $props();

  let targets = $state<TargetModel[]>([]);
  onMount(async () => {
    targets = await store.queries.listTargets();
  });
</script>

<div class="page">
  <h1>Targets</h1>
  <p class="muted">Diagnostic targets. Generating one creates a job that follows the normal flow.</p>
  <ul aria-label="Diagnostic targets">
    {#each targets as t (t.id)}
      <li class="card">
        <h2>{t.name}</h2>
        <p class="muted">{t.description}</p>
        <Button disabled>Generate target</Button>
      </li>
    {/each}
  </ul>
  <p class="muted">Generating targets needs backend integration. Fixture data.</p>
</div>

<style>
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 16px;
  }
  li {
    display: grid;
    gap: 8px;
    justify-items: start;
  }
</style>
