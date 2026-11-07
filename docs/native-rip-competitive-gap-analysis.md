# Veylora RIP: learning from Digital Factory and AcroRIP

November 7, 2026. Companion to [the original plan](et8550-rust-rip-plan.md) and [the initial review](et8550-plan-review.md).

## 1. Direction, now resolved

Your clarified goal is **Veylora RIP, an independent native RIP for personal high-end DTF work, designed so it can later be distributed and potentially sold**. The software must generate and submit its own printer-ready output with independent process and white control. A commercial RIP must not be required to use the finished product.

This supersedes the initial review's default recommendation to deliver a commercial-RIP-assisted workflow first. A legitimate commercial RIP remains useful as a temporary benchmark, not a production dependency or a substitute for native acceptance. Offline prepress can still be developed in parallel, but it is not completion of this product.

**Recommendation:** use Digital Factory as the benchmark for color, white control, diagnostics, and production workflow. Use AcroRIP as a benchmark for direct printer configuration and a relatively focused operator workflow. Aim for excellent output on one verified ET-8550 configuration before attempting their hardware breadth.

The research does not establish that either is objectively “the best.” Vendor feature lists are not comparative print-quality measurements. Their documented capabilities are valuable references, not proof that our implementation can match their output or speed.

### What high-end should mean here

- Correct source interpretation, dimensions, transparency, and separation.
- Predictable white backing and fine detail without avoidable halos.
- Measured color accuracy and repeatability for a named ink/film/garment recipe.
- Smooth ramps, controlled ink load, stable registration, and acceptable durability.
- A reliable, recoverable workflow with portable jobs and transparent settings.
- Honest limits: software cannot make a converted ET-8550 match an industrial printer's throughput, maintenance system, or every media capability.

High-end output is a measurable target, not the number of controls in the UI.

## 2. Evidence boundaries

I checked current public Fiery/CADlink and AcroRIP pages listed at the end. Fiery's product page describes Digital Factory 12.1 and distinguishes optional modules. AcroRIP's public pages list v11.4, with the release entry dated May 15, 2025. That is the latest release listed on the pages reviewed, not a guarantee about every reseller package.

Fiery's shared help covers multiple devices and workflows. Features such as layers, variable dots, plane shifts, or white passes must not be assumed available on the ET-8550 simply because the generic interface documents them. Some white-curve help includes DTG examples, which are not DTF curing or ink recommendations.

AcroRIP's detailed manual and ICC-creation guide are behind request forms. I did not submit personal information or claim to have read those guides. Missing public evidence is marked **not established**, not “AcroRIP cannot do this.” No vendor application or physical printer was tested. The Fiery shop upgrade page returned HTTP 403, so no pricing or edition detail is inferred from it.

## 3. Competitive gap matrix

Status refers to the original plan, not implemented software. **Covered** means already planned, **partial** means it needs an explicit contract or acceptance gate, and **missing** means it is not materially specified. P0 is foundation work, P1 is needed for the high-quality single-printer release, and P2 is a later enhancement.

