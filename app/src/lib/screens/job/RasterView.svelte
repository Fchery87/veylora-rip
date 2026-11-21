<script lang="ts">
  import type { AppStore } from "../../store.svelte";
  import type { PlaneImage, PlaneValues } from "../../services/types";

  let { store, jobId, files }: { store: AppStore; jobId: string; files: string[] } = $props();

  /** Stands for the composite picture, which is built from several planes and has no single loupe. */
  const COMPOSITE = "composite";
  const MAX_PX = 1024;
  const LOUPE = 7;
  /** Small planes are enlarged on screen by whole steps so single pixels stay visible. */
  const zoom = (img: PlaneImage) => Math.max(1, Math.min(16, Math.floor(480 / img.width)));

  let images = $state<Record<string, PlaneImage>>({});
  let loupe = $state<{ file: string; at: PlaneValues } | null>(null);
  let error = $state("");

  $effect(() => {
    const wanted = [...files];
    images = {};
    loupe = null;
    error = "";
    for (const file of wanted) {
      store.queries
        .getPlaneImage(jobId, file, MAX_PX)
        .then((img) => {
          if (files.includes(file)) images = { ...images, [file]: img };
        })
        .catch((e) => (error = e instanceof Error ? e.message : String(e)));
    }
  });

  async function inspect(event: MouseEvent, file: string, img: PlaneImage) {
    const box = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const px = Math.floor(((event.clientX - box.left) / box.width) * img.width) * img.scale;
    const py = Math.floor(((event.clientY - box.top) / box.height) * img.height) * img.scale;
    const half = Math.floor(LOUPE / 2);
    try {
      const at = await store.queries.getPlaneWindow(
        jobId,
        file,
        Math.max(0, Math.min(px, img.sourceWidth - 1) - half),
        Math.max(0, Math.min(py, img.sourceHeight - 1) - half),
        LOUPE,
        LOUPE,
      );
      loupe = { file, at };
      error = "";
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<div class="raster">
  {#if error}<p class="muted" role="status">{error}</p>{/if}
  {#each files as file (file)}
    {@const img = images[file]}
    <figure>
      <figcaption class="mono">
        {file === COMPOSITE ? "Composite simulation" : file}
      </figcaption>
      {#if img && file === COMPOSITE}
        <div class="pic flat">
          <img
            src={img.dataUrl}
            alt="Composite simulation of the retained logical planes"
            width={img.width * zoom(img)}
            height={img.height * zoom(img)}
          />
        </div>
        <p class="muted note">
          {img.sourceWidth} x {img.sourceHeight} px{img.scale > 1
            ? `, shown reduced ${img.scale}:1 (averaged)`
            : ", shown 1:1"}. Select a single plane to read exact values.
        </p>
      {:else if img}
        <button
          class="pic"
          type="button"
          aria-label={`Inspect values in ${file}`}
          onclick={(e) => inspect(e, file, img)}
        >
          <img src={img.dataUrl} alt={`Retained plane ${file}`} width={img.width * zoom(img)} height={img.height * zoom(img)} />
        </button>
        <p class="muted note">
          {img.sourceWidth} x {img.sourceHeight} px{img.scale > 1
            ? `, shown reduced ${img.scale}:1 (averaged)`
            : ", shown 1:1"}{zoom(img) > 1 ? `, enlarged ${zoom(img)}x on screen` : ""}. Click to read exact values.
        </p>
      {:else}
        <p class="muted">Loading…</p>
      {/if}
    </figure>
  {/each}
  {#if loupe}
    <section class="loupe" aria-label="Loupe">
      <h3>Loupe · {loupe.file}</h3>
      <p class="muted note">
        Values 0 to 1 from the retained bytes, starting at pixel ({loupe.at.x}, {loupe.at.y}).
      </p>
      <table>
        <tbody>
          {#each Array.from({ length: loupe.at.height }, (_, r) => r) as r (r)}
            <tr>
              {#each Array.from({ length: loupe.at.width }, (_, c) => c) as c (c)}
                <td class="mono">{loupe.at.values[r * loupe.at.width + c].toFixed(3)}</td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </section>
  {/if}
</div>

<style>
  .raster {
    display: grid;
    gap: 12px;
    width: 100%;
  }
  figure {
    margin: 0;
    display: grid;
    gap: 4px;
  }
  .pic {
    padding: 0;
    border: 1px solid var(--line);
    background: var(--canvas-surround);
    border-radius: 8px;
    overflow: auto;
    max-height: 360px;
    cursor: crosshair;
    justify-self: start;
    max-width: 100%;
  }
  .pic.flat {
    cursor: default;
  }
  img {
    display: block;
    image-rendering: pixelated;
    min-width: 8px;
    min-height: 8px;
  }
  .note {
    margin: 0;
    font-size: 12px;
  }
  table {
    border-collapse: collapse;
    font-size: 12px;
  }
  td {
    padding: 2px 6px;
    border: 1px solid var(--line);
    text-align: right;
  }
  h3 {
    margin: 0 0 4px;
  }
</style>
