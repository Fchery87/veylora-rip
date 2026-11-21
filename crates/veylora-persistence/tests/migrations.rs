use std::path::PathBuf;
use veylora_domain::ids::{
    AssetId, AttemptId, DeviceSetupId, JobId, PlacementId, PrintModeId, RecipeId,
};
use veylora_domain::job::{
    AssetReference, AttemptState, JobSnapshot, JobSnapshotInput, SubmissionAttempt,
};
use veylora_domain::layout::{LayoutInput, LayoutSnapshot, Placement, Rotation};
use veylora_domain::provenance::{ContentHash, Provenance};
use veylora_domain::recipe::*;
use veylora_persistence::{current_schema_version, FileJobStore, JobStore, PersistenceError};

fn tmp_dir(label: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "veylora-persistence-{}-{}-{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&base).unwrap();
    base
}

fn provenance() -> Provenance {
    Provenance {
        software_version: "0.0.0".into(),
        backend_version: "test".into(),
        content_hashes: vec![("layout".into(), ContentHash::digest(b"layout"))],
    }
}

fn snapshot(job_id: JobId) -> JobSnapshot {
    let asset_id = AssetId::new();
    let reference = || VersionedReference {
        version: 1,
        content_hash: ContentHash::digest(b"synthetic"),
    };
    JobSnapshot::new(JobSnapshotInput {
        schema_version: 1,
        id: job_id,
        created_at_ms: 1,
        assets: vec![AssetReference {
            id: asset_id,
            content_hash: ContentHash::digest(b"asset"),
        }],
        masks: vec![],
        layout: LayoutSnapshot::new(LayoutInput {
            schema_version: 1,
            page_width_um: 1000,
            page_height_um: 1000,
            mirror: false,
            placements: vec![Placement {
                id: PlacementId::new(),
                asset_id,
                mask_id: None,
                x_um: 0,
                y_um: 0,
                width_um: 1000,
                height_um: 1000,
                rotation: Rotation::Deg0,
                crop_x_px: 0,
                crop_y_px: 0,
                crop_width_px: 1,
                crop_height_px: 1,
            }],
        })
        .unwrap(),
        device_setup_id: DeviceSetupId::new(),
        device_setup: reference(),
        print_mode_id: PrintModeId::new(),
        print_mode: reference(),
        recipe: PrintRecipe::new(PrintRecipeInput {
            schema_version: 2,
            id: RecipeId::new(),
            revision: 1,
            created_at_ms: 1,
            status: RecipeStatus::Experimental,
            white: WhitePolicy {
                source: WhiteSource::Disabled,
                combine: MaskCombination::Replace,
                density_percent: 0,
                spread_um: 0,
                response: vec![0, 65535],
            },
            calibration: CalibrationPolicy {
                revision: 1,
                process_curves: std::array::from_fn(|_| vec![0, 65535]),
                channel_limits_percent: [100; 4],
                total_coverage_percent: 300,
            },
            color: ColorPolicy {
                profile: None,
                rendering_intent: RenderingIntent::Perceptual,
                black_point_compensation: false,
                untagged_rgb_policy: UntaggedRgbPolicy::Reject,
            },
            screen: ScreenPolicy {
                algorithm: ScreenAlgorithm::OrderedThreshold,
                revision: 1,
                seed: 0,
                phase_x: 0,
                phase_y: 0,
                cell_width: 1,
                cell_height: 1,
                thresholds: vec![32768],
            },
            clear: ClearPolicy::default(),
            validation: None,
            finish: FinishPolicy {
                ink_id: "synthetic".into(),
                film_id: "synthetic".into(),
                powder_id: "synthetic".into(),
                garment: "synthetic".into(),
                cure_temperature_c: 1,
                cure_seconds: 1,
                press_temperature_c: 1,
                press_seconds: 1,
                press_pressure_pa: 1,
                peel: PeelCondition::Cold,
            },
        })
        .unwrap(),
        provenance: provenance(),
    })
    .unwrap()
}

fn attempt(
    job: &JobSnapshot,
    attempt_id: AttemptId,
    sequence: u32,
    state: AttemptState,
) -> SubmissionAttempt {
    SubmissionAttempt {
        attempt_id,
        job_id: job.job_id(),
        snapshot_hash: job.content_hash(),
        sequence,
        state,
        bytes_written: 0,
        note: String::new(),
        recorded_at: "2026-11-08T22:01:00Z".into(),
    }
}

#[test]
fn migration_from_zero_creates_version_file_and_log() {
    let dir = tmp_dir("migration-zero");
    assert!(!dir.join("VERSION").exists());
    let store = FileJobStore::open(&dir).unwrap();
    assert_eq!(store.schema_version().unwrap(), current_schema_version());
    assert!(dir.join("VERSION").exists());
    assert!(dir.join("jobs").is_dir());
    assert!(dir.join("attempts.log").is_file());
}

#[test]
fn restart_recovery_rereads_existing_snapshots_and_attempts() {
    let dir = tmp_dir("restart");
    let job_id = JobId::new();
    let snap = snapshot(job_id);
    let attempt_id = AttemptId::new();
    {
        let store = FileJobStore::open(&dir).unwrap();
        store.create_snapshot(snap.clone()).unwrap();
        store
            .append_attempt(attempt(&snap, attempt_id, 1, AttemptState::Submitting))
            .unwrap();
        store
            .append_attempt(attempt(&snap, attempt_id, 2, AttemptState::Submitted))
            .unwrap();
    }
    let store = FileJobStore::open(&dir).unwrap();
    assert_eq!(store.get(job_id).unwrap(), snap);
    let attempts = store.list_attempts(job_id).unwrap();
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].sequence, 2);
    assert_eq!(attempts[0].state, AttemptState::Submitted);
}

