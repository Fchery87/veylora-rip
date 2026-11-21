# ADR 0008: Portable job archives

## Status

Accepted on 2026-11-20, recording choices made in task 9.1.

## Decisions

- The archive lives in `veylora-workflow` (`archive.rs`), not `veylora-persistence`, because it
  needs the stores, the device records and the safe tar container together, and persistence must
  not depend on the diagnostics crate. The format is documented in `docs/job-format.md`.
- History travels with the job and is replayed on import. Moving a job must never reset it to a
  printable state, so a submitted or ambiguous job stays submitted or ambiguous.
- Device verification does not travel. It is operator-asserted evidence about one machine, and the
  setup's content hash does not depend on it, so stripping it keeps the snapshot valid. The
  imported setup is unverified. A validated mode cannot be stripped without changing the hash the
  snapshot names, so it travels as part of the job definition.
- Renders do not travel. They are deterministic and are repeated; a test asserts the repeated
  render matches the original.
- Import validates fully before writing, refuses an existing job id, and never upgrades or guesses:
  an unknown layout version is refused by name.
- Re-exporting an imported job gives the same bytes, which a test asserts.

## Consequences

An imported job in a state such as Ready has no retained render on the new machine; the operator
holds it, renders again, and continues.

An import either completes or leaves nothing. The snapshot and attempt records are written
together; the lifecycle history is written to a staging directory that readers ignore and moved
into place in one step, last. If any step fails, what the import itself wrote is removed: the
snapshot, and the attempt log cut back to where it ended before the import. The cut is made only
after checking that everything past that point is this job's; if another job's record is found the
log is left as it is and the error says the job could not be undone. A failed import that was undone
says "nothing was imported" and the archive can be imported again.

Not covered: a crash (not a failed write) between the snapshot and the history leaves a job with no
history, which reads as a draft. An import is a deliberate operator action, and a job like that is
removed by importing into a clean workspace; nothing detects it.
