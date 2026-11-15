# ADR 0004: Area-averaging resampling and page compositing

## Status

Accepted on 2026-11-15.

## Context

A layout places one or more source images on a page at physical sizes. The device raster has a
fixed pixel pitch. Output must be deterministic, free of hidden-color leakage, and traceable to
the exact inputs. No resampling choice may be hidden from the retained artifacts.

## Decision

`veylora-compose` draws every resolved placement into one page raster before the band chain runs.

- Kernel: exact area-weighted averaging in premultiplied alpha. Upscaling therefore replicates
  source pixels; there is no smoothing option.
- Geometry: integer nanometres. Rotation (quarter turns) and the sheet mirror are inverted
  through the same `ResolvedPlacement` the preview uses, so the inverse map stays axis-aligned.
- Overlap: source-over in layout order. Placement boxes on fractional pixels yield partial alpha
  on edge pixels.
- Page size: page micrometres divided by the pixel pitch, rounded to the nearest pixel.
- Refusals instead of approximation: placement outside the page, crop outside the source, mixed
  color encodings or ICC profiles, gray sources whose encoding is not sRGB-like, non-Gray8 masks.
- Provenance: the algorithm identifier `veylora-compose-area-v1` and the composed-page hash are
  recorded in the render manifest. Any change to the arithmetic changes the identifier.

## Consequences

- The composed page is held in memory at 4 bytes per pixel, bounded by `max_page_pixels`. Only
  the band chain streams to disk. Streaming composition is a future change.
- Upscaled artwork is blocky. A smoother kernel would be a recipe change because it alters output.
- A trailing sliver smaller than half a pixel at the right or bottom edge is clipped by rounding.
- Non-quarter-turn rotation is not supported.
