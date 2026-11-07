<script lang="ts">
  import SegmentedControl from "../../components/SegmentedControl.svelte";
  import StageChip from "../../components/StageChip.svelte";
  import Tag from "../../components/Tag.svelte";
  import type { CapabilityState, JobDetail } from "../../services/types";
  import { STAGES, type StageId } from "../../status";

  let { job, caps }: { job: JobDetail; caps: CapabilityState } = $props();

  let stage = $state<StageId>("garment");
  // Sheet dimensions in mm for the sizes the fixture mode lists.
  const SIZES: Record<string, { w: number; h: number }> = {
    A3: { w: 297, h: 420 },
    A4: { w: 210, h: 297 },
  };
  const size = $derived(SIZES[job.sheet] ?? SIZES.A3);
  const chokeReason = $derived(`Not supported by mode ${caps.mode.id}`);
</script>

<div class="layout">
  <div class="stage-col">
    <div class="chip-row"><StageChip {stage} source="simulation" fixture={caps.dataSource === "fixture"} /></div>
    <div class="surround">
      <div
        class="sheet stage-{stage}"
        style="aspect-ratio: {size.w} / {size.h}"
        role="img"
        aria-label="Simulated {job.sheet} sheet with {job.placements.length} placement{job.placements.length === 1 ? '' : 's'}"
      >
        {#each job.placements as p (p.id)}
          <div
            class="placement"
            class:clipped={p.clipped}
            style="left:{(p.xMm / size.w) * 100}%; top:{(p.yMm / size.h) * 100}%; width:{(p.widthMm / size.w) * 100}%; height:{(p.heightMm / size.h) * 100}%"
          ></div>
        {/each}
      </div>
    </div>
    <SegmentedControl
      label="Preview stage"
      options={STAGES.map((s) => ({ id: s.id, label: s.label, sublabel: s.sublabel }))}
      value={stage}
      onValueChange={(v) => (stage = v as StageId)}
    />
    {#if job.rendered}<p class="frozen muted">Snapshot frozen. Read-only. Duplicate to edit.</p>{/if}
  </div>

  <aside class="inspector" aria-label="Sheet inspector">
    <h2>Placements</h2>
    <ul>
      {#each job.placements as p (p.id)}
        <li>
          <span>{p.name}</span>
          <span class="mono" class:bad={p.clipped}>
            {p.xMm} , {p.yMm} mm · {p.widthMm} x {p.heightMm} mm{p.clipped ? " · clipped" : ""}
          </span>
        </li>
      {/each}
    </ul>

    <h2>White</h2>
    <dl>
      <dt>Source</dt>
      <dd>{job.whitePolicy.source}</dd>
      <dt>Policy</dt>
      <dd>{job.whitePolicy.policy}</dd>
    </dl>
    <label for="white-choke">White choke (mm)</label>
    <input
      id="white-choke"
      type="number"
      value={job.whitePolicy.chokeMm}
      disabled={!caps.mode.whiteChoke || job.rendered}
      aria-describedby={!caps.mode.whiteChoke ? "white-choke-reason" : undefined}
    />
    {#if !caps.mode.whiteChoke}
      <span id="white-choke-reason" class="reason">{chokeReason}</span>
    {/if}
    <p class="muted note">Applied after mask combine, before choke. Never changes C, M, Y or K.</p>
    <Tag tone="warn">Experimental</Tag>
  </aside>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: 1fr 300px;
    gap: 16px;
  }
  .stage-col {
    display: grid;
    gap: 12px;
    justify-items: start;
  }
  .surround {
    width: 100%;
    background: var(--canvas-surround);
    border-radius: 8px;
    display: grid;
    place-items: center;
    padding: 36px 0;
  }
  .sheet {
    height: 340px;
    position: relative;
    box-shadow: var(--shadow-sheet);
    background: repeating-conic-gradient(var(--film-check-a) 0% 25%, var(--film-check-b) 0% 50%) 0 0 / 16px 16px;
  }
  .sheet.stage-garment {
    background: var(--garment);
  }
  .placement {
    position: absolute;
    background: rgb(46 107 230 / 0.35);
    outline: 1px solid var(--select-inner);
    box-shadow: 0 0 0 1px var(--select-outer);
  }
  .placement.clipped {
    background: var(--clip-hatch);
  }
  .inspector {
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 12px;
    padding: 16px;
    display: grid;
    gap: 8px;
    align-content: start;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 6px;
  }
  li {
    display: grid;
    font-size: 13px;
  }
  .bad {
    color: var(--bad-fg);
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 2px 12px;
    margin: 0;
    font-size: 13px;
  }
  dt {
    color: var(--muted);
  }
  dd {
    margin: 0;
  }
  input {
    height: 32px;
    border: 1px solid var(--line-strong);
    border-radius: 8px;
    padding: 0 10px;
    background: var(--surface);
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }
  input:disabled {
    background: var(--subtle);
    color: var(--muted);
  }
  label {
    font-size: 13px;
    font-weight: 500;
  }
  .reason {
    font-size: 12px;
    color: var(--warn-fg);
  }
  .note {
    font-size: 12px;
  }
</style>
