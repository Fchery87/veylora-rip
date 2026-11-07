# Veylora RIP Desktop Design

**Status:** Approved, including the shell prototype (variant E), November 7, 2026. Ready for implementation planning.
**Date:** November 7, 2026
**Applies to:** Tauri 2 desktop application (`app/`), Svelte + Bits UI frontend
**Approved prototypes:** Variant D, "Ledger + sheet" (layout and palette), and variant E, "Approved shell" (this document built as a working prototype), in `.scratch/veylora-ui-prototype/index.html?variant=E`
**Reference experience:** CADlink Digital Factory (production manager queue, layout editor, raw-data viewer). Used as a workflow benchmark, not a visual or behavioral copy.

This document defines how the Veylora RIP desktop application looks and behaves. It does not change product scope, domain contracts, or acceptance gates. Where this document and the PRD or requirements disagree, the PRD and `docs/native-rip-feature-requirements.md` win and this document must be corrected.

Read with:

- `docs/veylora-rip-prd.md`
- `docs/native-rip-feature-requirements.md` (Features 2, 6, 10, 12, 13, 14)
- `docs/adr/0001-tauri-svelte-bits-ui.md`
- `docs/superpowers/plans/2026-11-07-veylora-rip-implementation.md` (Tasks 8.2, 8.3)
- `GLOSSARY.md`

---

## 1. Design principles

1. **The job is the unit of work.** Every screen that touches output is anchored to a job, and a job shows its snapshot, attempts, and state at all times.
2. **Show which stage you are looking at.** Every preview carries a stage label. A simulation never looks like a retained artifact, and nothing is called "printed" until the operator confirms it.
3. **Uncertainty stops the line.** An ambiguous submission is the strongest visual state in the application, and it is visible from every screen until resolved.
4. **Controls exist only if the backend honors them.** Unsupported controls are hidden or disabled with a stated reason. Nothing silently does nothing.
5. **Quiet chrome, loud content.** Neutral surfaces and one accent color. Color is reserved for ink, status, and the operator's artwork.
6. **Color judgment happens on neutral ground.** The canvas surround is neutral grey in both themes.
7. **Deliberate actions are not shortcuts.** Releasing to the printer, resolving an ambiguous attempt, and entering experimental mode always take an explicit, confirmed click.
8. **One obvious next action.** In every state, the single primary button is the next correct step (Render and hold → Release to printer → Confirm it printed). A newcomer can follow the primary button; an expert uses the keyboard. If the next step isn't possible, the primary button is disabled with the reason beside it, so the operator is never left guessing what to do.
9. **Problems show up where you work.** Preflight runs continuously while you edit, and findings appear on the placement, the field, and the job header as they happen. Nobody has to remember to check.

---

## 2. Application shell

