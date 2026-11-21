<script lang="ts">
  import { Toggle } from "bits-ui";
  import PlaneToggle from "../../components/PlaneToggle.svelte";
  import SegmentedControl from "../../components/SegmentedControl.svelte";
  import StageChip from "../../components/StageChip.svelte";
  import RasterView from "./RasterView.svelte";
  import type { AppStore } from "../../store.svelte";
  import type { InspectView, JobDetail, Plane } from "../../services/types";

  let { store, job }: { store: AppStore; job: JobDetail } = $props();

  // The view comes from Rust, including the default selection, stages, planes,
  // labels and metadata. This panel only holds the latest view.
  let view = $state<InspectView | null>(null);
  let error = $state("");
  let loading = $state(false);
  // Only the newest request may change the view, so a slow older answer cannot overwrite it.
  let latest = 0;

  async function show(load: Promise<InspectView>) {
    const mine = ++latest;
    loading = true;
    try {
      const next = await load;
      if (mine !== latest) return;
      view = next;
      error = "";
    } catch (e) {
      if (mine !== latest) return;
      error = e instanceof Error ? e.message : String(e);
    } finally {
      if (mine === latest) loading = false;
    }
  }

  $effect(() => {
    // No selection: Rust returns the default one.
    void show(store.queries.getInspectView(job.id));
  });

  const togglePlane = (plane: Plane, on: boolean) =>
    view && show(store.queries.toggleInspectPlane(job.id, view.selection, plane, on));
  const toggleComposite = (on: boolean) =>
    view && show(store.queries.setInspectComposite(job.id, view.selection, on));
  const selectStage = (id: string) => {
    const stage = view?.stages.find((s) => s.id === id);
    if (view && stage) {
      // The composite is built from the logical planes, so it does not follow a change of stage.
      void show(
        store.queries.getInspectView(job.id, {
          ...view.selection,
          stage: stage.id,
          composite: false,
        }),
      );
    }
  };
</script>

{#if error}<p class="muted" role="status">{error}</p>{/if}
{#if !view && loading}
  <p class="muted" role="status" aria-label="Loading inspect view">Loading the retained output…</p>
{/if}
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
          <PlaneToggle
            plane={p.plane}
            pressed={p.pressed}
            onPressedChange={(on) => togglePlane(p.plane, on)}
          />
        {/each}
        <Toggle.Root
          pressed={view.composite.pressed}
          onPressedChange={toggleComposite}
          aria-label="Composite"
          class="plane composite"
        >
          Composite
        </Toggle.Root>
      </div>
    </div>
    <StageChip source={view.source} text={view.chip} />
    {#if loading}
      <div class="viewport" role="status" aria-label="Loading inspect view">
        <p>Loading the retained output…</p>
      </div>
    {:else if view.rasterShown && view.rasterFiles?.length}
      <RasterView {store} jobId={job.id} files={view.rasterFiles} />
      <p class="muted note">{view.placeholder}</p>
    {:else}
      <div class="viewport" role="img" aria-label={view.viewportLabel}>
        <p>{view.placeholder}</p>
      </div>
    {/if}
    <p class="muted note">{view.metadataNote}</p>
    <dl class="meta">
      {#each view.metadata as row (row.label)}
        <dt>{row.label}</dt>
        <dd class:mono={row.kind === "identifier"}>{row.value}</dd>
      {/each}
    </dl>
    <div class="panels">
      {#each view.panels as panel (panel.title)}
        <section class="card" aria-label={panel.title}>
          <h3>{panel.title}</h3>
          <p class="muted">{panel.message}</p>
        </section>
      {/each}
    </div>
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
    flex-wrap: wrap;
  }
  .planes {
    display: flex;
    gap: 8px;
  }
  :global(.plane.composite) {
    width: auto;
    padding: 8px 12px;
    font-size: 13px;
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
  .note {
    margin: 0;
    font-size: 12px;
  }
  .meta {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 4px 16px;
    margin: 0;
    font-size: 13px;
  }
  .panels {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }
  .panels h3 {
    margin: 0 0 4px;
    font-size: 13px;
  }
  dt {
    color: var(--muted);
  }
  dd {
    margin: 0;
  }
</style>
