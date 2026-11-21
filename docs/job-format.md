# Job archive format

A job archive is one POSIX ustar file that moves a job between workspaces and machines. It holds
no paths, so where it was made does not matter. Layout version: **1**.

## Contents

| Path | Holds |
| --- | --- |
| `archive.json` | `format` (`veylora-job-archive`), `schema_version`, `job_id`, `snapshot_hash`, `software_version`, and `files`: every other path with its size and SHA-256 |
| `job/snapshot.json` | the immutable job snapshot |
| `records/setup.json`, `records/mode.json`, `records/recipe.json` | the records the snapshot names by content hash |
| `sources/<sha256>` | each source image and mask, named by the hash the snapshot records |
| `history/lifecycle.json` | the append-only lifecycle events, in order |
| `history/attempts.json` | every submission-attempt record, in the order it was appended |

Not carried: rendered output (rendering is deterministic and is repeated on the other side) and
device verification (evidence about one physical machine; the setup arrives unverified and the
content hash does not depend on it). A validated print mode is part of the job definition (its
status is in its hash) and does travel; production submission still needs the setup verified
locally.

## Import rules

Everything is checked before anything is written; a refused archive leaves no trace.

1. The tar is well formed and every path passes the shared safe-path policy (see ADR 0007).
2. `format` matches and `schema_version` is supported. Another version is refused by name
   (`ArchiveVersion`); a migration for a newer layout is added as one more case in the version
   dispatch in `import_job_archive`.
3. The file list equals the archive contents, and each file's size and SHA-256 match.
4. The snapshot validates, and its hash and job id are the ones `archive.json` states.
5. Each record validates and has the content hash the snapshot names; the mode belongs to the setup.
6. Every source is present, no other source is, and each hashes to the name it has.
7. The lifecycle history is a legal run of the state machine from `Draft`, numbered from 1, and
   every history record belongs to this job and snapshot.
8. A job with the same id must not already exist in the destination. It is never overwritten or
   merged.

Then inputs are stored first, the job second, and its history last, replayed through the same
stores and state machine. A job that was submitted, or whose submission is ambiguous, stays that
way on the other side and cannot be printed again by moving it.

## Known limits

- A write failure part-way through the history replay leaves the job in the destination with
  incomplete history. The error says so and tells the operator not to print it and to import into a
  clean workspace. The stores are append-only, so the half-imported job cannot be removed.
- Archives are built and read in memory; the CLI bounds an input archive at 1 GiB.
- A recipe, setup or mode record is carried exactly as stored. Nothing is upgraded or approximated.