```text
┌────┬──────────────────────────────────────────────────────────────────────┐
│    │ [Review banner, only while an attempt is ambiguous]                  │
│ R  ├───────────────┬──────────────────────────────────────────────────────┤
│ a  │               │ Job header (title, state, meta, actions)             │
│ i  │  Job ledger   │ Lifecycle stepper                                    │
│ l  │  (320 px,     │ Tabs: Review · Sheet · Inspect · Preflight ·         │
│    │  collapsible) │       Attempts · Provenance                          │
│ 56 │               ├──────────────────────────────────────────────────────┤
│ px │               │ Tab content                                          │
│    │               │                                                      │
├────┴───────────────┴──────────────────────────────────────────────────────┤
│ Status bar (28 px): device · setup state · mode · sender · render · disk   │
└───────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Section rail (refinement 5)

A 56 px icon rail on the left replaces the prototype's top navigation and frees vertical space for the canvas.

- **Top group:** Jobs, Devices, Recipes, Targets, Reports.
- **Bottom group:** Settings, theme toggle (Light / Dark).
- Each item is an icon plus a tooltip showing its name and shortcut. The current section has an accent-soft background and a 2 px accent bar on the left edge.
- **Badges:** Jobs shows a review-tone dot when any attempt is ambiguous. Devices shows a warn dot while the active setup is unverified.
- Native window decorations are kept for the first release. A custom title bar is out of scope.

### 2.2 Device status bar (refinement 3)

A 28 px bar runs along the bottom of every screen. It answers "can I print right now?" without navigating anywhere.

| Segment | Content | Click target |
| --- | --- | --- |
| Device | `ET-8550 · Home bench rev 3` | Devices screen |
| Setup state | `Verified` (ok), `Unverified` (warn), or `Experimental mode` (warn, filled) | Devices, verification checklist |
| Mode | `PM-exp-01`, with `experimental` / `validated` / `retired` | Devices, mode detail |
| Sender | `Idle`, `Sending A-0142-1 · 42%`, `Paused`, or `Blocked: review required` (review tone, filled). See 2.3 for what is blocked. | The relevant job |
| Render | `Rendering T-0007 · band 25 of 40`, or empty | The relevant job |
| Disk | Free space for artifact storage, for example `214 GB free`. Neutral normally, warn and bad by the rules in 4.7. | Settings → Storage |
| Data source | `Fixture data` chip (warn) while running against mocked services | None |

The status bar is the only global place for the fixture-data indicator. The prototype's full-width fixture banner is removed. Stage labels on each preview still carry `Simulation` and `Fixture data` individually (section 6.3).

### 2.3 Ambiguous review banner (refinement 4)

While any submission attempt is ambiguous, a banner sits at the top of the content area on every screen, including Devices and Recipes:

> **Printer blocked.** J-0142 · North Ridge tee run may or may not have printed. Check the printer, then record what happened. **[Review now]**

- Review-tone soft background, 3 px review-tone left border, alert icon, 40 px tall.
- It can't be dismissed. It disappears only when every ambiguous attempt is resolved or explicitly kept in review. "Decide later" keeps the banner but collapses it to one line.
- **Review now** opens the job on its Review tab.

**What is blocked while a review is open** (resolved decision, section 11):

- **Blocked:** nothing can be sent to that printer while any of its attempts is ambiguous or kept in review with "Decide later". That covers new jobs, reprints, and diagnostic targets. There is no override; the only way forward is to record what happened.
- **Not blocked:** layout, import, preflight, and rendering continue. Rendering consumes no film or ink and can't create a duplicate print, so blocking it adds no safety. It would only push the operator to answer the review hastily just to get unstuck.
- **Waiting jobs explain themselves.** A ready or held job's **Release to printer** button is disabled, with "Waiting: printer blocked by review of J-0142" beside it, linked to the review.

### 2.4 Window sizes

- **Minimum window:** 1280 × 720.
- **Design targets:** 1366 × 768 (laptop) and 1920 × 1080 (desktop). Every screen must be usable at 1366 × 768 without focus mode, and comfortable for sheet work with focus mode.
- There is no mobile or tablet layout. The prototype's narrow-screen CSS is not a requirement.

---

## 3. Jobs screen

### 3.1 Job ledger

- 320 px wide and collapsible with `[`. Collapsed, it becomes a 0 px edge with a re-open handle.
- **Header:** "Jobs" title and **New job** (primary, small).
- **Search:** filters by job name, ID, and asset filename.
- **Filters** (pill group with counts): All, Needs review, Drafts, Held, Done.
- **Pinned group:** ambiguous and needs-review jobs always sit at the top under a "Needs review" heading, whatever the filter or sort.
- **Row:** job name (13.5 px, 500), state badge at the right; meta line (ID in mono, sheet size and placement count, relative time); optional attention line in the state's tone ("Printer result unknown. Review before printing anything else.").
- Implemented as an ARIA listbox with roving tabindex. `↑`/`↓` moves, `Enter` opens.
- **Empty and first-run states:** see 4.6.

### 3.2 Job header

Compact. It must fit in 104 px including the lifecycle stepper (the prototype used about 200 px).

- **Row 1:** job name (20 px, 600), state badge, meta (ID, sheet size, mode), live preflight summary, and actions aligned right.
- **Live preflight summary:** a chip that always shows the current result: `2 blocking · 1 to confirm` (bad), `1 to confirm` (warn), or `Ready to render` (ok). While a check is pending after an edit it reads `Checking…` (neutral). On rendered jobs it reads `Passed at render` (neutral). It updates while you edit, and clicking it opens the Preflight tab. It replaces the prototype's "Run preflight" button.
- Confirm-level findings block **Render and hold** until acknowledged, with the reason "Confirm 1 finding to render" beside the button.
- **Row 2:** lifecycle stepper, 2 px segments with 11.5 px labels. Done segments use the accent, the current one uses its state tone, future ones use the line color.
- **Actions by state.** Each state has at most one primary action (principle 8). When the primary action is unavailable, it stays visible but disabled, with the reason as text beside it, not hidden in a tooltip.

| State | Secondary | Primary | Note |
| --- | --- | --- | --- |
| Draft | | Render and hold | |
| Preflight failed | | Render and hold (disabled) | "Fix 2 blocking findings to render" |
| Rendering | Cancel render | | Progress shown in stepper and status bar |
| Ready / Held | Duplicate as new job | Release to printer | Release opens an inline confirmation under the header ("Send J-0143 to ET-8550 · Home bench rev 4? This creates attempt A-0143-1."). Disabled with "Waiting: printer blocked by review of J-0142" while a review is open (2.3), or "Verify the setup, or turn on experimental mode in Devices" on an unverified setup. |
| Submitting | | | No actions. Cancellation is honest about band and spool boundaries. |
| Submitted | Printed badly, reprint | Confirm it printed | Spooled is not printed |
| Ambiguous / Needs review | Duplicate as new job | Review attempt | Opens the Review tab |
| Finished | Reprint selected placements, Duplicate as new job | | |
| Cancelled / Failed | Duplicate as new job | | |

### 3.3 Job tabs

In order, shown only when relevant:

1. **Review:** only for ambiguous or needs-review jobs, and selected by default for them.
2. **Sheet:** layout and simulated stage previews.
3. **Inspect:** retained artifacts at true pixel scale. Disabled until the job has rendered, with the tooltip "Render the job to inspect its retained output."
4. **Preflight:** count badge in bad tone when blocking findings exist.
5. **Attempts:** append-only history, with a count badge.
6. **Provenance:** snapshot hashes and versions.

### 3.4 Review tab

- Card with review-tone border: a title ("Check the printer before printing anything else") and one paragraph saying what is known.
- **Facts row:** bytes written / intended, spool job ID, last signal time.
- **Choices:** It printed correctly · Nothing printed, reprint · Printed badly, reprint · Decide later.
- Each choice opens an inline confirmation stating exactly what will be recorded and whether a new attempt is created. Confirm labels repeat the action ("Mark finished", "Create reprint attempt", "Keep in review").
- After confirmation the outcome is appended to Attempts and the tab switches there. No outcome triggers a submission. A reprint attempt is created held.
- **What was sent:** artifact hash and size, snapshot hash, placement list.

### 3.5 Sheet tab

```text
┌──────────────────────────────────────────────────────┬──────────────────┐
│ [Stage label chip]                      [Size · zoom]│ Placements       │
│                                                      │ Selected (mm)    │
│                 neutral grey surround                │ White            │
│                    ┌────────┐                        │ White slots      │
│                    │ sheet  │                        │                  │
│                    └────────┘                        │ (300 px,         │
│                [Read-only note if frozen]            │  collapsible ])  │
├──────────────────────────────────────────────────────┤                  │
│ Stage: Garment│Film│Logical│Slots│Screened  Planes C M Y K W  [Focus]  │
└──────────────────────────────────────────────────────┴──────────────────┘
```

- **Canvas:** sheet centered on the neutral surround (section 5.3). Zoom with Ctrl + wheel and pan with Space + drag or the middle mouse button. Zoom is shown as `fit` or a percentage.
- **Stage strip:** segmented control with five stages, each with a sublabel (Approximate, Mirrored, CMYK + W, W1 + W2, Device truth). Plane toggles C M Y K W show the letter plus an ink bar, so color isn't the only signal. Toggling a plane switches to the Logical stage.
- **Before render:** stage views are simulations and labeled as such. **After render:** stage thumbnails come from retained artifacts, and the label reads `Retained artifact` (see Inspect for full fidelity).
- **Inspector panel (right, 300 px):**
  - Placements list with thumbnail, name, and meta. A clipped placement's meta is in bad tone.
  - Selected placement: X, Y, Width, Height in mm (tabular figures), rotation, quantity, plus a bad-tone callout with the exact correction if it is out of bounds.
  - White: source, policy, choke (mm), named response curve, and curve thumbnail. Note: "Applied after mask combine, before choke. Never changes C, M, Y or K."
  - White slots: logical W → physical slot split, with a warn callout while the slot map is inferred.
- **Live findings on the canvas:** a placement with a blocking finding gets a bad-tone outline and a small badge at its corner, and the offending edge is hatched. Confirm-level findings get a warn badge. Hovering the badge shows the finding and its fix, and clicking it selects the placement and focuses the field to change. Findings clear the moment the edit resolves them.
- **Garment colors:** the Garment stage shows a swatch row above the strip with the fixed palette in 5.3, plus **Custom…**. The last color used is remembered per job as a view setting. Garment color is never part of the job snapshot or any output hash, because it can't affect printed output.
- **Frozen snapshot:** once a job has rendered, all inputs are disabled and a chip under the sheet reads "Snapshot frozen. Read-only. Duplicate to edit."
- **Mirror ownership:** the Film stage shows the mirrored sheet because the device mode owns mirroring. The UI never applies a second mirror.

### 3.6 Sheet focus mode (refinement 1)

Focus mode gives layout work the full window, the equivalent of Digital Factory's separate layout editor, without leaving the job.

- **Enter:** `F`, or the **Focus** button at the right of the stage strip. **Exit:** `Esc` or the same button.
- **Hidden:** job ledger, lifecycle stepper, tabs.
- **Collapsed:** the job header becomes a 40 px bar with job name, state badge, live preflight summary, **Render and hold**, and **Exit focus**.
- **Kept:** section rail, status bar, review banner (it can't be hidden), stage strip, and inspector panel (collapsible with `]`).
- Focus state persists while switching between jobs in the same session, and resets on app restart.
- At 1366 × 768 with focus mode and the inspector panel open, an A3 sheet must render at least 520 px tall without the review banner. Prototype E measured about 559 px without the banner and about 519 px with it, using a 72 px vertical margin around the sheet (room for the stage chip). Keep that margin or smaller.
- Focus mode is offered only on the Sheet and Inspect tabs.

### 3.7 Inspect tab

The inspect tab is the product truth view (PRD principle 4, requirements Feature 12). It reads retained stage artifacts. It never recomputes.

```text
┌────────────────────────────────────────────────────────┬────────────────┐
│ Stage ▾  Plane/slot ▾  Fit 100% 200% 400% 800% 1600%   │ Artifact       │
│ Overlays: printable box · clear mask · out of bounds   │  dimensions    │
├────────────────────────────────────────────────────────┤  plane id      │
│                                                        │  mode id       │
│          tiled viewport, nearest-neighbor              │  hash          │
│                                                        │ Coverage       │
│                                          ┌────────┐    │  per plane     │
│                                          │navigator│   │ Loupe          │
│                                          └────────┘    │  x,y px / mm   │
│                                                        │  value / code  │
└────────────────────────────────────────────────────────┴────────────────┘
```

- **Planes:** one plane or slot at a time is the default, because isolating a channel is what reveals errors. A placement with no ink in the selected plane correctly disappears; for example, white-only artwork is invisible on the C plane. Add a **Composite** option per stage that overlays all process planes (and shows white planes on a dark ground), labeled `Composite simulation of retained planes`. This was identified while testing prototype E.
- **Orientation:** logical planes display in artwork orientation. Physical-slot, screened, and encoded stages display in film orientation, mirrored by the mode, and the metadata panel says which.
- **Navigator:** a small overview with the visible region outlined, shown only when zoomed in past fit.
- **Stages:** Logical plane, Physical-slot raster, Screened device raster, and Encoded artifact. The encoded artifact shows metadata only unless the bytes can be decoded. If they can't, the inspector shows the retained pre-encoding raster labeled "Pre-encoding truth".
- **Zoom:** at 100%, one device pixel equals one screen pixel (accounting for display scaling). Zooming in uses nearest-neighbor only. Below 100% the view shows a Rust-produced downsampled overview, labeled "Downsampled overview" in the stage chip.
- **Loupe:** hover shows pixel coordinates in px and mm and the value at the cursor for every plane in the current stage, with the selected plane in bold: `0–255 · %` for continuous tone, `drop 0–3 · none / small / medium / large` for screened.
- **Coverage panel:** per-plane coverage percentage, maximum combined load, clipped area, and out-of-bounds warnings.
- **Metadata:** dimensions, plane identity, mode ID, setup ID, recipe ID, profile hash, screen ID, transform ownership, artifact hash.
- **Export:** the current plane, or all planes, as files, with the artifact hash in the filename.
- **Tiles:** requested from Rust in bounded sizes for the visible region only (ADR 0001). While a tile loads, the UI shows a neutral placeholder. It never shows an interpolated stand-in.
- **Window:** Inspect lives in the job's tab for the first release, and focus mode (`F`) works here too, giving it the full window. Build `ArtifactViewport` window-independent: all of its state is keyed by job ID and artifact hash, and it reads from application services, never from main-window state. That way an **Open in new window** action for a second monitor can be added later without a rewrite. Revisit after Gate 3 if two-monitor use is routine.

### 3.8 Preflight, Attempts, Provenance tabs

- **Preflight:** findings are recomputed by the Rust preflight service after every output-affecting edit, debounced to about 300 ms. The tab shows "Checked just now against setup rev 3, mode PM-exp-01" so the operator knows the result is current. Confirm-level findings are acknowledged per job, with the operator and time recorded. Findings are grouped in one card per severity (Blocking, Confirm, Info). Each finding has a tone badge, a bold title, one sentence of cause, and a "fix" line saying exactly what to change. Every blocking finding has a fix line (Task 8.3, step 1). Findings that point to a placement select it on the Sheet tab when clicked.
- **Attempts:** append-only timeline with monospace timestamps, a tone dot per event, and plain-language events. A caption reads: "Append-only. Reprints add a new attempt; nothing here is ever overwritten."
- **Provenance:** key-value list of device setup, mode, recipe, profiles, assets, artifact, and software versions, with truncated hashes. Hover shows the full hash, and clicking copies it.

---

## 4. Other screens

### 4.1 Devices: setup and channel verification

This is the first screen needed in real use (Gate 1). It is designed now, not deferred.

- **Left:** setup list with revision history. A revision is immutable; editing creates a new revision and old jobs keep theirs.
- **Header:** setup name and revision, verification badge, and actions (**Generate channel identity target**, **Record test result**, **Create revision**).
- **Channel map table:** Logical ink | Physical slot (tank label) | Protocol channel | Evidence. Evidence is `Not tested` (warn), `Matched` (ok), or `Mismatch` (bad), and links to the bench record. Inferred values are labeled "(inferred)".
- **Verification checklist:** a stepped sequence (numbered markers are appropriate here):
  1. Generate the target.
  2. Print it through the native probe or reference path.
  3. For each patch, record which tank printed it, optionally with a photo.
  4. Review mismatches.
  5. Mark verified.

  Verified is unavailable until every required channel has a matched record. Behavior validated in prototype E:

  - Only the current step is expanded. Done steps show a check mark and collapse, except steps 3 and 4, which stay visible as evidence. Locked steps show one line saying what they will do.
  - Step 3 shows the target image beside six rows: "Patch 1 · protocol channel 0 · map says Black", then a tank picker ("Which tank printed this?", with every tank plus "Nothing printed"), then a Matched, Mismatch, or Not tested badge. A counter reads "4 of 6 recorded". Pickers are disabled until the target is printed.
  - Step 4 lists each mismatch in one sentence ("Channel 2 printed from Yellow tank, but the map says Cyan tank."). A wrong tank offers **Update map to match the print (creates rev N+1)**. "Nothing printed" never updates the map. It says to check that tank's ink and nozzles and offers **Print target again**, which clears only those answers.
  - Until verification is complete, the Devices rail tooltip shows progress ("Devices · step 4 of 5: review mismatches").
- **White allocation:** policy selector, with the "Duplicate coverage blocked" rule stated.
- **Experimental mode:** a switch that opens an AlertDialog explaining what experimental mode allows and how its output is labeled. While on, the status bar segment is warn and filled.
- **Identity:** printer identity, firmware observation, connection, conversion supplier, ink, film, feed path. These fields are read-only once used by a job.

### 4.2 Recipes

List and detail, following the Devices pattern: recipe identity, calibration references, profiles, white policy, screen, ink limits, finishing conditions, and state (experimental / validated / superseded) with evidence links. Changing an output-affecting value creates a new revision.

### 4.3 Targets

A gallery of diagnostic targets (channel identity, geometry, registration, white ramps, alpha edges, wet load, screening). Generating one creates a job of kind `target`, which then follows the normal job flow.

### 4.4 Reports

Diagnostic report and support bundle export, with redaction and omit-artwork options and a preview of included files before export.

### 4.5 New job and import

- **New job** opens a sheet-size picker (only sizes the active mode supports), then an empty Sheet tab with a drop zone.
- **Import:** each file is listed with pixels, color interpretation, alpha, and status before it is placed. Warnings ("Untagged, assumed sRGB") and rejections ("APNG is not supported. Export a static PNG.") appear here, never later.
- White masks are imported as their own asset type and paired with an artwork placement explicitly.

### 4.6 First run and empty states

A fresh install has an empty job list and an unverified setup. Nothing can print for real until the setup is verified, so the first run leads there.

- **No device setup:** the Jobs empty state reads "Set up your printer first" with **Create device setup**, which opens Devices with the identity form.
- **Setup exists but unverified:** the empty state reads "Verify your printer before your first job" with **Start verification**, which opens the Devices checklist (4.1). A secondary **Create a draft job anyway** lets layout work start; those jobs can render but can only be released in experimental mode.
- **Verified, no jobs yet:** "Import artwork to start a sheet" with **New job** and a drop zone.
- The checklist progress (for example, "Step 3 of 5: record test results") also appears on the Devices rail tooltip until verification is complete.
- Every other empty state invites the next action in one sentence, never mood text.

### 4.7 Storage: disk-space rules and retention

Retained artifacts are large. As a rough planning estimate, to be replaced with measured sizes once real artifacts exist: an A3 sheet at a 720 dpi grid is about 100 million pixels per plane, and keeping every stage of one job can take about 1–2 GB, more at higher resolutions. Because job sizes vary this much, warnings are based on what the next job needs, not a fixed number.

**Three layers of protection:**

| Layer | Rule | Where it shows |
| --- | --- | --- |
| Block | Preflight estimates the space a render needs. If free space is less than that estimate plus a 5 GB reserve, rendering is blocked. | Blocking preflight finding: "Needs 1.8 GB plus a 5 GB reserve; 4.1 GB free. Free up at least 2.7 GB in Settings → Storage." |
| Warn | Free space falls below the larger of 25 GB or ten times the median job size. | Status bar disk segment in warn tone |
| Critical | Free space is below what the next queued render needs. | Status bar disk segment in bad tone, plus the finding above on that job |

All thresholds are configurable defaults in Settings → Storage.

**Storage view (Settings → Storage):** total used by Veylora, broken down by job and by stage, with one-click cleanup of eligible intermediates and a preview of what will be removed.

**Proposed retention policy** (answers PRD open decision 8; needs PRD acceptance):

- **Always kept:** the job snapshot, the final printer-ready artifact for every submission attempt, and attempt records. These are needed for reprints and for proof of what was sent.
- **Cleaned up automatically:** intermediate stage artifacts (logical planes, physical-slot rasters, screened rasters) of finished or cancelled jobs, 30 days after the job closes. Cleanup never touches jobs that are open, held, ambiguous, or in review.
- **Pin to keep:** any job can be pinned to keep all its artifacts indefinitely.
- After cleanup, the Inspect tab shows "Intermediate stages were cleaned up on <date>. The printer-ready artifact is still retained." It never regenerates and passes off a replacement.

---

## 5. Visual system

### 5.1 Color tokens

Light is the default theme. Dark is a full theme, not an inversion. Tokens are CSS custom properties on `:root` and `[data-theme="dark"]`.

| Token | Light | Dark | Use |
| --- | --- | --- | --- |
| `--bg` | `#f6f7f9` | `#0b0f17` | App background |
| `--surface` | `#ffffff` | `#121826` | Panels, cards, ledger, inputs |
| `--subtle` | `#f2f4f7` | `#182031` | Hover, segmented track, disabled input |
| `--line` | `#e4e7ec` | `#232b3b` | Dividers, card borders |
| `--line-strong` | `#d0d5dd` | `#2f3a4f` | Input and button borders |
| `--text` | `#101828` | `#eef1f6` | Primary text |
| `--text-2` | `#344054` | `#c7cdd8` | Secondary text, button labels |
| `--muted` | `#667085` | `#8a94a7` | Meta, captions, placeholders |
| `--accent` | `#2e6be6` | `#5b8cff` | Primary actions, current tab, progress |
| `--accent-hover` | `#2459c9` | `#7aa2ff` | Primary hover |
| `--on-accent` | `#ffffff` | `#0b0f17` | Text on accent fills |
| `--accent-soft` | `#eef4ff` | `rgb(91 140 255 / .13)` | Selected row, rail current item |
| `--accent-text` | `#1d4fb8` | `#a9c2ff` | Accent-colored text |

