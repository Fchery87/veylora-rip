# Veylora RIP Feature Requirements Specification

> **Preparation document:** This is a requirements and validation specification for the native RIP project. It is not an implementation plan for one UI screen and it does not claim that the ET-8550 protocol or print quality has already been proven.

**Date:** November 7, 2026  
**Working product name:** Veylora RIP  
**Product direction:** Personal high-end DTF workflow first, with foundations suitable for future distribution.  
**Primary backend:** Native Windows backend for one converted Epson ET-8550 configuration.  
**Native requirement:** The finished workflow must generate printer-ready output without requiring Digital Factory, AcroRIP, PrintFab, or another commercial RIP at runtime.

## How to use this document

Every feature below has six sections:

- **Purpose:** user-visible outcome.
- **Requirements:** behavior that must exist before implementation is considered complete.
- **Data and interfaces:** information the feature consumes and produces.
- **Dependencies:** prerequisite work.
- **Validation:** fixtures, test methods, and failure cases.
- **Acceptance:** observable evidence required to call the feature ready.

The words **must**, **should**, and **may** mean required for the current native release, strongly recommended, and optional/future respectively. Vendor pages document advertised capabilities, not measured parity. Device-dependent claims remain experimental until verified on the actual converted printer.

## Global engineering contracts

These apply to every feature:

1. **Native output is the success criterion.** Commercial RIPs can be reference instruments only.
2. **One tested device configuration first.** The configuration includes printer identity, firmware, conversion map, film, ink, mode, resolution, feed path, screening, recipe, and finishing conditions.
3. **Logical and physical channels are separate.** Logical planes such as C, M, Y, K, W are not physical tank/head slots.
4. **Preview is not proof.** Garment preview, continuous-tone planes, physical-slot rasters, and screened device output must be labeled separately.
5. **Every transform has one owner.** Mirror, rotation, scaling, color conversion, white generation, calibration, screening, and device encoding must not be applied twice.
6. **Every reproducible job is immutable.** Artwork, masks, settings, profiles, device setup, software versions, and output artifacts are snapshotted.
7. **Unsupported input is rejected loudly.** Never silently flatten profiles, spot colors, overprint, HDR, animation, or unsupported PDF features.
8. **Physical acceptance precedes production claims.** A passing unit test or successful spool call is not evidence of a usable transfer.
9. **No automatic maintenance actions initially.** Do not automate firmware updates, head cleaning, waste-counter changes, or ink servicing.
10. **Bounded resources are mandatory.** Large images must render through disk-backed bands with cancellation and disk-space checks.
11. **Distribution readiness is not a storefront.** Design portable jobs, migrations, diagnostics, licensing records, and clean installation now. Defer accounts, payments, cloud queues, and DRM.

## Feature 1: Native printer backend and transport

### Purpose
Generate and submit the application's own printer-ready stream to the ET-8550 through a supported Windows transport, with enough observability to distinguish rendering, spooling, and physical completion.

### Requirements

- Implement a backend interface separate from layout, separation, screening, and UI.
- Support a file sink before a device sink. The file sink must retain exact output for inspection and replay.
- Define job framing, page framing, raster band framing, compression, checksums if used, and end-of-job behavior in a versioned backend specification.
- Use a dedicated worker for blocking Windows print calls. Microsoft documents `WritePrinter` as synchronous and potentially blocking.
- Track `rendering`, `ready`, `submitting`, `submitted`, `ambiguous`, `failed`, `cancelled`, and `operator-confirmed-finished` separately.
- Record bytes intended, bytes written, spooler identifiers where available, timestamps, device identity, and backend version.
- Never retry automatically after an ambiguous submission.
- Support cancellation at band boundaries and define what cancellation means after bytes have reached the spooler.
- Keep native transport optional in unit tests through a file sink and fake transport.

### Data and interfaces

`DeviceBackend` consumes a `PrintMode`, physical-slot raster bands, page geometry, and a job snapshot. It produces a retained printer-ready artifact and a `SubmissionAttempt`. The transport must not accept arbitrary UI settings directly.

### Dependencies

