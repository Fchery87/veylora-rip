# Printer-Independent Veylora RIP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use checkbox syntax and each task ends with its own validation and focused commit.

**Goal:** Implement and validate every Veylora RIP phase that does not require physical ET-8550 protocol or printer evidence, ending with a deterministic file-sink workflow and safety-labeled UI.

**Architecture:** Extend the existing Rust workspace through narrow crates and one-way dependencies. Domain snapshots and hashes flow into image/color/layout, separation/screening, rendering, persistence, application services, CLI, and Tauri read models. Native encoding, Windows submission, and physical promotion remain refused behind evidence gates.

**Tech Stack:** Rust 2021, serde/JSON, SHA-256, SQLite-compatible local persistence, bounded PNG decoder, explicit color-transform adapter, deterministic raster/screening code, Tauri/Svelte/TypeScript/Vite+, pnpm, Cargo tests, property/golden/fault-injection tests.

**Spec:** `docs/superpowers/specs/2026-11-08-printer-independent-implementation-design.md`

## Global Constraints

- Keep logical channels, physical slots, protocol channels, and screened rasters distinct.
- Every output-affecting input must be in an immutable snapshot or content hash.
- Unsupported inputs and controls fail explicitly.
- Rendering must be bounded, cancellable, deterministic, disk-space-aware, and inspectable.
- File-sink and simulated output must never be labeled printer-ready.
- Submission attempts are append-only; ambiguous outcomes are never automatically retried.
- Synthetic fixtures prove software behavior only and cannot promote a device mode.
- No commercial RIP is a runtime dependency.
- Do not implement evidence-dependent ET-8550 protocol fields or physical acceptance claims.
- Use TDD, focused commits, isolated workers, and full validation at phase boundaries.

---

## Task 1: Freeze Phase A domain contracts

**Files:**
- Create: `crates/veylora-domain/src/job.rs`
- Create: `crates/veylora-domain/src/recipe.rs`
- Create: `crates/veylora-domain/src/lifecycle.rs`
- Modify: `crates/veylora-domain/src/lib.rs`
- Test: `crates/veylora-domain/tests/job_snapshot.rs`
- Test: `crates/veylora-domain/tests/lifecycle.rs`

**Interfaces:**
- `JobSnapshot::new(...) -> Result<JobSnapshot, ValidationError>`
- `JobSnapshot::content_hash(&self) -> ContentHash`
- `JobSnapshot::begin_render(self) -> Result<RenderingSnapshot, SnapshotError>`
- `PrintRecipe` with versioned white, calibration, profile, screen, and finish references
- `JobState` and `LifecycleEvent` with `transition(current, event) -> Result<JobState, LifecycleError>`

- [ ] Write tests for snapshot hash changes, round trips, frozen mutation rejection, and legal/illegal lifecycle transitions.
- [ ] Run `cargo test -p veylora-domain --test job_snapshot --test lifecycle`; verify the new tests fail.
- [ ] Implement serializable immutable records using existing IDs, hashes, provenance, and validation findings.
- [ ] Run targeted tests and then `cargo test -p veylora-domain`.
- [ ] Commit `feat: add immutable job and recipe domain records`.

## Task 2: Add versioned device modes and compatibility checks

**Files:**
- Create: `crates/veylora-device/src/mode.rs`
- Create: `crates/veylora-device/src/capability.rs`
- Modify: `crates/veylora-device/src/lib.rs`
- Test: `crates/veylora-device/tests/mode_compatibility.rs`
- Create: `device-config/et8550/mode-experimental.json`

**Interfaces:**
- `PrintMode`, `DeviceCapability`, `ModeStatus`
- `check_compatibility(mode, setup, recipe) -> Vec<Finding>`
- `PrintMode::content_hash(&self) -> ContentHash`

- [ ] Write tests for incompatible setup/map/firmware/media/profile/recipe combinations and experimental-mode refusal.
- [ ] Run targeted device tests and verify failure.
- [ ] Implement only measured or explicitly experimental fields; do not invent protocol values.
- [ ] Add a fixture mode marked experimental and non-production.
- [ ] Run device tests, fmt, and strict Clippy.
- [ ] Commit `feat: add versioned print modes and capabilities`.

