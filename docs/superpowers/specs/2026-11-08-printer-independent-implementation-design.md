# Printer-Independent Veylora RIP Implementation Design

Date: 2026-11-08
Status: Proposed for implementation

## Purpose and scope

Complete every specification phase that can be implemented and validated without a configured physical Epson ET-8550 setup. The implementation must produce a deterministic, inspectable, file-sink workflow while refusing to claim native printer readiness.

This scope includes immutable jobs and recipes, local persistence, bounded image import, explicit color and layout boundaries, separation and rendering, application services, headless workflows, diagnostics, and UI integration. It excludes evidence-dependent protocol fields, native ET-8550 encoding, Windows printer submission, physical mode promotion, and physical recipe acceptance.

## Non-negotiable boundaries

- Logical artwork planes, physical-slot planes, screened planes, and printer artifacts remain separate types.
- UI code calls typed application services and read models. It does not own rendering, separation, encoding, persistence, or transport.
- Every output-affecting input is represented in an immutable snapshot or a content hash.
- Unsupported formats, metadata, controls, profiles, and combinations fail explicitly.
- Rendering is bounded, cancellable, deterministic, disk-space-aware, and inspectable.
- File-sink output is labeled diagnostic or simulated output, never printer-ready output.
- Submission attempts are append-only. Ambiguous outcomes are held for review and are never automatically retried.
- Synthetic fixtures establish software behavior only. They cannot promote an experimental device mode.
- No commercial RIP is a runtime dependency.

## Architecture and dependency order

Extend the existing Rust workspace with focused crates and one-way dependencies:

```text
veylora-domain
  -> image, color, layout, separation, screen
  -> render
  -> device, persistence, diagnostics
  -> app-services and CLI
  -> Tauri application bridge and frontend
```

The existing domain, device, diagnostics, CLI probe, and frontend foundation remains the base. New code must use public domain identities, hashes, provenance, validation findings, channel maps, and diagnostic target APIs rather than duplicating them.

## Phase A: immutable jobs, modes, recipes, and persistence

### Domain records

Add versioned, serializable records for `PrintMode`, `DeviceCapability`, `JobSnapshot`, `LayoutSnapshot`, `PrintRecipe`, asset references, calibration references, and submission attempts. Records must include all fields that can affect output, software/backend versions, and content hashes. Snapshot construction freezes the input graph. Mutation after render begins is rejected.

Mode compatibility must validate setup identity, firmware, channel-map version, media, raster, protocol fields, deposition policy, quality controls, profiles, and recipe status. Experimental records may exist, but only verified physical evidence can authorize production native submission.

### Persistence

Add a local persistence crate with versioned migrations and an SQLite-compatible implementation. Store immutable job snapshots, artifact references, lifecycle state, recipes, and append-only attempts. Reads reconstruct complete snapshots. Writes are transactional where possible and distinguish disk-full, corrupt database, conflict, and interrupted-operation errors. Restart recovery must not create a duplicate attempt.

### Acceptance

- Snapshot hashes change when any output-affecting field changes.
- Snapshot mutation is rejected after render starts.
- Records round-trip through the portable manifest format.
- Migrations apply from schema zero and reject unsupported future versions.
- Attempts are append-only and restart reconstruction is deterministic.

## Phase B: image, color, and layout core

### Image import

Add bounded PNG decoding with maximum pixels, dimensions, profile bytes, and metadata limits. Preserve original bytes hash, dimensions, alpha semantics, source profile metadata, and relevant resolution metadata. Handle hidden RGB under transparent pixels, opaque alpha, malformed files, conflicting color tags, unsupported APNG/HDR, and non-square resolution explicitly.

### Color boundary

Add a narrow color-policy and transform interface. Record source and working spaces, rendering intent, black-point policy, untagged-RGB policy, and profile hashes. Use a maintained, legally distributable transform implementation only after its dependency and license boundary is documented. No implicit double transform or silent profile assumption is allowed.

### Layout

Add physical-unit printable boxes, placements, rotation, mirror ownership, crop offsets, bounds checks, and deterministic gang-sheet models. One resolved transform must feed preview and production-stage artifacts. Placement duplication must distinguish linked copies from independent placements and preserve reprint identity.

### Acceptance

- Bounded import rejects unsafe or unsupported input before allocation.
- Alpha, hidden RGB, profile precedence, and resolution metadata have golden tests.
- Layout geometry is deterministic and remains inside the printable box.
- Mirror ownership is singular and testable.
- Gang-sheet duplication and nesting preserve hashes and placement identity.

## Phase C: separation, screening, and bounded rendering

### Separation

Represent logical C, M, Y, K, and W planes independently. Implement white sources from alpha, tonal policy, supplied masks, or disabled mode, with explicit replacement, union, intersection, and blend operations. Implement curve application, choke/spread geometry, maximum coverage, knockout rules, perforation rules, and final clear-mask enforcement over every plane.