Print-mode model, device setup, bounded renderer, Windows spooler adapter, job database, native probe, and physical test target.

### Validation

Test empty jobs, one-pixel jobs, one-band jobs, multi-band jobs, cancellation before submission, failure mid-write, timeout after final write, spooler disconnect, application restart, and duplicate-attempt prevention. Compare file-sink output byte-for-byte for deterministic inputs.

### Acceptance

A tiny native test containing isolated process K, isolated W, overlap, asymmetric text, and an orientation marker prints with the intended physical behavior. Repeated attempts have explicit outcomes, and an ambiguous failure never silently creates a duplicate.

## Feature 2: Device identity, conversion map, and setup wizard

### Purpose
Prevent a printer model name from being mistaken for a known physical ink configuration.

### Requirements

- Store printer model, serial or local identity when available, firmware observation, connection type, operating system, conversion supplier, physical tank labels, head/slot labels, ink brand/type, and installation date.
- Represent `LogicalInk`, `PhysicalSlot`, and `ProtocolChannel` as distinct records.
- Permit one logical ink to map to multiple physical slots, such as two white slots.
- Store allocation rules for duplicate white slots rather than copying full coverage into each slot.
- Require a labeled channel test before a configuration becomes `verified`.
- Show a prominent warning when a configuration is inferred, imported, or unverified.
- Version every map. A changed tank fill, firmware, driver mode, or white-slot policy creates a new device setup.
- Never translate AcroRIP's documented `KWCYMW` sequence into tank instructions without decoding its convention and checking the actual conversion.

### Data and interfaces

A `DeviceSetup` includes `PhysicalSlot[]`, `LogicalChannelMap`, tested firmware, supported modes, media/feed settings, calibration references, and verification status. A setup is referenced by hash from every job.

### Dependencies

Native channel test, print-mode registry, recipe storage, diagnostics, and operator confirmation flow.

### Validation

Create deliberately swapped maps and confirm the application refuses production submission. Print isolated patches for every physical slot, then a labeled sheet whose expected identity is known independently. Test two-white-slot allocation at 25%, 50%, 75%, and 100% logical coverage.

### Acceptance

An operator can identify every usable physical channel from a printed test. The system cannot submit a production job with an unverified or incompatible setup without an explicit experimental override and recorded reason.

## Feature 3: Validated PrintMode and device capability model

### Purpose
Represent the complete printer condition instead of exposing unsafe combinations of DPI, feed, media, and pass settings.

### Requirements

A `PrintMode` must specify:

- backend and device version;
- tested firmware range or exact observation;
- raster grid and physical output resolution separately;
- supported bit depth and drop codes;
- band height, packing, compression, and interleaving;
- logical-to-physical channel order;
- pass schedule and color/white deposition order;
- media source, sheet size range, printable box, origin, margins, and eject behavior;
- direction, speed, pass delay, and white-pass settings only when controllable;
- film and ink compatibility;
- screening, linearization, channel limits, wet-load limits, and profile compatibility;
- known failure modes and required nozzle/feed checks.

AcroRIP's ET-8550 release notes show that media options, film recognition, rear feed, central guides, page size, and ink sequence are model-specific concerns. Treat those as evidence for the model, not as values to copy blindly.

### Data and interfaces

A mode is immutable once used by a production job. New values create a new mode revision. `DeviceCapability` describes what the backend can honor; UI controls are generated or gated from it.

### Dependencies

Protocol investigation, device setup, physical media tests, screen engine, calibration, and recipe compatibility.

### Validation

For each mode, print geometry, feed, channel, pass, and overlap targets. Test each documented setting independently while holding others fixed. Record whether a setting changes bytes, physical output, or neither.

### Acceptance

Only a tested mode can be marked production-ready. Changing a setting that the backend cannot honor is impossible or produces a visible unsupported warning. The mode ID appears in the output inspector and job snapshot.

## Feature 4: Artwork import and source metadata

### Purpose
Load artwork without losing physical dimensions, transparency, color information, or unsupported-feature warnings.

### Initial supported subset

