# Native protocol investigation notes

Status: **investigation only**. No established ET-8550 protocol, encoder,
physical channel map, or printer-ready artifact exists here. Gate A remains
blocked by `.scratch/veylora-rip-build/native-evidence-blocker.md`.

## File-only commands

Run from the repository using `cargo run -p veylora-cli -- probe --help`.

```text
veylora-cli probe inspect-reference --input reference.bin
veylora-cli probe record-observation --input request.json --output observation.json
veylora-cli probe render-bounded-target --output target.json
```

Inspection emits JSON on stdout containing the path, exact byte count, SHA-256,
schema/software versions and `decoded_protocol: false`. Redirect stdout to a
new file when retaining it. It does not parse framing or infer channel IDs.

Target export uses the public `veylora-diagnostics` generator: channel identity
at 100 x 100 mm, logical coverage and geometry only. The deterministic JSON
contains the generated target, its stable hash, software version, and
`DIAGNOSTIC ONLY - NOT PRINTER-READY`. This command supports `.json` only.
It does not rasterize, screen, encode, or submit. SVG/raster export is unsupported.
The dimensions are generator dimensions, not verified printer media limits.

`decode-candidate` and `submit` always fail explicitly. `--experimental` is a
necessary future submission guard, never sufficient authorization in the absence
of an established encoder and physical evidence. No command uses printer APIs.
Unknown, repeated, missing and inappropriate flags are rejected.

## Observation request

Evidence paths are resolved relative to the request file, not the current
working directory. Keep the original request and evidence unchanged alongside
the record. A valid example (operator-supplied facts, not project evidence):

```json
{
  "operator": "your operator identity",
  "source": "owned reference export, software and version recorded here",
  "permission": "record the permission and acquisition basis",
  "command_family": "unknown",
  "framing": null,
  "notes": "Describe observed bytes or physical result without asserting unknown fields",
  "baseline": {
    "evidence": "baseline.bin",
    "settings": { "channel": "K", "media": "same recorded media" }
  },
  "candidate": {
    "evidence": "candidate.bin",
    "settings": { "channel": "W", "media": "same recorded media" }
  },
  "changed_variable": "channel"
}
```

The CLI hashes the actual baseline and candidate bytes and the exact request
bytes. It retains the request's settings and provenance in a new record. It
requires matching setting names and exactly one changed value with the declared
name. This validates the comparison metadata only, not that the operator held
all physical conditions fixed. Record setup identity, firmware, reference
software version, artwork, film, feed path and acquisition conditions among the
unchanged settings. Unknown JSON fields and malformed requests are rejected.

## Resource and retention boundaries

Each input is a regular file limited to 8 MiB. Reads are capped on the opened
handle even if the file grows. Two evidence files are read sequentially.
Output is capped at 1 MiB. Text fields and setting counts are additionally
bounded. Output paths use exclusive creation, never replacement, including
concurrent attempts. A failed write retains an explicitly incomplete file for
inspection and reports failure. Choose a new path after investigating it.

Hashes identify the bytes read, not an atomic filesystem snapshot. Do not edit
reference evidence during inspection. Evidence bytes are not copied into the
observation record, so retain the originals. These bounded local operations do
not implement long-running rendering, cancellation, or transport attempts.

## Legal and physical evidence

Only inspect legally obtained captures or exports for operator-owned artwork.
Record permission, acquisition source and versions. Do not commit private,
licensed or proprietary reference files without appropriate permission.
Commercial software may be a reference instrument, never a runtime dependency.

A hash proves byte identity, not framing or successful physical output. No
inferred mapping, fixture, synthetic target or successful software test can
promote a native mode. Collect channel identity, independent K/W behavior,
orientation, scale, deposition order, feed/ejection, repeated pressed transfer,
restart behavior, cancellation and partial-send evidence on the actual setup.
Unknown fields remain unknown. No automatic retry of ambiguous submission is
permitted. All physical operations remain under operator control.
