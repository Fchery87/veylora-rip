//! Immutable setup configuration and physical-channel verification boundary.
use crate::channels::{
    invalid, LogicalChannel, LogicalChannelMap, PhysicalSlotId, ProtocolChannel,
};
use serde::{Deserialize, Serialize};
use veylora_domain::{ids::DeviceSetupId, preflight::Finding, provenance::ContentHash};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceIdentity {
    pub model: String,
    pub serial_or_local_id: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalSlot {
    pub id: PhysicalSlotId,
    pub tank_label: String,
    pub head_label: String,
    pub ink: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetupOrigin {
    Recorded,
    Imported,
    Inferred,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationState {
    Unverified,
    Experimental { reason: String },
    Verified { evidence_hash: ContentHash },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelEvidence {
    pub target_label: String,
    pub slot: PhysicalSlotId,
    pub protocol: ProtocolChannel,
    pub expected_logical: LogicalChannel,
    pub observed_logical: LogicalChannel,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRecord {
    pub operator: String,
    pub setup_id: DeviceSetupId,
    pub revision: u32,
    pub setup_hash: ContentHash,
    pub map_version: u32,
    pub channels: Vec<ChannelEvidence>,
    pub target_id: String,
    pub observed_at: String,
    pub artifact_hash: ContentHash,
    pub evidence_hash: ContentHash,
}
#[derive(Clone, Debug)]
pub struct DeviceSetup {
    id: DeviceSetupId,
    revision: u32,
    details: SetupDetails,
    origin: SetupOrigin,
    verification: VerificationState,
    content_hash: ContentHash,
    evidence: Option<EvidenceRecord>,
}
/// Serialized form of a setup. Loading re-validates the details, recomputes the content hash and
/// replays the verification state against the retained evidence; nothing is trusted as written.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetupRecord {
    pub schema_version: u32,
    pub id: DeviceSetupId,
    pub revision: u32,
    pub origin: SetupOrigin,
    pub details: SetupDetails,
    pub verification: VerificationState,
    pub evidence: Option<EvidenceRecord>,
    pub content_hash: ContentHash,
}
pub const SETUP_RECORD_SCHEMA: u32 = 1;

impl DeviceSetup {
    pub fn to_record(&self) -> SetupRecord {
        SetupRecord {
            schema_version: SETUP_RECORD_SCHEMA,
            id: self.id,
            revision: self.revision,
            origin: self.origin,
            details: self.details.clone(),
            verification: self.verification.clone(),
            evidence: self.evidence.clone(),
            content_hash: self.content_hash,
        }
    }

    pub fn from_record(record: SetupRecord) -> Result<Self, Finding> {
        if record.schema_version != SETUP_RECORD_SCHEMA || record.revision == 0 {
            return Err(invalid(
                "Unsupported device setup record schema or revision.",
            ));
        }
        let mut setup = Self::new(record.id, record.details, record.origin)?;
        setup.revision = record.revision;
        setup.content_hash = hash_details(setup.id, &setup.details, setup.origin, setup.revision);
        if setup.content_hash != record.content_hash {
            return Err(invalid(
                "Device setup record does not match its content hash.",
            ));
        }
        match (record.verification, record.evidence) {
            (VerificationState::Unverified, None) => {}
            (VerificationState::Experimental { reason }, None) => {
                setup.mark_experimental(reason)?
            }
            (VerificationState::Verified { evidence_hash }, Some(evidence)) => {
                if evidence.evidence_hash != evidence_hash {
                    return Err(invalid(
                        "Verified setup record names different evidence than it retains.",
                    ));
                }
                setup.verify(evidence)?;
            }
            _ => {
                return Err(invalid(
                    "Setup verification state and retained evidence disagree.",
                ))
            }
        }
        Ok(setup)
    }

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
        let content_hash = hash_details(id, &details, origin, 1);
        Ok(Self {
            id,
            revision: 1,
            details,
            origin,
            verification: VerificationState::Unverified,
            content_hash,
            evidence: None,
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
    pub fn evidence(&self) -> Option<&EvidenceRecord> {
        self.evidence.as_ref()
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
        if evidence.target_id.trim().is_empty()
            || evidence.observed_at.trim().is_empty()
            || evidence.operator.trim().is_empty()
        {
            return Err(invalid(
                "Physical evidence requires a target ID and observation timestamp.",
            ));
        }
        let mut labels = std::collections::BTreeSet::new();
        for channel in &evidence.channels {
            if channel.target_label.trim().is_empty() || !labels.insert(&channel.target_label) {
                return Err(invalid(
                    "Every physical observation requires a distinct printed target label.",
                ));
            }
        }
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
        self.evidence = Some(evidence);
        Ok(())
    }
    pub fn revise(&self, details: SetupDetails) -> Result<Self, Finding> {
        let old_map = &self.details.channel_map;
        let new_map = &details.channel_map;
        if new_map.version() < old_map.version()
            || (new_map.bindings() != old_map.bindings() && new_map.version() <= old_map.version())
        {
            return Err(invalid(
                "Changed channel bindings require an increasing map version.",
            ));
        }
        let next = Self::new(self.id, details, self.origin)?;
        let revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("Setup revision overflow."))?;
        let content_hash = hash_details(self.id, &next.details, self.origin, revision);
        Ok(Self {
            id: self.id,
            revision,
            details: next.details,
            origin: self.origin,
            verification: VerificationState::Unverified,
            content_hash,
            evidence: None,
        })
    }
}

fn hash_details(
    id: DeviceSetupId,
    details: &SetupDetails,
    origin: SetupOrigin,
    revision: u32,
) -> ContentHash {
    let mut bytes = b"veylora-device-setup-v1\0".to_vec();
    bytes.extend_from_slice(&revision.to_le_bytes());
    bytes.push(origin as u8);
    let text = |out: &mut Vec<u8>, value: &str| {
        out.extend_from_slice(&(value.len() as u64).to_le_bytes());
        out.extend_from_slice(value.as_bytes());
    };
    let optional = |out: &mut Vec<u8>, value: Option<&str>| match value {
        Some(value) => {
            out.push(1);
            text(out, value);
        }
        None => out.push(0),
    };
    text(&mut bytes, &id.to_string());
    text(&mut bytes, &details.identity.model);
    optional(&mut bytes, details.identity.serial_or_local_id.as_deref());
    optional(&mut bytes, details.firmware.as_deref());
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
    optional(&mut bytes, details.installation_date.as_deref());
    bytes.extend_from_slice(&(details.physical_slots.len() as u64).to_le_bytes());
    for slot in &details.physical_slots {
        bytes.push(slot.id.0);
        text(&mut bytes, &slot.tank_label);
        text(&mut bytes, &slot.head_label);
        text(&mut bytes, &slot.ink);
    }
    details.channel_map.hash_bytes(&mut bytes);
    ContentHash::digest(&bytes)
}