`--on-accent` is dark in the dark theme. The prototype used white text on `#5b8cff`, which fails WCAG AA (3.2:1); dark text gives 6.1:1. All status tone pairs in 5.1 measure at least 4.8:1.

#### Status tones

Each tone has a foreground and a soft background. A tone is always paired with text and, for badges, a leading dot, so color is never the only signal.

| Tone | Light fg / bg | Dark fg / bg | Meaning |
| --- | --- | --- | --- |
| `ok` | `#067647` / `#ecfdf3` | `#47cd89` / `rgb(71 205 137 / .12)` | Verified, passed, finished |
| `info` | `#1d4fb8` / `#eef4ff` | `#a9c2ff` / `rgb(91 140 255 / .13)` | Held, ready, submitted, in progress |
| `warn` | `#b54708` / `#fffaeb` | `#fdb022` / `rgb(253 176 34 / .12)` | Unverified, experimental, assumption, needs confirmation |
| `bad` | `#b42318` / `#fef3f2` | `#f97066` / `rgb(249 112 102 / .12)` | Blocking, failed, clipped |
| `review` | `#6927da` / `#f4f3ff` | `#a48afb` / `rgb(164 138 251 / .14)` | Ambiguous or needs review only |
| `neutral` | `#344054` / `#f2f4f7` | `#c7cdd8` / `#1b2333` | Draft, cancelled, rendering |