- PNG truecolor and truecolor-with-alpha for SDR artwork.
- Embedded ICC profiles and explicit sRGB signaling.
- Untagged RGB with a visible, configurable sRGB assumption.
- Separate grayscale white masks imported as a first-class asset.

### Requirements

- Preserve original bytes and a decoded metadata record.
- Preserve alpha as coverage/opacity data, not gamma-encoded color.
- Follow PNG color metadata precedence and detect conflicting metadata. The W3C PNG specification documents `cICP`, `iCCP`, `sRGB`, and `cHRM/gAMA` precedence.
- Reject or explicitly warn on HDR signaling for the initial SDR renderer.
- Reject APNG or import only its defined static image with a warning. Do not silently choose an animation frame.
- Enforce decoded dimensions, pixel count, profile size, and archive/resource limits.
- Store physical dimensions independently from screen pixels and PPI metadata.
- Preserve hidden RGB for diagnostics but guarantee it cannot create ink where alpha is zero.
- Detect malformed profiles, invalid dimensions, unsupported bit depth, and unsupported color types.
- Never silently discard embedded profiles or convert twice.

### Data and interfaces

`SourceAsset` contains original hash, file type, dimensions, color metadata, alpha metadata, physical-size proposal, profile reference, and decoder version. `DecodedImage` declares encoding, channel meaning, alpha association, and working-space conversion status.

### Dependencies

PNG decoder, color-management library, asset store, security limits, and preview renderer.

### Validation

Fixtures must include hidden saturated RGB under zero alpha, partial alpha, opaque RGB-white, opaque white artwork with zero CMYK after conversion, embedded sRGB, embedded ICC, conflicting tags, HDR signaling, APNG, malformed profiles, huge decompression ratio, and non-square PPI.

### Acceptance

All supported fixtures render with declared metadata behavior. Unsupported files fail with a useful reason. Physical size is deterministic, and alpha ramps do not produce halos or hidden-RGB ink.

## Feature 5: Color management and rendering pipeline

### Purpose
Convert source color into a declared working space and device input consistently.

### Requirements

- Define source RGB, working linear RGB, proof/preview RGB, logical CMYK, white coverage, calibrated device coverage, and screened drop-code representations.
- Use a licensed color-management engine rather than inventing an ICC engine.
- Apply source profile conversion exactly once.
- Define rendering intent, black-point compensation, gamut handling, and untagged-source policy.
- Keep display proof separate from device output conversion.
- Define alpha-aware conversion: interpolate/composite in the chosen working representation and avoid hidden RGB leakage at zero coverage.
- Define pure-black and neutral policy. GCR/black generation must be part of the validated separation/profile strategy, not an unexplained slider.
- Support CMYK input only when its profile and conversion path are specified and tested.
- Keep source artwork, device recipe, calibration curves, output profile, and finishing recipe distinct.
- Record library version, profile hashes, intent, and all overrides.

### Dependencies

Color library, source importer, profile registry, calibration, white separation, preview, and test charts.

### Validation

Use known RGB/CMYK test values, neutrals, skin tones, saturated colors, black ramps, alpha edges, and profile round trips. Verify no double transform by comparing expected reference values and output metadata.

### Acceptance

The same job snapshot produces the same logical device values across band sizes and process runs. The preview declares its limitations and does not claim garment color accuracy before physical calibration.

## Feature 6: White generation, supplied masks, and white curves

### Purpose
Provide independent, predictable white control equivalent in concept to Digital Factory's generated underbase, highlight, supplied-layer, strength, choke, and manual-curve controls.

### Requirements

Provide explicit controls for:

- white source: alpha, automatic tonal generation, supplied grayscale mask, or disabled;
- policy: opaque underbase, tonal underbase, highlight, underbase-plus-highlight, or coverage-only;
- response curve input and output domain;
- white intensity under dark colors and highlights;
- mask combination: replace, union/max, intersection/min, or controlled blend;
- physical-unit choke/spread;
- minimum feature size warning;
- white slot allocation and independent slot calibration;
- maximum white coverage and combined wet-load limit;
- white-only and white-over-color diagnostic modes.

