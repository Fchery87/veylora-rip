# Durable persistence implementation brief

Queued after domain contracts and device serialization are accepted. Original Task 2.3 and Features 13/15 remain authoritative.

## Ownership and dependencies

Create veylora-persistence with narrow jobs, attempts, migrations, assets and recipes modules. Depend on domain and serialized versioned records, not UI or printer APIs. Coordinate workspace Cargo.toml/Cargo.lock integration serially. Before adding SQLite binding, follow integration discovery and document license.

## Public behavior

- Open/create local database with explicit schema version. Transactional migrations from zero, idempotent reopen, unknown future-version refusal without mutation.
- Create immutable snapshots, read by JobId, verify retained record hashes and refuse conflicting same-ID content. An identical replay may be idempotent but never create duplicate records.
- Retain original asset/profile/configuration bytes by content hash with size limits and exclusive writes. Verify exact bytes against references on read. Do not silently trust a filename or hash supplied by caller.
- Store complete recipe/layout/configuration records needed to reconstruct snapshots. Reject missing dependencies before claiming a reproducible job.
- Snapshot + associated durable initial lifecycle state updates are transactional. Artifact files are not a database transaction: explicitly stage, publish, and reconcile partial files without overwriting existing evidence.
- Attempts are immutable identities plus append-only progress/outcome events, never destructive history updates. Include job/snapshot/artifact hashes, intended/written byte counts, backend/device identity, timestamps and spooler ID where present.
- A crash during Submitting reconstructs NeedsReview/Ambiguous, not Ready or automatic retry. Before-write failures may be safely distinguished only by durable evidence.
- Conditional state transitions prevent concurrent double starts. Reprints explicitly request a new attempt and preserve original snapshot references.
- Errors distinguish corrupt database, unsupported schema, unavailable storage, disk full, checksum failure, invalid reference, conflict and cancelled operation.

## TDD acceptance

Use temporary local databases and injectable failure boundaries. Cover create/read/reopen, migration rollback, future schema refusal, duplicate/conflicting snapshot IDs, identical replay, missing or corrupt content, append-only attempts, concurrent reservation, transaction rollback, disk-full error mapping and recovery after interrupted publication. Assert no automatic new attempt after restart.

Run targeted tests, fmt, strict Clippy and workspace regression suite. Report API, schema and recovery rules with exact tests and limitations. Do not claim production hardware recovery from fake transport tests.
