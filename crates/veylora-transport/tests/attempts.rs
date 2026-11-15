use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::tempdir;
use veylora_transport::{
    fake::{FailurePoint, FakeTransport},
    FileSink, PrinterArtifact, PrinterTransport, SubmissionAttempts, SubmissionObserver,
    TransportError, TransportStatus,
};

#[derive(Default)]
struct Probe {
    bytes: u64,
    statuses: Vec<TransportStatus>,
}
impl SubmissionObserver for Probe {
    fn on_bytes(&mut self, count: u64) {
        self.bytes += count;
    }
    fn on_status(&mut self, status: TransportStatus) {
        self.statuses.push(status);
    }
}
struct Cancelled(AtomicBool);
impl veylora_transport::Cancellation for Cancelled {
    fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

struct CancelOnWrite {
    cancelled: Arc<Cancelled>,
}
impl SubmissionObserver for CancelOnWrite {
    fn on_bytes(&mut self, _count: u64) {
        self.cancelled.0.store(true, Ordering::Relaxed);
    }
    fn on_status(&mut self, _status: TransportStatus) {}
}

#[test]
fn fake_transport_records_before_during_and_ambiguous_write_facts() {
    let artifact = PrinterArtifact::from_encoded_bytes(vec![1, 2, 3, 4]);
    for (failure, expected) in [
        (FailurePoint::BeforeWrite, 0),
        (FailurePoint::DuringWrite { bytes: 2 }, 2),
        (FailurePoint::AfterWrite, 4),
    ] {
        let mut probe = Probe::default();
        let error = FakeTransport::new(failure)
            .submit(&artifact, &mut probe)
            .unwrap_err();
        let written = match error {
            TransportError::BeforeWrite => 0,
            TransportError::DuringWrite { bytes_written, .. }
            | TransportError::Ambiguous { bytes_written, .. } => bytes_written,
            _ => panic!("unexpected error"),
        };
        assert_eq!(written, expected);
        assert_eq!(probe.bytes, expected);
    }
}

#[test]
fn ambiguous_attempt_is_preserved_without_retry_api() {
    let artifact = PrinterArtifact::from_encoded_bytes(vec![7, 8]);
    let mut probe = Probe::default();
    let error = FakeTransport::new(FailurePoint::AfterWrite)
        .submit(&artifact, &mut probe)
        .unwrap_err();
    let mut attempts = SubmissionAttempts::default();
    attempts.record(veylora_transport::TransportResult {
        attempt_id: uuid::Uuid::new_v4(),
        outcome: veylora_transport::TransportOutcome::Ambiguous,
        bytes_intended: 2,
        bytes_written: 2,
    });
    assert!(matches!(error, TransportError::Ambiguous { .. }));
    assert_eq!(attempts.facts().len(), 1);
}

#[test]
fn cancellation_before_and_after_write_is_explicit() {
    let cancellation = Arc::new(Cancelled(AtomicBool::new(true)));
    let artifact = PrinterArtifact::from_encoded_bytes(vec![1, 2]);
    let mut probe = Probe::default();
    let error = FakeTransport::new(FailurePoint::None)
        .with_cancellation(cancellation)
        .submit(&artifact, &mut probe)
        .unwrap_err();
    assert!(matches!(
        error,
        TransportError::Cancelled {
            bytes_written: 0,
            ..
        }
    ));
    assert!(probe.statuses.contains(&TransportStatus::Cancelled));

    let cancellation = Arc::new(Cancelled(AtomicBool::new(false)));
    let mut observer = CancelOnWrite {
        cancelled: Arc::clone(&cancellation),
    };
    let error = FakeTransport::new(FailurePoint::None)
        .with_cancellation(cancellation)
        .submit(&artifact, &mut observer)
        .unwrap_err();
    assert!(matches!(
        error,
        TransportError::Cancelled {
            bytes_written: 2,
            ..
        }
    ));
}

#[test]
fn file_sink_retains_exact_bytes_and_refuses_overwrite() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nested").join("artifact.bin");
    let artifact = PrinterArtifact::from_encoded_bytes(vec![0, 1, 255, 4]);
    let sink = FileSink::new(&path);
    sink.write(&artifact).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), artifact.bytes());
    assert!(matches!(
        sink.write(&artifact),
        Err(veylora_transport::FileSinkError::AlreadyExists(_))
    ));
}
