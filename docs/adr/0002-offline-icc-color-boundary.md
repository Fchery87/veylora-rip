# ADR 0002: Offline ICC color transform boundary

## Status

Accepted on 2026-11-08 for feature branch `feat/icc-color-boundary`.

## Context

The Veylora RIP PRD and native-RIP feature requirements both require a real
ICC color management engine rather than hand-rolled matrix math. The
competitive gap analysis explicitly calls out that "supports ICC" is not the
same as a proper color pipeline, and the implementation plan names Little CMS
as a practical candidate for native transforms.

Three constraints shape the choice here:

1. The project's engineering rules require that we not silently approximate
   unsupported input, color metadata, device settings, or protocol fields. A
   real ICC engine is therefore not negotiable.
2. The project's release gate currently blocks on an UNRESOLVED dependency
   inventory. Adding a native build dependency (C compiler, Little CMS source
   drop, vcpkg, or bindgen) materially widens licensing, provenance, and
   build-surface review.
3. The development environment available for this change has no C/C++
   toolchain on `PATH`, so a `cc`-driven Little CMS build was not an option
   for the scoped task.

## Decision

We ship the initial offline ICC color transform boundary on top of the
`qcms` crate (`0.3.0`, MIT-licensed, pure Rust, derived from Firefox's color
management library). `qcms` provides:

- ICC v2 and v4 profile parsing (`iccv4-enabled` feature).
- RGB and CMYK input (`cmyk` feature) with RGB/RGBA output.
- All four ICC rendering intents.
- A deterministic `Transform::apply` over a `&mut [u8]` slice.

The boundary is implemented in `veylora-domain::color` and exposes:

- `SourceProfile::from_bytes` for tagged inputs, carrying the SHA-256 of the
  exact profile bytes.
- `TransformConfig` with explicit `RenderingIntent`,
  `BlackPointCompensation`, `UntaggedPolicy`, and `PixelFormat` fields.
- `ColorTransform::from_source` and `ColorTransform::from_untagged`, where
  the untagged path is guarded by `UntaggedPolicy::Reject` by default and
  only produces output under `UntaggedPolicy::AssumeSrgb`.
- `PixelBuffer` with a `transformed` flag so the boundary refuses to
  transform the same buffer twice without an explicit `mark_fresh` call.
- `TransformReceipt` carrying the source profile hash, the configuration
  hash, the intent, BPC setting, untagged policy, pixel format, pixel count,
  and the engine identifier, suitable for inclusion in an immutable job
  snapshot.

The working/output target in this first scope is the engine's built-in sRGB
profile. The existing `ErrorCode::UnsupportedProfile` is reused for the
unsupported-profile and untagged-reject surfaces so operator-facing messages
remain consistent.

## Consequences and unresolved items

- **BPC inside the engine.** `qcms` does not expose an independent
  black-point compensation switch. The boundary records the operator's
  selection in the configuration hash and provenance, so the setting is
  auditable and will be honored once an engine with explicit BPC (such as
  Little CMS or a future `qcms` release) is wired in. This is called out
  honestly in the module documentation rather than silently approximated.
- **CMYK output.** The device raster pipeline will need CMYK and
  CMYK-plus-white output profiles. `qcms` accepts CMYK input but its
  high-level transform targets are currently RGB/RGBA. Replacing or
  augmenting the engine for DTF device output is a separate, gated change
  with its own physical validation.
- **Licensing.** `qcms` 0.3.0 is MIT-licensed (per its crates.io metadata).
  It has no C build dependencies, no vendored proprietary code, and no
  runtime requirements beyond the Rust standard library and the `bytemuck`
  prerequisite that is already reviewed in the workspace graph. Adding it
  does not pull in Little CMS, Ghostscript, or any commercial RIP. The
  workspace dependency inventory still needs its UNRESOLVED entries cleared
  before the release gate passes; this ADR does not claim that gate has
  moved.
- **Toolchain dependency.** Keeping the engine pure-Rust preserves the
  current workspace's "no C toolchain required" posture on Windows, matching
  the current CI reality.
- **Future migration.** If physical ET-8550 evidence demands Little CMS'
  specific BPC math or its CMYK rendering, the boundary's `ColorTransform`
  type is the only consumer of the ICC engine. Swapping engines is an
  internal change and does not require callers to be rewritten.

## References

- `docs/et8550-rust-rip-plan.md` (Little CMS and `lcms2` binding named as
  practical candidates).
- `docs/native-rip-feature-requirements.md` (ICC engine, rendering intent,
  untagged policy, profile identity).
- `docs/native-rip-competitive-gap-analysis.md` ("supports ICC" is not
  sufficient).
- `qcms` upstream: https://github.com/FirefoxGraphics/qcms