| Capability | Digital Factory evidence | AcroRIP evidence | Our plan and recommendation |
| --- | --- | --- | --- |
| Independent white and process output | White/color processing and layer controls documented, device-dependent. [F2] | One-pass white/color and custom channels advertised. [A1] | **Covered but unproven, P0.** Native addressing and physical deposition order are the first gate. |
| Ink-order setup | Ink-order wizard available where supported. [F6] | Custom mapping and ET-8500/8550 sequence update documented. [A1,A2] | **Partial, P0.** Add a setup wizard, labeled physical slot test, mapping validation, and versioned device configuration. |
| Media/feed/mode reliability | Queue controls include media, direction, pass delay, white passes, margins, varying by device. [F3] | ET-8550 media options, film-sensor fixes, central-guide placement, and rear-feeder instructions. [A2] | **Partial, P0.** Define a validated print-mode bundle rather than arbitrary independent dropdowns. |
| White underbase and highlight | None, underbase, highlight, combined, coverage underbase, strength, choke, adaptive alpha. [F2] | One-pass white/color established; equivalent detailed control set not established in reviewed public pages. [A1,A4] | **Partial, P1.** Existing density/highlight/shadow controls need named modes and exact semantics. |
| White response curves | Manual curve and presets documented. [F4] | Detailed curve editor not established. | **Missing, P1.** Add editable white-response curves, preset comparison, and a declared curve input. |
| User-authored white | Layer documentation distinguishes supplied white from automatically generated white. [F2] | Arbitrary supplied white-plane import not established. | **Partial, P1.** Add aligned grayscale mask import, replace/combine modes, and explicit precedence. Native output removes the PNG-handoff constraint. |
| Fine registration correction | X/Y plane shifts documented for certain 8-bit devices. [F2] | Equivalent UI not established. | **Missing explicit requirement, P1.** Add measured offsets only where the native mode can honor them. Do not use shifts to conceal bad feed or nozzle health. |
| Color controls | ICC, photo/graphics adjustments, CMYK curves, ink limits, saturation option. [F1,F2,F3] | ICC profiles advertised; supplied profile examples are model/ink-specific. [A1,A2] | **Partial, P1.** Add neutral/black policy, per-channel curves, tagged CMYK handling, and quality validation. |
| Calibration and profile creation | Spectrophotometer-based Color Profiler is an optional module. [F1] | An ICC-creation guide is offered by request, implementation not inspected. [A4] | **Covered workflow, partial UI, P1.** Integrate measurement import and calibration records before building a profile wizard. |
| Named-color replacement | Job color/range replacement documented. [F6] | Equivalent feature not established. | **Missing, P1/P2.** Start with saved custom named colors and swatch testing. Add true PDF spot preservation with vector input later. |
| Background removal and knockout | Knockout tools and optional FluidMask module. [F1,F7] | Equivalent public detail not established. | **Partial, P1.** Knockout is planned. Add reversible masks and clean-edge preview. Advanced automatic masking is P2. |
| Final separation inspection | RIP Only, raw-data viewer, plane toggles, true-pixel view, export. [F5] | Equivalent final-raster inspector not established. | **Partial, P0/P1.** Inspect the actual retained native separations, not merely regenerated previews. Include physical slots and screened dots. |
| Gang sheets | Bulk import, manual/block nesting, crop, copies, labels, portable bundles. [F1,F8] | Multi-image handling in v11. [A1,A2] | **Covered core, P1.** Add transparency-aware crop, explicit copy counts, undo/redo, and portable bundles. |
| Selective reprint | Builder can create a sheet from selected jobs. [F8] | Equivalent workflow not established. | **Missing, P1.** Reprint selected placements/copies, preserving their recipe and artwork settings, as a new attempt. |
| Queue and retained jobs | Hold, release, RIP-only, reserve list, job backup, printer query. [F6] | Public pages reviewed do not establish comparable durable queue semantics. | **Covered, P1.** Keep the planned crash-safe submission model and add RIP-only release after inspection. |
| Production labels and automation | Markup, barcodes, multi-lane printing, optional job sorter. [F1,F6,F8] | Roll feeding is model-specific. [A1,A2] | **Covered later, P2.** Labels useful early. Multi-lane/roll production is not an ET-8550 cut-sheet milestone. |
| Cost and consumption | Costing entry in interface, optional InkSaver. [F1,F6] | Equivalent costing not established. | **Missing, P2.** Estimate media/ink with measured drop-volume calibration and explicit uncertainty. Do not call coverage exact milliliters. |
| Wider ink systems | Fluorescent process inks and optional broader workflows. [F1] | UV varnish and WAVE features scoped to particular wide/UV devices. [A1,A2] | **Out of initial scope.** Extensible plane identity is enough now. Do not add fluorescence/varnish switches to a CMYK+W printer. |

## 4. The most important additions

### 4.1 A validated print-mode model, not just a resolution setting

This is the largest technical specification gap. AcroRIP's ET-8550 fixes show that driver support is a combination of ink sequence, feed behavior, page geometry, and media settings, not just six independently writable channels. [A2]

A native `PrintMode` should identify:

- Supported device/backend version and tested firmware information.
- Raster sampling grid separately from the device's actual dot-placement/drop capabilities.
- Addressable physical slots and permitted logical ink assignments.
- Bit depth/drop codes, compression, band packing, and device command framing.
- Required interleaving/pass schedule and physical color/white order.
- Media source, reliable size range, printable box, origin, feed/eject behavior.
- Direction and speed settings, if actually controllable.
- Associated screening, linearization, ink limits, and compatible recipe versions.

