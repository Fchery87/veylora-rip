# Native protocol investigation notes

Status: **investigation only**. This document does not establish an ET-8550
protocol, encoder, channel map, or printer-ready artifact.

## Task 1.3 probe boundary

`veylora-cli probe` is intentionally file-only. It performs no printer
discovery, spooler operation, transport, or submission. Input is bounded to
8 MiB and generated output is bounded to 1 MiB. Existing output files are
never overwritten.

- `inspect-reference --input <path>` retains the exact byte count and SHA-256
  hash, plus limited filesystem metadata. It explicitly reports that no
  protocol was decoded.
- `record-observation --input <json> --output <json>` preserves the input
  evidence hash and adds observation metadata. A one-variable comparison must
  be represented by the caller in the JSON, for example `changed_variable`,
  `baseline_evidence_sha256`, and `comparison_evidence_sha256`. The CLI does
  not infer protocol meaning from those fields.
- `render-bounded-target --output <path>` emits deterministic diagnostic JSON
  or SVG. Every target is labeled `DIAGNOSTIC ONLY - NOT PRINTER-READY`.
- `decode-candidate` refuses with or without `--experimental`. There is no
  established protocol or encoder evidence, so an explicit flag cannot make
  submission safe.

## Evidence handling

Reference captures and exported observations must be retained with their exact
SHA-256 values, source identity, acquisition conditions, and legal provenance.
Do not normalize, reinterpret, or invent bytes. A file hash proves byte
identity only. It does not prove framing, channel order, orientation, scale,
feed behavior, or a valid native command path.

## Gate A blocker

The native-evidence blocker remains active. Physical ET-8550 evidence is still
required for protocol fields, logical-to-physical mapping, independent K/W
behavior, orientation, scale, feed/ejection, repeatability, cancellation, and
partial-send behavior. Until that evidence exists, the probe remains an
investigation tool and no native RIP support is claimed.