Changing white policy must not modify process coverage. Clear-mask enforcement occurs after all relevant operations and cannot be undone by later screening.

### Allocation and screening

Allocate logical planes to measured physical slots through the existing channel-map model. Duplicate-white slots may share logical coverage only according to explicit allocation limits and must never accidentally duplicate full coverage. Add calibration curves, ink limits, deterministic screen methods, page-origin phase, and band-equivalence tests.

### Renderer

Add a bounded, disk-backed band renderer with explicit halo handling, cancellation tokens, disk-space checks, intermediate artifact references, exact stage hashes, and resource statistics. Produce separate logical, physical-slot, and screened artifacts. Artifact inspection must not recompute a different preview.

### Acceptance

- Hidden-RGB, alpha, opaque-white, clear-mask, knockout, perforation, and white-policy fixtures pass.
- Band sizes produce equivalent deterministic output and no seams or phase resets.
- Cancellation and disk-full behavior leave inspectable state without claiming completion.
- Stage artifacts are hashable, bounded, and labeled non-printer-ready.
- Logical-to-physical allocation is validated against setup maps and limits.

## Phase D: application services, lifecycle, CLI, and diagnostics

### Application services

Introduce a service layer that orchestrates import, mode and recipe compatibility, preflight, rendering, artifact inspection, lifecycle transitions, and file-sink export. Services accept immutable requests and return typed reports, artifacts, findings, or durable state changes.

### Lifecycle and preflight

Implement missing asset, profile, setup, mode, recipe, clipping, limits, disk-space, resource, and unsupported-control findings. Implement legal lifecycle transitions for draft, preflight failure, held, rendering, ready, submitting, submitted, ambiguous, review, cancelled, failed, and operator-confirmed completion. Ambiguous results remain held and cannot trigger automatic retries.

### Headless workflow

Complete file-sink commands for setup inspection, target generation, asset inspection, job creation, preflight, render, artifact inspection, review, reprint, and report export. A complete PNG-to-file-artifact workflow must work without a printer and must label every non-physical result.

### Diagnostics

Add deterministic job and attempt reports, redacted support bundles, safe archive extraction, path traversal protection, and no-upload guarantees. Reports include hashes and provenance but do not embed private source data unless explicitly requested by the operator.

### Acceptance

- End-to-end file-sink workflow succeeds with fake or synthetic device data while native submission remains refused.
- Every blocking finding has an actionable correction.
- All legal and illegal lifecycle transitions are tested.
- Fault injection covers decode, render, artifact write, persistence, cancellation, and restart.
- Reports and support bundles are deterministic, redacted, and safe to extract.

## Phase E: Tauri application and frontend integration

Connect the current Svelte/Tauri foundation to typed service commands and read-model projections. Add setup/channel status, asset import warnings, mode and recipe selection, layout, white/mask controls, plane inspectors, preflight, queue, attempt review, reprint, and diagnostic export views.

Fixture data and simulated output must retain visible labels such as `Fixture data`, `Simulation`, `Experimental`, and `Not physical output`. Unsupported controls must be disabled or explain why they are unavailable. The UI must surface facts, hashes, findings, and corrections without inventing measurements.

### Acceptance

- Navigation and command enablement derive from capability state.
- UI tests cover all major screens and safety labels.
- Every blocking finding displays its correction.
- Logical, physical-slot, and screened views remain distinguishable.
- File-sink workflow is reachable through services, not direct backend calls.
- No UI path presents a fixture or simulation as physical printer output.

## Explicitly deferred native work

Do not implement or enable these as production behavior until the real setup is available and Gate A evidence exists:

- ET-8550 protocol specification and encoder fields
- Native stream golden fixtures based on actual evidence
- Windows spooler submission to the printer
- Physical channel, layer order, orientation, scale, feed, ejection, restart, and cancellation acceptance
- Promotion of an experimental mode to verified/native status
- Physical calibration and validated production recipes

The existing refusal paths remain in place. A future implementation may add narrow experimental adapters only when their input fields are evidence-backed and the command remains guarded by setup and physical acceptance state.

## Verification strategy

Each slice uses test-first development and focused commits. Run, as applicable:

```text
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cd app && pnpm run check && pnpm test && pnpm run build
```

Use unit, property, golden, fault-injection, migration, and end-to-end file-sink tests. Keep synthetic fixtures clearly marked. Run the full suite at phase boundaries. Physical evidence is not inferred from any software result.

## Execution order

1. Phase A domain manifests, modes, recipes, persistence, and attempts.
2. Phase B image import, color boundary, layout, and gang sheets.
3. Phase C separation, allocation, screening, and renderer artifacts.
4. Phase D services, preflight, lifecycle, file-sink CLI, and diagnostics.
5. Phase E Tauri application-service integration and frontend workflows.
6. Stop at the native evidence boundary and record the remaining Gate A/D/E requirements.
