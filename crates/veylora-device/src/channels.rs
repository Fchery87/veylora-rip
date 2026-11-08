//! Logical planes, physical slots and protocol addresses are deliberately distinct.
use veylora_domain::{
    errors::ErrorCode,
    preflight::{Finding, Severity},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogicalChannel {
    Cyan,
    Magenta,
    Yellow,
    Black,
    White,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalSlotId(pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtocolChannel(pub u8);

#[derive(Clone, Debug, PartialEq)]
pub struct ChannelBinding {
    pub logical: LogicalChannel,
    pub slot: PhysicalSlotId,
    pub protocol: ProtocolChannel,
    /// Fraction of the logical coverage assigned to this slot, not an ink multiplier.
    pub fraction: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LogicalChannelMap {
    version: u32,
    bindings: Vec<ChannelBinding>,
}

impl LogicalChannelMap {
    pub fn new(version: u32, bindings: Vec<ChannelBinding>) -> Result<Self, Finding> {
        if version == 0 {
            return Err(invalid("Channel map version must be positive."));
        }
        let mut slots = std::collections::BTreeSet::new();
        let mut protocols = std::collections::BTreeSet::new();
        for binding in &bindings {
            if !slots.insert(binding.slot) || !protocols.insert(binding.protocol) {
                return Err(invalid(
                    "Physical slots and protocol channels must be unique.",
                ));
            }
            if !binding.fraction.is_finite() || binding.fraction <= 0.0 || binding.fraction > 1.0 {
                return Err(invalid(
                    "Allocation fractions must be finite and in (0, 1].",
                ));
            }
        }
        for logical in [
            LogicalChannel::Cyan,
            LogicalChannel::Magenta,
            LogicalChannel::Yellow,
            LogicalChannel::Black,
            LogicalChannel::White,
        ] {
            let matching: Vec<_> = bindings.iter().filter(|b| b.logical == logical).collect();
            if matching.is_empty() || matching.iter().map(|b| b.fraction).sum::<f64>() > 1.0 {
                return Err(invalid(
                    "Every required logical channel needs an allocation totaling at most one.",
                ));
            }
            if logical != LogicalChannel::White
                && (matching.len() != 1 || matching[0].fraction != 1.0)
            {
                return Err(invalid("Process channels require one full-coverage slot."));
            }
        }
        Ok(Self { version, bindings })
    }

    pub fn version(&self) -> u32 {
        self.version
    }
    pub fn bindings(&self) -> &[ChannelBinding] {
        &self.bindings
    }

    pub(crate) fn hash_bytes(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(&self.version.to_le_bytes());
        for binding in &self.bindings {
            output.push(binding.logical as u8);
            output.push(binding.slot.0);
            output.push(binding.protocol.0);
            output.extend_from_slice(&binding.fraction.to_le_bytes());
        }
    }

    pub fn allocate(
        &self,
        logical: LogicalChannel,
        coverage: f64,
    ) -> Result<Vec<(PhysicalSlotId, f64)>, Finding> {
        if !coverage.is_finite() || !(0.0..=1.0).contains(&coverage) {
            return Err(invalid("Logical coverage must be finite and in [0, 1]."));
        }
        Ok(self
            .bindings
            .iter()
            .filter(|b| b.logical == logical)
            .map(|b| (b.slot, coverage * b.fraction))
            .collect())
    }
}

pub(crate) fn invalid(message: &str) -> Finding {
    Finding {
        code: ErrorCode::UnverifiedDeviceSetup,
        severity: Severity::Blocking,
        message: message.into(),
        subject: None,
    }
}
