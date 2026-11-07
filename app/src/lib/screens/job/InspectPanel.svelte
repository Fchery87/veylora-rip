<script lang="ts">
  import PlaneToggle from "../../components/PlaneToggle.svelte";
  import SegmentedControl from "../../components/SegmentedControl.svelte";
  import StageChip from "../../components/StageChip.svelte";
  import type { CapabilityState, JobDetail } from "../../services/types";
  import { simulationSource, type StageId } from "../../status";

  let { job, caps }: { job: JobDetail; caps: CapabilityState } = $props();

  type Plane = "C" | "M" | "Y" | "K" | "W";
  const PLANES: Plane[] = ["C", "M", "Y", "K", "W"];

  let stage = $state<StageId>("logical");
  let active = $state<Plane[]>(["K"]);

  function toggle(plane: Plane, on: boolean) {
    active = on ? [...active, plane] : active.filter((p) => p !== plane);
    // Toggling a plane always shows logical planes.
    stage = "logical";
  }
</script>

<div class="wrap">
  <div class="tools">
    <SegmentedControl
      label="Inspect stage"
      options={[
        { id: "logical", label: "Logical" },
        { id: "slots", label: "Slots" },
        {
          id: "screened",
          label: "Screened",
          disabled: !caps.mode.screenedInspect,
        },
      ]}
      value={stage}
      onValueChange={(v) => (stage = v as StageId)}
    />
    <div class="planes" role="group" aria-label="Planes">
      {#each PLANES as plane (plane)}
        <PlaneToggle {plane} pressed={active.includes(plane)} onPressedChange={(on) => toggle(plane, on)} />
      {/each}
    </div>
  </div>
  <StageChip {stage} source={simulationSource(caps.dataSource)} />
  <div class="viewport" role="img" aria-label="Placeholder for plane {active.join(', ') || 'none'}">
    <p>No raster is shown. Retained artifacts arrive when backend integration is unblocked.</p>
  </div>
  <dl class="meta">
    <dt>Job</dt><dd class="mono">{job.id}</dd>
    <dt>Mode</dt><dd class="mono">{caps.mode.id}</dd>
    <dt>Orientation</dt><dd>{stage === "logical" ? "Artwork orientation" : "Film orientation, mirrored by the mode"}</dd>
  </dl>
</div>

<style>
  .wrap {
    display: grid;
    gap: 12px;
    justify-items: start;
  }
  .tools {
    display: flex;
    gap: 16px;
    align-items: center;
  }
  .planes {
    display: flex;
    gap: 8px;
  }
  .viewport {
    width: 100%;
    height: 280px;
    background: var(--canvas-surround);
    border-radius: 8px;
    display: grid;
    place-items: center;
    color: var(--canvas-text);
    font-size: 13px;
  }
  .meta {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 4px 16px;
    margin: 0;
    font-size: 13px;
  }
  dt {
    color: var(--muted);
  }
  dd {
    margin: 0;
  }
</style>