Recommended explicit precedence:

`source white -> selected combination rule -> tonal response -> choke/spread -> knockout/perforation clear mask -> calibrated slot allocation -> device screening`.

- A custom white mask must not modify process alpha implicitly.
- Explicit white artwork must remain eligible when CMYK is zero.
- Fully transparent pixels must remain clear unless the user deliberately invokes an extension mask.
- Automatic crop must not remove intentional opaque white.
- Curve presets must be named and serializable, not represented only by UI slider positions.

### Dependencies

Alpha-aware renderer, mask engine, morphology engine, calibration, physical-slot mapping, screening, and inspector.

### Validation

Print isolated white, alpha ramps, white under black, white under saturated colors, opaque white artwork, thin strokes, holes, choke/spread ramps, supplied-mask replacement/union cases, and two-slot allocation charts.

### Acceptance

The generated white plate matches the declared policy, process coverage remains unchanged when only the white mask changes, and physical output demonstrates independent white control. Any uncalibrated curve is visibly labeled experimental.

## Feature 7: Knockout, perforation, and clear-mask enforcement

### Purpose
Remove selected artwork colors or create deliberate openings while guaranteeing that no other plane reintroduces ink.

### Requirements

- Support reversible garment-color knockout with tolerance, transition, and preview.
- Support selected color/range knockout as a non-destructive mask.
- Store the chosen garment/substrate color in the artwork recipe.
- Use one final clear mask for CMYK, white, and every future channel.
- Apply clear-mask enforcement after white spread/geometry operations.
- Provide perforation controls in physical units: hole diameter/range, spacing, eligible areas, edge protection, and deterministic screen/phase behavior.
- Warn if holes erase fine text or undermine transfer opacity.
- Keep solid and perforated variants as separate recipe revisions.

Digital Factory documents black and color knockout controls with tolerance, hardclip, opaque, midtone, and transparency behavior. Use that as a behavior reference, not a copied implementation.

### Dependencies

Mask algebra, morphology, white separation, screening, preview, and physical press/wash tests.

### Validation

Use black, navy, near-black, gradients, antialiased edges, transparent backgrounds, thin lettering, holes inside letters, and overlapping objects. Verify zero coverage on every plane inside a clear mask.

### Acceptance

A knocked-out pixel cannot reappear through white generation, screening, or device encoding. Physical tests establish acceptable hand feel and durability before perforation is called production-ready.

## Feature 8: Device screening and ink limiting

### Purpose
Convert continuous calibrated coverage to printable dot/drop codes while managing channel deposition.

### Requirements

- Start with one deterministic screening method and document its phase, cell, threshold, and supported levels.
- Keep screen phase in page coordinates so band rendering does not create seams.
- Define whether screening occurs before or after calibration and ink limiting.
- Support per-channel and combined CMYK limits.
- Treat white slots as measured physical deposition, not duplicate percentages.
- Store screen parameters in the mode/recipe hash.
- Provide diagnostic continuous-tone and screened views.
- Support a file sink for unencoded screened output.
- Defer advanced stochastic/halftone choices until ramps, moiré, grain, band seams, and dot gain are measured.

### Dependencies

PrintMode, calibration, band renderer, physical-slot allocation, inspector, and benchmark fixtures.

### Validation

Render identical images at different band heights and compare screen phase. Test ramps, flat fields, gradients, isolated channels, overprints, white/process overlaps, and maximum combined load.

### Acceptance

Banded and unbanded output match the declared tolerance. No channel exceeds calibrated limits. Physical samples show no unacceptable pooling, banding, or repeatable screen artifacts in the validated mode.

## Feature 9: Physical calibration, linearization, and ICC recipe management

### Purpose
Turn one printer/ink/film/mode/finishing condition into a named repeatable recipe.

### Requirements

