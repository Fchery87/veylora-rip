# Quality baseline record template

**Blank template, not physical evidence. Hardware work and acceptance remain pending.** Copy for a study, then repeat session/sample sections as needed. Follow the [quality protocol](quality-protocol.md) and [retention rules](../../fixtures/physical/README.md). Blank cells intentionally contain no results, tolerances, sample claims, dates, or hashes.

Every field is required for review unless explicitly marked not applicable with rationale. Use `unknown` or `not measured` with a reason for missing facts. Neither means pass. Do not populate tolerance or decision fields before the required real evidence exists. Preserve raw records and append corrections. All recorded times include timezone, all measurements include units, and all file references include SHA-256 of retained bytes. Application content hashes additionally identify algorithm/schema.

## A. Study plan and authorization

- Study ID, record revision, protocol revision/hash:
- Purpose and scope of comparison:
- Plan author, operator, reviewer:
- Plan creation and approval timestamps:
- Study role (baseline / fitting / candidate validation):
- Baseline source (commercial reference / agreed alternative), availability and limitations:
- Alternative-baseline agreement, reviewer and reason if commercial reference unavailable:
- Planned conditions and controlled variables:
- Planned samples per condition, within-session repetitions and measurement repetitions:
- Planned number/spacing of independent sessions, restart and fresh-media procedure:
- Sheet order/randomization method and seed or retained order:
- Conditioning intervals and destructive-test allocation:
- Wash cycle checkpoints and follow-up schedule:
- Planned summaries, uncertainty treatment and exclusion rules:
- Supplier safety/handling instruction references and revisions:
- Workspace, ventilation/PPE and equipment safety review:
- Separate physical-work authorization, scope, operator and timestamp:

## B. Immutable configuration and artifacts

Keep IDs/revisions and hashes distinct. Add rows for every output-affecting asset. Reference-path artifacts that cannot be obtained must have an explicit unavailability reason and claim limitation.

| Record or artifact | ID/revision | Retained path | SHA-256 | Application content hash and algorithm/schema | Verification status or limitation |
| --- | --- | --- | --- | --- | --- |
| DeviceSetup and channel map | | | | | |
| PrintMode | | | | | |
| Recipe and finishing specification | | | | | |
| Calibration curves and channel/wet-load limits | | | | | |
| Output profile and color policy | | | | | |
| White policy and per-slot allocation | | | | | |
| Screen configuration | | | | | |
| Source artwork, fonts and masks | | | | | |
| Target layout and patch manifest | | | | | |
| Fitting/holdout split manifest | | | | | |
| Immutable job snapshot | | | | | |
| Logical planes | | | | | |
| Physical-slot rasters | | | | | |
| Screened device rasters | | | | | |
| Exact printer-ready artifact | | | | | |
| Reference settings/capture | | | | | |

- Printer model, stable local identity, firmware observation and date:
- Device bench record reference/hash and channel verification evidence:
- Native/reference path, application/backend version and build commit:
- OS, transport, driver/reference software version if applicable:
- Mode geometry, page size, origin, margins, feed path/guides and orientation:
- Pass/deposition order, direction and delays, supported/experimental controls:
- Preflight and retained-raster inspection report/hash, inspector and timestamp:
- Compatibility findings and unresolved warnings:

## C. Legal provenance and custody

Repeat for targets, artwork, profiles, fonts, captures, reference prints and photographs. Ownership of a print does not imply rights to distribute source assets or vendor software. Do not include license keys or private identifying data in shared records.

| Asset/evidence ID | Creator/source and acquisition timestamp | Rights holder/license or permission reference | Allowed use and redistribution restrictions | Local evidence path/hash | Custodian and access/redaction limits |
| --- | --- | --- | --- | --- | --- |
| | | | | | |

- Reference print provider, software/version/settings and lawful acquisition evidence:
- Original private evidence location and custodian:
- Redacted derivative links/hashes and redaction reason, if any:
- Physical storage location, label method, handling and retention plan:

