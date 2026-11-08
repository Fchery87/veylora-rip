# Physical quality evidence

This directory currently defines evidence-handling conventions only. **No physical samples or measurements are supplied by this README. Hardware validation is pending.** Synthetic fixtures, file-sink artifacts and screenshots of previews cannot establish physical output, native support or recipe acceptance.

Use the [quality protocol](../../docs/bench/quality-protocol.md) and copy the [empty baseline template](../../docs/bench/quality-baseline-template.md) for future operator-run studies. Link the [device bench record](../../docs/bench/device-record-template.md) rather than inferring channel identity from the printer model.

## Local evidence organization

The following is a proposed layout, not an inventory of existing evidence:

```text
<study-id>/
  plan/                  approved plan and fitting/holdout split manifest
  records/               baseline, sessions, sample ledger, decisions, corrections
  snapshots/             immutable setup, mode, recipe, job and target manifests
  artifacts/             exact retained streams and stage-labeled rasters
  measurements/          original instrument exports and separate derived summaries
  photos/                original labeled film/garment/control photographs
  provenance/            rights, source and permission records
  manifest.sha256        SHA-256 and relative path of each retained evidence file
```

Keep private or non-redistributable evidence in an operator-controlled local store outside the repository. A local record may reference that store by stable evidence ID and hash without publishing private paths. Do not upload, commit binary evidence, or redistribute assets merely because this layout exists. Obtain appropriate authorization and review rights first. Do not retain software license keys or unnecessary personal identifiers in shared records.

## Identity and integrity

- Assign unique study, session, sheet, sample, patch and submission-attempt IDs. Map every cut specimen to its parent sheet and location. Label physical specimens and photographs consistently, including stage, orientation, control role and wash cycle where applicable.
- Record capture/measurement operator and timestamp with timezone. Preserve raw readings, units, instrument/calibration metadata, environment, consumable batches and actual finishing conditions using the template.
- Retain exact artifact bytes and their SHA-256. Separately retain application content hashes with algorithm/schema for DeviceSetup, PrintMode, Recipe and job snapshots. A manifest of files is not a substitute for those domain records.
- Hash originals before analysis. Preserve original instrument exports and photographs unchanged. Derived summaries, cropped/redacted images and conversions get distinct names, hashes, parent links, transformation details and software versions.
- List each file's relative path and SHA-256 in the manifest. Do not include the manifest's own hash inside itself. Record the manifest hash in a separate review record, and retain prior manifest versions. Missing files or hash mismatches block review.
- Keep append-only attempt history, unsuccessful samples, exclusions and correction records. Never replace failed evidence with a later successful reprint. A reprint has a new attempt/sample ID even if its artifact hash is identical.
- Record physical storage location, custodian, storage/conditioning and retention duration. Log destructive testing, loss, damage or disposal so missing samples cannot be mistaken for surviving controls.

## Legal provenance and evidence classes

For every asset or captured reference, record creator/source, acquisition date, rights holder, license/permission evidence, allowed use, redistribution limits and retained-file hash. Commercial reference output must come from a lawful source and record software/version/settings where known. It is an optional comparator, not a commercial runtime dependency. Unknown rights block sharing, not permission to invent a replacement observation.

Keep these evidence classes explicit:

| Class | What it can establish | What it cannot establish |
| --- | --- | --- |
| Synthetic/software-only | Rendering or record-format behavior under stated assumptions | Any physical channel, color, durability or native-support claim |
| Captured reference | Observed reference behavior with documented provenance and conditions | Native backend operation or parity under unmatched conditions |
| Physical baseline | Actual measured behavior for the recorded setup and finishing process | Universal tolerances or candidate acceptance before comparison |
| Physical candidate/holdout | Independent observations against a frozen baseline criterion | Validation when data, repeat sessions or durability checkpoints are missing |

Do not place synthetic data in a physical-baseline record as if measured. Mark unavailable reference artifacts or measurements explicitly with their limitations. A photograph supports visual observations but does not replace calibrated color, opacity or dimensional measurement.

## Review before acceptance

1. Reconcile planned and actual samples, targets, patch counts and attempt IDs. Explain missing, duplicate, failed and excluded observations.
2. Verify every retained file hash, sample/photo link, legal provenance entry, instrument check and environment/finishing record.
3. Confirm fitting data and independently printed holdout samples remain separate. Tuning on holdouts requires fresh holdouts for validation.
4. Confirm baseline measurements precede the tolerance freeze and candidate judgment follows it. Preserve approved limit/rubric versions and evidence hashes. Never invent numeric tolerances or physical results.
5. Compare repeated pressed samples across sessions, including wash follow-up. Record reviewer/time, incomplete evidence, failed criteria and bounded follow-up actions. Keep acceptance pending until all required evidence exists.
6. Preserve ambiguous attempts in review and never retry automatically. Record separately authorized reprints without overwriting history.

Original Task 6.3's physical reference recording, measurements, tolerance freeze and sample capture remain pending. This documentation alone does not satisfy the native hardware gates or Gate E personal production recipe acceptance.
