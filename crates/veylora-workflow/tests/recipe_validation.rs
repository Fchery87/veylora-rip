mod common;
use common::*;
use std::collections::BTreeMap;
use veylora_domain::{
    errors::ErrorCode,
    layout::Rotation,
    provenance::ContentHash,
    recipe::{PressedSample, RecipeStatus},
};
use veylora_transport::{
    fake::{FailurePoint, FakeTransport},
    PrinterArtifact,
};
use veylora_workflow::{
    JobSpec, LayoutSpec, PlacementSpec, RecipeValidationRequest, RenderOptions, WhiteEdit,
    WorkflowError, Workspace,
};

struct Bench {
    ws: Workspace,
    setup: ContentHash,
    mode: ContentHash,
    recipe: ContentHash,
}

/// A workspace with a verified setup, a validated mode and an experimental recipe.
fn bench() -> Bench {
    let ws = Workspace::open(dir("recipe-validation")).unwrap();
    let s = setup();
    let setup_hash = ws.import_setup(&s.to_record()).unwrap();
    ws.verify_setup(&setup_hash, evidence_for_record(&s.to_record()))
        .unwrap();
    let mut m = mode(s.id());
    m.mark_validated().unwrap();
    let mode_hash = ws.import_mode(&m.to_record()).unwrap();
    let recipe = ws.import_recipe(&recipe()).unwrap();
    Bench {
        ws,
        setup: setup_hash,
        mode: mode_hash,
        recipe,
    }
}

fn samples() -> Vec<PressedSample> {
    ["a", "b", "c"]
        .iter()
        .enumerate()
        .map(|(i, label)| PressedSample {
            session: format!("session {}", i.min(1)),
            label: (*label).into(),
            within_tolerance: true,
            note: "pressed and measured".into(),
        })
        .collect()
}

fn request(b: &Bench) -> RecipeValidationRequest {
    RecipeValidationRequest {
        recipe: b.recipe,
        setup: b.setup,
        mode: b.mode,
        operator: "operator".into(),
        tolerances: "fixture tolerances".into(),
        samples: samples(),
        signed_off_by: "operator".into(),
        signed_off_at: "2026-11-20T09:00:00Z".into(),
        sign_off_note: "reviewed the samples".into(),
        created_at_ms: 1735000100000,
    }
}

fn job_on(b: &Bench, recipe: ContentHash, mode: ContentHash) -> veylora_domain::ids::JobId {
    let spec = JobSpec {
        schema_version: 1,
        job_id: None,
        created_at_ms: 1735000000000,
        setup: b.setup,
        mode,
        recipe,
        assets: BTreeMap::from([(
            "art".to_string(),
            b.ws.import_asset(&fixture("alpha-ramp.png")).unwrap(),
        )]),
        masks: BTreeMap::new(),
        layout: LayoutSpec {
            page_width_um: 80,
            page_height_um: 20,
            mirror: false,
            placements: vec![PlacementSpec {
                asset: "art".into(),
                mask: None,
                x_um: 0,
                y_um: 0,
                width_um: 80,
                height_um: 20,
                rotation: Rotation::Deg0,
                crop: None,
            }],
        },
    };
    let id = b.ws.create_job(&spec).unwrap().job_id;
    b.ws.render(id, &RenderOptions::default()).unwrap();
    id
}

fn artifact() -> PrinterArtifact {
    PrinterArtifact::from_encoded_bytes(vec![7u8; 4096])
}

fn submit_findings(b: &Bench, job: veylora_domain::ids::JobId) -> Vec<ErrorCode> {
    match b
        .ws
        .submit(job, &artifact(), &FakeTransport::new(FailurePoint::None))
    {
        Err(WorkflowError::PreflightBlocked(f)) => f.iter().map(|x| x.code).collect(),
        Ok(_) => vec![],
        Err(other) => panic!("{other:?}"),
    }
}

#[test]
fn validating_makes_a_new_revision_and_leaves_the_original_alone() {
    let b = bench();
    let made = b.ws.validate_recipe(&request(&b)).unwrap();
    assert_eq!(made.revision, 2);
    assert_ne!(made.hash, b.recipe);
    let old = b.ws.recipe(&b.recipe).unwrap();
    let new = b.ws.recipe(&made.hash).unwrap();
    assert_eq!(old.inputs().status, RecipeStatus::Experimental);
    assert_eq!(new.inputs().status, RecipeStatus::Validated);
    assert_eq!(new.inputs().id, old.inputs().id);
    // Only the status, revision, time and evidence differ: every policy is exactly the same.
    let mut same = new.inputs().clone();
    same.status = old.inputs().status;
    same.revision = old.inputs().revision;
    same.created_at_ms = old.inputs().created_at_ms;
    same.validation = None;
    assert_eq!(&same, old.inputs());
    let evidence = &new.inputs().validation.as_ref().unwrap().evidence;
    assert_eq!(evidence.validated_hash, b.recipe);
    assert_eq!((evidence.setup_hash, evidence.mode_hash), (b.setup, b.mode));
}

