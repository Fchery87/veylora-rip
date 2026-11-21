# ADR 0015: White-setting edits make a new job

Status: accepted, 2026-11-21

## Decision
- A job is an immutable snapshot and its white settings live in its recipe, so changing them never
  edits anything stored. `Workspace::derive_job_with_white` stores the next revision of the job's
  recipe (same id, revision = highest stored revision of that id + 1, stamped with the edit time)
  with the changed white policy, and a new job that is the old one on that recipe. The layout, assets,
  setup and mode references, and every other recipe field are copied unchanged. The new job's
  provenance names the new recipe. The first job, its recipe and its runs are untouched.
- The editable settings are the ones a recipe states: white on (source `Alpha`) or off (`Disabled`),
  and a choke, which is a negative spread in micrometres. Growing white past the artwork (positive
  spread) is refused, and so is an edit that changes nothing: it creates no job.
- A recipe whose white comes from a tonal source or a supplied mask cannot be edited this way and is
  refused with that reason.
- `live::edit_white` (app) turns the operator's change into that edit. The Sheet controls "Invert
  mask" and the "Whole placement" mask source are not things a recipe can state, so they are refused
  with a message instead of being approximated or ignored.
- One Tauri command, `live_edit_white`, serves all three white controls. It returns the new job's
  detail and the app opens that job. Live jobs stay editable after a render, because the edit never
  touches the rendered one.

## Parent link
A derived job records the snapshot it was made from as a `derived_from` provenance entry holding
that snapshot's content hash. Provenance labels are free-form and unique, so this needs no job
format change, and the entry is covered by the new job's own hash. `Workspace::job_parent` returns the
recorded hash and, when a job with exactly that snapshot is in the workspace, its job id. A parent that
is not in the workspace (an archive carries one job, not its parent) is reported by hash only and is
never guessed. A chain of edits points one step at a time: each job names the job just before it.

The Provenance tab shows it as "Derived from". The entry records that a job was derived, not by
which kind of edit; what changed is read from the two jobs' recipes. This is not the reprint of the
product requirements: a selective reprint is a new attempt on the same job and stays refused until
a submission path exists.
