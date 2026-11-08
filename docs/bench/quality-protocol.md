# Physical quality protocol

## Scope and current status

This is the printer-independent procedure for [original Task 6.3](../superpowers/plans/2026-11-07-veylora-rip-implementation.md#task-63-define-physical-quality-protocol-and-baseline-records). **Hardware work is pending.** This document supplies no measurements, printed samples, validated recipes, or evidence of native support. Preparing records does not authorize printing, maintenance, uploads, or submission retries.

Use the [blank baseline template](quality-baseline-template.md) and [physical evidence retention rules](../../fixtures/physical/README.md). Requirements come from [PRD section 10](../veylora-rip-prd.md#10-quality-and-success-metrics), [Features 9, 14, and 20](../native-rip-feature-requirements.md), and the [glossary](../../GLOSSARY.md). Logical channels, physical slots, protocol channels, and screened device rasters are separate identities.

## 1. Prepare the study before physical work

1. Create a study ID and versioned plan. Name an operator and reviewer. Select one converted ET-8550 DeviceSetup, PrintMode, Recipe, ink/film/garment condition and finishing process. Link the [device bench record](device-record-template.md). Record verification status without promoting an experimental setup.
2. Assign distinct baseline, calibration-fitting, and validation-holdout roles to targets and samples. Preserve target generator/version, patch IDs/order/count, geometry, nominal channel inputs, white allocation and expected orientation. Include isolated C/M/Y/K/W, each addressable physical slot, combined white, ramps, overlaps, rulers, registration crosses at center and corners, asymmetric text, fine positive/negative type, alpha/white edges, gradients, neutrals, skin tones, saturated colors, near-black, and matched solid/perforated controls.
3. Plan independently printed holdout sheets, not just withheld readings from the fitting sheet. Freeze the split manifest before fitting. Never use holdouts to select curves, profiles, limits, or white policies. If a holdout informs tuning, retire it as validation evidence and reserve a fresh independent holdout set.
4. Declare sample count per condition, within-session repetitions, number and spacing of separate sessions, sheet order/randomization method, measurement repetitions, conditioning intervals, wash checkpoints, and handling of exclusions. Include fresh media and a documented restart between sessions where safe and supported. Counts and timing are operator-selected study parameters, not universal pass thresholds. Do not select them retrospectively to conceal variation.
5. Select a legally available commercial/reference print when available. Record source, software/version/settings, rights and acquisition date. A commercial RIP is only a benchmark, never a native runtime dependency. If unavailable, state why and agree on an alternative measured baseline. No commercial-parity claim is possible without a comparable reference.
6. Record supplier safety and handling instructions for ink, powder, film, cure and press equipment, workspace ventilation/PPE requirements, and operator authorization. This protocol gives no universal temperature, pressure, exposure, cleaning, or ink-servicing instructions. Unknown safe conditions block the physical stage.

## 2. Establish identity, provenance, and measurement conditions

Before each session, complete template sections A through F. Use unique study/session/sheet/sample IDs. Mark film and garment orientation and patch coordinates without contaminating measurement areas. Link cut pieces and destructive-test specimens to their parent sheet. Record operator and timestamps with timezone at printing, each finishing stage, observation, and measurement.

Retain immutable source assets, masks, target layout, job snapshot, DeviceSetup, PrintMode, Recipe, calibration, profile, white policy/allocation, screen configuration, software/backend versions, and exact printer artifact. Record SHA-256 hashes of retained bytes and preserve application content hashes separately with their algorithm/schema. Do not substitute a filename or screenshot for an artifact hash. If a reference path cannot expose an artifact, record that limitation explicitly. It cannot demonstrate native byte reproducibility.

Record ink, film, powder, garment and backing product/batch identifiers, storage/conditioning, printer state, feed path/guides, ambient temperature and relative humidity with units and timestamps. Record actual conditions separately from requested settings. Capture elapsed print-to-powder, powder-to-cure, cure-to-press, press-to-peel and finish-to-measure times.

For each instrument, record identity/model/serial, software/version, calibration/check date and result, resolution and uncertainty or known limitations. Specify spectrophotometer geometry, aperture, measurement mode, illuminant/observer, backing, sample orientation, repeated-read method, and color-difference formula. Specify ruler/microscope scale and camera calibration where used. Record viewing illumination, background, angle/distance and observer. Photos need sample labels, scale, orientation, lighting, camera/exposure/white-balance settings and original-file hashes. Photos alone are not calibrated color or opacity measurements.

Missing or expired instrument checks block affected quantitative claims. Mark unavailable measurements as not measured with a reason, never zero or pass. Do not guess unknown metadata.

## 3. Run the controlled baseline sequence

The following is a future operator procedure, not a command to execute now. Inspect retained logical planes, physical-slot rasters and screened device rasters before separately authorized submission. Check target labels, dimensions, mirror owner, coverage and compatibility. Unsupported or unverified controls remain experimental. Each physical submission must have a unique append-only attempt record and explicit physical outcome.

| Stage | Operator procedure | Required observations and evidence |
| --- | --- | --- |
| Nozzle health and channel identity | Use an approved diagnostic path before quality sheets and after a session, or immediately on a suspected change. Compare labeled patterns with the independently established slot map. Do not infer nozzle health from a preview. | Pattern/attempt hash, per-slot missing or deflected marks, gaps/banding, expected versus observed logical ink and protocol channel, timestamp, photos and operator assessment. Missing or uncertain channels stop quality acceptance. No automatic maintenance. |
| Per-channel ramps | Measure declared input steps for each process channel, each independently addressable white slot, and combined white under the declared allocation. Keep all other settings fixed. Do not duplicate full white coverage into each slot. | Patch ID, nominal input, actual measured response and units, repeat reads, monotonicity, saturation onset, dot gain, banding, gradient smoothness, moire and fine-detail survival. Record maximum usable coverage as an observation, not a preassigned limit. Unsupported slot isolation is a blocker, not a simulated result. |
| Geometry and registration | Measure rulers in both axes, page origin, skew, feed scale, and process-to-white crosses at center and every corner on film and after pressing. Read asymmetric text and orientation marks on both. | Requested/observed dimensions, signed X/Y offsets with defined axes and units, measurement positions, sheet-to-sheet spread, mirror/orientation result and photos. Separate feed/scale faults from plane alignment. Do not hide nozzle or feed faults with plane shifts. |
| Overlap and wet load | Use a planned, bounded patch progression within authorized safe conditions. Compare process-only, white-only and process/white overlaps with documented deposition order and timing. Stop escalation at pooling, handling hazards or media failure. | Per-channel inputs, combined process load and white load separately, pass/order/delay, bleed, pooling, coalescence, drying/handling observations, adhesion and powder pickup after finishing. Coverage is not measured ink volume. |
| White opacity and edges | Measure matched white and process-over-white patches on declared light/dark backings or garments using a fixed measurement geometry. Inspect opaque white, fine edges, transparent boundaries and halos. | Raw reflectance/color readings on each backing, chosen opacity definition/formula and units, substrate blanks, white allocation, edge halo width/location and photos. Do not compare opacity numbers calculated by different methods. |
| Color and detail controls | Measure separately printed holdout patches with recorded reference values and the selected color-difference method. Inspect fine positive/negative type, gradients, neutrals, skin tones, saturated colors and near-black. Compare solid/perforated pairs. | Raw readings, patch-level errors, summary distribution, repeatability, smallest surviving declared feature, banding/moire, edge artifacts and tactile observations. Keep qualitative judgments separate from numeric measurements. |

## 4. Finish, then test durability

Use supplier-approved conditions for the selected materials and equipment. Record setpoints and independently observed values, not an assumed equivalence. Keep an untested matched control and allocate separate labeled specimens for destructive tests so one test does not silently confound another.

1. **Powder and cure:** record powder identity/batch, application/removal method, elapsed times, cure equipment, actual temperature measurement location/method, duration and observed cure/adhesion defects. Record deviations or unavailable actual measurements.
2. **Press:** record garment composition/color/batch, pretreatment if any, pre-press, platen/protective layers, equipment, setpoint and observed temperature, pressure value/unit or reproducible equipment setting with limitation, dwell time, positioning and repress conditions. Measure geometry and registration again after transfer.
3. **Peel:** record supplier-specified peel condition and actual delay/sample temperature, peel direction/method, film release, lift, residue and incomplete transfer by area/patch. Retain before/after photographs.
4. **Stretch:** record conditioning time, gauge length, direction, extension or load with units, rate, hold time, repetitions and recovery interval. Measure cracking, delamination and recovered dimensions against the untested control. Do not claim a standard test unless its method was actually followed and identified.
5. **Wash:** record care protocol or standard/version, machine/program, water temperature, detergent/product/dose, load, garment orientation, wash and dry method, cycle count and inspection checkpoints. At each checkpoint record color change, cracks, edge lift, adhesion, dimensional change and photos under matched conditions. Preserve raw readings and the unwashed control. A skipped checkpoint remains missing, not passed.

## 5. Measure baseline, then freeze acceptance rules

1. Collect actual baseline measurements through the planned sessions and finishing checkpoints. Keep all samples, including failures and exclusions with reasons. Summarize within-sheet, within-session and between-session variation separately. Record sample counts, raw ranges, chosen summary statistics and measurement uncertainty.
2. Review comparability: ink/film/garment, geometry, finishing and instrument conditions must match or differences must be declared. An unmatched reference is informative only, not proof of candidate parity.
3. **Only after real baseline measurement**, agree and freeze a versioned tolerance record. For each metric specify method, units, baseline sample/evidence hashes, comparator, statistic, allowed range or qualitative rubric, uncertainty treatment, replicate/session requirements and rationale. Record reviewer approval, timestamp and hash. No numerical tolerances are supplied by this document.
4. Freeze before judging candidate output. Do not relax tolerances after seeing candidate failures. Changed limits require a new baseline/tolerance revision, justification and new independent candidate evaluation, preserving the old decision.
5. Run the candidate with the same planned conditions and holdouts across repeated sessions. Compare every required metric against the frozen record, including durability, reprint accuracy, throughput, spoilage and duplicate-print incidents. Record inspected-artifact preparation time, print/finish time, usable/rejected counts and units rather than claiming universal speed.
6. A recipe can be called validated only after repeated pressed samples meet the frozen rules across sessions. Missing data, incomplete wash follow-up, missing reference comparability, or unresolved failures mean pending/inconclusive or failed as applicable, not pass. Restrict any claim to the exact tested setup/mode/recipe and supplies. Software equality, spool success and a single attractive sample cannot pass Gate E.

## 6. Failures, changes, and review

Stop unsafe work and preserve the sample and logs. Never automatically retry an ambiguous submission: hold it for operator review, inspect the device and output, and record the unresolved/confirmed outcome. A later operator-authorized reprint receives a new attempt and sample ID linked to the original, even when reusing identical artifact bytes.

Diagnose in bounded steps, recording evidence and the next proposed test:

1. Compare intended target with retained logical/physical/screened output. A raster mismatch goes to source, transform, separation or screening investigation without another physical print.
2. Check independently verified mapping and slot identities. A mapping mismatch blocks acceptance and requires a new verified setup revision before production use.
3. Check mode, geometry and feed observations. Unsupported settings or unstable feed block comparison. Do not compensate blindly with registration offsets.
4. Review nozzle observations. Stop acceptance for missing/uncertain nozzle evidence. Any maintenance is a separate human decision under appropriate instructions, never an automatic troubleshooting action.
5. If film evidence is consistent but transfer evidence differs, compare powder/cure/press/peel and garment records before changing rendering.

Change one factor per investigation where possible. Ink, film, firmware, map, mode, screen, calibration, profile, white policy or finishing changes require new affected record hashes and compatibility review. Keep recipes experimental until relevant baseline and holdout tests are repeated. Append corrections and superseding records instead of rewriting historical observations. Unexplained exclusions or missing patch IDs/counts block quantitative acceptance.

## 7. Record review and pending hardware checklist

Before review, reconcile every planned target and patch with sample, attempt, raw reading and photo references. Check hashes, rights, instruments, environment, finishing, reference comparability, missing fields and holdout isolation. Use the template's decision section to record reviewer, timestamp, scope, failures and next action.

Still pending after this documentation task:

- Actual nozzle/channel checks and authorized reference/native physical samples.
- Real instrument readings, photos, finishing and durability observations.
- Repeated-session baseline and independent holdout measurements.
- Tolerance freeze based on those measurements, then candidate comparison.
- Physical native gates and Gate E recipe acceptance by the operator/reviewer.
