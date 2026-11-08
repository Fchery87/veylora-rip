<script lang="ts">
  import type { PreflightView } from "../services/types";
  import Tag from "./Tag.svelte";

  let { summary }: { summary: PreflightView["summary"] } = $props();
</script>

<Tag tone={summary.tone}>{summary.text}</Tag>
