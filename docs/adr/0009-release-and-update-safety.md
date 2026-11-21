# ADR 0009: Release verification and update safety

## Status

Accepted on 2026-11-20, recording choices made in task 9.2.

## Decisions

- Updates are side-by-side. `releases/<version>/` holds verified copies; one atomically replaced
  `install-state.json` decides what runs. A crash leaves the old release or the new one.
- The commit point is the final state write. A journal entry written before the change lets
  recovery recognise and abandon an unfinished update. Recovery never completes an update.
- An update or rollback is refused while a submission or render is in flight, and when job state
  cannot be read: an unknown answer is treated as unsafe. A caller must name a workspace or say
  `--no-workspace`; there is no silent default.
- Integrity rests on a manifest of SHA-256 hashes checked twice: by `veylora-release` and by
  `verify-release.ps1`, which does not trust the program inside the release. A zip needs a hash from
  outside it. There is no skip switch.
- Development tooling is excluded by name and extension at build and verify time.
- Versions are numeric triples and must strictly increase; data newer than the release reads is
  refused.
- Workspace data is outside the install root and is not rolled back with the program.
- The installer technology, the signing certificate and the desktop app packaging are **not**
  decided here. Signature checking exists (`-RequireSignature`) but nothing is signed.

## Consequences

`package-release.ps1` cannot yet produce a release because the license check (9.3) fails closed.
The installation test has run only on a development machine, so Gate F's clean-machine requirement
is open. A rollback after an update that migrated data needs a workspace backup.
