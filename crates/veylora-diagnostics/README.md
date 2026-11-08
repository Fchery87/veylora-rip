# Diagnostic target geometry

Task 1.2 implements seven synthetic logical targets, not rendering, screening,
device mapping, transport, or printer-ready output. No physical device facts are
encoded. `Screening` supplies continuous-tone fields and physical line pairs to
inspect a future screen implementation, not screened drop codes.

`PhysicalSize::new` accepts finite dimensions from 50 through 1000 mm inclusive.
These are generator layout/resource limits, not a printer's supported media.
Objects use millimetres from a top-left origin, with x right and y down. Bounds
include strokes. Rulers have 1 mm intervals independent of page size. The
generator emits at most 17 objects and never allocates a raster.

Coverage fields are logical CMYK/W values in [0, 1], before calibration or
allocation. Alpha is straight coverage and must be applied exactly once by a
future renderer. A K/W overlap means both logical planes cover the same region,
not an assumed physical deposition order. Text objects are labeled geometry
intent, not resolved glyphs. A future renderer must record its font and fit the
declared bounds rather than silently substitute fonts.

The standalone serde JSON includes schema, version, kind, physical size, all
objects, and synthetic provenance. Deserialization accepts only the exact
generated geometry for that schema, rejecting modified targets rather than
acting as an arbitrary artwork format. Callers accepting external data must
bound input byte size before deserializing. No physical verification is implied.

`stable_hash()` uses the shared domain `ContentHash` over compact serde JSON in
declared field/object order. All geometry, labels, coverage and alpha are covered.
Changes to generator geometry or serialization require a schema revision. The
channel fixture has a pinned digest regression test. Reproduce its pretty JSON:

```text
cargo run -p veylora-diagnostics --example channel_identity
```

The example prints JSON to stdout and its digest to stderr. The checked-in
fixture is original synthetic geometry, not captured commercial or printer data.
