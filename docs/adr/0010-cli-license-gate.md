# ADR 0010: Release-scoped license gate for the CLI

Status: accepted, 2026-11-20

## Context
The repository-wide license gate fails closed (UNKNOWN licenses, a frontend lock defect,
uninventoried crates), so nothing could be packaged. The owner chose to ship the CLI first.

## Decision
- Add a per-package scope (`check-licenses.ps1 -Package`) rather than weakening the global gate.
- Derive the shipped set from `cargo tree` for the target, never from a hand-kept list; require the
  toolchain's Rust standard library; bind the review to `Cargo.lock` and member manifest hashes.
- Generate inventory and notices from checksum-verified crate sources with
  `generate-package-notices.ps1`; non-permissive, missing-text or unverifiable crates block.
- Own terms: proprietary, all rights reserved; binary distribution authorized by the owner.
- Link the C runtime statically; verify the binary imports no VC++ runtime.
- `package-release.ps1` runs the scoped check and ships notices plus the std notice file.

## Consequences
The CLI can be packaged. The desktop app, frontend, profiles and fonts remain blocked and listed.
Automated permissive-license review is not legal advice; a human should skim the notices before
the first public release. Nothing is signed; no clean-machine run yet.