Do not expose every combination. Offer only tested presets and put experimental settings behind explicit warnings. A firmware identifier is evidence about the tested setup, not a guarantee that all other revisions fail.

“One pass white and color” means the operator need not manually refeed the sheet in AcroRIP's advertised workflow. It does **not** prove a single carriage sweep, a particular nozzle timing model, or identical capabilities in our encoder. [A1]

### 4.2 A complete white-separation system

The original plan has the right fundamentals but needs an explicit operating model inspired by Digital Factory's documented separation between generated white, highlight, and supplied white. [F2,F4]

Add:

1. **White source:** automatic alpha, tonal response, supplied grayscale mask, or disabled for diagnostics.
2. **White policy:** opaque backing, tonal backing, and highlight response with named, documented behavior.
3. **Custom-mask combination:** replace, maximum-union, or intersection. Do not hide an arbitrary additive rule that can exceed limits.
4. **Response editor:** curve with declared input, such as normalized working-space luminance. Avoid an unlabeled “strength” slider whose math changes between modes.
5. **Geometry:** physical-unit choke/spread, optional measured registration offsets, thin-feature warnings, and before/after overlay.
6. **Limits:** clamp/calibrate white output and allocate it to the physical white slots using the tested mode.

Suggested precedence to specify and test:

`source/automatic white -> explicit combination rule -> response and geometry -> enforce knockout/perforation clear mask -> calibrated white allocation -> screening`

Keep process-color coverage separate. A custom white mask must not alter process alpha implicitly. A mask replacement that can intentionally extend beyond source alpha should be an explicit option, while intentional knockout/perforation must remain clear on every plane.

The first native release can import a separate grayscale mask at a declared physical alignment. It does not need PDF spot support to provide genuine independent white. Add spot-white extraction when the document renderer can preserve named separations reliably.

### 4.3 Color quality beyond “supports ICC”

Both products advertise ICC support, but Digital Factory's documentation is more explicit about adjustment tools and optional profiling. Its proprietary saturation rendering behavior is not something we can claim to reproduce merely by selecting the standard ICC saturation intent. [F1,F3]

Add these explicit requirements:

- Source-profile policy for RGB and, when supported, CMYK input.
- Documented rendering intent and black-point-compensation policy, with compatible transforms only.
- A predictable pure-black/neutral policy. GCR and black generation belong in a validated profile or separation strategy, not a late uncalibrated slider.
- Per-channel linearization curves and independent white calibration.
- CMYK total-ink limits plus a separate mode-specific check of combined process and white wet load.
- Named custom color targets, printed swatch charts, measured selection, and saved recipe-specific matches.
- Profile/recipe compatibility checks and warning on quality-affecting overrides.
- Color verification with independently printed holdout patches, repeated measurements, and recorded viewing/measurement conditions.

A four-channel profile can characterize a consistent backing condition. It cannot fully model arbitrary independently edited white or garment show-through. Freeze a profiling white policy, then label departures as unvalidated. Color-accurate soft proof is similarly limited by the monitor, garment, white coverage, and measurement model.

**Do not invent a new color engine to be “native.”** Native here means no commercial RIP is needed. A properly licensed color-management library and external profile-building tools are compatible with that goal. Preserve library/tool version provenance and decide distribution obligations before bundling anything.

### 4.4 A final-output inspector

Take direct inspiration from Digital Factory's RIP-only and raw-data viewer. [F5,F6]

Provide four levels of inspection:

- Layout/garment preview, clearly labeled simulation.
- Continuous-tone logical CMYK+W planes.
- Allocated physical-slot rasters after calibration and limiting.
- Screened device raster/drop codes at true pixel scale.

Show effective dimensions, mode ID, plane identity, coverage statistics, and out-of-bounds/clipping warnings. Allow target and plane export. If the final stream cannot be decoded for inspection, retain the exact pre-encoding rasters and verify encoder round-trip invariants where feasible. Do not call an independently recomputed preview the final output.

This inspector is a development instrument and a support tool. Build it before advanced nesting.

### 4.5 More useful editing, not a full graphics suite

