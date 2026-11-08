# Printer-independent execution ledger

Plan: docs/superpowers/plans/2026-11-08-printer-independent-implementation-plan.md
Base: d4c8cb2

## Scope ruling

The user's 2026-11-08 instruction authorizes all printer-independent work, not another stop after diagnostics. The original 2026-11-07 plan and product specifications remain the requirements authority. The new plan is an execution grouping, not permission to omit original tasks. Physical evidence and actual submission remain blocked.

Coordinator reviews and independently tests implementer changes. No reviewer agents are spawned.

## Preflight findings and rulings

| Tasks | Shared contract or issue | Resolution |
| --- | --- | --- |
| 1, 2, 3 | Domain references feed mode checks and durable records | Domain must stay independent of device and database crates; references include identity and content hashes, not mutable IDs alone. |
| 1, 6 | Layout snapshots versus geometry ownership | Domain holds immutable layout values; layout crate implements transforms and operations. |
| 1, 7, 8 | Recipe policy versus numerical processing | Domain owns serializable policy values; separation and screen own computation. |
| 3, 9 | Attempts and lifecycle atomicity | Persistence records immutable attempts/events; services orchestrate transactional state changes. |
| 4, 5, 7 | Import to process coverage | A color interface alone is insufficient: implement actual supported ICC transformations and process separations, with licensed fixtures. |
| 7, 8 | Banded masks and screening | Retain absolute page coordinates and halos; prove band equivalence. |
| 8, 10, 11 | Stage artifacts and inspection | Inspector reads retained bytes, not an independently recomputed preview. |
| 9, original 5.2 | Fake transport missing from task files | Implement fake transport and fault-injection contract without physical printer calls. Native submission remains disabled. |
| 12, original 6.1-6.2 | Calibration storage, charts and measurements under-specified | Implement software calibration/recipe storage and chart export/measurement import; no fabricated measured values or validated recipe promotion. |
| 12, original 9.2 | Packaging/update safety omitted | Add packaging, compatibility, release verification, update-safety tests and clean-machine procedure. Distinguish locally verified packaging from unperformed clean-machine acceptance. |
| 2 | Proposed ET-8550 fixture could imply measured fields | Synthetic mode data belongs under fixtures, explicitly non-hardware; do not invent a real ET-8550 configuration. |

These rulings preserve the original requested scope. They may require additional task splits and interface refinement before dependent dispatches.

## Progress

- Design committed: 0dea505.
- Execution grouping committed: d4c8cb2.
- Task 1 in progress: worker session_chick_1794170144594_0019d6b64bf19eda. No implementation accepted yet.
- Task 4 (original 3.1) in progress independently: worker session_chipmunk_1794170342800_ca79030abfdccb9e in C:\Coding-Projects\dtf-epson-image, branch feat/bounded-png-import. Uses existing IDs/hashes only; coordinator merges workspace manifests serially.
- Tasks 2-3 and 5-12 pending, including the coverage corrections above.
- Initial Task 1 contract inspection found hash-only layout/recipe references insufficient without retained typed content or a validated resolver. Worker instructed to supply typed values, creation metadata and consistency tests before acceptance.
- Task 1 intermediate commit e6f1e27 independently passed domain tests, including six snapshot tests, two lifecycle tests and three compile-fail immutability examples. Not integrated: recipe and layout remain unresolved hash references. Worker resumed for typed retained configuration and creation metadata correction.
- Original Task 6.3 documentation in progress: worker session_cow_1794170453042_d57d14b8342c5d84 in C:\Coding-Projects\dtf-epson-quality. Owns only quality protocol, empty baseline template and physical-fixture README. No physical results authorized or claimed.
- Device-mode dispatch acceptance brief retained in device-modes-brief.md. Broader observable review checklist retained in offline-acceptance.md.

## Acceptance policy

A passing stub, interface, or fixture is not completed product behavior. Each task requires coordinator spec review, quality review and fresh tests. Tests validate software behavior only. Production protocol fields, hardware mapping and physical quality gates remain unverified until real evidence exists.
