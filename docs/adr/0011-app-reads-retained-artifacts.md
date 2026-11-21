# ADR 0011: The app's Inspect view reads retained artifacts

Status: accepted, 2026-11-20

## Decision
- `Workspace::inspect_latest_run` (workflow) re-reads the newest completed run (one with a manifest)
  and returns the verified `ArtifactReport`; a damaged run is returned with its problems.
- `veylora-app` depends on `veylora-workflow` (one-way: app -> workflow). `artifacts::retained_for_job`
  maps the report to an app read model; `inspect::present_retained` builds the view from it.
- The view is labeled `retained artifact`, never Fixture; failed verification is shown in the metadata and note.
- Values are statistics computed from the retained bytes. No raster is drawn (`raster_shown` stays false).
- Slot letters (W1/W2, C..K on physical stages) belong to the mode's channel map, so physical and
  screened stages list files by name instead of guessing a mapping. Composite is refused until built.
- Tauri command `get_retained_inspect_view` uses `VEYLORA_WORKSPACE`; without it the app says only fixture jobs exist.

## Not done
The Svelte UI does not call the command yet (the app has no live job list), and there is no raster or loupe view.
