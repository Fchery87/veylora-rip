<script lang="ts">
  import { ToggleGroup } from "bits-ui";

  interface Option {
    id: string;
    label: string;
    sublabel?: string;
    disabled?: boolean;
  }
  interface Props {
    label: string;
    options: Option[];
    value: string;
    onValueChange: (value: string) => void;
  }

  let { label, options, value, onValueChange }: Props = $props();

  // A single-select group reports "" when the active item is clicked again.
  function change(next: string) {
    if (next) onValueChange(next);
  }
</script>

<ToggleGroup.Root type="single" {value} onValueChange={change} aria-label={label} class="seg">
  {#each options as option (option.id)}
    <ToggleGroup.Item value={option.id} disabled={option.disabled} class="seg-item">
      <span>{option.label}</span>
      {#if option.sublabel}<span class="sub">{option.sublabel}</span>{/if}
    </ToggleGroup.Item>
  {/each}
</ToggleGroup.Root>

<style>
  :global(.seg) {
    display: inline-flex;
    gap: 4px;
    padding: 4px;
    border-radius: 8px;
    background: var(--subtle);
  }
  :global(.seg-item) {
    min-height: 28px;
    padding: 0 12px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--text-2);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
  }
  :global(.seg-item[data-state="on"]) {
    background: var(--surface);
    box-shadow: var(--shadow-1);
    color: var(--text);
  }
  :global(.seg-item[data-disabled]) {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .sub {
    font-size: 11.5px;
    line-height: 14px;
    color: var(--muted);
  }
</style>
