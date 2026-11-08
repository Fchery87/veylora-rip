<script lang="ts">
  import ReviewCard from "../../components/ReviewCard.svelte";
  import type { AppStore } from "../../store.svelte";
  import type { ReviewView } from "../../services/types";

  let { store, view }: { store: AppStore; view: ReviewView | null } = $props();
</script>

<ReviewCard {view} onResolve={(id, outcome) => store.resolveReview(id, outcome)} />
