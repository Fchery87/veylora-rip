<script lang="ts">
  import { stageChipText, type PreviewSource, type StageId } from "../status";

  let { stage, source }: { stage: StageId; source: PreviewSource } = $props();
</script>

<span class="chip {source}" data-testid="stage-chip">{stageChipText(stage, source)}</span>

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
