//! Reserved platform adapter boundary. Native Windows spooler calls are intentionally not implemented here.
//! The printer-independent fake transport and file sink are the only supported transports in this task.

use crate::{
    PrinterArtifact, PrinterTransport, SubmissionObserver, TransportError, TransportResult,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsSpoolerTransport;

impl PrinterTransport for WindowsSpoolerTransport {
    fn submit(
        &self,
        _artifact: &PrinterArtifact,
        _observer: &mut dyn SubmissionObserver,
    ) -> Result<TransportResult, TransportError> {
        Err(TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Windows spooler transport is not implemented",
        )))
    }
}