#[test]
fn snapshots_are_immutable_once_written() {
    let dir = tmp_dir("immutable");
    let store = FileJobStore::open(&dir).unwrap();
    let snap = snapshot(JobId::new());
    store.create_snapshot(snap.clone()).unwrap();
    let err = store.create_snapshot(snap).unwrap_err();
    assert!(matches!(
        err,
        PersistenceError::ImmutableRecordExists { .. }
    ));
}

#[test]
fn attempts_log_enforces_ordering_and_hash_alignment() {
    let dir = tmp_dir("attempts-order");
    let store = FileJobStore::open(&dir).unwrap();
    let snap = snapshot(JobId::new());
    store.create_snapshot(snap.clone()).unwrap();
    let a = AttemptId::new();

    // Second event cannot reorder the sequence.
    store
        .append_attempt(attempt(&snap, a, 1, AttemptState::Submitting))
        .unwrap();
    let err = store
        .append_attempt(attempt(&snap, a, 1, AttemptState::Submitted))
        .unwrap_err();
    assert!(matches!(
        err,
        PersistenceError::AttemptContractViolation { .. }
    ));

    // New attempt cannot open at sequence 1 while a prior exists.
    let b = AttemptId::new();
    let err = store
        .append_attempt(attempt(&snap, b, 1, AttemptState::Submitting))
        .unwrap_err();
    assert!(matches!(
        err,
        PersistenceError::AttemptContractViolation { .. }
    ));

    // Snapshot-hash drift is rejected.
    let mut drift = attempt(&snap, a, 2, AttemptState::Submitted);
    drift.snapshot_hash = ContentHash::digest(b"wrong");
    let err = store.append_attempt(drift).unwrap_err();
    assert!(matches!(
        err,
        PersistenceError::AttemptContractViolation { .. }
    ));
}

#[test]
fn ambiguous_attempt_blocks_automatic_retry_but_allows_operator_confirmation() {
    let dir = tmp_dir("ambiguous");
    let store = FileJobStore::open(&dir).unwrap();
    let snap = snapshot(JobId::new());
    store.create_snapshot(snap.clone()).unwrap();
    let a = AttemptId::new();
    store
        .append_attempt(attempt(&snap, a, 1, AttemptState::Submitting))
        .unwrap();
    store
        .append_attempt(attempt(&snap, a, 2, AttemptState::Ambiguous))
        .unwrap();

    // Any automatic follow-on attempt is refused.
    let b = AttemptId::new();
    let err = store
        .append_attempt(attempt(&snap, b, 3, AttemptState::Submitting))
        .unwrap_err();
    assert!(matches!(err, PersistenceError::AmbiguousNoRetry { .. }));

    // Operator confirmation on the same attempt is permitted.
    store
        .append_attempt(attempt(
            &snap,
            a,
            3,
            AttemptState::OperatorConfirmedFinished,
        ))
        .unwrap();
    let views = store.list_attempts(snap.job_id()).unwrap();
    assert_eq!(
        views.last().unwrap().state,
        AttemptState::OperatorConfirmedFinished
    );
}

#[test]
fn corrupted_version_file_is_reported_not_silently_recovered() {
    let dir = tmp_dir("corrupt-version");
    {
        let _ = FileJobStore::open(&dir).unwrap();
    }
    std::fs::write(dir.join("VERSION"), "not-a-number").unwrap();
    let err = FileJobStore::open(&dir).unwrap_err();
    assert!(matches!(err, PersistenceError::Corrupt { .. }));
}

#[test]
fn newer_schema_is_rejected_rather_than_downgraded() {
    let dir = tmp_dir("future-schema");
    {
        let _ = FileJobStore::open(&dir).unwrap();
    }
    std::fs::write(
        dir.join("VERSION"),
        (current_schema_version() + 7).to_string(),
    )
    .unwrap();
    let err = FileJobStore::open(&dir).unwrap_err();
    assert!(matches!(err, PersistenceError::SchemaOutOfRange { .. }));
}

#[test]
fn corrupt_attempt_log_entry_is_surfaced() {
    let dir = tmp_dir("corrupt-log");
    let store = FileJobStore::open(&dir).unwrap();
    let snap = snapshot(JobId::new());
    store.create_snapshot(snap.clone()).unwrap();
    store
        .append_attempt(attempt(
            &snap,
            AttemptId::new(),
            1,
            AttemptState::Submitting,
        ))
        .unwrap();
    // Append a junk line directly.
    std::fs::OpenOptions::new()
        .append(true)
        .open(dir.join("attempts.log"))
        .unwrap();
    use std::io::Write as _;
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(dir.join("attempts.log"))
        .unwrap();
    f.write_all(b"{not valid json}\n").unwrap();
    drop(f);
    let err = store.list_attempts(snap.job_id()).unwrap_err();
    assert!(matches!(err, PersistenceError::Corrupt { .. }));
}
