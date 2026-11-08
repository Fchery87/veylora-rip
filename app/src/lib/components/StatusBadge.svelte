<script lang="ts">
  import { stateBadge } from "../status";
  import type { JobState } from "../services/types";
  import Tag from "./Tag.svelte";

  let { state, progressPercent }: { state: JobState; progressPercent?: number } = $props();
  const badge = $derived(stateBadge(state, progressPercent));
</script>

<Tag tone={badge.tone} dot>{badge.text}</Tag>
