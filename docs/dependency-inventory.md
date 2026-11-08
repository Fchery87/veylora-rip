# Dependency Inventory

Task 9.3 deliverable. Lists every third-party component that ships in a Veylora RIP
build, with license and where the notice lives. This is an engineering record, not
legal advice.

Status: partial. Fonts are complete. Rust crates, npm packages, native libraries,
external tools, and fixtures are pending Task 9.3 step 1.

## Fonts (bundled in the app)

| Component | Version | License | SPDX | Notice |
| --- | --- | --- | --- | --- |
| Geist (`@fontsource-variable/geist`) | 5.3.0 | SIL Open Font License 1.1 | `OFL-1.1` | `THIRD_PARTY_NOTICES.md` |
| Geist Mono (`@fontsource-variable/geist-mono`) | 5.3.0 | SIL Open Font License 1.1 | `OFL-1.1` | `THIRD_PARTY_NOTICES.md` |

Detail and obligations: `docs/profile-and-font-licensing.md`.

## Pending inventory (Task 9.3 step 1)

- Rust crates (`Cargo.lock`), including Tauri 2 and its transitive dependencies.
- npm packages (`app/pnpm-lock.yaml`), including Svelte, Bits UI, and Lucide
  (`@lucide/svelte`, ISC per `docs/design.md` section 5.7).
- Native libraries and external tools.
- ICC profiles and fixture assets.

## Release check (Task 9.3 step 3)

`scripts/check-licenses.ps1` is not written yet. When it is, it must fail the release
if any bundled component above lacks a notice in `THIRD_PARTY_NOTICES.md`.