**Refinement 4:** `review` is reserved for ambiguous and needs-review states. Amber (`warn`) never means "stop and decide"; it means "you can proceed, knowingly". Violet was chosen because it is distinct from every other status color and from the CMYK ink colors.

### 5.2 Ink colors

Used only where the meaning is an ink or plane: plane toggles, channel map swatches, the logo mark, target artwork. Never used for status or chrome.

| Ink | Value |
| --- | --- |
| C | `#00a0c6` |
| M | `#d4307a` |
| Y | `#e8c200` |
| K | `#2a2a2a` |
| W | `#f4f4ef`, always with a 1 px `--line-strong` outline so it is visible on light surfaces |

### 5.3 Canvas tokens (refinements 2 and 4)

The canvas does not follow the UI theme.

| Token | Value | Use |
| --- | --- | --- |
| `--canvas-surround` | `#7f7f7f` (default) | Area around the sheet. Neutral grey, close to an 18% reflectance card. |
| `--canvas-surround` options | `#7f7f7f`, `#000000`, `#ffffff` | Set under View → Canvas background, persisted per user |
| `--canvas-grid` | `rgb(255 255 255 / .12)` | Optional dot grid, off by default |
| `--film-check-a` / `--film-check-b` | `#d9d9d9` / `#f2f2f2` | Clear-film checkerboard, neutral and theme-independent |
| `--garment` | `#1c1c1e` (black, default) | Garment simulation color, chosen from the palette below. Always labeled approximate. |
| `--select-outer` / `--select-inner` | `#ffffff` / `#101828` | Selection is a 1 px double outline. It stays visible over any plane, including cyan, so it does not use the accent. |
| `--printable` | `#101828`, dashed, with a `#ffffff` 1 px halo | Printable box |
| `--clip-hatch` | `#dc2626` at 55% | Out-of-bounds hatching |

