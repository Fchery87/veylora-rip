# Device bench record template

Status: **UNVERIFIED**. This blank template supplies no printer facts, tank instructions, protocol IDs, or proof of native support. Enter `unknown` for facts not observed. Do not infer a conversion map from a printer model or a commercial RIP sequence.

## Identity and provenance

- Setup ID:
- Setup revision and content hash:
- Channel-map version:
- Record author and date:
- Source: directly recorded / imported / inferred
- Source document or import hash:
- Printer model:
- Serial number or stable local identity (if available):
- Firmware observation, observation date, and evidence reference:
- Connection type and endpoint description:
- Operating system and version:
- Conversion supplier and conversion identifier:
- Installation date (or unknown):
- Installation notes, modifications, and supporting evidence:
- Driver/reference software and version (observation only, never a runtime dependency):
- Reference mode/settings and evidence hash:

## Consumables and feed

- Ink brand, product/type, batch, and installation/refill observations:
- Film brand, product, batch, size, and printable-side observation:
- Feed path, guide placement, and media settings:
- Additional installation/feed observations:

## Physical inventory and proposed map

Record actual tank and head/slot labels separately. Protocol identifiers are hypotheses until matched by labeled physical tests. Add a row for **every** physical slot, including both white slots when present. No example map is supplied intentionally.

| Physical slot ID | Tank label observed | Head/slot label observed | Ink brand/type | Proposed logical channel | Proposed protocol channel | White allocation fraction | Observation source |
| --- | --- | --- | --- | --- | --- | --- | --- |
| | | | | | | | |

The logical channels C, M, Y, K, and W are distinct from slot IDs and protocol channels. Each slot and protocol channel must occur once. White fractions must be finite, positive, at most 1, and sum to at most 1 across all white slots. Do not duplicate full logical white coverage into each slot. Record allocations at 25%, 50%, 75%, and 100% logical coverage and compare actual behavior, not just arithmetic.

## Labeled physical channel evidence

Preserve the target, exact submitted artifact, photograph/scan, and observation record with SHA-256 hashes. Record who observed the result and when. Printed labels must identify the expected mapping independently of the software preview. Isolated patches must establish **every mapped physical slot**, not only one sample of each logical ink.

- Tested setup ID, revision, and content hash:
- Tested channel-map version:
- Target/artifact reference and hash:
- Physical record/photo reference and hash:
- Observer and observation date:
- Test path/backend and version (reference or native):
- Mode hash and Recipe hash (if applicable, otherwise not established):
- Printer state and nozzle/feed observations:
- Film orientation, geometry measurements, and repeatability observations:
- Press/cure/garment conditions and transfer evidence (if performed):

| Printed target label | Physical slot ID | Protocol channel | Expected logical channel | Observed logical channel | Physical result and evidence reference/hash | Pass/fail |
| --- | --- | --- | --- | --- | --- | --- |
| | | | | | | |

A missing, duplicate, swapped, stale, unlabeled, or mismatched observation blocks verification. A spooler success, imported confidence flag, generated target, or software-only test is not physical evidence.

## Verification decision

- Current state: unverified / experimental / verified
- Explicit experimental reason (required for experimental use):
- Outstanding unknowns and failed observations:
- Complete matched labeled evidence checked by:
- Decision date and retained evidence hash:

Imported and inferred configurations start unverified. Only complete matched physical observations can establish channel-map verification. Firmware, tank fill/map, media, feed, installation, driver settings, or allocation changes require a new setup revision/hash and reset verification. Keep previous revisions and evidence unchanged.

This record's channel verification is **not** PrintMode validation, press-quality acceptance, native-backend acceptance, or permission to submit a job. Production requires separate compatible mode/recipe and downstream preflight gates. Experimental use remains explicitly experimental and does not assert validation. No automatic print, retry, maintenance, or firmware operation is authorized by filling out this template.
