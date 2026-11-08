# Device modes implementation brief

Status: queued until immutable recipe API is reviewed and integrated.

## Requirements authority

Read AGENTS.md, GLOSSARY.md, original implementation plan Task 2.1, and feature specification Feature 3. The execution grouping is not a reduced specification.

## Ownership

Own crates/veylora-device/src/mode.rs, capability.rs, associated tests, and lib.rs exports. Add setup serialization only if needed to retain complete immutable configuration, with tests preserving all existing evidence validation. Do not edit domain policies without coordinator agreement.

## Required behavior

- Versioned immutable PrintMode identifying exact DeviceSetup ID, revision and content hash, channel-map version and hash, software/backend version and firmware requirements.
- Keep raster grid separate from physical DPI and dimensions, and logical channels separate from slots/protocol addresses.
- Explicit optional or unavailable states for unmeasured packing, compression, interleaving, pass scheduling, deposition, feed and ejection. Never fill in Epson defaults.
- Supported controls must name capabilities that backend can honor, not merely UI settings. Unknown requested controls produce blocking findings.
- Validate sheet bounds/margins/origin, grid/bit depths/levels, film/ink compatibility, profile/calibration/recipe compatibility and resource dimensions without silent clamp.
- Diagnostic software configurations may render offline without pretending to satisfy production evidence. Native submission always stays disabled while no established encoder exists.
- Do not ship a guessed device-config/et8550 mode. Synthetic configurations are fixtures with explicit synthetic provenance.
- Hash every output-affecting input, distinguish unavailable from empty values, and reject unsupported schema versions and invalid deserialization.
- Revision returns new immutable record and invalidates previous acceptance.

## Tests first

Write failing public behavior tests for invalid dimensions/nonfinite inputs, mismatched setup ID/hash/revision/map, stale firmware, unsupported control, incompatible profile/recipe, changed-field hashes, serialization rejection, experimental versus production eligibility and synthetic configurations unable to authorize transport. Keep physical evidence helpers test-only and clearly synthetic.

## Acceptance

Run crate tests, formatting, strict Clippy and the existing setup/probe regression suite. Report exact public API, commits, test evidence and any unsupported software requirements. Coordinator reviews spec/quality and integrates. No actual printer calls, no worker-created subagents.
