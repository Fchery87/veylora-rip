<script lang="ts">
  import { onMount } from "svelte";
  import { deriveNavigation, type SectionId } from "./lib/capability";
  import ReviewBanner from "./lib/components/ReviewBanner.svelte";
  import SectionRail from "./lib/components/SectionRail.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import DevicesScreen from "./lib/screens/DevicesScreen.svelte";
  import JobsScreen from "./lib/screens/JobsScreen.svelte";
  import RecipesScreen from "./lib/screens/RecipesScreen.svelte";
  import ReportsScreen from "./lib/screens/ReportsScreen.svelte";
  import SettingsScreen from "./lib/screens/SettingsScreen.svelte";
  import TargetsScreen from "./lib/screens/TargetsScreen.svelte";
  import type { AppServices } from "./lib/services/types";
  import { AppStore } from "./lib/store.svelte";

  let { services }: { services: AppServices } = $props();

  // svelte-ignore state_referenced_locally
  const store = new AppStore(services);
  onMount(() => store.load());

  const caps = $derived(store.caps);
  const nav = $derived(caps ? deriveNavigation(caps) : []);

  function select(id: SectionId) {
    store.section = id;
  }
</script>

{#if caps}
  <div class="app">
    <SectionRail
      items={nav}
      current={store.section}
      theme={store.theme}
      onSelect={select}
      onTheme={(t) => store.setTheme(t)}
    />
    <div class="column">
      {#if caps.openReview}
        <ReviewBanner
          jobId={caps.openReview.jobId}
          jobName={caps.openReview.jobName}
          onReview={() => store.openReview()}
        />
      {/if}
      <main>
        {#if store.section === "jobs"}<JobsScreen {store} />
        {:else if store.section === "devices"}<DevicesScreen {store} />
        {:else if store.section === "recipes"}<RecipesScreen {store} />
        {:else if store.section === "targets"}<TargetsScreen {store} />
        {:else if store.section === "reports"}<ReportsScreen {store} />
        {:else}<SettingsScreen />
        {/if}
      </main>
    </div>
    <StatusBar {caps} />
    <div class="sr-only" aria-live="polite">{store.announcement}</div>
  </div>
{/if}

<style>
  .app {
    height: 100%;
    display: grid;
    grid-template-columns: 56px 1fr;
    grid-template-rows: 1fr 28px;
  }
  .app > :global(footer) {
    grid-column: 1 / -1;
  }
  .column {
    display: flex;
    flex-direction: column;
    min-height: 0;
    min-width: 0;
  }
  main {
    flex: 1;
    display: flex;
    min-height: 0;
    overflow: auto;
  }
  main > :global(.page) {
    flex: 1;
  }
</style>
