# Implementation audit and task graph

Date: 2026-11-08
Baseline: `4c03d9f` on `main`
Integration branch: `integration/native-foundation`

## Documentation audit

Read all 22 project Markdown documents (excluding generated dependency/build directories): root instructions, glossary and notices, all `docs/` documents, the build specification, prototype decision record, and app README. The build specification is the only open implementation issue. Task-level work is in the 2026-11-07 implementation plan, not separate ticket files.

The old statement that no source exists is obsolete. Tasks 8.2 and 8.3 have delivered the fixture-backed Tauri/Svelte shell and presenters. They have not delivered a native RIP pipeline. Historical commercial handoff recommendations are superseded by the native-only PRD and gap analysis.

## Existing code and evidence

- Workspace contains only `veylora-app` at baseline.
- `src/jobs.rs` is an in-memory fixture store. Render changes flags. File sink adds a synthetic path without writing a file. Review changes fixture attempt state. None is a durable application service.
- Tauri commands expose these fixtures. Setup, recipes, targets, artwork candidates and diagnostics still fall back to TypeScript mocks.
- Inspection has stage metadata and selection, but no retained raster tiles, coverage or loupe values.
- Physical submission is refused. Preserve this safety boundary.
- No `device-config`, bench record, protocol capture, measured channel mapping, native encoder, profiles, renderer or physical acceptance evidence exists.
- Baseline Rust formatting, clippy and tests pass (38 tests). Frontend validation is recorded below when complete.

## Dependency graph and status

`pending` means unimplemented, not failed. Dependencies below are implementation prerequisites. Physical acceptance is separate and never inferred from software tests.

| Task | Outcome | Prerequisites | Audit status |
| --- | --- | --- | --- |
| 0.1 | Workspace, domain version, CLI, Rust CI | None | Complete, ee05451 |
| 0.2 | IDs, hashes, provenance | 0.1 | Complete, merged 6930740 |
| 0.3 | Errors and validation severity | 0.2 | Complete, 3cb3df9 |
| 1.1 | Device setup, distinct channels, bench template | 0.2, 0.3 | Complete, 724b694 corrected by 2e5251d, integrated 5576e9c |
| 1.2 | Deterministic diagnostic targets | 0.2, 0.3 | Complete, 2d6fd4c and e3d8cb2 |
| 1.3 | File-only investigation CLI and observations | 1.1, 1.2 | Complete, SVG correction d2c8f0f integrated c10b166 |
| 2.1 | PrintMode and capability contracts | 1.1, protocol facts | Complete, 0f66351; white check fixed 2026-11-15. Mode JSON is placeholders only |
| 2.2 | Immutable snapshot and recipe | 0.2, 0.3, 2.1 contracts | Complete, 1edb22c |
| 2.3 | Durable store and migrations | 2.2 | Complete, 4c96afe. File-based store, not SQLite (ADR 0003) |
| 3.1 | Bounded PNG metadata/import | 0.2, 0.3 | Complete, c926f01/e3c2ada |
| 3.2 | Licensed ICC engine boundary | 3.1, licensed profiles | Complete, 0884320. qcms RGB output only (ADR 0002) |
| 3.3 | Physical layout and transforms | 2.1, 2.2 | Complete, 6a64d50 (ADR 0005) |
| 3.4 | Gang sheet model | 3.3 | Complete, 29fb24d (ADR 0005) |
| 4.1 | White and supplied masks | 3.1, 3.2, 3.3 | Complete, d9d0c20/975c545. No Blend mode; tonal white needs the color engine |
| 4.2 | Knockout and final clear mask | 4.1 | Complete, 0122f5d/0f7cec5 |
| 4.3 | Logical planes and slot allocation | 1.1, 4.1 | Complete, 975c545 |
| 4.4 | Calibration and screening | 4.3, 2.1 | Complete, a8161ab |
| 4.5 | Bounded render and retained artifacts | 3.4, 4.2, 4.4 | Complete, fd37912; layouts via compositor 9ac2055 (ADR 0004), validation gate 8cab3c9. Synthetic separator only |
| 5.1 | Native ET-8550 encoder | 2.1, 4.5, legally obtained protocol evidence | Blocked on evidence |
| 5.2 | Windows and fake transport | retained artifact/attempt contracts | Complete, d9572e1. Windows spooler returns not implemented |
| 5.3 | Integrated probe and physical submission | 2.3, 5.1, 5.2, actual staged printer | Blocked on evidence and hardware |
| 6.1 | Calibration and recipe storage | 2.3, 3.2, 4.4, native gate | Complete, 79a050f and 4b3b3fb (durable recipe and calibration store, recipe-to-calibration limit check) |
| 6.2 | Chart export and measurements | 1.2, 6.1 | Complete, 79a050f |
| 6.3 | Physical quality baseline | physical samples and supplies | Documentation accepted, 3d9fb66. Baseline blocked on hardware |
| 7.1 | Preflight services | 2.1, 2.2, 3.x, 4.x | Complete in `veylora-workflow`: missing/altered sources, profile, clipping, limits, stated disk budget, unverified setup (ADR 0006) |
| 7.2 | Durable lifecycle | 2.3, 5.2, 7.1 | Complete: append-only lifecycle store, submission service with fault injection, recover, operator review. Fake transport only; no encoder (ADR 0006) |
| 7.3 | Diagnostics/support archives | 2.3, 7.2 | Complete: local redacted ustar support bundle (artwork opt-in, unreadable sections named), safe path policy for write and read, `report support-bundle` (ADR 0007) |
| 8.1 | Complete headless workflow | 4.5, 5.x, 7.x | 8.1a and 8.1b done for what can be real offline: setup create/verify (operator evidence), target generate, read-only job review. `job submit` and `job reprint` are refused by design (no encoder, no submission path). Recording an operator review decision waits for 7.2 |
| 8.2 | Desktop shell | approved mock exception | Shell done, real file-sink integration pending |
| 8.3 | Preflight/inspection presentation | 8.2 | Presentation done, retained-artifact integration pending |
| 9.1 | Portable archives | 2.3, artifact contracts | Complete: path-independent job archive with checksums, version dispatch, history replay, safe import; `archive export/import`; docs/job-format.md (ADR 0008) |
| 9.2 | Packaging/update safety | production workflow, clean-machine test | Machinery complete and tested: release manifest/verifier, side-by-side install with journaled switch, rollback, recovery, update guard, scripts (ADR 0009). Open: installer choice, signing, clean-machine run, desktop app packaging; packaging of the CLI unblocked by 9.3 |
| 9.3 | License inventory/check | actual dependencies | CLI scope done: generated inventory/notices for what the Windows CLI links (49 components, 0 blockers), scoped gate, static CRT, package-release passes for the CLI (ADR 0010). Still blocked and listed: Tauri app, frontend packages/lock defect, icons, ICC profiles, fonts, fixtures; global gate still fails closed |

