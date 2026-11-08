<script lang="ts">
  import type { JobDetail } from "../../services/types";

  let { job }: { job: JobDetail } = $props();
</script>

<dl class="card kv">
  {#each job.provenance as row (row.label)}
    <dt>{row.label}</dt>
    <dd class="mono">{row.value}</dd>
  {/each}
</dl>

<style>
  .kv {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 8px 16px;
    max-width: 720px;
    margin: 0;
  }
  dt {
    color: var(--muted);
  }
  dd {
    margin: 0;
  }
</style>
