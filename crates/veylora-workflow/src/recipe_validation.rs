//! Validating a recipe. A recipe becomes validated only through physical evidence (pressed samples
//! from more than one session, measured against stated tolerances) and a named sign-off. The result
//! is a new revision that carries both; the revision it was made from is never changed, and jobs
//! already made on it stay exactly as they were.
use crate::{
    error::{WorkflowError, WorkflowResult},
    workspace::Workspace,
};
use veylora_domain::{
    provenance::ContentHash,
    recipe::{PressedSample, PrintRecipe, RecipeEvidence, RecipeStatus, RecipeValidation},
};

/// What an operator supplies to validate one recipe revision under one setup and mode.
#[derive(Debug, Clone)]
pub struct RecipeValidationRequest {
    /// The experimental revision being validated.
    pub recipe: ContentHash,
    /// The setup and mode the samples were printed with. The setup must be physically verified
    /// and the mode validated; the evidence is recorded against exactly these.
    pub setup: ContentHash,
    pub mode: ContentHash,
    /// Who pressed and measured the samples.
    pub operator: String,
    pub tolerances: String,
    pub samples: Vec<PressedSample>,
    pub signed_off_by: String,
    pub signed_off_at: String,
    pub sign_off_note: String,
    /// Stamps the new revision (UTC Unix milliseconds).
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatedRecipe {
    pub hash: ContentHash,
    pub revision: u32,
}

fn invalid(message: impl Into<String>) -> WorkflowError {
    WorkflowError::Invalid(message.into())
}

impl Workspace {
    /// The setup must be physically verified, the mode validated, and the mode this setup's.
    pub(crate) fn check_validation_conditions(
        &self,
        setup: &ContentHash,
        mode: &ContentHash,
    ) -> WorkflowResult<()> {
        let setup = self.devices.get_setup(setup)?;
        if setup.require_production().is_err() {
            return Err(invalid(
                "The setup has not been verified with physical channel evidence, so samples printed with it cannot validate a recipe.",
            ));
        }
        let mode = self.devices.get_mode(mode)?;
        if mode.device_setup() != setup.id() {
            return Err(invalid("The mode does not belong to that setup."));
        }
        if mode.status() != veylora_device::mode::ModeStatus::Validated {
            return Err(invalid(
                "The mode has not been validated, so samples printed with it cannot validate a recipe.",
            ));
        }
        Ok(())
    }

    /// A stored recipe by content hash.
    pub fn recipe(&self, hash: &ContentHash) -> WorkflowResult<PrintRecipe> {
        Ok(self.records.get_recipe(hash)?)
    }

    /// Stores a validated revision of `request.recipe` and returns it. Refuses, and stores
    /// nothing, when the recipe is already validated or retired, when the setup is not physically
    /// verified, when the mode is not validated or does not belong to the setup, or when the
    /// evidence or sign-off is incomplete.
    pub fn validate_recipe(
        &self,
        request: &RecipeValidationRequest,
    ) -> WorkflowResult<ValidatedRecipe> {
        let base = self.recipe(&request.recipe)?;
        match base.inputs().status {
            RecipeStatus::Experimental => {}
            RecipeStatus::Validated => {
                return Err(invalid(
                    "This recipe revision is already validated. Evidence is never added to it; a \
                     changed recipe is a new revision that is validated again.",
                ))
            }
            RecipeStatus::Retired => {
                return Err(invalid("A retired recipe cannot be validated."));
            }
        }
        self.check_validation_conditions(&request.setup, &request.mode)?;
        let setup = self.devices.get_setup(&request.setup)?;
        let mode = self.devices.get_mode(&request.mode)?;

        let recipe_id = base.inputs().id;
        let revision = self
            .catalog()?
            .recipes
            .iter()
            .filter(|r| r.id == recipe_id.to_string())
            .map(|r| r.revision)
            .max()
            .unwrap_or(0)
            .max(base.inputs().revision)
            + 1;
        let mut input = base.inputs().clone();
        input.revision = revision;
        input.created_at_ms = request.created_at_ms;
        input.status = RecipeStatus::Validated;
        input.validation = Some(RecipeValidation {
            evidence: RecipeEvidence {
                validated_revision: base.inputs().revision,
                validated_hash: base.content_hash(),
                setup_hash: setup.content_hash(),
                mode_hash: mode.content_hash(),
                operator: request.operator.clone(),
                tolerances: request.tolerances.clone(),
                samples: request.samples.clone(),
            },
            signed_off_by: request.signed_off_by.clone(),
            signed_off_at: request.signed_off_at.clone(),
            sign_off_note: request.sign_off_note.clone(),
        });
        let validated = PrintRecipe::new(input).map_err(|e| invalid(e.message))?;
        // The recipe must still suit the mode and setup it was validated under.
        let blocking: Vec<String> = veylora_device::mode::check_compatibility(
            &mode,
            &setup,
            &crate::workspace::diagnostic_capability(&setup),
            &validated,
        )
        .into_iter()
        .filter(|f| f.severity.is_blocking())
        .map(|f| f.message)
        .collect();
        if !blocking.is_empty() {
            return Err(invalid(format!(
                "The recipe does not suit that setup and mode: {}",
                blocking.join(" ")
            )));
        }
        let hash = self.import_recipe(&validated)?;
        Ok(ValidatedRecipe { hash, revision })
    }
}