## Gate policy

On 2026-11-08 the user explicitly authorized completion of all printer-independent tasks while the printer is not configured. Continue domain, persistence, import, color, layout, separation, rendering, calibration-data tooling, fake transport, services, CLI, UI integration, bundles and packaging in dependency order. Gate A still blocks native support claims, evidence-dependent protocol fields, actual printer submission and physical promotion, not offline software implementation. Synthetic configurations and measurements must remain labeled and cannot establish device truth. See `printer-independent-progress.md` for execution status and scope rulings.

No print, firmware, maintenance, payment or external upload is authorized as a side effect of implementation. A staged physical experiment needs actual setup facts and operator coordination.

## Verification log

- Task 1.3 accepted at c10b166 after SVG correction d2c8f0f: actual shared shapes, visible diagnostic disclaimer, explicit approximate-color/font policy, exact SVG artifact hash and bounded exclusive paired writes. Coordinator reviewed code, independently ran CLI tests and a real SVG export, and passed integrated workspace fmt, strict Clippy and tests. No protocol or physical acceptance is established. Gate A remains blocked as documented in native-evidence-blocker.md.

- Task 1.3 foundation integrated at 5fee126: bounded file inspection, explicit operator observations with evidence-file hashes, strict CLI arguments, exclusive output creation, and JSON diagnostic targets. The later SVG visualization and manifest correction was accepted at c10b166 and is recorded above.

- Task 1.1: rejected initial synthetic evidence helper and mutable revisions. Correction 2e5251d removes fabricated observations, retains explicit evidence, returns new revisions, and length-delimits optional and collection hash inputs. Coordinator independently passed 12 device tests and integrated workspace fmt, strict Clippy and tests at 5576e9c. This validates software contracts only, not physical mappings or native support.

- Baseline frontend: `pnpm run check` (including Svelte type checks), `pnpm test`, and `pnpm run build` passed.
- Baseline: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` passed.
- Task 0.1: reviewed domain/CLI/tests/CI diff, worker passed complete workspace checks (40 tests) in clean worktree without frontend dist. Coordinator independently passed integrated fmt, headless clippy/tests and CLI invocation. Hosted CI/Linux execution remains unobserved.

## Re-verification 2026-11-15

Statuses above were re-checked against the code and plan. Gate B: the headless output inspector exists (`inspect_artifacts`, `artifact inspect`) and the app's Inspect presenter now reads a real retained run (`inspect::present_retained`, `get_retained_inspect_view`, ADR 0011): verified statistics per plane, no raster drawn, composite and slot-letter mapping not built. Not yet met: the Svelte UI is not wired to the command (no live job list), and no raster/loupe view exists. Real RGB-to-CMYK separation needs the licensed profile and color engine and is not built. Remaining printer-independent work: 8.1 residue, UI integration of real artifacts, the 9.2 open items (installer choice, signing, clean-machine run) and the 9.3 license gate. Blocked on evidence or hardware: 5.1, 5.3, 6.3 baseline, native support claims.

Live job store (ADR 0012, slices 1-2): with `VEYLORA_WORKSPACE` set, the app lists and opens real jobs, runs preflight, acknowledges confirm findings, renders and inspects retained output through Rust commands, and the Svelte adapter routes to them with no fixture fallback. Remaining: job creation from the UI, attempt itemization/review, file-sink writes, real mode capabilities, raster/loupe.
