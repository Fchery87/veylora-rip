# ADR 0005: Layout units, anchors and mirror ownership

## Status

Accepted on 2026-11-15, recording choices made in tasks 3.3 and 3.4.

## Context

The plan sketches layout in millimetres as floats, with `resolve_transform(layout, mode)`.
Retained layout records are hashed and must not contain floats. The device capability, not the
print mode, says who owns mirroring.

## Decision

- Layout is integer micrometres (`veylora-domain::layout`). `x_um`/`y_um` is the top-left of the
  rotated bounding box; rotation is clockwise by quarter turns.
- `resolve_transform(layout, MirrorOwner)` applies the sheet mirror once, after rotation, to
  every placement. A requested mirror is refused unless the RIP owns it, so it can neither
  double-mirror nor mirror silently.
- Layout checks return findings: clipping and unsupported page size block; safe margin and low
  effective ppi need acknowledgement. Margin, minimum ppi and supported sizes are caller-supplied
  policy because the media spec carries no margin or size data yet.
- Gang sheets hold placements plus non-printing notes. Copies share the source asset with fresh
  placement identities. Nesting is a deterministic shelf nest and does not search rotations.
  Printed labels are not supported.
- The render path runs the same validation and records acknowledged findings in the manifest.

## Consequences

- No `PrintableBox` or margin type exists until the device spec supplies margin data.
- `duplicate_placement(independent)` from the plan is not implemented; placements are values, so
  a copy is always independent of its source.
