<script lang="ts">
  import type { PreviewSource } from "../status";

  /** `text` is the finished label. Rust builds it for Inspect. */
  let { source, text }: { source: PreviewSource; text: string } = $props();
</script>

<span class="chip {source}" data-testid="stage-chip">{text}</span>

<style>
  .chip {
    display: inline-block;
    padding: 4px 12px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 500;
    max-width: 100%;
  }
  .simulation,
  .fixture {
    color: var(--warn-fg);
    background: var(--warn-bg);
  }
  .retained {
    color: var(--neutral-fg);
    background: var(--neutral-bg);
  }
</style>