#### Garment palette

A fixed set of common garment colors, one click each, plus **Custom…** for an exact color. These are display approximations for judging contrast and white coverage, not color proofs.

| Name | Value |
| --- | --- |
| Black | `#1c1c1e` |
| White | `#f7f7f5` |
| Heather grey | `#9a9da3` |
| Navy | `#1f2a44` |
| Red | `#b3202c` |
| Royal blue | `#2b4fa8` |
| Forest green | `#24452f` |
| Natural | `#e8dcc4` |

### 5.4 Typography

- **Faces:** Geist for UI text, Geist Mono for IDs, hashes, timestamps, and loupe values. Both are SIL OFL and must be bundled with the app; no runtime font fetching. Add them to the font and license inventory (Task 9.3).
- Measurements in forms and tables use `font-variant-numeric: tabular-nums`.

| Role | Size / line height | Weight |
| --- | --- | --- |
| Page title (job name, section title) | 20 / 26 | 600, tracking −0.015em |
| Card title | 15 / 22 | 600 |
| Body | 14 / 21 | 400 |
| Dense UI (rows, inputs, buttons, tabs) | 13–13.5 / 20 | 400–500 |
| Meta, captions, badges | 12 / 16 | 400–500 |
| Stage sublabels, stepper labels | 11.5 / 14 | 400 |
| Mono values | 12 / 16 | 400 |