Digital Factory's builder provides useful inspiration without copying its interface. [F8]

Add undo/redo, transparency-aware crop, align/distribute, multi-selection, numeric size/position, clone versus independent-copy semantics, per-design quantities, and selected-item reprint.

Preserve original assets and non-destructive masks. Automatic cropping of opaque white must be off by default for DTF because white may be intentional artwork. Store the crop offset so removing transparent borders does not silently shift registration or mask alignment.

A rich drawing application is unnecessary. PNG plus aligned white masks is a sensible first input contract. For a distributable high-end release, add TIFF and a carefully chosen PDF/SVG path only after the renderer can preserve physical size, embedded profiles, fonts, transparency, overprint, and named spot colors correctly. Reject unsupported features explicitly rather than silently flattening them incorrectly. This is a recommendation for our scope, not an assertion that both competitors support an identical format list.

### 4.6 A quality lab built into the workflow

The original plan includes test targets. Promote them into a maintained quality workflow:

- Channel identity and nozzle-health record.
- Feed scale, geometric distortion, and process/white registration target.
- CMYK ramps, white ramps, process+white overlaps, and combined-load checks.
- Fine positive/negative type, thin lines, gradients, partial transparency, and isolated white.
- Color chart and independent holdout verification.
- Solid versus perforated transfer, controlled press and wash comparison.
- Quality-versus-throughput measurements for each validated mode.

Set acceptance numbers from the physical baseline, then freeze them before evaluating our output. Retain failures as regression fixtures where possible. “Looks great once” is insufficient for a high-end claim.

## 5. Build for future distribution without building a storefront

Your personal-first approach changes the priority of market discovery: it is no longer a prerequisite for building a tool you want. Future commercial readiness still requires deliberate engineering.

### Design in now

- **A headless core and thin desktop UI.** Keep artwork/separation, device encoding, and transport behind clear module boundaries. A modular desktop application is sufficient, not microservices.
- **A versioned device capability model.** One implemented ET-8550 backend, not a speculative universal driver framework.
- **Portable job bundles.** Assets, masks, layout, recipe snapshot, profile hashes and permitted embedded profiles, engine/backend versions, and checksums. Fiery's bundled gang sheets validate the usefulness of this workflow. [F8]
- **Recipe packages separate from device identity.** Share a recipe without silently overwriting someone's physical ink map. Imported recipes remain unverified until compatibility and setup checks pass.
- **Schema versions and migrations.** Keep originals/backups and reject incompatible future formats clearly.
- **Durable local job/attempt records.** Rendering and submission must survive UI crashes without uncontrolled duplicate output.
- **Settings provenance.** Distinguish device defaults, recipe values, per-artwork edits, and per-attempt overrides. Display effective values.
- **A dependency/license inventory.** This includes native libraries, document renderers, bundled utilities, fonts, profiles, and calibration data. Personal use does not settle later redistribution rights.
- **Portable diagnostics.** Redacted logs, support bundles, explicit consent before including artwork, and no automatic uploads by default.
- **Reproducible builds and regression fixtures.** Numerical/format tests, crash injection, band-size equivalence, and physical reference records.

### Required before giving it to others

- An installer tested on a clean supported Windows machine, with no development-tool dependency.
- A documented supported OS/device/firmware/mode matrix and setup instructions.
- Signed releases when practically available, verified update artifacts, and rollback. Never update the encoder during an active job.
- Import/resource limits, safe archive extraction, malformed-image/profile handling, and sandboxing of higher-risk document renderers where practical.
- Compatibility checks before printer-ready stream replay.
- Backup/restore, explicit retention limits for large raster artifacts, and disk-full recovery.
- A documented way to report issues and export a diagnostic target without sending private artwork.
- A clear license for your application and redistribution clearance for bundled assets.

### Defer until there is a reason

Subscriptions, activation servers, payments, dongles, accounts, cloud queues, remote telemetry, and a plug-in marketplace. Commercial-ready does not mean DRM-ready. An entitlement interface can be added at application boundaries later without placing license checks throughout the renderer.

Do not imitate AcroRIP's hardware channel-substitution example as an easy software fix. Changing what ink is physically in a channel is a maintenance/conversion operation. If ever supported, it requires operator/supplier procedures, a new verified map, and recalibration, not merely a dropdown change. [A1]

