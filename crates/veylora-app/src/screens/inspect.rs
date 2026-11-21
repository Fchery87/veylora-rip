//! Inspect tab presentation (design 3.7): stage and plane selection, the
//! preview label, and metadata. It never recomputes a raster.

use crate::state::{CapabilityState, DataSource};
use crate::view_models::{
    CompositeOption, InfoPanel, InspectView, JobDetail, MetaKind, MetaRow, Plane, PlaneOption,
    PreviewSource, RetainedArtifact, StageId, StageOption,
};

pub use crate::view_models::InspectSelection;

struct StageDef {
    id: StageId,
    label: &'static str,
    chip: &'static str,
}

// Garment and Film belong to the Sheet tab preview; Inspect never offers them.
// They stay in the table so the shared chip vocabulary has one owner.
const STAGES: [StageDef; 5] = [
    StageDef {
        id: StageId::Garment,
        label: "Garment",
        chip: "Garment simulation \u{b7} approximate, not a color proof",
    },
    StageDef {
        id: StageId::Film,
        label: "Film",
        chip: "Film view \u{b7} mirrored for transfer",
    },
    StageDef {
        id: StageId::Logical,
        label: "Logical",
        chip: "Logical planes: C M Y K W \u{b7} continuous tone",
    },
    StageDef {
        id: StageId::Slots,
        label: "Slots",
        chip: "Physical-slot raster \u{b7} W split to W1 and W2",
    },
    StageDef {
        id: StageId::Screened,
        label: "Screened",
        chip: "Screened device raster",
    },
];

const INSPECT_STAGES: [StageId; 3] = [StageId::Logical, StageId::Slots, StageId::Screened];

const NOT_RENDERED: &str = "Render the job to inspect its retained output.";
const NO_SCREENED: &str = "The active print mode does not support screened inspection.";
const COMPOSITE_LABEL: &str = "Composite simulation of retained planes";
/// Stands in the raster list for the composite picture, which is not a plane file.
pub const COMPOSITE_RASTER: &str = "composite";
const COMPOSITE_NOTE: &str = "Composite of the retained logical planes on a mid-gray backdrop: white ink lifts the backdrop toward white and each colored ink then darkens what is under it. It is a simulation of where ink goes, not color managed and not what the print will look like. Drawn after checking every plane against the manifest.";
const NEEDS_RASTER: &str = "Available when a raster exists";

// Fixture values for artifact metadata. They are labeled as fixture data in the
// view and replaced by retained-artifact values when a backend emits them.
const FIXTURE_RECIPE_ID: &str = "RCP-fixture-01";
const FIXTURE_PROFILE_HASH: &str = "sha256:fixture-profile-0000";
const FIXTURE_SCREEN_ID: &str = "SCR-fixture-01";
const FIXTURE_ARTIFACT_HASH: &str = "sha256:fixture-artifact-0000";

fn stage_def(stage: StageId) -> &'static StageDef {
    STAGES
        .iter()
        .find(|s| s.id == stage)
        .expect("stage table is complete")
}

/// Source for a preview given what the app is running on. No retained
/// artifact exists yet, so a preview is never labeled retained.
pub fn preview_source(data_source: DataSource) -> PreviewSource {
    match data_source {
        DataSource::Fixture => PreviewSource::Fixture,
        DataSource::Live => PreviewSource::Simulation,
    }
}

/// The label every preview carries. It always ends with "Not physical output".
pub fn stage_chip_text(stage: StageId, source: PreviewSource, composite: bool) -> String {
    let mut parts = vec![stage_def(stage).chip];
    if composite {
        parts.push(COMPOSITE_LABEL);
    }
    parts.push(if source == PreviewSource::Retained {
        "retained artifact"
    } else {
        "simulation"
    });
    if source == PreviewSource::Fixture {
        parts.push("Fixture data");
    }
    parts.push("Not physical output");
    parts.join(" \u{b7} ")
}

/// Isolating one plane reveals errors, so inspect opens on one logical plane.
pub fn default_selection() -> InspectSelection {
    InspectSelection {
        stage: StageId::Logical,
        planes: vec![Plane::K],
        composite: false,
    }
}

