use std::path::PathBuf;
use veylora_domain::{
    ids::{AttemptId, JobId},
    lifecycle::{JobState, LifecycleEvent},
};
use veylora_persistence::{FileLifecycleStore, PersistenceError};

fn dir(label: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let p = std::env::temp_dir().join(format!(
        "veylora-lifecycle-{label}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&p);
    p
}

const T: &str = "2026-11-15T10:00:00Z";

fn walk(store: &FileLifecycleStore, job: JobId, events: &[LifecycleEvent]) {
    for e in events {
        store.append(job, *e, T, "", None).unwrap();
    }
}

#[test]
fn a_job_with_no_events_is_a_draft() {
    let store = FileLifecycleStore::open(dir("fresh")).unwrap();
    let job = JobId::new();
    assert_eq!(store.state(job).unwrap(), JobState::Draft);
    assert!(store.events(job).unwrap().is_empty());
}

#[test]
fn events_advance_the_state_and_are_retained_in_order() {
    use LifecycleEvent::*;
    let store = FileLifecycleStore::open(dir("walk")).unwrap();
    let job = JobId::new();
    walk(
        &store,
        job,
        &[PreflightPassed, BeginRender, RenderCompleted],
    );
    assert_eq!(store.state(job).unwrap(), JobState::Ready);
    let attempt = AttemptId::new();
    let rec = store
        .append(job, BeginSubmission, T, "start", Some(attempt))
        .unwrap();
    assert_eq!(
        (rec.sequence, rec.from, rec.to),
        (4, JobState::Ready, JobState::Submitting)
    );
    let events = store.events(job).unwrap();
    assert_eq!(
        events.iter().map(|e| e.sequence).collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
    assert_eq!(events[3].attempt_id, Some(attempt));
    assert_eq!(events[3].note, "start");
}

#[test]
fn an_illegal_transition_is_refused_and_writes_nothing() {
    use LifecycleEvent::*;
    let store = FileLifecycleStore::open(dir("illegal")).unwrap();
    let job = JobId::new();
    walk(&store, job, &[PreflightPassed]);
    // Held cannot be submitted.
    let err = store.append(job, BeginSubmission, T, "", None).unwrap_err();
    assert!(
        matches!(err, PersistenceError::LifecycleViolation { .. }),
        "{err:?}"
    );
    assert_eq!(store.events(job).unwrap().len(), 1);
    assert_eq!(store.state(job).unwrap(), JobState::Held);
}

#[test]
fn state_is_reconstructed_after_a_restart() {
    use LifecycleEvent::*;
    let root = dir("restart");
    let job = JobId::new();
    {
        let store = FileLifecycleStore::open(&root).unwrap();
        walk(
            &store,
            job,
            &[
                PreflightPassed,
                BeginRender,
                RenderCompleted,
                BeginSubmission,
            ],
        );
    }
    let reopened = FileLifecycleStore::open(&root).unwrap();
    // An open submission is still open; nothing assumed it finished.
    assert_eq!(reopened.state(job).unwrap(), JobState::Submitting);
}

#[test]
fn jobs_do_not_share_history() {
    use LifecycleEvent::*;
    let store = FileLifecycleStore::open(dir("isolated")).unwrap();
    let (a, b) = (JobId::new(), JobId::new());
    walk(&store, a, &[PreflightPassed]);
    assert_eq!(store.state(b).unwrap(), JobState::Draft);
}

#[test]
fn damaged_history_is_reported_not_repaired() {
    use LifecycleEvent::*;
    let root = dir("damage");
    let store = FileLifecycleStore::open(&root).unwrap();
    let job = JobId::new();
    walk(&store, job, &[PreflightPassed, BeginRender]);
    let first = root
        .join("lifecycle")
        .join(job.to_string())
        .join("00000001.json");
    let second = root
        .join("lifecycle")
        .join(job.to_string())
        .join("00000002.json");
    let original = std::fs::read_to_string(&first).unwrap();

    // Edited result state.
    std::fs::write(&first, original.replace("\"held\"", "\"ready\"")).unwrap();
    assert!(matches!(
        store.state(job),
        Err(PersistenceError::Corrupt { .. })
    ));
    std::fs::write(&first, &original).unwrap();

    // Missing step.
    std::fs::remove_file(&first).unwrap();
    assert!(matches!(
        store.state(job),
        Err(PersistenceError::Corrupt { .. })
    ));
    std::fs::write(&first, &original).unwrap();

    // Garbage and a record filed under the wrong sequence.
    std::fs::write(&second, b"not json").unwrap();
    assert!(matches!(
        store.state(job),
        Err(PersistenceError::Corrupt { .. })
    ));
    std::fs::write(&second, &original).unwrap();
    assert!(matches!(
        store.state(job),
        Err(PersistenceError::Corrupt { .. })
    ));
    // Appending to damaged history is refused too.
    assert!(store.append(job, RenderCompleted, T, "", None).is_err());
}

#[test]
fn leftover_temporary_files_from_a_crash_are_ignored() {
    use LifecycleEvent::*;
    let root = dir("tmp");
    let store = FileLifecycleStore::open(&root).unwrap();
    let job = JobId::new();
    walk(&store, job, &[PreflightPassed]);
    let d = root.join("lifecycle").join(job.to_string());
    std::fs::write(d.join("00000002.json.123.tmp"), b"{ half").unwrap();
    assert_eq!(store.state(job).unwrap(), JobState::Held);
    walk(&store, job, &[BeginRender]);
    assert_eq!(store.state(job).unwrap(), JobState::Rendering);
}

#[test]
fn concurrent_writers_cannot_both_make_the_same_transition() {
    let root = dir("race");
    let job = JobId::new();
    let results: Vec<bool> = (0..8)
        .map(|_| {
            let root = root.clone();
            std::thread::spawn(move || {
                let store = FileLifecycleStore::open(&root).unwrap();
                store
                    .append(job, LifecycleEvent::PreflightPassed, T, "", None)
                    .is_ok()
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    assert_eq!(results.iter().filter(|ok| **ok).count(), 1);
    let store = FileLifecycleStore::open(&root).unwrap();
    assert_eq!(store.events(job).unwrap().len(), 1);
}

/// Each tampering below keeps every other property of the history intact, so only one check can
/// notice it.
#[test]
fn every_integrity_check_catches_damage_only_it_can_see() {
    use LifecycleEvent::*;
    let root = dir("isolated-damage");
    let store = FileLifecycleStore::open(&root).unwrap();
    let file = |job: JobId, n: u32| {
        root.join("lifecycle")
            .join(job.to_string())
            .join(format!("{n:08}.json"))
    };
    let corrupt = |job| matches!(store.state(job), Err(PersistenceError::Corrupt { .. }));

    // The recorded result alone is wrong.
    let job = JobId::new();
    walk(&store, job, &[PreflightPassed]);
    let original = std::fs::read_to_string(file(job, 1)).unwrap();
    std::fs::write(
        file(job, 1),
        original.replace("\"to\":\"held\"", "\"to\":\"ready\""),
    )
    .unwrap();
    assert!(corrupt(job), "result");

    // The recorded start state alone is wrong, yet the move is legal from it.
    let job = JobId::new();
    walk(&store, job, &[PreflightPassed]);
    let original = std::fs::read_to_string(file(job, 1)).unwrap();
    std::fs::write(
        file(job, 1),
        original.replace("\"from\":\"draft\"", "\"from\":\"preflight_failed\""),
    )
    .unwrap();
    assert!(corrupt(job), "from");

    // Only the sequence is off: the record is renumbered to 2 and filed as 2, with no 1.
    let job = JobId::new();
    walk(&store, job, &[PreflightPassed]);
    let original = std::fs::read_to_string(file(job, 1)).unwrap();
    std::fs::remove_file(file(job, 1)).unwrap();
    std::fs::write(
        file(job, 2),
        original.replace("\"sequence\":1", "\"sequence\":2"),
    )
    .unwrap();
    assert!(corrupt(job), "contiguity");

    // A record belonging to another job.
    let (a, b) = (JobId::new(), JobId::new());
    walk(&store, a, &[PreflightPassed]);
    walk(&store, b, &[PreflightPassed]);
    std::fs::copy(file(b, 1), file(a, 1)).unwrap();
    assert!(corrupt(a), "job id");

    // An unknown schema version.
    let job = JobId::new();
    walk(&store, job, &[PreflightPassed]);
    let original = std::fs::read_to_string(file(job, 1)).unwrap();
    std::fs::write(
        file(job, 1),
        original.replace("\"schema_version\":1", "\"schema_version\":2"),
    )
    .unwrap();
    assert!(corrupt(job), "schema");
}

#[test]
fn events_need_a_timestamp() {
    let store = FileLifecycleStore::open(dir("stamp")).unwrap();
    let job = JobId::new();
    let err = store
        .append(job, LifecycleEvent::PreflightPassed, "  ", "", None)
        .unwrap_err();
    assert!(matches!(err, PersistenceError::LifecycleViolation { .. }));
    assert!(store.events(job).unwrap().is_empty());
}
