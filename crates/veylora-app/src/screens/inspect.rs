//! Inspect tab presentation (design 3.7): stage and plane selection, the
//! preview label, and metadata. It never recomputes a raster.

use crate::state::{CapabilityState, DataSource};
use crate::view_models::{
    InspectView, JobDetail, MetaRow, Plane, PlaneOption, PreviewSource, StageId, StageOption,
};

pub use crate::view_models::InspectSelection;

struct StageDef {
    id: StageId,
    label: &'static str,
    chip: &'static str,
}

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

/// Source for a preview given what the app is running on. No retained
/// artifact exists yet, so a preview is never labeled retained.
pub fn preview_source(data_source: DataSource) -> PreviewSource {
    match data_source {
        DataSource::Fixture => PreviewSource::Fixture,
        DataSource::Live => PreviewSource::Simulation,
    }
}

/// The label every preview carries. It always ends with "Not physical output".
pub fn stage_chip_text(stage: StageId, source: PreviewSource) -> String {
    let def = STAGES
        .iter()
        .find(|s| s.id == stage)
        .expect("stage table is complete");
    let mut parts = vec![
        def.chip,
        if source == PreviewSource::Retained {
            "retained artifact"
        } else {
            "simulation"
        },
    ];
    if source == PreviewSource::Fixture {
        parts.push("Fixture data");
    }
    parts.push("Not physical output");
    parts.join(" \u{b7} ")
}

/// Toggling a plane always shows logical planes, in canonical C M Y K W order.
pub fn toggle_plane(current: &InspectSelection, plane: Plane, on: bool) -> InspectSelection {
    let mut planes: Vec<Plane> = current
        .planes
        .iter()
        .copied()
        .filter(|p| *p != plane)
        .collect();
    if on {
        planes.push(plane);
    }
    planes.sort();
    InspectSelection {
        stage: StageId::Logical,
        planes,
    }
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

    let stages = INSPECT_STAGES
        .iter()
        .map(|id| {
            let def = STAGES
                .iter()
                .find(|s| s.id == *id)
                .expect("stage table is complete");
            let blocked = *id == StageId::Screened && !caps.mode.screened_inspect;
            StageOption {
                id: *id,
                label: def.label.to_owned(),
                disabled: blocked,
                reason: blocked.then(|| NO_SCREENED.to_owned()),
            }
        })
        .collect();
    let planes = Plane::ALL
        .iter()
        .map(|plane| PlaneOption {
            plane: *plane,
            pressed: selection.planes.contains(plane),
        })
        .collect();

    let source = preview_source(caps.data_source);
    let orientation = if selection.stage == StageId::Logical {
        "Artwork orientation"
    } else {
        "Film orientation, mirrored by the mode"
    };
    let meta = |label: &str, value: &str| MetaRow {
        label: label.to_owned(),
        value: value.to_owned(),
    };

    Ok(InspectView {
        selection: InspectSelection {
            stage: selection.stage,
            planes: selection.planes.clone(),
        },
        stages,
        planes,
        chip: stage_chip_text(selection.stage, source),
        source,
        metadata: vec![
            meta("Job", &job.summary.id),
            meta("Mode", &caps.mode.id),
            meta("Orientation", orientation),
        ],
        raster_shown: false,
        placeholder:
            "No raster is shown. Retained artifacts arrive when backend integration is unblocked."
                .to_owned(),
    })
}
