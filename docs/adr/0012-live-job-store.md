# ADR 0012: Live job list and detail in the app

Status: accepted (slice 1 of 2), 2026-11-20

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

## Not done (slice 2)
Preflight, acknowledge and render commands over the real workspace; wiring the Svelte adapter; capabilities from the
real setup and mode; attempt itemization; job creation from the UI.
