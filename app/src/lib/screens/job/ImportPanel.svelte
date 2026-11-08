<script lang="ts">
  import Tag from "../../components/Tag.svelte";
  import type { ArtworkCandidate } from "../../services/types";

  let { candidates }: { candidates: ArtworkCandidate[] } = $props();

  const tone = { ok: "ok", warning: "warn", rejected: "bad" } as const;
  const label = { ok: "Ready", warning: "Warning", rejected: "Rejected" } as const;
</script>

<section class="card">
  <h2>Artwork import</h2>
  <p class="muted">Metadata warnings and rejections appear here, before anything is placed.</p>
  <ul aria-label="Import candidates">
    {#each candidates as c (c.filename)}
      <li>
        <span class="file">{c.filename}</span>
        <span class="mono">{c.pixels}</span>
        <span>{c.colorInterpretation}</span>
        <Tag tone={tone[c.status]}>{label[c.status]}</Tag>
        <span class="msg">{c.message}</span>
      </li>
    {/each}
  </ul>
</section>

<style>
  .card {
    display: grid;
    gap: 12px;
    max-width: 840px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 8px;
  }
  li {
    display: grid;
    grid-template-columns: 160px 110px 100px 90px 1fr;
    gap: 12px;
    align-items: center;
    padding: 8px 0;
    border-top: 1px solid var(--line);
    font-size: 13px;
  }
  .file {
    font-weight: 500;
  }
</style>