- Separate device setup, print mode, calibration, output profile, and garment/finishing recipe.
- Measure single-channel response and set conservative limits.
- Measure combined process-channel wet load.
- Calibrate each white slot and combined white policy.
- Record chart generator version, patch layout, instrument, illuminant/observer, measurement date, operator, substrate, press settings, and sample identifiers.
- Support importing measurement data from external tools such as ArgyllCMS when appropriate.
- Treat ArgyllCMS's documented CMY/CMYK profile capabilities and lack of N-color profiles as a constraint. Do not promise a generic CMYKW profile wizard without another validated method.
- Require independently printed holdout patches. Do not judge a profile only on its training chart.
- Invalidate or warn on recipe changes affecting ink, film, firmware, mode, screen, white policy, or finishing.
- Make calibration curves and profiles immutable by hash once used.

### Dependencies

Test target generator, measurement import, profile registry, white policy, screen engine, recipe store, and physical lab procedure.

### Validation

Recalibrate the same condition, compare holdouts, change one condition at a time, and verify that incompatible profiles are rejected or clearly marked. Test missing instrument data and partial chart files.

### Acceptance

A named recipe can be reproduced from its record, and color/registration/white results meet tolerances frozen against the commercial reference or an agreed baseline. No unmeasured factory profile is presented as a validated DTF output profile.

## Feature 10: Layout, transforms, and film orientation

### Purpose
Place artwork at physical dimensions and guarantee one correct mirror/rotation/origin transform.

### Requirements

- Store positions and dimensions in physical units, not UI pixels.
- Support aspect-lock, numeric width/height, rotation, duplication, alignment, distribution, and bounds warnings.
- Keep garment view and film view distinct.
- Declare one mirror owner. Apply mirror to all planes and marks together exactly once.
- Preserve an off-center marker and asymmetric text in every orientation fixture.
- Make origin, printable box, margins, feed direction, and page size explicit in the mode.
- Warn on low effective PPI, clipping, unsafe margins, and unsupported media size.
- Store all transforms in the job snapshot and expose the effective transform in the inspector.

### Dependencies

Asset model, printable-box geometry, renderer, preview, gang-sheet layout, and backend mode.

### Validation

Use rulers, crosses, asymmetric text, off-center markers, rotated objects, masks, and multi-plane marks. Compare garment view, film view, logical planes, and physical output.

### Acceptance

The printed ruler and registration marks are within measured tolerances at center and corners. No double mirror, shifted mask, crop-induced displacement, or rotation mismatch occurs.

## Feature 11: Gang sheets, nesting, and portable job packages

### Purpose
Prepare repeatable sheets efficiently without sacrificing placement, mask, recipe, or reprint fidelity.

### Requirements

- Start with manual placement and deterministic rectangle/block nesting.
- Support bulk import, folder import, quantities, gutters, margins, rotation rules, copy independence, and printable-area warnings.
- Use transparency-aware crop with an explicit crop offset.
- Keep automatic crop of opaque white disabled by default.
- Support labels, notes, job names, rulers, and reserved margins.
- Save a path-independent bundled package containing source assets, masks, layout, settings, profiles, device/recipe hashes, software/backend versions, and checksums.
- Permit path-dependent lightweight layouts only as an explicit option.
- Preserve individual placement identity for selective reprint.
- Add shape nesting, roll support, barcode automation, and multi-lane output only after cut-sheet quality is stable.

Digital Factory documents block/manual layouts, copies, auto-crop, labels, selected-job reprint, and bundled versus path-dependent layouts. These define useful behavior to emulate, not a requirement to copy its format.

### Dependencies

Layout model, asset store, job schema, renderer, inspector, queue, and archive security.

### Validation

Test missing assets, moved directories, duplicate sources, independent versus cloned copies, crop offsets, rotated masks, oversized designs, exact gutters, and partial reprint selection.

### Acceptance

A bundled sheet opens on a clean machine with identical geometry. Selected items can be reprinted as a new attempt without changing the original sheet or recipe.

## Feature 12: Preview and final-output inspector

### Purpose
Make every transformation observable before submission.

### Required views

1. Garment simulation, clearly labeled approximate.
2. Film orientation with mirror state.
3. Logical CMYK+W planes.
4. Calibrated physical-slot rasters.
5. Screened device raster/drop codes.
6. Coverage/statistics and out-of-bounds warnings.

### Requirements