## 6. Answers to the earlier open questions

| Question | Decision now |
| --- | --- |
| Learning, personal shop, or product for sale? | Personal high-end native RIP, with distribution-ready foundations. Selling is optional later. |
| Is commercial RIP dependency acceptable? | No runtime/production dependency. A legitimate reference RIP may be used for development comparison if available. |
| What is the first supported platform? | Windows desktop, targeting a currently supported Windows release and testing the actual print stack. Do not copy stale Windows 7/8 recommendations from vendor pages. AcroRIP advertises Windows 11 support for v11, while its support page also contains older cautionary language. [A1,A4] |
| Which printer and media scope? | One ET-8550, cut-sheet film, one physically confirmed map and recipe initially. Competitor breadth is a long-term reference, not initial scope. |
| Which ink map? | Still a hardware fact, not answerable from marketing. AcroRIP's `KWCYMW` sequence cannot safely be translated into tank-fill instructions without decoding its slot convention and checking the actual conversion. [A2] |
| Which exact DPI, direction, pass sequence, and film margins? | Must be selected from a proven native mode. AcroRIP's model-specific size/feed guidance proves these need validation, not that its values transfer to our encoder. [A2] |
| What existing RIP failure are we solving? | Independence and full control are now the primary motivations. No claim about a specific vendor failure is needed. Operator experience should still guide UI priorities. |
| Do we need a spectrophotometer? | For credible measured color claims, obtain access to a compatible instrument or profiling service after establishing physical stability. Vendor profiling features reinforce the need for measurements, not a specific instrument purchase. [F1,A4] |
| What budget or deadline? | Not inferable from competitor features. Keep an explicit feasibility checkpoint, but do not turn the earlier ten-day proposal into a promise or user-approved budget. If native output is blocked, report the technical blocker instead of redefining prepress as success. |
| Do we need to sell to justify the work? | No. Personal usefulness is a sufficient goal. Future sale requires additional legal, support, security, and compatibility work, not a different color pipeline. |

Research can answer what controls and workflows are worth learning from. It cannot tell us what is in your tanks, how your firmware behaves, or what a transferred sample will measure.

## 7. Native-first implementation roadmap

### Gate A: device proof, before broad UI

Deliver a native probe, retained test rasters/streams, one verified mapping/mode, and physical reference results. Independently address process K and white, then all required channels. Prove overlap order, orientation, scale, feed/eject, and repeated transfer after restart. The final application must not invoke a commercial RIP to produce these samples.

If this gate fails, continue focused driver investigation only with a bounded decision checkpoint. Do not hide the failure behind a working gang-sheet editor.

### Gate B: correct native raster pipeline

Implement transparent PNG, explicit physical placement, independent supplied white masks, deterministic separation/calibration/screening, bounded bands, file sink, and output inspector. Define preview versus production representations and every transform's owner.

Acceptance includes zero ink in intentional clear regions, preserved opaque white, no double alpha or mirror, correct physical choke under unequal X/Y sampling, bounded memory, and stable band boundaries.

### Gate C: quality on one recipe

Add measured channel limits, linearization, white curves, registration calibration where feasible, a compatible output profile, and color verification. Tune screening with ramps and fine detail rather than assuming a basic threshold screen is production quality. A calibrated quality mode is more valuable than many unmeasured DPI choices.

Acceptance is repeated pressed samples meeting pre-agreed tolerances, including fine detail, neutrals, skin tones, saturated colors, white opacity, and adhesion. Keep throughput measurements separate from quality claims.

### Gate D: daily-use application

Build manual layout, quantities, crop, undo/redo, recipe selection, RIP-only/inspect/release, durable single-device queue, retained attempts, portable bundles, and selected-item reprint. Add diagnostics before automation.

Acceptance includes paper-out, disconnect, cancellation, disk-full, application restart, and ambiguous submission without silent duplicate printing.

### Gate E: high-end workflow additions

Add named-color matching, richer inputs with verified rendering, better masking, validated perforation, improved nesting, labels, and calibrated consumption estimates. Add a profile-creation UI only if it demonstrably improves the external measurement workflow.

### Gate F: distributable beta

