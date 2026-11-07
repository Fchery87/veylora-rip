<script lang="ts">
  import EmptyState from "../components/EmptyState.svelte";
</script>

<div class="page">
  <h1>Settings</h1>
  <section class="card">
    <EmptyState title="Storage settings are not connected">
      Disk thresholds and retention need backend integration. Nothing here changes output.
    </EmptyState>
  </section>
</div>
