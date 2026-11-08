<script lang="ts">
  import { stageChipText, type PreviewSource, type StageId } from "../status";

  /** `text` is the label built by Rust; the local text is the mocked fallback. */
  let { stage, source, text }: { stage: StageId; source: PreviewSource; text?: string } = $props();
</script>

<span class="chip {source}" data-testid="stage-chip">{text ?? stageChipText(stage, source)}</span>

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