Test clean installation, portable job import on another machine, setup wizard, incompatible-recipe rejection, schema migration, update rollback, support export, and license inventory. Keep native printer coverage limited to tested hardware configurations.

These gates define sequencing, not dates. Printer reverse engineering and physical quality work remain the schedule uncertainty.

## 8. Specific changes to make to the original plan

1. Change the top-level success criterion to **native output without a commercial RIP dependency**. Keep the reference RIP only as a development benchmark.
2. Add a Print Mode/Device Capability specification alongside Device Setup and Print Recipe.
3. Add independent white-mask import, curve editing, precedence rules, and explicit white-only/color-only diagnostics.
4. Add final physical-slot and screened-raster inspection, not just logical white preview.
5. Add basic calibration/ink-load gates before the native queue is called production-ready.
6. Add black/neutral policy, custom named-color matching, profile compatibility, and measurement provenance.
7. Add portable bundles, selected-item reprint, undo/redo, and settings inheritance/provenance.
8. Add a defined supported input-format subset and a later vector/PDF fidelity gate.
9. Add a distribution-readiness track for installer, migrations, signed updates, diagnostics, security, and licensing.
10. Keep roll handling, multi-lane output, fluorescent/varnish channels, broad printer support, and monetization outside the initial implementation.

**Bottom line:** the existing plan is not missing the basic idea of a professional RIP. It is missing several detailed contracts that make a RIP reliably professional. Prioritize native mode correctness, controllable white, measured color, final-raster visibility, and recoverable jobs. Digital Factory provides the richer workflow reference; AcroRIP provides valuable evidence about model-specific printer behavior. Neither removes the need to prove our own backend.

## Sources

All accessed November 7, 2026. Documentation establishes advertised/documented functionality, not independently measured performance.

- **F1:** Fiery, [Digital Factory Direct To Film](https://www.fiery.com/products/digital-factory-dtf/). 12.1 features, Desktop/Production positioning, and optional Color Profiler, FluidMask, job sorter, DeviceLink, and InkSaver modules.
- **F2:** Fiery, [Layer Tab](https://help.cadlink.com/website/digital_factory/en/production/menus/layer_tab.htm). Generated/supplied white, highlight, adaptive alpha, layer settings, advanced curves and variable-dot controls. Generic and device-dependent.
- **F3:** Fiery, [Queue tab](https://help.cadlink.com/website/digital_factory/en/production/queue_tab.htm). Device-dependent mode controls and photo/graphics/color adjustments.
- **F4:** Fiery, [Underbase Strength and Manual Curve](https://help.cadlink.com/website/digital_factory/en/bonus_documentation/underbase_strength_and_manual_curve.htm). White tonal control. Includes DTG examples, not transferable DTF recipes.
- **F5:** Fiery, [View Raw Data](https://help.cadlink.com/website/digital_factory/en/bonus_documentation/view_raw_data.htm). RIP-only output inspection, plane toggles, true-pixel display, image export.
- **F6:** Fiery, [Digital Factory Interface](https://help.cadlink.com/website/digital_factory/en/production/vpm_interface.htm). Queue/reserve workflow, job backup, ink-order wizard, color replacement, costing entry, and production tools.
- **F7:** Fiery, [KnockMeOut Tools](https://help.cadlink.com/website/digital_factory/en/bonus_documentation/knockmeout.htm). Black/color removal and transparency controls.
- **F8:** Fiery, [Gang Sheet Builder](https://help.cadlink.com/website/digital_factory/en/production/menus/gang_sheets.htm). Bundled versus path-dependent layouts, selected-job reprint, crop, quantities, spacing, and labels.
- **A1:** AcroRIP, [Features](https://acrorip.com/features.php). One-pass white/color, custom channels, v11 multi-image handling, ICC support, and model-scoped speed/UV features.
- **A2:** AcroRIP, [Updates](https://acrorip.com/updates.php). Version-specific ET-8550 sequence, film/media/feed/placement changes and model-specific capabilities.
- **A3:** AcroRIP, [Supported printers](https://www.acrorip.com/supported-printers.php). ET-8550 in v11.4 desktopDTF list.
- **A4:** AcroRIP, [Support and training](https://acrorip.com/support-training.php). Request-only manuals and ICC guide, OS statements and licensing/support information. Detailed manuals were not obtained.
