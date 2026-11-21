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
    recipe::{PrintRecipe, WhiteSource},
};
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
}