- Display true pixel scale and physical dimensions.
- Toggle planes and layers independently.
- Show source mask, generated white, final clear mask, and final screened result.
- Show mode ID, device setup ID, recipe ID, profile hash, screen ID, and transform ownership.
- Allow exporting diagnostic planes and target sheets.
- If final bytes cannot be decoded, retain the exact pre-encoding artifact and display that as pre-encoding truth.
- Never show a regenerated approximation as the submitted raster.

Digital Factory's RIP-only/raw-data viewer is the direct benchmark: plane toggling, true-pixel display, substrate preview, and export are all valuable behaviors.

### Validation and acceptance

A diagnostic fixture must reveal any accidental mirror, double alpha, white under transparent pixels, clipped placement, screen seam, wrong slot allocation, and changed profile. Inspector output must identify the exact job and recipe hash.

## Feature 13: Queue, reprint, and crash-safe job lifecycle

### Purpose
Make native production recoverable without silent duplicate printing.

### Requirements

- Use durable states: draft, preflight-failed, held, rendering, ready, submitting, submitted, ambiguous, needs-review, cancelled, failed, and operator-confirmed-finished.
- Persist a job snapshot before rendering.
- Persist a submission attempt before writing bytes.
- Retain exact raster/stream artifacts when configured and disk allows.
- Provide RIP-only/inspect/release flow before native submission.
- Allow selected-placement reprint as a new attempt linked to the original.
- Distinguish successful spooling from usable pressed transfer.
- Require operator review after crash/disconnect at uncertain boundaries.
- Provide queue pause, hold, release, cancel, priority, and one sender per device.
- Reject replay when device setup, mode, firmware, or recipe compatibility is not verified.

### Validation

Inject faults before render, after render, before submission, mid-write, after final write, during cancellation, during restart, and on disk-full. Verify no path silently retries an ambiguous job.

### Acceptance

A restart reconstructs the correct state and artifact. Every physical attempt has a unique ID, and reprints never overwrite history.

## Feature 14: Diagnostics and operator troubleshooting

### Purpose
Reduce diagnosis time without hiding physical causes behind software settings.

### Requirements

Generate diagnostic targets for:

- nozzle/channel identity;
- isolated logical and physical channels;
- white ramps and process ramps;
- registration crosses and rulers;
- feed scale and page origin;
- screen/raster inspection;
- overlap and wet-load patches;
- alpha/hidden-RGB/edge fixtures.

Provide a compact job report containing device setup, mode, media, ink, recipe, profile, screen, software version, hashes, warnings, timestamps, and attempt state. Keep artwork optional and support redaction.

Troubleshooting order must distinguish raster error, mapping error, mode/feed error, nozzle health, and finishing error. Do not suggest cleaning or maintenance automatically.

### Acceptance

An operator can reproduce a reported failure from a diagnostic package without uploading private artwork. Each common failure has a bounded decision tree and a next test.

## Feature 15: Source-aware knockout, color replacement, and named colors

### Purpose
Provide high-end artwork preparation without pretending to be a full graphics editor.

### Requirements

- Make knockout masks reversible and non-destructive.
- Support black/near-black and selected color ranges with tolerance, hard clip, opaque, midtone, and transparency behavior where meaningful.
- Keep garment color and selected source color separate concepts.
- Add a color replacement operation with before/after preview and saved parameters.
- Later support named spot-color preservation only through a document renderer that can prove spot channel identity and overprint behavior.
- Preserve unsupported vector features as warnings, not approximations silently.

### Dependencies

Source importer, color engine, mask engine, preview, white separation, and PDF/SVG strategy.

### Acceptance

Knockout clears every printed plane and can be disabled without losing original artwork. Named-color behavior is tested with actual source fixtures before being advertised.

## Feature 16: Input formats and document fidelity

### Initial scope
PNG plus aligned grayscale white masks. Add TIFF after metadata/alpha behavior is stable. Defer PDF/SVG until a renderer can preserve physical dimensions, profiles, transparency, overprint, fonts, and named spot colors.

### Requirements

