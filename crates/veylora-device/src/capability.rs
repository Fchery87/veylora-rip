//! Backend capability model. Capabilities describe what the printer backend can
//! actually honor. UI controls and compatibility checks are gated from this record.
//! A capability record does not authorize submission; it only reports what the
//! backend is prepared to do.

use std::collections::BTreeSet;

/// Three-value capability so the UI can distinguish "not offered" from "supported
/// but not validated on this device".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability {
    /// Not offered by this backend at all.
    Unsupported,
    /// Offered but not physically validated for the active device setup.
    Experimental,
    /// Offered and validated for the active device setup.
    Validated,
}

impl Capability {
    pub const fn is_offered(self) -> bool {
        !matches!(self, Self::Unsupported)
    }
}

/// Which actor owns horizontal mirroring. The RIP never silently mirrors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MirrorOwner {
    /// Operator or driver is expected to perform the mirror.
    Operator,
    /// RIP performs the mirror explicitly.
    Rip,
    /// No mirror. DTF workflows normally require a mirror; this value only applies
    /// to diagnostic rasters.
    None,
}

/// Stable identifier for a backend-honored control. Not an enum because future
/// controls arrive without recompiling consumers.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ControlId(pub String);

impl ControlId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

/// Immutable record of what the backend can honor for one device setup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceCapability {
    /// Whether the backend supports independent white-channel control.
    pub independent_white: Capability,
    /// Logical process channel names supported by the backend, order-insensitive.
    pub process_channels: BTreeSet<String>,
    pub mirror_owner: MirrorOwner,
    pub supported_controls: BTreeSet<ControlId>,
}

impl DeviceCapability {
    pub fn offers(&self, control: &ControlId) -> bool {
        self.supported_controls.contains(control)
    }
}
