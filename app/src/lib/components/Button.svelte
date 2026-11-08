<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    variant?: "primary" | "secondary" | "danger";
    size?: "default" | "sm";
    disabled?: boolean;
    type?: "button" | "submit";
    onclick?: (event: MouseEvent) => void;
    children: Snippet;
    [key: string]: unknown;
  }

  let {
    variant = "secondary",
    size = "default",
    disabled = false,
    type = "button",
    onclick,
    children,
    ...rest
  }: Props = $props();
</script>

<button class="btn {variant} {size}" {type} {disabled} {onclick} {...rest}>
  {@render children()}
</button>

<style>
  .btn {
    height: 34px;
    padding: 0 16px;
    border-radius: 8px;
    border: 1px solid var(--line-strong);
    background: var(--surface);
    color: var(--text-2);
    font-weight: 500;
    font-size: 13.5px;
    box-shadow: var(--shadow-1);
    cursor: pointer;
    transition: background 120ms ease-out;
  }
  .btn.sm {
    height: 30px;
    padding: 0 12px;
  }
  .btn:hover:not(:disabled) {
    background: var(--subtle);
  }
  .primary {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--on-accent);
  }
  .primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }
  .danger {
    color: var(--bad-fg);
  }
  .btn:disabled {
    cursor: not-allowed;
    opacity: 0.55;
    box-shadow: none;
  }
</style>
