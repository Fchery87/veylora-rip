//! Immutable setup configuration and physical-channel verification boundary.
use crate::channels::{
    invalid, LogicalChannel, LogicalChannelMap, PhysicalSlotId, ProtocolChannel,
};
use veylora_domain::{ids::DeviceSetupId, preflight::Finding, provenance::ContentHash};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceIdentity {
    pub model: String,
    pub serial_or_local_id: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalSlot {
    pub id: PhysicalSlotId,
    pub tank_label: String,
    pub head_label: String,
    pub ink: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupOrigin {
    Recorded,
    Imported,
    Inferred,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SetupDetails {
    pub identity: DeviceIdentity,
    pub firmware: Option<String>,
    pub connection: String,
    pub operating_system: String,
    pub conversion_supplier: String,
    pub ink: String,
    pub film: String,
    pub feed_path: String,
    pub installation_date: Option<String>,
    pub installation_notes: String,
    pub reference_settings: String,
    pub physical_slots: Vec<PhysicalSlot>,
    pub channel_map: LogicalChannelMap,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerificationState {
    Unverified,
    Experimental { reason: String },
    Verified { evidence_hash: ContentHash },
}
#[derive(Clone, Debug, PartialEq)]
pub struct ChannelEvidence {
    pub slot: PhysicalSlotId,
    pub protocol: ProtocolChannel,
    pub expected_logical: LogicalChannel,
    pub observed_logical: LogicalChannel,
}
#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceRecord {
    pub setup_id: DeviceSetupId,
    pub revision: u32,
    pub setup_hash: ContentHash,
    pub map_version: u32,
    pub channels: Vec<ChannelEvidence>,
    pub evidence_hash: ContentHash,
}
impl EvidenceRecord {
    pub fn complete(setup: &DeviceSetup) -> Self {
        let channels = setup
            .details
            .channel_map
            .bindings()
            .iter()
            .map(|b| ChannelEvidence {
                slot: b.slot,
                protocol: b.protocol,
                expected_logical: b.logical,
                observed_logical: b.logical,
            })
            .collect();
        Self {
            setup_id: setup.id,
            revision: setup.revision,
            setup_hash: setup.content_hash,
            map_version: setup.details.channel_map.version(),
            channels,
            evidence_hash: ContentHash::digest(b"synthetic physical evidence fixture"),
        }
    }
}
#[derive(Clone, Debug)]
pub struct DeviceSetup {
    id: DeviceSetupId,
    revision: u32,
    details: SetupDetails,
    origin: SetupOrigin,
    verification: VerificationState,
    content_hash: ContentHash,
}
impl DeviceSetup {
    pub fn new(
        id: DeviceSetupId,
        details: SetupDetails,
        origin: SetupOrigin,
    ) -> Result<Self, Finding> {
        if details.identity.model.trim().is_empty()
            || details.physical_slots.len() != details.channel_map.bindings().len()
        {
            return Err(invalid("Device identity and physical inventory must be complete and match the channel map."));
        }
        let mut slots = std::collections::BTreeSet::new();
        for slot in &details.physical_slots {
            if !slots.insert(slot.id)
                || slot.tank_label.trim().is_empty()
                || slot.head_label.trim().is_empty()
            {
                return Err(invalid(
                    "Physical slot inventory must be unique and labeled.",
                ));
            }
        }
        if !details
            .channel_map
            .bindings()
            .iter()
            .all(|binding| slots.contains(&binding.slot))
        {
            return Err(invalid(
                "Every channel-map slot must have physical inventory.",
            ));
        }
        let content_hash = hash_details(&details, origin, 1);
        Ok(Self {
            id,
            revision: 1,
            details,
            origin,
            verification: VerificationState::Unverified,
            content_hash,
        })
    }
    pub fn id(&self) -> DeviceSetupId {
        self.id
    }
    pub fn revision(&self) -> u32 {
        self.revision
    }
    pub fn details(&self) -> &SetupDetails {
        &self.details
    }
    pub fn origin(&self) -> SetupOrigin {
        self.origin
    }
    pub fn content_hash(&self) -> ContentHash {
        self.content_hash
    }
    pub fn verification(&self) -> &VerificationState {
        &self.verification
    }
    pub fn mark_experimental(&mut self, reason: impl Into<String>) -> Result<(), Finding> {
        let reason = reason.into();
        if reason.trim().is_empty() {
            return Err(invalid("Experimental setups require an explicit reason."));
        }
        self.verification = VerificationState::Experimental { reason };
        Ok(())
    }
    /// Setup-only gate. Passing never authorizes transport or validates a PrintMode.
    pub fn require_production(&self) -> Result<(), Finding> {
        if matches!(self.verification, VerificationState::Verified { .. }) {
            Ok(())
        } else {
            Err(invalid(
                "Production requires complete matched physical channel evidence.",
            ))
        }
    }
    pub fn verify(&mut self, evidence: EvidenceRecord) -> Result<(), Finding> {
        if evidence.setup_id != self.id
            || evidence.revision != self.revision
            || evidence.setup_hash != self.content_hash
            || evidence.map_version != self.details.channel_map.version()
            || evidence.channels.len() != self.details.channel_map.bindings().len()
        {
            return Err(invalid("Evidence must match the current setup revision, hash, map, and complete channel set."));
        }
        for binding in self.details.channel_map.bindings() {
            let Some(found) = evidence
                .channels
                .iter()
                .find(|e| e.slot == binding.slot && e.protocol == binding.protocol)
            else {
                return Err(invalid("Evidence is missing a mapped physical channel."));
            };
            if found.expected_logical != binding.logical
                || found.observed_logical != binding.logical
            {
                return Err(invalid(
                    "Observed channel evidence does not match the expected mapping.",
                ));
            }
        }
        self.verification = VerificationState::Verified {
            evidence_hash: evidence.evidence_hash,
        };
        Ok(())
    }
    pub fn revise(&mut self, details: SetupDetails) -> Result<(), Finding> {
        let next = Self::new(self.id, details, self.origin)?;
        self.revision += 1;
        self.details = next.details;
        self.content_hash = hash_details(&self.details, self.origin, self.revision);
        self.verification = VerificationState::Unverified;
        Ok(())
    }
}

fn hash_details(details: &SetupDetails, origin: SetupOrigin, revision: u32) -> ContentHash {
    let mut bytes = b"veylora-device-setup-v1\0".to_vec();
    bytes.extend_from_slice(&revision.to_le_bytes());
    bytes.push(origin as u8);
    let text = |out: &mut Vec<u8>, value: &str| {
        out.extend_from_slice(&(value.len() as u64).to_le_bytes());
        out.extend_from_slice(value.as_bytes());
    };
    text(&mut bytes, &details.identity.model);
    text(
        &mut bytes,
        details.identity.serial_or_local_id.as_deref().unwrap_or(""),
    );
    text(&mut bytes, details.firmware.as_deref().unwrap_or(""));
    for value in [
        &details.connection,
        &details.operating_system,
        &details.conversion_supplier,
        &details.ink,
        &details.film,
        &details.feed_path,
        &details.installation_notes,
        &details.reference_settings,
    ] {
        text(&mut bytes, value);
    }
    text(
        &mut bytes,
        details.installation_date.as_deref().unwrap_or(""),
    );
    for slot in &details.physical_slots {
        bytes.push(slot.id.0);
        text(&mut bytes, &slot.tank_label);
        text(&mut bytes, &slot.head_label);
        text(&mut bytes, &slot.ink);
    }
    details.channel_map.hash_bytes(&mut bytes);
    ContentHash::digest(&bytes)
}
