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
  fallback to fixture jobs; file sink, release, review, and white-setting edits reject with "not available for live jobs yet".

## Not done
Creating jobs from the UI; attempt itemization and review for live jobs; file-sink writes from the app; white-setting edits
(jobs are immutable snapshots, so they would create a new job); real sheet/white capabilities from the mode; raster/loupe view.
