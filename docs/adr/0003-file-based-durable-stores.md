# ADR 0003: File-based durable stores instead of SQLite

## Status

Accepted on 2026-11-15, recording a decision taken during task 2.3 and extended by task 6.1.

## Context

The implementation plan names SQLite for local persistence. The persistence brief said to
follow integration discovery and document the license before adding a SQLite binding. The
workspace has no C toolchain requirement, the dependency inventory still has unresolved entries,
and the data to keep is small: immutable snapshots, an append-only attempt log, and immutable
recipes and calibration records.

## Decision

Persistence uses plain files in `veylora-persistence`:

- `FileJobStore`: `jobs/<job_id>.json` write-once snapshots and an append-only JSON-Lines
  `attempts.log`, with a `VERSION` file and a forward-only migration runner.
- `FileRecordStore`: `recipes/<hash>.json` and `calibrations/<hash>.json`, named by content hash.

Every write goes to a temporary file, is fsynced, and is renamed into place; directories are
synced where the platform allows it. Existing records are never rewritten. A stored file that
differs from its name or hash, a newer `VERSION`, or a corrupt log entry is returned as
`PersistenceError::Corrupt` or `SchemaOutOfRange`; the store never repairs or downgrades.
Disk-full is classified separately. An ambiguous submission blocks a follow-up attempt.

## Consequences

- No native dependency and no new license to clear.
- No transactions across files and no query engine. Listing is a directory scan. Atomicity is
  per record, and the attempt log is the only multi-record structure.
- Concurrent writers from several processes are not supported. One process owns a store.
- Moving to SQLite later is a migration, not a rewrite, because the store traits and the
  content-addressed record formats are independent of the backing files. It needs its own ADR
  and license review.
