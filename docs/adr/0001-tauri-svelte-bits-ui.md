# ADR 0001: Tauri 2 with Svelte and Bits UI for the Desktop Application

- **Status:** Accepted
- **Date:** 2026-11-07
- **Decision owners:** Project author and implementation team

## Context

Veylora RIP needs a polished operator-facing desktop application for setup verification, artwork import, physical layout, white-policy editing, plane inspection, preflight, queue management, ambiguous-submission review, and diagnostics.

The product must initially run on Windows while preserving a credible path to Linux and macOS. The performance-sensitive and correctness-sensitive portions of the product are the Rust domain, rendering, color, separation, persistence, diagnostics, encoding, and transport layers. The UI should be able to evolve independently and must not become the source of truth for RIP behavior.

Bits UI provides headless, accessible, customizable primitives for Svelte. It is a component foundation rather than a complete visual design system, which fits Veylora's need for a distinctive operator interface without coupling the product to a prebuilt visual style.

## Decision

Use the following desktop stack:

- **Desktop shell:** Tauri 2
- **Frontend language:** TypeScript
- **Frontend framework:** Svelte
- **Component primitives:** Bits UI
- **Frontend toolchain:** Vite+
- **Declared package manager:** pnpm
- **Backend and source of truth:** Rust application services and domain crates
- **Communication boundary:** typed Tauri commands and Rust-produced read models/events

The Svelte frontend owns presentation concerns:

- Navigation and screen composition
- Form state before submission
- Layout interaction gestures and local editing affordances
- Accessible controls and keyboard behavior
- Loading, progress, empty, error, and review states
- Preview tile display and artifact inspection controls

Rust owns all authoritative behavior:

- Domain models and invariants
- DeviceSetup, PrintMode, Recipe, and JobSnapshot validation
- PNG decoding and metadata policy
- Color management
- Layout resolution used for output
- CMYK and white separation
- Masks, morphology, calibration, and screening
- Bounded rendering and artifact retention
- Persistence and migrations
- Encoder and transport
- Submission lifecycle and ambiguity handling
- Diagnostics and support bundles

The frontend must call application-service operations using stable domain IDs and typed request models. It must not pass arbitrary UI settings directly into the renderer or encoder.

Bits UI components will be wrapped by Veylora-owned components where product semantics, styling, validation, or accessibility behavior need to be standardized. The project will not expose Bits UI primitives throughout the application as an uncontrolled design dependency.

Vite+ is used as the unified Vite-based toolchain for frontend development, checking, testing, building, and package-management delegation. The project declares pnpm as its package manager. `package.json` and the lockfile remain the dependency source of truth. Vite+ may detect and invoke pnpm, but it does not replace the project's dependency manifest or lockfile.

Large rasters and printer artifacts will remain in Rust-managed artifact storage. The frontend receives metadata, hashes, dimensions, status, and controlled preview tiles rather than full-resolution planes through JSON or ordinary IPC payloads.

## Alternatives considered

### Pure Rust UI with Slint

This would maximize Rust implementation consistency and could provide a cross-platform desktop UI. It was not selected as the default because Veylora needs complex layout editing, image inspection, forms, tables, overlays, and rapid UX iteration. Slint remains a possible alternative if a focused prototype demonstrates a better fit.

### Pure Rust UI with egui

This is attractive for diagnostic tooling and fast internal prototypes. It was not selected as the primary product UI because the intended operator experience requires polished workflow composition, accessibility, rich layout interactions, and a larger component ecosystem.

### Tauri with React or another frontend framework

This would also satisfy the shell boundary, but Svelte is the selected frontend framework for this project and Bits UI provides the desired headless component foundation.

### Native platform UI toolkits

Separate Windows, Linux, and macOS UI implementations would increase long-term maintenance and make workflow parity harder to preserve.

## Consequences

### Positive

- Rust remains authoritative for all output-affecting behavior.
- Svelte supports fast UX iteration and complex operator interfaces.
- Bits UI supplies accessible, unstyled primitives without imposing a visual theme.
- Tauri provides a cross-platform desktop shell and packaging path.
- The CLI, UI, and future clients can share Rust application services.
- The rendering core remains testable without a WebView or desktop session.

### Negative

- The project maintains both Rust and TypeScript toolchains.
- Tauri IPC contracts require explicit versioning and serialization discipline.
- WebView rendering and large-image inspection need careful performance design.
- Bits UI does not provide a complete visual system, so Veylora must build and maintain its own tokens, wrappers, and interaction conventions.
- Platform-specific packaging and WebView behavior still require Windows, Linux, and macOS validation.
- Vite+ is an additional toolchain dependency that must be version-pinned and validated in local development and CI.

## Validation and follow-up

Before treating the desktop stack as production-ready:

1. Build a Svelte and Bits UI prototype for setup, import, layout, plane inspection, preflight, queue, and ambiguous review using clearly labeled fixture data.
2. Verify keyboard navigation, focus management, error messaging, and reduced-motion behavior.
3. Verify that large artifact inspection uses tiles or bounded previews rather than transferring full-resolution planes through ordinary IPC.
4. Verify the same application-service contract can drive the CLI and desktop UI.
5. Build and smoke-test the Tauri shell on Windows first, then Linux and macOS.
6. Record any platform-specific limitations in the compatibility matrix.
7. Pin the Vite+ and pnpm versions used by the project before the first frontend implementation commit.