## Task 3: Implement local persistence and migrations

**Files:**
- Create: `crates/veylora-persistence/Cargo.toml`
- Create: `crates/veylora-persistence/src/lib.rs`
- Create: `crates/veylora-persistence/src/migrations.rs`
- Create: `crates/veylora-persistence/src/jobs.rs`
- Create: `crates/veylora-persistence/src/attempts.rs`
- Modify: workspace `Cargo.toml` and `Cargo.lock`
- Test: `crates/veylora-persistence/tests/migrations.rs`
- Test: `crates/veylora-persistence/tests/jobs.rs`

**Interfaces:**
- `JobStore::create_snapshot`, `get_snapshot`, `append_attempt`, `list_attempts`
- `Migration::version`, `Migration::apply`
- Explicit errors for corrupt DB, unsupported schema, conflict, disk-full, and interrupted writes

- [ ] Write migration, create/read, append-only, restart, duplicate-attempt, and future-schema tests.
- [ ] Run targeted persistence tests and verify failure.
- [ ] Implement a local SQLite-compatible store with versioned migrations and transactional writes.
- [ ] Verify restart reconstruction and no automatic duplicate attempt.
- [ ] Run workspace checks and commit `feat: add durable job persistence`.

## Task 4: Add bounded image import

**Files:**
- Create: `crates/veylora-image/Cargo.toml`
- Create: `crates/veylora-image/src/lib.rs`
- Create: `crates/veylora-image/src/limits.rs`
- Create: `crates/veylora-image/src/png.rs`
- Create: `crates/veylora-image/src/asset.rs`
- Test: `crates/veylora-image/tests/png_fixtures.rs`
- Add: bounded synthetic fixtures under `fixtures/images/`

**Interfaces:**
- `DecodeLimits`
- `SourceAsset` and `ImageMetadata`
- `decode_png(bytes, limits) -> Result<SourceAsset, ImageError>`

- [ ] Add tests for hidden RGB, alpha, profile metadata, conflicting tags, non-square resolution, malformed input, APNG/HDR rejection, and limits.
- [ ] Verify tests fail before implementation.
- [ ] Implement bounded decoding and explicit metadata precedence.
- [ ] Hash original bytes and preserve source metadata without silent normalization.
- [ ] Run image tests and commit `feat: add bounded image asset import`.

## Task 5: Define color policy and transform boundary

**Files:**
- Create: `crates/veylora-color/Cargo.toml`
- Create: `crates/veylora-color/src/lib.rs`
- Create: `crates/veylora-color/src/space.rs`
- Create: `crates/veylora-color/src/transform.rs`
- Test: `crates/veylora-color/tests/transform_contract.rs`
- Create: `docs/dependency-inventory.md` if a transform dependency is selected

**Interfaces:**
- `ColorPolicy`, `ColorSpace`, `ColorTransform`
- `transform_pixel(transform, source) -> Result<WorkingPixel, ColorError>`

- [ ] Write contract tests for tagged/untagged RGB policy, profile hashes, intent, BPC, determinism, and double-transform rejection.
- [ ] Verify failure.
- [ ] Implement the narrow adapter and document dependency/license constraints before adding a transform engine.
- [ ] Reject unsupported profiles and ambiguous metadata.
- [ ] Run tests and commit `feat: define explicit color transform boundary`.

## Task 6: Implement physical layout and gang sheets

**Files:**
- Create: `crates/veylora-layout/Cargo.toml`
- Create: `crates/veylora-layout/src/lib.rs`
- Create: `crates/veylora-layout/src/geometry.rs`
- Create: `crates/veylora-layout/src/placement.rs`
- Create: `crates/veylora-layout/src/transforms.rs`
- Create: `crates/veylora-layout/src/gang_sheet.rs`
- Test: `crates/veylora-layout/tests/orientation.rs`
- Test: `crates/veylora-layout/tests/gang_sheet.rs`