- Publish supported format/version and color-feature matrix.
- Store original files and renderer version.
- Reject unsupported blend modes, masks, fonts, overprint, animations, HDR, and spot channels with actionable diagnostics.
- Include flattening only as an explicit user-confirmed conversion that shows the resulting preview.
- Add fixture documents for transparency, clipping paths, embedded profiles, text, gradients, overprint, and spot white.

### Acceptance

Every supported feature has a fixture and expected raster/plane result. Every unsupported feature produces a clear preflight failure or explicit user-confirmed approximation.

## Feature 17: Preflight and safety checks

### Purpose
Prevent predictable failures before irreversible output.

### Requirements

Preflight must check:

- missing or changed assets;
- unsupported profiles/formats;
- unverified device setup;
- incompatible mode/recipe/profile;
- dimensions, margins, clipping, PPI, and rotation;
- alpha/white policy conflicts;
- clear-mask violations;
- channel/wet-load limits;
- unsupported controls for the selected backend;
- disk space, decoded pixel limits, and retained-artifact requirements;
- firmware/setup changes affecting replay.

Warnings must be categorized as blocking, requires confirmation, or informational. The report must explain the exact setting and artifact responsible.

### Acceptance

Known invalid fixtures cannot reach native submission. A user can export the preflight report and understand how to correct every blocking issue.

## Feature 18: Distribution-ready architecture and operations

### Purpose
Make future sharing possible without prematurely building a commercial service.

### Requirements

- Headless core, device encoder, transport, and desktop UI remain separate modules.
- Version job schemas and provide migrations.
- Use portable archives with safe extraction and path traversal protection.
- Maintain dependency, license, profile, font, and bundled-asset inventory.
- Build clean-machine installer tests.
- Sign releases when practical, verify updates, support rollback, and never update during active submission.
- Provide support bundles with redacted logs, diagnostic targets, hashes, and optional artwork.
- Add resource limits and malformed-input handling before accepting third-party files.
- Provide backup/restore and disk-full recovery.
- Store no telemetry or artwork remotely by default.
- Keep license/entitlement concerns at application boundaries, not in render math.

### Acceptance

A clean supported Windows installation can import a bundled job, reject an incompatible recipe, render a diagnostic, and recover from a failed update without development tools. A second machine can inspect a job without silently changing its device setup.

## Feature 19: Performance, memory, and cancellation

### Purpose
Make large sheets practical and prevent resource exhaustion.

### Requirements

- Render in bounded bands from the first full-sheet implementation.
- Use disk-backed intermediate artifacts when memory would exceed limits.
- Carry explicit row halos for morphology, choke/spread, and filters.
- Keep screen phase in page coordinates.
- Check cancellation at decode, transform, band, encode, and submission boundaries.
- Check disk space before allocating large artifacts and during long jobs.
- Use decoded pixel and profile-size limits.
- Benchmark CPU rendering on representative sheets before considering GPU acceleration.
- Define equality/tolerance between banded and unbanded output.

### Acceptance

The 13 × 19 inch, 1440 × 720 engineering example does not require multiple gigabytes of unbounded heap memory. Output is stable across band sizes, and cancellation leaves no corrupt job state.

## Feature 20: Measurement and quality acceptance system

### Purpose
Turn “high-end” into repeatable evidence.

### Requirements

Maintain controlled tests for:

- channel identity;
- nozzle health;
- feed scale and registration;
- CMYK and white ramps;
- overlaps and wet-load limits;
- fine positive/negative type;
- transparency and white edges;
- neutrals, skin tones, saturated colors, near-black;
- solid versus perforated transfers;
- press, cure, peel, stretch, and wash conditions;
- throughput, waste, and reprint accuracy.

Use a commercial reference when available, but freeze tolerances only after measuring the baseline. Hold out validation patches from profile creation. Record instrument and viewing conditions.

### Acceptance

A mode/recipe is “validated” only when repeated pressed samples meet agreed tolerances across sessions. Digital equality alone cannot establish physical color or durability.

## Feature 21: Deferred capabilities and explicit non-goals

Do not implement these in the first ET-8550 native release unless a test proves a need:

