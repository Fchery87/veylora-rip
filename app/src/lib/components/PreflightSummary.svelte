<script lang="ts">
  import type { JobDetail } from "../services/types";
  import Tag from "./Tag.svelte";

  let { job }: { job: JobDetail } = $props();

  const blocking = $derived(job.findings.filter((f) => f.severity === "blocking").length);
  const toConfirm = $derived(
    job.findings.filter((f) => f.severity === "confirm" && !f.acknowledged).length,
  );
  const summary = $derived.by(() => {
    if (blocking > 0) {
      const parts = [`${blocking} blocking`];
      if (toConfirm > 0) parts.push(`${toConfirm} to confirm`);
      return { text: parts.join(" · "), tone: "bad" as const };
    }
    if (toConfirm > 0) return { text: `${toConfirm} to confirm`, tone: "warn" as const };
    if (job.rendered) return { text: "Passed at render", tone: "neutral" as const };
    return { text: "Ready to render", tone: "ok" as const };
  });
</script>

<Tag tone={summary.tone}>{summary.text}</Tag>
