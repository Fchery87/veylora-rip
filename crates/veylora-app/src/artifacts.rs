//! Retained render artifacts as the app sees them: a read model over the workflow service.
//! Nothing here renders; it re-reads what a completed run kept and reports it.

use veylora_device::channels::LogicalChannel;
use veylora_domain::ids::JobId;
use veylora_workflow::{JobChannel, Workspace};

use crate::view_models::{RetainedArtifact, RetainedPlane, RetainedSlot};

/// The Inspect letter for each binding, in the order given. C, M, Y and K name a binding only when
/// the map has exactly one of that channel. White slots are lettered W1, W2 by ascending slot
/// number, and only when there are no more than two. Anything else gets no letter, so nothing is
/// guessed.
pub fn slot_letters(channels: &[JobChannel]) -> Vec<Option<&'static str>> {
    let count = |c: LogicalChannel| channels.iter().filter(|b| b.logical == c).count();
    let mut whites: Vec<u8> = channels
        .iter()
        .filter(|b| b.logical == LogicalChannel::White)
        .map(|b| b.slot)
        .collect();
    whites.sort_unstable();
    channels
        .iter()
        .map(|b| match b.logical {
            _ if count(b.logical) != 1 && b.logical != LogicalChannel::White => None,
            LogicalChannel::Cyan => Some("C"),
            LogicalChannel::Magenta => Some("M"),
            LogicalChannel::Yellow => Some("Y"),
            LogicalChannel::Black => Some("K"),
            LogicalChannel::White if whites.len() <= 2 => {
                match whites.iter().position(|s| *s == b.slot) {
                    Some(0) => Some("W1"),
                    Some(1) => Some("W2"),
                    _ => None,
                }
            }
            LogicalChannel::White => None,
        })
        .collect()
}

fn logical_name(c: LogicalChannel) -> &'static str {
    match c {
        LogicalChannel::Cyan => "cyan",
        LogicalChannel::Magenta => "magenta",
        LogicalChannel::Yellow => "yellow",
        LogicalChannel::Black => "black",
        LogicalChannel::White => "white",
    }
}

/// The newest completed run of a job, or `None` when it was never rendered.
pub fn retained_for_job(ws: &Workspace, job_id: &str) -> Result<Option<RetainedArtifact>, String> {
    let id: JobId = job_id
        .parse()
        .map_err(|_| format!("'{job_id}' is not a job id."))?;
    let Some(inspected) = ws.inspect_latest_run(id).map_err(|e| e.to_string())? else {
        return Ok(None);
    };
    let device = ws.job_device(id).map_err(|e| e.to_string())?;
    let letters = slot_letters(&device.channels);
    let slots = device
        .channels
        .iter()
        .zip(letters)
        .map(|(c, letter)| RetainedSlot {
            letter: letter.map(str::to_owned),
            logical: logical_name(c.logical).to_owned(),
            protocol: c.protocol,
            slot: c.slot,
            fraction: c.fraction,
            tank_label: c.tank_label.clone(),
            head_label: c.head_label.clone(),
        })
        .collect();
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
        slots,
        channel_map_version: device.channel_map_version,
        channel_map_verified: device.setup_physically_verified,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bind(logical: LogicalChannel, slot: u8) -> JobChannel {
        JobChannel {
            logical,
            slot,
            protocol: 10 + slot,
            fraction: 1.0,
            tank_label: String::new(),
            head_label: String::new(),
        }
    }

    #[test]
    fn white_slots_are_lettered_by_ascending_slot_number_whatever_the_stored_order() {
        use LogicalChannel::*;
        let map = [
            bind(White, 5),
            bind(Cyan, 0),
            bind(White, 4),
            bind(Black, 3),
        ];
        assert_eq!(
            slot_letters(&map),
            vec![Some("W2"), Some("C"), Some("W1"), Some("K")]
        );
    }

    #[test]
    fn a_map_that_does_not_name_a_slot_unambiguously_gets_no_letter() {
        use LogicalChannel::*;
        // Two cyan slots: neither is "the" C. Three white slots do not fit W1 and W2.
        let map = [
            bind(Cyan, 0),
            bind(Cyan, 1),
            bind(White, 2),
            bind(White, 3),
            bind(White, 4),
        ];
        assert_eq!(slot_letters(&map), vec![None; 5]);
    }
}
