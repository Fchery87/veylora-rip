<script lang="ts">
  import SegmentedControl from "../../components/SegmentedControl.svelte";
  import StageChip from "../../components/StageChip.svelte";
  import Tag from "../../components/Tag.svelte";
  import type { AppStore } from "../../store.svelte";
  import { WHITE_MASK_SOURCES, type CapabilityState, type JobDetail } from "../../services/types";
  import { simulationSource, STAGES, type StageId } from "../../status";

  let { store, job, caps }: { store: AppStore; job: JobDetail; caps: CapabilityState } = $props();

  let stage = $state<StageId>("garment");
  // Sheet dimensions in mm for the sizes the fixture mode lists.
  const SIZES: Record<string, { w: number; h: number }> = {
    A3: { w: 297, h: 420 },
    A4: { w: 210, h: 297 },
  };
  const size = $derived(SIZES[job.sheet] ?? SIZES.A3);
  const unsupported = $derived(`Not supported by mode ${caps.mode.id}`);
  const frozen = $derived(job.rendered);

  function commitChoke(e: Event) {
    const value = (e.currentTarget as HTMLInputElement).valueAsNumber;
    if (Number.isFinite(value)) store.setWhiteChoke(job.id, value);
  }
</script>

<div class="layout">
  <div class="stage-col">
    <div class="chip-row"><StageChip {stage} source={simulationSource(caps.dataSource)} /></div>
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
    <label for="white-policy">White policy</label>
    <select
      id="white-policy"
      value={job.whitePolicy.policy}
      disabled={frozen}
      onchange={(e) => store.setWhitePolicy(job.id, e.currentTarget.value)}
    >
      {#each caps.mode.whitePolicies as policy (policy)}
        <option value={policy}>{policy}</option>
      {/each}
    </select>
    <label for="mask-source">Mask source</label>
    <select
      id="mask-source"
      value={job.whitePolicy.source}
      disabled={frozen}
      onchange={(e) =>
        store.setWhiteMask(job.id, {
          source: e.currentTarget.value,
          invert: job.whitePolicy.invertMask,
        })}
    >
      {#each WHITE_MASK_SOURCES as source (source)}
        <option value={source}>{source}</option>
      {/each}
    </select>
    <label class="check">
      <input
        type="checkbox"
        checked={caps.mode.whiteMaskInvert && job.whitePolicy.invertMask}
        disabled={!caps.mode.whiteMaskInvert || frozen}
        aria-describedby={!caps.mode.whiteMaskInvert ? "mask-invert-reason" : undefined}
        onchange={(e) =>
          store.setWhiteMask(job.id, {
            source: job.whitePolicy.source,
            invert: e.currentTarget.checked,
          })}
      />
      Invert mask
    </label>
    {#if !caps.mode.whiteMaskInvert}
      <span id="mask-invert-reason" class="reason">{unsupported}</span>
    {/if}
    <label for="white-choke">White choke (mm)</label>
    <input
      id="white-choke"
      type="number"
      step="0.1"
      value={caps.mode.whiteChoke ? job.whitePolicy.chokeMm : null}
      placeholder={caps.mode.whiteChoke ? undefined : "n/a"}
      disabled={!caps.mode.whiteChoke || frozen}
      aria-describedby={!caps.mode.whiteChoke ? "white-choke-reason" : undefined}
      onchange={commitChoke}
    />
    {#if !caps.mode.whiteChoke}
      <span id="white-choke-reason" class="reason">{unsupported}</span>
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
    padding: 32px 0;
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
    background: var(--placement-fill);
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
    gap: 8px;
  }
  li {
    display: grid;
    font-size: 13px;
  }
  .bad {
    color: var(--bad-fg);
  }
  .reason {
    font-size: 12px;
    color: var(--warn-fg);
  }
  .note {
    font-size: 12px;
  }
</style>