Sentence case everywhere. No all-caps labels.

### 5.5 Spacing, radius, elevation

- **Spacing scale (px):** 4, 8, 12, 16, 20, 24, 32. Panel padding is 16; page padding is 24.
- **Radius:** 6 for inner controls (segment buttons, thumbnails), 8 for buttons, inputs and the selected ledger row, 12 for cards, 999 for badges and pills.
- **Elevation:**
  - `--shadow-1` (`0 1px 2px rgb(16 24 40 / .05), 0 1px 3px rgb(16 24 40 / .07)`) for buttons, cards and the active segment.
  - `--shadow-sheet` (`0 12px 32px -8px rgb(16 24 40 / .18)`) for the sheet only.
  - Dark theme uses black shadows at higher opacity.
- **Control heights:** 34 px buttons, 32 px inputs, 30 px small buttons, 28 px segment items, 22 px badges.

### 5.6 Motion

- Motion only responds to the operator: tab switch, panel collapse, focus mode, confirmation reveal. Durations are 120–200 ms, ease-out.
- New attempt entries get a single 500 ms soft highlight.
- There is no ambient or looping motion except progress indicators.
- `prefers-reduced-motion` turns off all transitions.

### 5.7 Icons

Lucide (`@lucide/svelte`, ISC license), confirmed. 16 px in controls and 20 px in the rail, 1.5 px stroke. Add it to the dependency and license inventory (Task 9.3).