/// Selecting a plane shows that plane alone and keeps the stage. Turning the
/// selected plane off leaves nothing selected.
pub fn toggle_plane(current: &InspectSelection, plane: Plane, on: bool) -> InspectSelection {
    let planes = if on {
        vec![plane]
    } else {
        current
            .planes
            .iter()
            .copied()
            .filter(|p| *p != plane)
            .collect()
    };
    InspectSelection {
        stage: current.stage,
        planes,
        composite: false,
    }
}

/// The composite option replaces the single plane; it is a simulation view.
pub fn set_composite(current: &InspectSelection, on: bool) -> InspectSelection {
    InspectSelection {
        stage: current.stage,
        planes: Vec::new(),
        composite: on,
    }
}

fn plane_prefix(stage: StageId) -> &'static str {
    match stage {
        StageId::Logical => "Logical plane",
        StageId::Slots => "Physical slot",
        _ => "Screened slot",
    }
}

fn sheet_mm(sheet: &str) -> Option<(u32, u32)> {
    match sheet {
        "A3" => Some((297, 420)),
        "A4" => Some((210, 297)),
        _ => None,
    }
}

fn meta(label: &str, value: impl Into<String>, kind: MetaKind) -> MetaRow {
    MetaRow {
        label: label.to_owned(),
        value: value.into(),
        kind,
    }
}

fn plane_names(selection: &InspectSelection) -> String {
    let names: Vec<&str> = selection.planes.iter().map(|p| p.name()).collect();
    names.join(", ")
}

fn metadata(job: &JobDetail, caps: &CapabilityState, selection: &InspectSelection) -> Vec<MetaRow> {
    let stage = selection.stage;
    let dimensions = match sheet_mm(&job.summary.sheet) {
        Some((w, h)) => format!("{w} x {h} mm \u{b7} pixel size unavailable until a raster exists"),
        None => "Unknown sheet size".to_owned(),
    };
    let plane = if selection.composite {
        "Composite of retained planes".to_owned()
    } else if selection.planes.is_empty() {
        "None selected".to_owned()
    } else {
        format!("{} {}", plane_prefix(stage), plane_names(selection))
    };
    let (transform, orientation) = if stage == StageId::Logical {
        (
            "None: artwork orientation, no transform".to_owned(),
            "Artwork orientation",
        )
    } else {
        (
            format!("Mode {} mirrors for film; the UI does not", caps.mode.id),
            "Film orientation, mirrored by the mode",
        )
    };
    vec![
        meta("Job", &job.summary.id, MetaKind::Identifier),
        meta("Dimensions", dimensions, MetaKind::Text),
        meta("Plane", plane, MetaKind::Text),
        meta("Mode ID", &caps.mode.id, MetaKind::Identifier),
        meta("Setup ID", &caps.setup.name, MetaKind::Identifier),
        meta("Recipe ID", FIXTURE_RECIPE_ID, MetaKind::Identifier),
        meta("Profile hash", FIXTURE_PROFILE_HASH, MetaKind::Identifier),
        meta("Screen ID", FIXTURE_SCREEN_ID, MetaKind::Identifier),
        meta("Transform ownership", transform, MetaKind::Text),
        meta("Artifact hash", FIXTURE_ARTIFACT_HASH, MetaKind::Identifier),
        meta("Orientation", orientation, MetaKind::Text),
    ]
}

