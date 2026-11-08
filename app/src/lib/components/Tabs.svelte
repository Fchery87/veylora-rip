<script lang="ts">
  import { Tabs } from "bits-ui";
  import type { Snippet } from "svelte";
  import type { JobTab } from "../jobTabs";

  interface Props {
    tabs: JobTab[];
    value: string;
    onValueChange: (value: string) => void;
    panel: Snippet<[string]>;
  }

  let { tabs, value, onValueChange, panel }: Props = $props();
</script>

<Tabs.Root {value} {onValueChange}>
  <Tabs.List class="tab-list" aria-label="Job sections">
    {#each tabs as tab (tab.id)}
      <Tabs.Trigger
        class="tab"
        value={tab.id}
        disabled={tab.disabled}
        title={tab.disabled ? tab.reason : undefined}
      >
        {tab.label}
        {#if tab.count}<span class="count">{tab.count}</span>{/if}
      </Tabs.Trigger>
    {/each}
  </Tabs.List>
  {#each tabs as tab (tab.id)}
    <Tabs.Content value={tab.id} class="tab-panel">
      <!-- Inactive panels stay unmounted so only one preview surface exists at a time. -->
      {#if tab.id === value}{@render panel(tab.id)}{/if}
    </Tabs.Content>
  {/each}
</Tabs.Root>

<style>
  :global(.tab-list) {
    display: flex;
    gap: 4px;
    padding: 0 24px;
    border-bottom: 1px solid var(--line);
    background: var(--surface);
  }
  :global(.tab) {
    height: 38px;
    padding: 0 12px;
    border: 0;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--muted);
    font-size: 13.5px;
    font-weight: 500;
    cursor: pointer;
    display: inline-flex;
    gap: 8px;
    align-items: center;
  }
  :global(.tab[data-state="active"]) {
    color: var(--accent-text);
    border-bottom-color: var(--accent);
  }
  :global(.tab[data-disabled]) {
    cursor: not-allowed;
    opacity: 0.55;
  }
  .count {
    min-width: 18px;
    height: 18px;
    padding: 0 4px;
    border-radius: 999px;
    background: var(--subtle);
    font-size: 11.5px;
    line-height: 18px;
  }
  :global(.tab-panel) {
    padding: 24px;
  }
</style>