## D. Session conditions and supplies

- Session ID, start/end timestamps, operator and location:
- Previous session link and elapsed time:
- Printer state, restart/fresh-media observations and deviations:
- Before/after nozzle evidence paths/hashes and times:
- Per-slot missing/deflected marks, channel identity observations and disposition:
- Ambient temperature, relative humidity, units, instrument and observation times:
- Material storage and conditioning duration/conditions:

| Supply | Brand/product | Batch/lot | Relevant properties and preparation | Supporting reference/hash |
| --- | --- | --- | --- | --- |
| Ink per physical slot | | | | |
| Film | | | Size, printable side | |
| Powder | | | Application/removal method | |
| Garment/substrate | | | Composition, color, pretreatment | |
| Light/dark measurement backing | | | Reflectance/identity | |
| Protective press layers | | | | |

## E. Instruments and viewing

| Instrument ID | Model/serial and software version | Calibration/check date, method and result | Resolution/uncertainty or limitation | Evidence path/hash |
| --- | --- | --- | --- | --- |
| | | | | |

- Spectrophotometer geometry, aperture and measurement mode:
- Illuminant/observer, backing and sample orientation:
- Repeated-read count/aggregation and repositioning method:
- Color space, reference values and color-difference formula/version:
- Opacity definition/formula, backing method and units:
- Ruler/microscope scale and measurement axes/sign convention:
- Viewing illumination, background, angle/distance and observer:
- Camera/lens, exposure, white balance, scale and color-reference method:
- Instrument unavailability, failed checks and blocked claims:

## F. Target and sample plan

- Target ID, generator/version and target hash:
- Patch layout/order/count, nominal input values and expected orientation:
- Fitting patch IDs and holdout patch IDs, split manifest hash/freeze time:
- Independent holdout sheet plan and isolation from fitting/tuning:
- Holdout contamination check, reviewer and replacement plan if needed:

| Planned sample ID | Session/sheet ID | Parent sample or cut location | Role: baseline/fitting/holdout/control | Target/patch IDs | Planned finishing/test branch | Matched reference/control ID |
| --- | --- | --- | --- | --- | --- | --- |
| | | | | | | |

## G. Actual sample and submission log

Repeat for every sheet and derived specimen, including failed or ambiguous attempts.

- Sample ID, parent sheet/sample ID and patch coordinates:
- Session ID, operator and actual print timestamp:
- Job snapshot hash and exact artifact SHA-256:
- Append-only attempt ID, prior attempt link and reprint reason:
- Transport outcome, spool identifier, intended/written byte counts and timestamps:
- Physical outcome and confirming operator/time (distinct from spool success):
- Film/garment orientation and sample label/storage location:
- Ambiguous state, hold/review record and operator decision:
- Missing/duplicate patches or samples and exclusion reason:

| Stage | Operator and timestamp | Equipment/method | Requested settings | Observed values with units and measurement location | Duration/elapsed interval | Defects/deviations and evidence path/hash |
| --- | --- | --- | --- | --- | --- | --- |
| Powder | | | | | Print-to-powder | |
| Cure | | | Temperature/duration | | Powder-to-cure | |
| Press/pre-press | | | Temperature, pressure, dwell | | Cure-to-press | |
| Peel | | | Peel condition/direction | Sample temperature | Press-to-peel | |
| Repress if applicable | | | | | | |
| Conditioning/measurement | | | | | Finish-to-measure | |

- Stretch specimen and untested control IDs:
- Stretch gauge length, direction, extension/load, rate, hold, repetitions, recovery interval:
- Stretch cracking/delamination and recovered dimensions, evidence paths/hashes:
- Wash specimen and unwashed control IDs:
- Wash standard/version or declared care procedure, machine/program:
- Water temperature, detergent/product/dose, load and garment orientation:
- Dry method/settings, cycle count and inspection checkpoints:

