# Veylora RIP Agent Instructions

Read this file before modifying the repository.

## Project direction

Veylora RIP is a native Windows DTF RIP initially targeting one converted Epson ET-8550 configuration.

The project must:

- Generate printer-ready output without Digital Factory, AcroRIP, PrintFab, or another commercial RIP at runtime.
- Keep logical channels, physical slots, protocol channels, and screened device rasters distinct.
- Treat physical printer evidence as stronger than software-only assumptions.
- Preserve immutable job snapshots, provenance, artifacts, and append-only submission attempts.
- Never automatically retry an ambiguous printer submission.
- Keep rendering bounded, cancellable, deterministic, and inspectable.
- Keep the initial scope focused on one verified ET-8550 configuration.
- Defer broad printer support, accounts, cloud services, payments, DRM, telemetry, and industrial production features.

Read the relevant documents before implementation:

- `docs/veylora-rip-prd.md`
- `docs/native-rip-feature-requirements.md`
- `docs/native-rip-competitive-gap-analysis.md`
- `docs/design.md` (desktop UI design; read before any frontend work)
- `docs/superpowers/plans/2026-11-07-veylora-rip-implementation.md`
- `GLOSSARY.md`
- Relevant ADRs under `docs/adr/`

## Engineering rules

- Use test-first development for behavior changes.
- Keep modules narrowly scoped with one-way dependency direction.
- Do not let UI code own rendering, separation, encoding, persistence, or Windows transport logic.
- Do not claim native support before the physical ET-8550 gate passes.
- Do not silently approximate unsupported input, color metadata, device settings, or protocol fields.
- Record output-affecting inputs in hashes and job snapshots.
- Run formatting, linting, and tests before declaring work complete.
- Commit focused changes with clear messages.
- Do not use commercial RIPs as runtime dependencies.

## Agent skills

### Issue tracker

Issues are tracked as local markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the default triage vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, and `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

This is a single-context project. Read the root `GLOSSARY.md` and relevant ADRs under `docs/adr/`. See `docs/agents/domain.md`.