**Interfaces:**
- `Placement`, `PrintableBox`, `ResolvedTransform`
- `resolve_transform(layout, mode) -> Result<ResolvedTransform, LayoutError>`
- `duplicate_placement(sheet, placement, copies, independent)`
- `block_nest(sheet, constraints)`

- [ ] Write geometry, mirror ownership, bounds, crop, duplication, gutters, and deterministic nesting tests.
- [ ] Verify failure.
- [ ] Implement physical-unit geometry and one-owner transform resolution.
- [ ] Ensure preview and production artifacts consume the same resolved transform.
- [ ] Run tests and commit `feat: add physical layout and gang sheets`.

## Task 7: Implement separation, masks, and physical-slot allocation

**Files:**
- Create: `crates/veylora-separation/Cargo.toml`
- Create: `crates/veylora-separation/src/lib.rs`
- Create: `crates/veylora-separation/src/planes.rs`
- Create: `crates/veylora-separation/src/white.rs`
- Create: `crates/veylora-separation/src/mask.rs`
- Create: `crates/veylora-separation/src/morphology.rs`
- Create: `crates/veylora-separation/src/knockout.rs`
- Create: `crates/veylora-separation/src/perforation.rs`
- Create: `crates/veylora-separation/src/allocation.rs`
- Test: `crates/veylora-separation/tests/white_policy.rs`
- Test: `crates/veylora-separation/tests/clear_mask.rs`
- Test: `crates/veylora-separation/tests/allocation.rs`

**Interfaces:**
- `LogicalPlanes`, `PhysicalPlanes`, `Plane`, `ClearMask`
- `generate_white(input, policy)`
- `build_clear_mask(input, rules)` and `enforce_clear_mask(planes, clear)`
- `allocate_logical_to_physical(logical, map, limits)`

- [ ] Write tests for alpha/tonal/supplied/disabled white, mask combinations, curve, choke/spread, knockout, perforation, clear-mask invariants, and one/two-white allocation.
- [ ] Verify failure.
- [ ] Implement deterministic plane operations with bounded dimensions.
- [ ] Prove white policy changes do not alter process coverage and allocation never silently duplicates white.
- [ ] Run tests and commit `feat: add logical separation and mask pipeline`.

## Task 8: Add deterministic screening and bounded renderer

**Files:**
- Create: `crates/veylora-screen/Cargo.toml`
- Create: `crates/veylora-screen/src/lib.rs`
- Create: `crates/veylora-screen/src/calibration.rs`
- Create: `crates/veylora-screen/src/halftone.rs`
- Create: `crates/veylora-render/Cargo.toml`
- Create: `crates/veylora-render/src/lib.rs`
- Create: `crates/veylora-render/src/bands.rs`
- Create: `crates/veylora-render/src/pipeline.rs`
- Create: `crates/veylora-render/src/artifacts.rs`
- Test: screen band-equivalence tests
- Test: renderer pipeline fixtures and fault injection

**Interfaces:**
- `ChannelCurve`, `ScreenSpec`, `apply_calibration`, `screen_band`
- `RenderRequest`, `RenderLimits`, `RenderArtifacts`, `render(request)`

- [ ] Write ramp, ink-limit, phase, band-equivalence, cancellation, disk-full, halo, and deterministic-artifact tests.
- [ ] Verify failure.
- [ ] Implement bounded disk-backed bands with explicit halo ownership and cancellation checks.
- [ ] Record logical, physical-slot, screened artifact hashes and resource statistics.
- [ ] Ensure artifacts are labeled non-printer-ready.
- [ ] Run phase-boundary workspace checks and commit `feat: add bounded deterministic renderer`.

## Task 9: Add application services, preflight, lifecycle, and diagnostics

