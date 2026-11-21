# ADR 0007: Local, redacted support bundles

## Status

Accepted on 2026-11-20, recording choices made in task 7.3.

## Context

Someone other than the operator sometimes has to diagnose a job. The job's records are hashes and
states, but free text (transport errors, operator notes) can carry the machine's paths and the
user's profile name, and the stored artwork belongs to the customer.

## Decisions

- `veylora-cli report support-bundle` writes one plain POSIX ustar file to a path the operator
  chooses, refusing to overwrite. Nothing is uploaded; the diagnostics crate has no network code.
- Artwork (sources and masks) is included only with `--artwork include`. The default bundle holds
  the job report, lifecycle history and render manifests, all of which are hashes and metadata.
- Free text is redacted: the workspace location becomes `<workspace>` and user-profile directory
  names become `<user>`, checked on decoded strings, because JSON doubles Windows backslashes.
  Hashed records (render manifests) are never rewritten, since that would break their hashes. A
  manifest that names a local path is withheld and listed instead.
- A section that cannot be read is named under `unavailable` in `manifest.json`; runs that never
  wrote a manifest are listed under `incomplete_runs`. A bundle never looks more complete than it
  is. Only a missing job is an error.
- The container is deterministic (sorted paths, fixed metadata), so equal state gives equal bytes,
  and `manifest.json` lists each file's size and SHA-256.
- One path policy covers writing and reading: relative, `/`-separated, at most 100 bytes, no
  `..`, drive letters, backslashes, trailing dots or spaces, or Windows device names. The reader
  accepts regular files only and refuses links, directories, prefix fields, duplicates (also by
  case), oversize entries and data after the end marker, so extraction cannot write outside its
  destination.

## Limits

Redaction covers known roots and user-profile paths, not arbitrary secrets an operator types into
a note. Operators should read the bundle before sending it. Diagnostic target jobs need no special
case: a job built from a generated target has no customer artwork to omit.
