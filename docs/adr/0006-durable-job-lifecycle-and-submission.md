# ADR 0006: Durable job lifecycle and submission semantics

## Status

Accepted on 2026-11-15, recording choices made in tasks 7.1 and 7.2.

## Context

The domain already had a pure state machine (`veylora-domain::lifecycle`) and an append-only
attempt log, but nothing recorded job state, and no service tied them together. A crash between
"the printer may have received data" and "the outcome was saved" must never look like success or
like a safe retry.

## Decision

- Job state is never stored. `FileLifecycleStore` keeps one immutable file per event under
  `state/lifecycle/<job>/<nnnnnnnn>.json`, hard-linked into place so racing writers cannot both
  take a sequence number. State is the replay of the events through the domain machine, and
  every read re-checks the whole history. Damaged history is reported, never repaired.
- A new job is a draft with no events. Preflight records `PreflightRejected` or
  `PreflightPassed` only when it changes what is known.
- The machine gained one edge, `Failed + Release -> Held`, so an operator can retry a failed
  render or a clean submission failure. There is still no edge out of `Ambiguous`, `Review` or
  `Submitted` that re-renders or re-submits.
- `submit` requires a `Ready` job, a verified setup, a validated mode and a recipe that is not
  retired. It records `BeginSubmission`, then the attempt, and only then calls the transport.
  Only proof that nothing was sent is a clean failure. A short "success", any write failure, a
  cancellation after bytes, an I/O error and an ambiguous result are all `Ambiguous`.
- If the outcome cannot be saved the job stays `Submitting` and `submit` returns
  `OutcomeNotRecorded`. `recover` completes what the attempt log proves, and otherwise closes the
  attempt as uncertain and moves the job to `Review`. It is an explicit operator action because
  the store cannot tell a crashed process from a running one.
- An operator settles `Ambiguous`, `Review` or `Submitted` with a name and a note:
  confirmed finished, or not printed (which cancels the job; a new print is a new job).
- Preflight now also blocks on missing or altered sources, on a recipe that names a color profile
  (no color engine is connected, so it would be ignored), on the page pixel limit and on a stated
  disk budget. The budget is a number the operator gives, not a measurement of free space.
- `job submit` and `job reprint` stay refused in the CLI. There is no encoder, so there are no
  printer bytes to send; the service is exercised with the fake transport only.

## Consequences

- A restart reconstructs the same state and attempts. An interrupted submission is never
  retried; it waits for a person.
- Recipes cannot be validated yet (`RecipeStatus` has no validated state), so the submission gate
  checks the setup and mode and refuses retired recipes only.
- `render`, `submit` and `recover` each claim the job for their whole run (`Workspace::try_lock_job`,
  ADR 0017), so `recover` refuses a job another process is working on instead of assuming it is
  free. The claim is an operating-system file lock that the system drops when its holder dies, so
  a crashed process never blocks the recovery meant to clean up after it.
- A render that succeeded but whose completion could not be recorded leaves the job `Rendering`;
  `recover` marks it failed and its run directory stays on disk.
