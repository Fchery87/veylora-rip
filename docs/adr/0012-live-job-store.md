# ADR 0012: Live job list and detail in the app

Status: accepted (slices 1 and 2), 2026-11-20

## Decision
- `Workspace::list_jobs` (newest first) and `Workspace::job_snapshot` expose stored jobs; `FileJobStore::list_ids`
  reads file names only, so a damaged snapshot still appears and fails when opened.
- `veylora_app_lib::live` builds `JobSummary` and `JobDetail` from the real snapshot, lifecycle state and report.
  Tauri commands `live_list_jobs` and `live_get_job` use `VEYLORA_WORKSPACE`.
- Nothing is approximated. A recipe white source the UI model cannot express (Tonal, Supplied) or a white expansion
  is an error naming the gap. Jobs have no operator name, so the label is `Job <id prefix>`; placements are
  `Asset <id prefix>`.
- `findings` stay empty in detail: they come from an explicit preflight run (which records a lifecycle event, so it is not a read).
  Attempts are counted in provenance but not itemized, because the operator-review outcomes (Finished, Printed badly...)
  cannot be derived from transport states without inventing meaning.

## Slice 2
- `live::capabilities` reads the job's real setup and mode (`Workspace::job_device`): setup verification, mode maturity,
  no physical printer. What the stored records do not state (sheet list, white choke/invert/policies, screened inspect)
  is reported conservatively off, not guessed. An empty workspace reports "no setup".
- `live::run_preflight` maps real findings: title from the error code, cause is the workflow's message, correction is the
  domain's stable `ErrorCode::message`. Ids are `code:subject`. Only confirmation-required findings can be acknowledged;
  unknown ids are errors.
- Acknowledgements live in Rust for the session (`live::Acknowledgements`) and become permanent in the manifest of the render
  that uses them; `live::render_job` refuses blocking or unacknowledged findings and names them.
- Operator identity does not exist, so acknowledgements say "Operator at this workstation".
- The frontend adapter has a live mode (`createTauriServices(..., live)`), chosen when `VEYLORA_WORKSPACE` is set. It has no
  fallback to fixture jobs; file sink, release and review reject with "not available for live jobs yet".

## Slice 3 (completion)
- Attempts are itemized from the stored attempt records with an honest outcome vocabulary ("Accepted" is not "Finished"; only an
  operator confirmation is Finished). Review offers only the outcomes the workflow can record.
- Capabilities come from the stored mode: printable area, white capability. Anything the record does not state stays off.
- File sink: `Workspace::export_latest_run` copies the newest verified run (manifest and planes, byte for byte) to
  `exports/<job>-<run>`, via a `.partial` folder, re-verified, never overwriting. It is diagnostic output, not printer bytes (no
  encoder exists), nothing is sent, so it is not a submission attempt; the folder is listed under Provenance as "Exported to".
- Job creation: `Workspace::catalog` lists stored setups, modes and recipes; `live::create_job` validates every number, file
  size and hash of every placement before importing any artwork or creating anything, then calls `Workspace::create_job`. A sheet
  holds 1 to 32 artworks (`MAX_PLACEMENTS`), each with its own position, size and rotation; errors name the artwork ("Artwork 2:
  ..."). A file placed twice is one stored asset. Fit and overlap are preflight's judgement, not the form's. Artwork is chosen
  with a native file dialog opened by the Rust command `live_pick_artwork` (PNG filter; the webview only receives the path) or a
  typed path.
- Render is offered for Held and PreflightFailed un-rendered jobs, not only Draft (preflight moves a job out of Draft).
- Raster/loupe: `Workspace::plane_overview` (box-averaged gray PNG, at most 4096 px) and `plane_window` (at most 64 x 64 exact
  values) check the plane against the manifest hash before use; a damaged plane is refused rather than drawn. The Inspect view
  lists `rasterFiles` only for a verified run. Screened planes are shown as value / dot level. Composite simulation was added later (ADR 0014).

## Not done
Nothing printer-independent remains; slot letters were added in ADR 0016. for physical planes.