#[test]
fn it_refuses_conditions_that_are_not_physically_established() {
    let b = bench();
    // A setup that was never verified.
    let loose = setup();
    let loose_hash = b.ws.import_setup(&loose.to_record()).unwrap();
    let mut m = mode(loose.id());
    m.mark_validated().unwrap();
    let loose_mode = b.ws.import_mode(&m.to_record()).unwrap();
    let r = RecipeValidationRequest {
        setup: loose_hash,
        mode: loose_mode,
        ..request(&b)
    };
    assert!(b
        .ws
        .validate_recipe(&r)
        .unwrap_err()
        .to_string()
        .contains("verified"));
    // A mode that was never validated.
    let s2 = setup();
    let h2 = b.ws.import_setup(&s2.to_record()).unwrap();
    b.ws.verify_setup(&h2, evidence_for_record(&s2.to_record()))
        .unwrap();
    let experimental = b.ws.import_mode(&mode(s2.id()).to_record()).unwrap();
    let r = RecipeValidationRequest {
        setup: h2,
        mode: experimental,
        ..request(&b)
    };
    assert!(b
        .ws
        .validate_recipe(&r)
        .unwrap_err()
        .to_string()
        .contains("validated"));
    // A mode that belongs to a different setup.
    let r = RecipeValidationRequest {
        setup: h2,
        ..request(&b)
    };
    assert!(b.ws.validate_recipe(&r).is_err());
    assert_eq!(
        b.ws.catalog().unwrap().recipes.len(),
        1,
        "nothing was stored"
    );
}

#[test]
fn incomplete_evidence_and_unsuitable_recipes_are_refused_and_store_nothing() {
    let b = bench();
    let mut one_session = request(&b);
    for s in &mut one_session.samples {
        s.session = "only".into();
    }
    let mut failing = request(&b);
    failing.samples[0].within_tolerance = false;
    let mut no_name = request(&b);
    no_name.signed_off_by = " ".into();
    for r in [one_session, failing, no_name] {
        assert!(b.ws.validate_recipe(&r).is_err());
    }
    let missing = RecipeValidationRequest {
        recipe: ContentHash::digest(b"not stored"),
        ..request(&b)
    };
    assert!(b.ws.validate_recipe(&missing).is_err());
    assert_eq!(b.ws.catalog().unwrap().recipes.len(), 1);
    // An already validated recipe is not validated again; the evidence is not extended.
    let made = b.ws.validate_recipe(&request(&b)).unwrap();
    let again = RecipeValidationRequest {
        recipe: made.hash,
        ..request(&b)
    };
    assert!(b
        .ws
        .validate_recipe(&again)
        .unwrap_err()
        .to_string()
        .contains("already"));
}

#[test]
fn only_a_job_on_a_validated_recipe_can_be_submitted() {
    let b = bench();
    let experimental_job = job_on(&b, b.recipe, b.mode);
    assert!(submit_findings(&b, experimental_job).contains(&ErrorCode::IncompatibleRecipe));
    assert_eq!(
        b.ws.state(experimental_job).unwrap(),
        veylora_domain::lifecycle::JobState::Ready,
        "refusing it touched nothing"
    );
    let made = b.ws.validate_recipe(&request(&b)).unwrap();
    let validated_job = job_on(&b, made.hash, b.mode);
    assert_eq!(submit_findings(&b, validated_job), vec![]);
}

#[test]
fn a_recipe_validated_under_one_mode_is_not_validated_for_another() {
    let b = bench();
    let made = b.ws.validate_recipe(&request(&b)).unwrap();
    // A second validated mode on the same setup is a different physical condition.
    let s = b.ws.catalog().unwrap();
    let setup_id = s.modes[0].setup_id.parse().unwrap();
    let mut other = mode(setup_id);
    other.mark_validated().unwrap();
    let other_hash = b.ws.import_mode(&other.to_record()).unwrap();
    let job = job_on(&b, made.hash, other_hash);
    assert!(submit_findings(&b, job).contains(&ErrorCode::IncompatibleRecipe));
}

#[test]
fn editing_a_validated_jobs_white_does_not_carry_the_validation() {
    let b = bench();
    let made = b.ws.validate_recipe(&request(&b)).unwrap();
    let job = job_on(&b, made.hash, b.mode);
    let derived =
        b.ws.derive_job_with_white(
            job,
            &WhiteEdit {
                enabled: false,
                spread_um: 0,
            },
            1735000200000,
        )
        .unwrap();
    let recipe =
        b.ws.job_snapshot(derived.job_id)
            .unwrap()
            .inputs()
            .recipe
            .clone();
    assert_eq!(recipe.inputs().status, RecipeStatus::Experimental);
    assert!(recipe.inputs().validation.is_none());
    b.ws.render(derived.job_id, &RenderOptions::default())
        .unwrap();
    assert!(submit_findings(&b, derived.job_id).contains(&ErrorCode::IncompatibleRecipe));
}

#[test]
fn a_validated_recipe_cannot_be_imported_around_the_checks() {
    let b = bench();
    let base = b.ws.recipe(&b.recipe).unwrap();
    // Evidence that names conditions this workspace does not hold at all.
    let nowhere = validated_recipe(
        &base,
        ContentHash::digest(b"no such setup"),
        ContentHash::digest(b"no such mode"),
    );
    assert!(b.ws.import_recipe(&nowhere).is_err());
    // Evidence that names a setup that was never verified.
    let loose = setup();
    let loose_hash = b.ws.import_setup(&loose.to_record()).unwrap();
    let mut m = mode(loose.id());
    m.mark_validated().unwrap();
    let loose_mode = b.ws.import_mode(&m.to_record()).unwrap();
    let unverified = validated_recipe(&base, loose_hash, loose_mode);
    let err = b.ws.import_recipe(&unverified).unwrap_err().to_string();
    assert!(err.contains("verified"), "{err}");
    assert_eq!(
        b.ws.catalog().unwrap().recipes.len(),
        1,
        "nothing was stored"
    );
    // The same recipe with conditions that do hold is accepted.
    let fine = validated_recipe(&base, b.setup, b.mode);
    assert!(b.ws.import_recipe(&fine).is_ok());
    // An experimental or retired recipe needs no such check.
    assert!(b.ws.import_recipe(&base).is_ok());
}
