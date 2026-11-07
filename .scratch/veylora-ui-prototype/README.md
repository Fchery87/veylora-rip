---
status: needs-triage
labels:
  - needs-triage
---

# PROTOTYPE: Veylora RIP desktop UI direction

Throwaway. Not production code. Delete or archive once a direction is chosen.

## Question

What should the Veylora RIP desktop shell look like, and which interaction model should own the operator workflow, before `design.md` is written?

Three variants of the full operator shell, switchable via `?variant=A|B|C`, in one static file.

- **A, Pipeline rail.** A guided, gated sequence: Setup → Artwork → Layout → White → Inspect → Preflight → Submit. One stage at a time, and the job snapshot is always visible on the right.
- **B, Light table.** Canvas-first workbench. The film sheet is the centre of gravity, with placements on the left, white policy on the right, and a stage strip along the bottom (garment, film, logical planes, physical slots, screened).
- **C, Job ledger.** Queue and evidence first. Jobs, lifecycle state, append-only attempts, preflight corrections, and explicit ambiguous-submission review lead; artwork is a detail of a job.

- **D, Ledger + sheet** (default, requested 2026-11-07). C's job ledger hosts B's light table as each job's main tab. Tabs: Review (only for ambiguous jobs), Sheet, Preflight, Attempts, Provenance. Jobs with a frozen snapshot show the sheet read-only. New palette with light and dark themes: cool neutrals, one blue accent (`#2e6be6` light, `#5b8cff` dark), soft status tints, Geist and Geist Mono.

- **E, Approved shell** (default, 2026-11-07). Built from `docs/design.md`: section rail, device status bar, persistent review banner with the violet review tone, sheet focus mode (`F` / `Esc`, `[` and `]` collapse panels), live preflight with markers on the canvas, garment swatches, and an Inspect tab. Inspect rasterizes, separates and dithers the fixture artwork in the browser, so the loupe shows real per-pixel values. Also includes the Devices verification checklist with mismatch handling and revisions, plus first-run empty states. A pink scenario pill switches between Fresh install, Unverified setup and Working day.

## Run

Open `index.html` in a browser (double-click). Use the arrows on the bottom bar or the keyboard `←` / `→` to switch variants. The variant is kept in the URL.

All data is fixture data. Previews are SVG-filter simulations, not Veylora output and not physical printer output.

## Decision

**Approved 2026-11-07: variant D (Ledger + sheet), then variant E (approved shell built from `docs/design.md`).** Light theme is the default, with a dark toggle. The job ledger is the home screen, and each job's sheet, review, preflight, attempts and provenance live in tabs. The user named CADlink Digital Factory as the reference experience. All five refinements (sheet focus mode, device status bar, neutral canvas surround, violet review tone with a persistent banner, and a section rail) are specified in `docs/design.md`.

## Related

- `docs/adr/0001-tauri-svelte-bits-ui.md` (validation step 1: prototype the UI with labeled fixture data)
- `docs/superpowers/plans/2026-11-07-veylora-rip-implementation.md` Task 8.2 and 8.3
- `docs/native-rip-feature-requirements.md` Features 6, 12, 13
- `.scratch/veylora-rip-build/issue.md`
- `docs/design.md` (design spec, approved with variants D and E)
