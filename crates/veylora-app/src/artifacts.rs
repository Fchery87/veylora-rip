//! Retained render artifacts as the app sees them: a read model over the workflow service.
//! Nothing here renders; it re-reads what a completed run kept and reports it.

use veylora_domain::ids::JobId;
use veylora_workflow::Workspace;

use crate::view_models::{RetainedArtifact, RetainedPlane};

/// The newest completed run of a job, or `None` when it was never rendered.
pub fn retained_for_job(ws: &Workspace, job_id: &str) -> Result<Option<RetainedArtifact>, String> {
    let id: JobId = job_id
        .parse()
        .map_err(|_| format!("'{job_id}' is not a job id."))?;
    let Some(inspected) = ws.inspect_latest_run(id).map_err(|e| e.to_string())? else {
        return Ok(None);
    };
    let report = inspected.report;
    let m = &report.manifest;
    Ok(Some(RetainedArtifact {
        job_id: id.to_string(),
        run: inspected.run,
        evidence: m.evidence.clone(),
        manifest_hash: report.manifest_hash.to_string(),
        snapshot_hash: m.snapshot_hash.to_string(),
        recipe_hash: m.recipe_hash.to_string(),
        layout_hash: m.layout_hash.to_string(),
        width_px: m.page_width_px as u64,
        height_px: m.page_height_px as u64,
        pitch_x_nm: m.pitch_x_nm,
        pitch_y_nm: m.pitch_y_nm,
        verified: report.verified,
        problems: report.problems,
        acknowledged_findings: m
            .acknowledged_layout_findings
            .iter()
            .map(|f| format!("{}:{}", f.code, f.subject.as_deref().unwrap_or("-")))
            .collect(),
        planes: report
            .planes
            .into_iter()
            .map(|p| RetainedPlane {
                file: p.name,
                pixels: p.pixels,
                nonzero: p.nonzero,
                min: p.min,
                max: p.max,
                mean: p.mean,
            })
            .collect(),
    }))
}
