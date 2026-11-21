# ADR 0017: One claim at a time on a job

Status: accepted, 2026-11-21

## Decision
- `Workspace::try_lock_job` claims one job and returns a guard; dropping the guard releases it. It
  never waits: if something else holds the job it refuses at once, naming the cause ("another process
  is working on this job"), and changes nothing.
- `render`, `submit` and `recover` take the claim before they read or write anything and hold it for
  their whole run, on every exit path. A refused one records no lifecycle event, touches no printer
  and writes no run.
- The claim is an operating-system lock (`File::try_lock`) on `<workspace>/state/locks/<job>.lock`.
  The files are left in place; the lock, not the file, is the claim. The system releases a lock when
  its holder exits for any reason, including a crash or a kill, so there is no stale claim to expire,
  steal or clean up, and no timer to get wrong.
- A claim covers one job. Other jobs, and operations that only read or settle a finished state
  (preflight, review decisions), are not blocked.

## Why not a lock file with a process id
A file that exists while a job is busy outlives a crash, and `recover` exists for crashes. Deciding
whether its writer is still alive means guessing from a process id or a clock, which is the kind of
guess this project refuses to make about printer state.

## Verified
A test starts a second process that takes the claim: `recover` is refused while it runs, the process
is killed with no chance to clean up, and `recover` then succeeds.

## Not done
Opening the same workspace from two machines over a network share is not covered; whether a file
lock holds there depends on the share. The workspace is local.
