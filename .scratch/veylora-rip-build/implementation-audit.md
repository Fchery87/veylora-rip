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
| 0.1 | Workspace, domain version, CLI, Rust CI | None | In progress |
| 0.2 | IDs, hashes, provenance | 0.1 | Pending |
| 0.3 | Errors and validation severity | 0.2 | Pending |
| 1.1 | Device setup, distinct channels, bench template | 0.2, 0.3 | Pending |
| 1.2 | Deterministic diagnostic targets | 0.2, 0.3 | Pending |
| 1.3 | File-only investigation CLI and observations | 1.1, 1.2 | Pending |
| 2.1 | PrintMode and capability contracts | 1.1, protocol facts | Pending, evidence needed for real mode |
| 2.2 | Immutable snapshot and recipe | 0.2, 0.3, 2.1 contracts | Pending |
| 2.3 | Durable store and migrations | 2.2 | Pending |
| 3.1 | Bounded PNG metadata/import | 0.2, 0.3 | Pending |
| 3.2 | Licensed ICC engine boundary | 3.1, licensed profiles | Pending |
| 3.3 | Physical layout and transforms | 2.1, 2.2 | Pending |
| 3.4 | Gang sheet model | 3.3 | Pending |
| 4.1 | White and supplied masks | 3.1, 3.2, 3.3 | Pending |
| 4.2 | Knockout and final clear mask | 4.1 | Pending |
| 4.3 | Logical planes and slot allocation | 1.1, 4.1 | Pending |
| 4.4 | Calibration and screening | 4.3, 2.1 | Pending |
| 4.5 | Bounded render and retained artifacts | 3.4, 4.2, 4.4 | Pending |
| 5.1 | Native ET-8550 encoder | 2.1, 4.5, legally obtained protocol evidence | Blocked on evidence |
| 5.2 | Windows and fake transport | retained artifact/attempt contracts | Pending |
| 5.3 | Integrated probe and physical submission | 2.3, 5.1, 5.2, actual staged printer | Blocked on evidence and hardware |
| 6.1 | Calibration and recipe storage | 2.3, 3.2, 4.4, native gate | Pending |
| 6.2 | Chart export and measurements | 1.2, 6.1 | Pending |
| 6.3 | Physical quality baseline | physical samples and supplies | Blocked on hardware |
| 7.1 | Preflight services | 2.1, 2.2, 3.x, 4.x | Pending, presenter is not service |
| 7.2 | Durable lifecycle | 2.3, 5.2, 7.1 | Pending, fixture actions are not durable |
| 7.3 | Diagnostics/support archives | 2.3, 7.2 | Pending |
| 8.1 | Complete headless workflow | 4.5, 5.x, 7.x | Pending |
| 8.2 | Desktop shell | approved mock exception | Shell done, real file-sink integration pending |
| 8.3 | Preflight/inspection presentation | 8.2 | Presentation done, retained-artifact integration pending |
| 9.1 | Portable archives | 2.3, artifact contracts | Pending |
| 9.2 | Packaging/update safety | production workflow, clean-machine test | Pending |
| 9.3 | License inventory/check | actual dependencies | Partial, fonts only |

## Gate policy

Gate A after Task 1.3 explicitly requires a pause if a native command path cannot be established. The plan's implementation-order section also describes renderer/encoder work before physical proof. Resolve this tension conservatively: implement the ready foundation and investigation tools, retain truthful diagnostics, and do not invent protocol fields, validated mode facts or physical results to close later tasks. Later production phases stay open until the evidence prerequisite is met. The existing explicitly permitted UI fixture work is not grounds to bypass this rule.

No print, firmware, maintenance, payment or external upload is authorized as a side effect of implementation. A staged physical experiment needs actual setup facts and operator coordination.

## Verification log

- Baseline: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` passed.
