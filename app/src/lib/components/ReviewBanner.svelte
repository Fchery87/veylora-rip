<script lang="ts">
  import AlertTriangle from "@lucide/svelte/icons/alert-triangle";
  import Button from "./Button.svelte";

  let {
    jobId,
    jobName,
    onReview,
  }: { jobId: string; jobName: string; onReview: () => void } = $props();
</script>

<!-- Not dismissable: it goes away only when the attempt is resolved. -->
<div class="banner" role="alert">
  <AlertTriangle size={18} strokeWidth={1.5} aria-hidden="true" />
  <p>
    <strong>Printer blocked.</strong>
    {jobId} · {jobName} may or may not have printed. Check the printer, then record what happened.
  </p>
  <Button size="sm" onclick={onReview}>Review now</Button>
</div>

<style>
  .banner {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 40px;
    padding: 4px 16px;
    background: var(--review-bg);
    color: var(--review-fg);
    border-left: 3px solid var(--review-fg);
    font-size: 13px;
  }
  p {
    flex: 1;
  }
</style>