| Wash checkpoint/cycle | Sample ID | Operator/time | Actual wash/dry conditions and deviations | Color change, cracks, lift, adhesion, dimensions | Raw reading/photo paths and hashes | Next checkpoint |
| --- | --- | --- | --- | --- | --- | --- |
| | | | | | | |

## H. Raw measurements and photographs

Use one row per patch, location, stage and repetition, or link an immutable raw table with these columns. Reconcile planned patch IDs/counts with actual readings. Missing readings are not zero.

| Sample/patch ID | Stage/location/axis | Metric and method version | Nominal input/reference | Raw value and units | Repetition | Operator/time | Instrument ID | Raw file/photo path and SHA-256 | Missing/exclusion reason |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| | | | | | | | | | |

For each required metric below, link raw rows and observations. Leave results blank until measured.

| Required metric/control | Sample IDs and raw evidence references | Observations or missing reason |
| --- | --- | --- |
| Nozzle health and logical/physical/protocol channel identity | | |
| C/M/Y/K ramps and per-slot/combined white ramps | | |
| Linearity, saturation, maximum usable coverage and dot gain | | |
| Center/corner registration on film and pressed garment | | |
| Feed scale, origin, skew, dimensions and orientation | | |
| Process wet load, white load, overlaps, pooling and adhesion | | |
| White opacity, substrate blanks and edge halo | | |
| Fine positive/negative type, alpha/white edges and detail survival | | |
| Gradient smoothness, banding and moire | | |
| Holdout color error: neutrals, skin tones, saturated colors, near-black | | |
| Solid/perforated controls and tactile observations | | |
| Cure, press, peel, stretch and wash observations | | |
| Within-session and between-session repeatability | | |
| Reprint accuracy, throughput, waste and duplicate-print incidents | | |

- Photo manifest: sample/stage, capture operator/time, scale/orientation, lighting/settings, original path/hash:
- Measured throughput intervals and units, usable/rejected counts and spoilage reasons:
- Summary method/script version/hash, raw counts, ranges and uncertainty:
- Within-sheet, within-session and between-session statistics separately:

## I. Baseline review and tolerance freeze

**Complete only after real baseline measurements, before candidate judgment.** No default tolerances or approval are supplied here.

- Baseline record version/hash and actual sample/session IDs:
- Baseline completeness, planned/actual counts and exclusions:
- Reference comparability and differences in supplies, finishing or measurement:
- Reviewer, review timestamp and measurement evidence hashes:
- Freeze record ID/revision/hash, approver and approval timestamp:

| Metric | Baseline raw evidence/sample hashes | Method, units and statistic | Observed baseline variation/uncertainty | Agreed limit or qualitative rubric | Replicate/session requirement | Rationale and uncertainty treatment |
| --- | --- | --- | --- | --- | --- | --- |
| | | | | | | |

## J. Candidate comparison and decision

- Candidate setup/mode/recipe hashes and sample/session IDs:
- Frozen tolerance record hash and freeze timestamp:
- Candidate evaluation timestamp, operator and reviewer:
- Independent holdout isolation evidence and fitting profile/calibration hashes:

| Metric | Baseline result/evidence | Candidate result/evidence | Difference and uncertainty | Frozen criterion reference | Pass/fail/pending/inconclusive and rationale |
| --- | --- | --- | --- | --- | --- |
| | | | | | |

- Completeness review of A through J, missing fields and blocked claims:
- Overall decision, tested scope and reviewer signature/time:
- Unresolved failures, wash follow-ups and next bounded test:
- Failure classification (raster / mapping / mode-feed / nozzle / finishing):
- Quarantined samples, ambiguous attempts and recovery decisions:
- Changed factors, new revisions/hashes and required revalidation:
- Supersedes/superseded-by record links and append-only correction history:
- Hardware gate evidence still required and recipe status:

No validated status is justified until repeated pressed samples across sessions meet frozen tolerances. This empty template completes no hardware gate.