- roll handling and cutters;
- multi-lane production;
- fluorescent, varnish, or UV channels;
- broad printer-family support;
- automatic shape nesting;
- barcode automation and hot folders;
- cloud queues, accounts, payments, subscriptions, or DRM;
- automatic cleaning, firmware, or waste-counter operations;
- a full vector design editor;
- a generic N-color ICC profile wizard;
- advanced screening algorithms before the baseline screen is measured.

These are not rejected permanently. They require separate device capability contracts and acceptance tests.

## Dependency order

1. Device probe, protocol notes, and physical setup verification.
2. Device setup and versioned PrintMode.
3. File sink, banded raster model, and output inspector.
4. PNG/metadata import and physical layout transforms.
5. Color working-space contract.
6. White generation and supplied-mask semantics.
7. Clear masks, calibration, screening, and physical-slot allocation.
8. Native encoder and transport.
9. Preflight and crash-safe queue.
10. Quality recipe and measurement workflow.
11. Daily-use layout/gang sheets and selective reprints.
12. Distribution readiness and later high-end enhancements.

Do not build a polished production queue before the file sink, inspector, native mode, and physical test targets exist. Do not build a profile wizard before the printer condition is physically stable.

## Definition of ready for the first native release

The first native release is ready for personal production only when all of the following are true:

- One ET-8550 setup and one cut-sheet mode are physically verified.
- Independent K and W addressing is demonstrated, plus required CMY channels.
- Mirror, placement, scale, feed, and layer order pass asymmetric targets.
- White policy, supplied masks, choke/spread, and clear masks pass digital fixtures and physical samples.
- Channel limits, white allocation, screening, and wet-load behavior are measured for the recipe.
- A compatible color condition/profile is documented, or output is honestly labeled unprofiled.
- Final logical, physical-slot, and screened representations are inspectable.
- Jobs are portable, immutable, resumable, and reprintable without silent duplication.
- Paper-out, disconnect, cancellation, restart, and disk-full paths are tested.
- Unsupported inputs and incompatible settings are blocked before submission.
- No commercial RIP is needed to create the printer-ready artifact.

Future distribution requires the additional installer, migration, security, licensing, support, and compatibility gates in Feature 18.

## Research references

- Fiery Digital Factory DTF product and 12.1 capabilities: https://www.fiery.com/products/digital-factory-dtf/
- Fiery Layer Tab: https://help.cadlink.com/website/digital_factory/en/production/menus/layer_tab.htm
- Fiery Queue Tab: https://help.cadlink.com/website/digital_factory/en/production/queue_tab.htm
- Fiery Digital Factory Interface: https://help.cadlink.com/website/digital_factory/en/production/vpm_interface.htm
- Fiery Raw Data viewer: https://help.cadlink.com/website/digital_factory/en/bonus_documentation/view_raw_data.htm
- Fiery Underbase Strength and Manual Curve: https://help.cadlink.com/website/digital_factory/en/bonus_documentation/underbase_strength_and_manual_curve.htm
- Fiery KnockMeOut: https://help.cadlink.com/website/digital_factory/en/bonus_documentation/knockmeout.htm
- Fiery Gang Sheet Builder: https://help.cadlink.com/website/digital_factory/en/production/menus/gang_sheets.htm
- AcroRIP Features: https://acrorip.com/features.php
- AcroRIP Updates: https://acrorip.com/updates.php
- AcroRIP Supported Printers: https://www.acrorip.com/supported-printers.php
- AcroRIP Support and Training: https://acrorip.com/support-training.php
- Microsoft WritePrinter: https://learn.microsoft.com/en-us/windows/win32/printdocs/writeprinter
- W3C PNG Specification: https://www.w3.org/TR/png-3/
- ArgyllCMS print calibration: https://www.argyllcms.com/doc/printcal.html
- ArgyllCMS ICC profile creation: https://www.argyllcms.com/doc/colprof.html
- Epson film precautions: https://files.support.epson.com/docid/cpd6/cpd63670/source/pro_graphics/source/usage_notes/reference/film_safety.html