pub fn present(
    job: &JobDetail,
    caps: &CapabilityState,
    selection: &InspectSelection,
) -> Result<InspectView, String> {
    if !job.rendered {
        return Err(NOT_RENDERED.to_owned());
    }
    if !INSPECT_STAGES.contains(&selection.stage) {
        return Err("Inspect shows the Logical, Slots, and Screened stages.".to_owned());
    }
    if selection.stage == StageId::Screened && !caps.mode.screened_inspect {
        return Err(NO_SCREENED.to_owned());
    }

    // Switching stage can leave a plane the new stage does not offer, such as
    // logical W in the slot stages. Drop it rather than guess a mapping.
    let offered = Plane::for_stage(selection.stage);
    let selection = InspectSelection {
        stage: selection.stage,
        planes: selection
            .planes
            .iter()
            .copied()
            .filter(|p| offered.contains(p))
            .collect(),
        composite: selection.composite,
    };

    let stages = INSPECT_STAGES
        .iter()
        .map(|id| {
            let blocked = *id == StageId::Screened && !caps.mode.screened_inspect;
            StageOption {
                id: *id,
                label: stage_def(*id).label.to_owned(),
                disabled: blocked,
                reason: blocked.then(|| NO_SCREENED.to_owned()),
            }
        })
        .collect();
    let planes = offered
        .iter()
        .map(|plane| PlaneOption {
            plane: *plane,
            pressed: selection.planes.contains(plane),
        })
        .collect();

    let source = preview_source(caps.data_source);
    let metadata_note = match caps.data_source {
        DataSource::Fixture => "Fixture data. No retained artifact exists yet.",
        DataSource::Live => "No retained artifact exists yet.",
    };
    let viewport_label = if selection.composite {
        "Placeholder for composite of retained planes".to_owned()
    } else if selection.planes.is_empty() {
        "Placeholder for no plane".to_owned()
    } else {
        format!("Placeholder for plane {}", plane_names(&selection))
    };
    let panel = |title: &str| InfoPanel {
        title: title.to_owned(),
        message: NEEDS_RASTER.to_owned(),
    };

    Ok(InspectView {
        chip: stage_chip_text(selection.stage, source, selection.composite),
        metadata: metadata(job, caps, &selection),
        composite: CompositeOption {
            label: COMPOSITE_LABEL.to_owned(),
            pressed: selection.composite,
        },
        selection,
        stages,
        planes,
        source,
        metadata_note: metadata_note.to_owned(),
        panels: vec![panel("Coverage"), panel("Loupe")],
        raster_shown: false,
        raster_files: Vec::new(),
        viewport_label,
        placeholder:
            "No raster is shown. Retained artifacts arrive when backend integration is unblocked."
                .to_owned(),
    })
}

fn plane_file(plane: Plane) -> Option<String> {
    match plane {
        Plane::C | Plane::M | Plane::Y | Plane::K | Plane::W => {
            Some(format!("logical-{}.f32", plane.name().to_lowercase()))
        }
        // Slot letters belong to the mode's channel map; the file names carry protocol ids.
        Plane::W1 | Plane::W2 => None,
    }
}

fn stage_prefix(stage: StageId) -> &'static str {
    match stage {
        StageId::Logical => "logical-",
        StageId::Slots => "physical-",
        _ => "screened-",
    }
}

fn percent(part: u64, whole: u64) -> String {
    if whole == 0 {
        "0%".to_owned()
    } else {
        format!("{:.2}%", part as f64 * 100.0 / whole as f64)
    }
}

fn plane_row(label: String, p: &crate::view_models::RetainedPlane) -> MetaRow {
    meta(
        &label,
        format!(
            "{} of {} nonzero ({}) \u{b7} min {} \u{b7} max {} \u{b7} mean {:.6}",
            p.nonzero,
            p.pixels,
            percent(p.nonzero, p.pixels),
            p.min,
            p.max,
            p.mean
        ),
        MetaKind::Text,
    )
}

