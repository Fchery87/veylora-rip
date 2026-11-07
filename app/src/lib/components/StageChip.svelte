<script lang="ts">
  import { stageChipText, type PreviewSource, type StageId } from "../status";

  let {
    stage,
    source,
    fixture,
  }: { stage: StageId; source: PreviewSource; fixture: boolean } = $props();
</script>

<span class="chip {source}" data-testid="stage-chip">{stageChipText(stage, source, fixture)}</span>

<style>
  .chip {
    display: inline-block;
    padding: 3px 10px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 500;
    max-width: 100%;
  }
  .simulation { color: var(--warn-fg); background: var(--warn-bg); }
  .retained { color: var(--neutral-fg); background: var(--neutral-bg); }
</style>
