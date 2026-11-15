use crate::{
    Cancellation, PrinterArtifact, PrinterTransport, SubmissionObserver, TransportError,
    TransportOutcome, TransportResult, TransportStatus,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailurePoint {
    None,
    BeforeWrite,
    DuringWrite { bytes: u64 },
    AfterWrite,
}

#[derive(Clone)]
pub struct FakeTransport {
    failure: FailurePoint,
    cancel: Option<Arc<dyn Cancellation>>,
}
impl FakeTransport {
    pub fn new(failure: FailurePoint) -> Self {
        Self {
            failure,
            cancel: None,
        }
    }
    pub fn with_cancellation(mut self, cancellation: Arc<dyn Cancellation>) -> Self {
        self.cancel = Some(cancellation);
        self
    }
}

impl PrinterTransport for FakeTransport {
    fn submit(
        &self,
        artifact: &PrinterArtifact,
        observer: &mut dyn SubmissionObserver,
    ) -> Result<TransportResult, TransportError> {
        let id = Uuid::new_v4();
        let intended = artifact.len();
        observer.on_status(TransportStatus::Started);
        if self.cancel.as_ref().is_some_and(|c| c.is_cancelled()) {
            observer.on_status(TransportStatus::Cancelled);
            return Err(TransportError::Cancelled {
                bytes_written: 0,
                bytes_intended: intended,
            });
        }
        if self.failure == FailurePoint::BeforeWrite {
            return Err(TransportError::BeforeWrite);
        }
        let written = match self.failure {
            FailurePoint::DuringWrite { bytes } => bytes.min(intended),
            _ => intended,
        };
        if written > 0 {
            observer.on_status(TransportStatus::Writing);
            observer.on_bytes(written);
        }
        if self.cancel.as_ref().is_some_and(|c| c.is_cancelled()) {
            observer.on_status(TransportStatus::Cancelled);
            return Err(TransportError::Cancelled {
                bytes_written: written,
                bytes_intended: intended,
            });
        }
        match self.failure {
            FailurePoint::DuringWrite { .. } => Err(TransportError::DuringWrite {
                bytes_written: written,
                bytes_intended: intended,
            }),
            FailurePoint::AfterWrite => Err(TransportError::Ambiguous {
                bytes_written: written,
                bytes_intended: intended,
            }),
            FailurePoint::None => {
                observer.on_status(TransportStatus::Finished);
                Ok(TransportResult {
                    attempt_id: id,
                    outcome: TransportOutcome::Submitted,
                    bytes_intended: intended,
                    bytes_written: written,
                })
            }
            FailurePoint::BeforeWrite => unreachable!(),
        }
    }
}
