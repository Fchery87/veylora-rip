<script lang="ts">
  import ReviewCard from "../../components/ReviewCard.svelte";
  import type { AppStore } from "../../store.svelte";
  import type { JobDetail } from "../../services/types";

  let { store, job }: { store: AppStore; job: JobDetail } = $props();

  const attempt = $derived(
    job.attempts.find((a) => a.outcome === "ambiguous" || a.outcome === "kept-in-review"),
  );
</script>

<ReviewCard {attempt} onResolve={(id, outcome) => store.resolveReview(id, outcome)} />
