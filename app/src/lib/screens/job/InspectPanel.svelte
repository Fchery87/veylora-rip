<script lang="ts">
  import PlaneToggle from "../../components/PlaneToggle.svelte";
  import SegmentedControl from "../../components/SegmentedControl.svelte";
  import StageChip from "../../components/StageChip.svelte";
  import type { AppStore } from "../../store.svelte";
  import type { InspectSelection, InspectView, JobDetail, Plane } from "../../services/types";

  let { store, job }: { store: AppStore; job: JobDetail } = $props();

  // Selection state only; Rust builds the view (stages, labels, metadata).
  let view = $state<InspectView | null>(null);
  let error = $state("");

  async function show(load: Promise<InspectView>) {
    try {
      view = await load;
      error = "";
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  $effect(() => {
    const initial: InspectSelection = { stage: "logical", planes: ["K"] };
    void show(store.queries.getInspectView(job.id, initial));
  });

  const toggle = (plane: Plane, on: boolean) =>
    view && show(store.queries.toggleInspectPlane(job.id, view.selection, plane, on));
  const selectStage = (stage: string) =>
    view &&
    show(
      store.queries.getInspectView(job.id, {
        stage: stage as InspectSelection["stage"],
        planes: view.selection.planes,
      }),
    );
</script>

{#if error}<p class="muted" role="status">{error}</p>{/if}
{#if view}
  <div class="wrap">
    <div class="tools">
      <SegmentedControl
        label="Inspect stage"
        options={view.stages.map((s) => ({ id: s.id, label: s.label, disabled: s.disabled }))}
        value={view.selection.stage}
        onValueChange={selectStage}
      />
      <div class="planes" role="group" aria-label="Planes">
        {#each view.planes as p (p.plane)}
          <PlaneToggle plane={p.plane} pressed={p.pressed} onPressedChange={(on) => toggle(p.plane, on)} />
        {/each}
      </div>
    </div>
    <StageChip stage={view.selection.stage} source={view.source} text={view.chip} />
    <div
      class="viewport"
      role="img"
      aria-label="Placeholder for plane {view.selection.planes.join(', ') || 'none'}"
    >
      <p>{view.placeholder}</p>
    </div>
    <dl class="meta">
      {#each view.metadata as row (row.label)}
        <dt>{row.label}</dt>
        <dd class:mono={row.label === "Job" || row.label === "Mode"}>{row.value}</dd>
      {/each}
    </dl>
  </div>
{/if}

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
