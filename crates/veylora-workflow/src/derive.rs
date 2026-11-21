//! Changing a stored job's white settings. A job is an immutable snapshot, so an edit never touches
//! it: it stores a new recipe revision with the changed white policy and a new job that is the old
//! one with that recipe. The first job, its recipe and its runs stay exactly as they were.
use crate::{
    error::{WorkflowError, WorkflowResult},
    workspace::{JobCreated, Workspace},
};
use veylora_domain::{
    ids::JobId,
    job::{JobSnapshot, JobSnapshotInput},
    provenance::ContentHash,
    recipe::{PrintRecipe, RecipeStatus, WhiteSource},
};

/// Provenance label under which a derived job records the snapshot it was made from.
pub const DERIVED_FROM: &str = "derived_from";

/// The job a derived job was made from. The snapshot hash is what was recorded; `job_id` is set
/// only when a job with exactly that snapshot is in this workspace (an archive carries one job,
/// not its parent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobParent {
    pub snapshot_hash: ContentHash,
    pub job_id: Option<JobId>,
}
use veylora_persistence::JobStore;

/// The white settings a recipe can state that an operator may change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhiteEdit {
    /// False prints no white at all.
    pub enabled: bool,
    /// Micrometres; negative contracts white inside the artwork (a choke).
    pub spread_um: i32,
}

fn invalid(message: impl Into<String>) -> WorkflowError {
    WorkflowError::Invalid(message.into())
}

impl Workspace {
    /// Makes a new job from `id` with `edit` applied to its recipe's white policy, on the next free
    /// revision of that recipe. `created_at_ms` stamps both new records. Refuses an edit that
    /// changes nothing, a white source the edit cannot express (tonal or a supplied mask), and a
    /// positive spread, which would print white beyond the artwork.
    pub fn derive_job_with_white(
        &self,
        id: JobId,
        edit: &WhiteEdit,
        created_at_ms: u64,
    ) -> WorkflowResult<JobCreated> {
        let snapshot = self.jobs.get(id)?;
        let old = snapshot.inputs();
        let current = &old.recipe.inputs().white;
        if edit.spread_um > 0 {
            return Err(invalid(
                "White can be choked (a negative spread) but not grown past the artwork.",
            ));
        }
        let was_enabled = match current.source {
            WhiteSource::Alpha => true,
            WhiteSource::Disabled => false,
            _ => {
                return Err(invalid(
                    "This job's recipe takes white from a source this edit cannot express.",
                ))
            }
        };
        if was_enabled == edit.enabled && current.spread_um == edit.spread_um {
            return Err(invalid(
                "The job already has these white settings; nothing was created.",
            ));
        }

        let recipe_id = old.recipe.inputs().id;
        let revision = self
            .catalog()?
            .recipes
            .iter()
            .filter(|r| r.id == recipe_id.to_string())
            .map(|r| r.revision)
            .max()
            .unwrap_or(0)
            .max(old.recipe.inputs().revision)
            + 1;
        let mut input = old.recipe.inputs().clone();
        input.revision = revision;
        input.created_at_ms = created_at_ms;
        // Changed white is a different recipe, so what validated the old one does not carry over.
        if input.status == RecipeStatus::Validated {
            input.status = RecipeStatus::Experimental;
        }
        input.validation = None;
        input.white.source = if edit.enabled {
            WhiteSource::Alpha
        } else {
            WhiteSource::Disabled
        };
        input.white.spread_um = edit.spread_um;
        let recipe = PrintRecipe::new(input).map_err(|e| invalid(e.message))?;
        self.import_recipe(&recipe)?;

        let mut next: JobSnapshotInput = old.clone();
        next.id = JobId::default();
        next.created_at_ms = created_at_ms;
        // The new job points at the one it was made from, not at that job's own parent, so a chain
        // of edits reads one step at a time.
        let parent_hash = snapshot.content_hash();
        next.provenance
            .content_hashes
            .retain(|(label, _)| label != DERIVED_FROM);
        next.provenance
            .content_hashes
            .push((DERIVED_FROM.to_owned(), parent_hash));
        for (label, hash) in &mut next.provenance.content_hashes {
            if label == "recipe" {
                *hash = recipe.content_hash();
            }
        }
        next.recipe = recipe;
        let made = JobSnapshot::new(next).map_err(|e| invalid(e.message))?;
        let created = JobCreated {
            job_id: made.job_id(),
            snapshot_hash: made.content_hash(),
        };
        self.jobs.create_snapshot(made)?;
        Ok(created)
    }

    /// The job `id` was derived from, or `None` for a job made from scratch.
    pub fn job_parent(&self, id: JobId) -> WorkflowResult<Option<JobParent>> {
        let snapshot = self.jobs.get(id)?;
        let Some((_, hash)) = snapshot
            .inputs()
            .provenance
            .content_hashes
            .iter()
            .find(|(label, _)| label == DERIVED_FROM)
        else {
            return Ok(None);
        };
        let snapshot_hash = *hash;
        // Another job's damaged record must not hide this one's parent, so unreadable ones are
        // skipped rather than failing the lookup.
        let job_id = self.list_jobs()?.into_iter().find(|other| {
            self.jobs
                .get(*other)
                .is_ok_and(|s| s.content_hash() == snapshot_hash)
        });
        Ok(Some(JobParent {
            snapshot_hash,
            job_id,
        }))
    }
}