---

## 6. Content and states

### 6.1 Job state mapping

| Domain state | Badge text | Tone |
| --- | --- | --- |
| draft | Draft | neutral |
| preflight-failed | Preflight failed | bad |
| rendering | Rendering 62% | neutral |
| ready | Ready | info |
| held | Held | info |
| submitting | Sending 42% | info |
| submitted | Submitted | info, with attention line "Spooled. Confirm what came out of the printer." |
| ambiguous | Ambiguous | review |
| needs-review | Needs review | review |
| cancelled | Cancelled | neutral |
| failed | Failed | bad |
| operator-confirmed-finished | Finished | ok |

The UI never says "Printed" for any state except an operator-confirmed outcome.

### 6.2 Action vocabulary

An action keeps the same name from button to confirmation to history entry.

Render and hold · Release to printer · Cancel render · Cancel job · Confirm it printed · Mark finished · Create reprint attempt · Keep in review · Duplicate as new job · Reprint selected placements · Generate channel identity target · Record test result · Create revision · Import PNG · Import white mask.

### 6.3 Stage labels

Every preview shows a chip at the top left using this vocabulary:

- `Garment simulation · approximate, not a color proof`
- `Film view · mirrored for transfer · simulation`
- `Logical planes: C M Y K W · continuous tone · simulation | retained artifact`
- `Physical-slot raster · W split to W1 and W2 · simulation | retained artifact`
- `Screened device raster · simulation | retained artifact`
- `Downsampled overview` (Inspect, below 100%)
- `Pre-encoding truth` (Inspect, when final bytes can't be decoded)
- `Fixture data` is appended to any chip while running on mocked services.

Simulation chips use the warn tone; retained-artifact chips use neutral.

### 6.4 Copy rules

- Plain verbs, sentence case, active voice. Errors say what happened and what to change; they don't apologize.
- Every blocking finding has a "fix" line with a concrete number when one exists ("Move it left by at least 4.2 mm").
- Empty states invite the next action ("No attempts yet. Render and hold the job, inspect it, then release it to the printer.").

---

## 7. Interaction

### 7.1 Keyboard

| Key | Action |
| --- | --- |
| `F` / `Esc` | Enter / exit sheet focus mode |
| `[` / `]` | Toggle job ledger / inspector panel |
| `1`–`5` | Stage: Garment, Film, Logical, Slots, Screened |
| `Alt+C/M/Y/K/W` | Toggle a plane |
| `Ctrl+0` / `Ctrl+1` | Zoom to fit / zoom to 100% |
| `Ctrl` + wheel, `Space` + drag | Zoom / pan |
| Arrow keys / `Shift` + arrows | Nudge the selected placement 0.1 mm / 1 mm |
| `Ctrl+Z` / `Ctrl+Shift+Z` | Undo / redo layout changes |
| `Ctrl+D` | Duplicate placement |
| `↑` / `↓` / `Enter` in ledger | Move / open job |

There are deliberately no shortcuts for: Release to printer, resolving a review, entering experimental mode, or creating a setup revision.

### 7.2 Confirmations

- Release to printer, review outcomes, and experimental mode require explicit confirmation.
- Review outcomes use an inline confirmation panel inside the review card, so the facts stay visible.
- Experimental mode and destructive setup changes use an AlertDialog.
- Confirm buttons repeat the action name. A "Go back" option is always present.

### 7.3 Capability gating

Controls are generated from `DeviceCapability` and the active `PrintMode`.

- Unsupported controls are hidden when they don't apply to the device at all.
- They are disabled with a tooltip ("Not supported by mode PM-exp-01") when the device could support them but the current mode doesn't.
- Sheet sizes, resolutions, and white-slot options list only what the mode honors.

### 7.4 Read-only snapshots

Once rendering begins, the job's inputs are read-only in every tab. Editing requires **Duplicate as new job**, which creates a draft linked to the original.

---

## 8. Components

Veylora-owned Svelte components wrap Bits UI primitives where semantics, validation, styling, or accessibility are standardized (ADR 0001). Components live in `app/src/lib/components/` and consume only the tokens in section 5.

| Veylora component | Built on | Notes |
| --- | --- | --- |
| `Button` | native `<button>` | Variants: primary, secondary, danger; sizes: default, sm |
| `StatusBadge` | none | Takes a domain state and maps it to text and tone per section 6.1. UI code never picks tones ad hoc. |
| `StageChip` | none | Takes stage and source (simulation, retained, fixture) and renders the vocabulary in 6.3 |
| `Tabs` | Bits UI `Tabs` | Job tabs, with count badges |
| `SegmentedControl` | Bits UI `ToggleGroup` (single) | Stage strip, theme, canvas background |
| `PlaneToggle` | Bits UI `Toggle` | Letter plus ink bar |
| `Select` | Bits UI `Select` | White policy, curve, allocation |
| `MeasureField` | native input + `Label` | Unit suffix, tabular numbers, step and range from the capability model |
| `JobLedger` | ARIA listbox pattern | Roving tabindex, pinned review group |
| `ReviewCard` | none | Owns the inline confirmation flow |
| `Finding` | none | Severity, title, cause, fix |
| `PreflightSummary` | none | Live chip in the job header and focus bar. Opens the Preflight tab. |
| `FindingMarker` | Bits UI `Tooltip` | Corner badge on a placement with a finding. Selects the placement and focuses the field to fix. |
| `GarmentSwatches` | Bits UI `ToggleGroup` (single) + `Popover` | Fixed palette plus Custom…. View setting only, never in the snapshot. |
| `EmptyState` | none | One sentence plus one primary action (4.6) |
| `StorageView` | none | Usage by job and stage, cleanup preview, threshold settings (4.7) |
| `AttemptTimeline` | none | Append-only, highlights new entries |
| `Callout` | none | Tones: warn, bad, review |
| `ConfirmDialog` | Bits UI `AlertDialog` | Experimental mode, destructive setup actions |
| `Tooltip` | Bits UI `Tooltip` | Rail labels, disabled-control reasons, full hashes |
| `ContextMenu` | Bits UI `ContextMenu` | Canvas placement actions |
| `Progress` | Bits UI `Progress` | Render and send progress |
| `ScrollArea` | Bits UI `ScrollArea` | Ledger, inspector panel |
| `SectionRail`, `StatusBar`, `ReviewBanner` | none | Shell components (section 2) |
| `SheetCanvas`, `ArtifactViewport` | none | Rendering surfaces. They draw what Rust provides and own only view state (zoom, pan, selection). |

---

## 9. Data boundary

- The frontend renders read models and sends typed application-service commands by stable domain ID. It never sends arbitrary settings to the renderer or encoder.
- Large rasters stay in Rust. The frontend requests bounded preview tiles and receives metadata, hashes, dimensions, and status.
- Sheet-tab simulations before render are display approximations. They must be produced by Rust preview services, or clearly marked client-side approximations, and always carry a `simulation` chip.
- The UI never presents a simulation or regenerated preview as the submitted raster.

---

## 10. Accessibility

- WCAG 2.2 AA contrast for all text and meaningful non-text elements in both themes. Verify the token tables with an automated contrast check in CI.
- Visible 2 px focus ring (`--accent`, 2 px offset) on every interactive element.
- Color is never the only signal: status badges have text, plane toggles have letters, and clipping has hatching plus a callout.
- State changes (job state, new attempt, preflight results) are announced through an `aria-live="polite"` region. The review banner uses `role="alert"` when it first appears.
- Full keyboard operation for the ledger, tabs, stage strip, inspector panel, and review flow. Canvas placement editing is also possible through numeric fields.
- Reduced motion is respected (section 5.6).

---

## 11. Resolved decisions

| Question | Decision | Why |
| --- | --- | --- |
| Icon set | Lucide | Consistent, open-licensed, broad coverage for this workflow |
| Garment preview colors | Fixed palette of eight common garment colors plus Custom…, remembered per job as a view setting (5.3, 3.5) | The preview answers "does this hold up on dark or light?", which a few swatches answer in one click. Free pickers invite fiddling without adding accuracy, and orders arrive as "navy tee", not hex codes. |
| Inspect in its own window | Not in the first release. In-tab with focus mode; viewport built window-independent so it can be added later (3.7) | Two windows must agree on job, artifact, and review state, and a stale window would undermine principle 2. Focus mode gives most of the benefit now. |
| Disk-space thresholds | Need-based: block render when free < estimate + 5 GB; warn below max(25 GB, 10 × median job); critical when the next render won't fit. Configurable. Retention policy proposed (4.7) | Job sizes vary too much for a fixed number. The PRD already requires disk shortages to block before rendering. |
| "Decide later" blocks renders? | No. It blocks sending to that printer, with no override. Layout and rendering continue (2.3) | Only sending can waste film or create a duplicate. Blocking everything would pressure hasty answers, which is the risk the review exists to prevent. |
| Preflight trigger | Live and continuous, not a button (principle 9, 3.2, 3.5, 3.8) | Findings appear as you work, so correctness doesn't depend on remembering a step |

### Still open

1. Accept or amend the retention policy in 4.7 as the answer to PRD open decision 8.
2. Replace the storage size estimates in 4.7 with measured artifact sizes after the first full renders.

---

## 12. Prototype record

- **Prototype:** `.scratch/veylora-ui-prototype/index.html`, variants A–E. D set the layout and palette; E implements this document and is approved. Its scenario switcher (Fresh install, Unverified setup, Working day) covers first run, verification, live preflight, render, review blocking, focus mode, and Inspect. It is throwaway, and its code is not to be promoted to `app/`.
- **Decision record:** `.scratch/veylora-ui-prototype/README.md`.
- **Prototyped in E:** section rail, status bar, review banner and violet review tone, neutral canvas surround and selection outline, focus mode, Inspect tab with loupe, coverage and navigator, Devices verification checklist with revisions, compact job header, live preflight on the canvas and in the header, garment palette, first-run states, inline release confirmation, and dark-theme on-accent text.
- **Specified here but not prototyped:** Recipes, Targets, Reports, Settings → Storage view, the Inspect composite option, the import flow, and the keyboard nudge and undo behavior. Inspect in E separates and screens fixture artwork in the browser as an approximation. In the product, Inspect reads retained artifacts from Rust (section 9).