**Files:**
- Create: `crates/veylora-app-services/Cargo.toml`
- Create: `crates/veylora-app-services/src/lib.rs`
- Create: `crates/veylora-app-services/src/preflight.rs`
- Create: `crates/veylora-app-services/src/workflow.rs`
- Create: `crates/veylora-diagnostics/src/report.rs`
- Create: `crates/veylora-diagnostics/src/support_bundle.rs`
- Test: application service workflow and fault-injection tests
- Test: diagnostics redaction and archive traversal tests

**Interfaces:**
- `run_preflight(request) -> PreflightReport`
- File-sink workflow service returning typed artifact/report records
- `export_report` and `build_support_bundle`

- [ ] Write tests for missing assets, unsupported profiles, incompatible setup/recipe, clipping, limits, disk space, lifecycle transitions, restart, cancellation, and ambiguous attempts.
- [ ] Verify failure.
- [ ] Implement orchestration through domain, image, layout, separation, render, device, and persistence APIs.
- [ ] Implement deterministic redacted reports and safe no-upload bundles.
- [ ] Run tests and commit `feat: add application services and diagnostics`.

## Task 10: Complete headless file-sink CLI

**Files:**
- Modify: `crates/veylora-cli/src/main.rs`
- Create: `crates/veylora-cli/src/commands.rs`
- Test: `crates/veylora-cli/tests/workflows.rs`

- [ ] Write an end-to-end PNG-to-file-artifact workflow test and native refusal tests.
- [ ] Verify failure.
- [ ] Wire setup, target, asset, job, preflight, render, artifact, review, reprint, and report commands through application services.
- [ ] Ensure every file-sink artifact has diagnostic/simulation metadata and no command calls printer APIs.
- [ ] Run CLI tests and commit `feat: complete headless file-sink workflow`.

## Task 11: Connect Tauri application services and frontend views

**Files:**
- Modify: `crates/veylora-app/src/state.rs`
- Create/modify: typed Tauri command bridge and read-model modules
- Modify: `app/src/` screens and components
- Add: frontend workflow and accessibility tests

- [ ] Write tests for capability-driven navigation, disabled unsupported controls, visible correction messages, plane distinction, review actions, and safety labels.
- [ ] Verify failure.
- [ ] Connect typed commands and projections without direct backend ownership in UI code.
- [ ] Add file-sink workflow and report export paths.
- [ ] Run `cd app && pnpm run check && pnpm test && pnpm run build`.
- [ ] Commit `feat: connect desktop workflow to application services`.

## Task 12: Add portable bundles, quality records, and release inventory

**Files:**
- Create: `crates/veylora-persistence/src/bundle.rs`
- Create: `docs/job-format.md`
- Create: `docs/bench/quality-protocol.md`
- Create: `docs/bench/quality-baseline-template.md`
- Create: `fixtures/physical/README.md`
- Create: `THIRD_PARTY_NOTICES.md`
- Create: `docs/profile-and-font-licensing.md`
- Create: `docs/dependency-inventory.md`
- Create: `scripts/check-licenses.ps1`

- [ ] Write bundle round-trip, moved-path, missing-asset, checksum, schema, and path-traversal tests.
- [ ] Implement path-independent bundles and safe extraction.
- [ ] Document physical quality protocol and mark measurements as operator-supplied, not fabricated.
- [ ] Inventory dependencies, licenses, profiles, fonts, and fixture permissions.
- [ ] Run release/documentation checks and commit `docs: add portable jobs and release inventory`.

## Phase gates and final validation

- [ ] After Tasks 1-3, run domain/device/persistence tests, fmt, locked Clippy, and workspace check.
- [ ] After Tasks 4-6, run image/color/layout fixtures and verify no build artifacts are tracked.
- [ ] After Tasks 7-8, run renderer property/golden/fault tests and verify logical/physical/screened artifacts remain separate.
- [ ] After Tasks 9-10, run the complete file-sink workflow and explicit native refusal checks.
- [ ] After Task 11, run frontend check/tests/build and full Rust validation.
- [ ] After Task 12, run bundle/license/documentation checks and final clean-tree validation.
- [ ] Record the remaining native Gate A/D/E requirements without marking them satisfied.