/// The Inspect view for a real retained run. Values come from the bytes the run kept, are
/// verified against its manifest, and a damaged run is shown as such. No raster is drawn.
pub fn present_retained(
    artifact: &RetainedArtifact,
    selection: &InspectSelection,
) -> Result<InspectView, String> {
    if !INSPECT_STAGES.contains(&selection.stage) {
        return Err("Inspect shows the Logical, Slots, and Screened stages.".to_owned());
    }
    if selection.composite && selection.stage != StageId::Logical {
        return Err(
            "The composite simulation is built from the Logical planes; choose the Logical stage."
                .to_owned(),
        );
    }
    let composite = selection.composite;
    let prefix = stage_prefix(selection.stage);
    let stage_files: Vec<&crate::view_models::RetainedPlane> = artifact
        .planes
        .iter()
        .filter(|p| p.file.starts_with(prefix))
        .collect();
    if stage_files.is_empty() {
        return Err("This run retained no planes for that stage.".to_owned());
    }

    let offered = Plane::for_stage(selection.stage);
    let selection = InspectSelection {
        stage: selection.stage,
        planes: selection
            .planes
            .iter()
            .copied()
            .filter(|p| offered.contains(p))
            .collect(),
        composite,
    };
    let stages = INSPECT_STAGES
        .iter()
        .map(|id| {
            let has = artifact
                .planes
                .iter()
                .any(|p| p.file.starts_with(stage_prefix(*id)));
            StageOption {
                id: *id,
                label: stage_def(*id).label.to_owned(),
                disabled: !has,
                reason: (!has).then(|| "This run retained no planes for that stage.".to_owned()),
            }
        })
        .collect();
    let planes = offered
        .iter()
        .map(|plane| PlaneOption {
            plane: *plane,
            pressed: selection.planes.contains(plane),
        })
        .collect();

    let mm = |px: u64, pitch_nm: u32| px as f64 * f64::from(pitch_nm) / 1_000_000.0;
    let mut metadata = vec![
        meta("Job", &artifact.job_id, MetaKind::Identifier),
        meta("Run", &artifact.run, MetaKind::Identifier),
        meta(
            "Pixels",
            format!("{} x {}", artifact.width_px, artifact.height_px),
            MetaKind::Text,
        ),
        meta(
            "Dimensions",
            format!(
                "{:.3} x {:.3} mm at {} x {} nm per pixel",
                mm(artifact.width_px, artifact.pitch_x_nm),
                mm(artifact.height_px, artifact.pitch_y_nm),
                artifact.pitch_x_nm,
                artifact.pitch_y_nm
            ),
            MetaKind::Text,
        ),
        meta("Evidence", &artifact.evidence, MetaKind::Text),
        meta("Recipe hash", &artifact.recipe_hash, MetaKind::Identifier),
        meta(
            "Snapshot hash",
            &artifact.snapshot_hash,
            MetaKind::Identifier,
        ),
        meta("Layout hash", &artifact.layout_hash, MetaKind::Identifier),
        meta(
            "Artifact hash",
            &artifact.manifest_hash,
            MetaKind::Identifier,
        ),
        meta(
            "Acknowledged findings",
            if artifact.acknowledged_findings.is_empty() {
                "None".to_owned()
            } else {
                artifact.acknowledged_findings.join(", ")
            },
            MetaKind::Text,
        ),
        meta(
            "Verification",
            if artifact.verified {
                "Verified against the manifest".to_owned()
            } else {
                format!("FAILED: {}", artifact.problems.join("; "))
            },
            MetaKind::Text,
        ),
    ];
    let mut note = if artifact.verified {
        format!(
            "Read from retained run {}; every file matches its manifest hash.",
            artifact.run
        )
    } else {
        format!(
            "Retained run {} failed verification. Do not trust these values.",
            artifact.run
        )
    };
    if selection.stage == StageId::Logical {
        for plane in &selection.planes {
            let Some(file) = plane_file(*plane) else {
                continue;
            };
            if let Some(p) = stage_files.iter().find(|p| p.file == file) {
                metadata.push(plane_row(format!("Coverage {}", plane.name()), p));
            }
        }
    } else {
        for p in &stage_files {
            metadata.push(plane_row(p.file.clone(), p));
        }
        note.push_str(
            " Slot letters come from the mode's channel map and are not mapped here, so files are listed by name.",
        );
    }

    // Only a verified run is drawn; a damaged one is shown as statistics and a warning.
    let raster_files: Vec<String> = if !artifact.verified {
        Vec::new()
    } else if composite {
        vec![COMPOSITE_RASTER.to_owned()]
    } else if selection.stage == StageId::Logical {
        selection
            .planes
            .iter()
            .filter_map(|plane| plane_file(*plane))
            .filter(|file| stage_files.iter().any(|p| p.file == *file))
            .collect()
    } else {
        stage_files.iter().map(|p| p.file.clone()).collect()
    };
    let raster_shown = !raster_files.is_empty();
    let panel = |title: &str| InfoPanel {
        title: title.to_owned(),
        message: NEEDS_RASTER.to_owned(),
    };
    Ok(InspectView {
        chip: stage_chip_text(selection.stage, PreviewSource::Retained, composite),
        metadata,
        composite: CompositeOption {
            label: COMPOSITE_LABEL.to_owned(),
            pressed: composite,
        },
        viewport_label: format!("Statistics for {}", plane_names(&selection)),
        selection,
        stages,
        planes,
        source: PreviewSource::Retained,
        metadata_note: note,
        panels: if raster_shown {
            Vec::new()
        } else {
            vec![panel("Coverage"), panel("Loupe")]
        },
        raster_shown,
        raster_files,
        placeholder: if raster_shown && composite {
            COMPOSITE_NOTE.to_owned()
        } else if raster_shown {
            "Drawn from the retained bytes after checking them against the manifest.".to_owned()
        } else {
            "Nothing is drawn: select a plane, or the run failed verification.".to_owned()
        },
    })
}
