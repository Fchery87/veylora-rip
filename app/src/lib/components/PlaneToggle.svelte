<script lang="ts">
  import { Toggle } from "bits-ui";

  interface Props {
    plane: "C" | "M" | "Y" | "K" | "W";
    pressed: boolean;
    onPressedChange: (pressed: boolean) => void;
    disabled?: boolean;
  }

  let { plane, pressed, onPressedChange, disabled = false }: Props = $props();
</script>

<Toggle.Root {pressed} {onPressedChange} {disabled} aria-label="Plane {plane}" class="plane">
  <span>{plane}</span>
  <span class="ink ink-{plane}" aria-hidden="true"></span>
</Toggle.Root>

<style>
  :global(.plane) {
    width: 36px;
    padding: 4px 0 8px;
    border: 1px solid var(--line-strong);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text-2);
    font-weight: 600;
    display: grid;
    gap: 4px;
    justify-items: center;
    cursor: pointer;
  }
  :global(.plane[data-state="on"]) {
    background: var(--accent-soft);
    border-color: var(--accent);
  }
  :global(.plane[data-disabled]) {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .ink {
    width: 20px;
    height: 4px;
    border-radius: 2px;
  }
  .ink-C { background: var(--ink-c); }
  .ink-M { background: var(--ink-m); }
  .ink-Y { background: var(--ink-y); }
  .ink-K { background: var(--ink-k); }
  .ink-W { background: var(--ink-w); outline: 1px solid var(--line-strong); }
</style>
