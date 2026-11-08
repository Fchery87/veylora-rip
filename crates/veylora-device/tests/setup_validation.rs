use veylora_device::channels::{
    ChannelBinding, LogicalChannel, LogicalChannelMap, PhysicalSlotId, ProtocolChannel,
};
use veylora_device::setup::{ChannelEvidence, EvidenceRecord};
include!("setup_gate.inc");

#[test]
fn changed_mapping_requires_new_map_version() {
    let setup = DeviceSetup::new(DeviceSetupId::new(), details(), SetupOrigin::Recorded).unwrap();
    let mut d = details();
    let mut b = bindings();
    b[4].fraction = 0.25;
    d.channel_map = LogicalChannelMap::new(1, b.clone()).unwrap();
    assert!(setup.revise(d.clone()).is_err());
    d.channel_map = LogicalChannelMap::new(2, b).unwrap();
    assert!(setup.revise(d).is_ok());
}

#[test]
fn optional_empty_values_are_not_unknown_values() {
    for field in 0..3 {
        let mut absent = details();
        absent.identity.serial_or_local_id = None;
        let mut present = absent.clone();
        match field {
            0 => present.identity.serial_or_local_id = Some(String::new()),
            1 => present.firmware = Some(String::new()),
            _ => present.installation_date = Some(String::new()),
        }
        let id = DeviceSetupId::new();
        assert_ne!(
            DeviceSetup::new(id, absent, SetupOrigin::Recorded)
                .unwrap()
                .content_hash(),
            DeviceSetup::new(id, present, SetupOrigin::Recorded)
                .unwrap()
                .content_hash()
        );
    }
}

#[test]
fn evidence_rejects_stale_duplicate_unlabeled_and_missing_provenance() {
    let setup = DeviceSetup::new(DeviceSetupId::new(), details(), SetupOrigin::Recorded).unwrap();
    for change in 0..10 {
        let mut record = evidence_for(&setup);
        match change {
            0 => record.setup_id = DeviceSetupId::new(),
            1 => record.revision += 1,
            2 => record.setup_hash = veylora_domain::provenance::ContentHash::digest(b"wrong"),
            3 => record.map_version += 1,
            4 => record.channels[1] = record.channels[0].clone(),
            5 => record.target_id.clear(),
            6 => record.observed_at.clear(),
            7 => record.operator.clear(),
            8 => record.channels[0].target_label.clear(),
            _ => record.channels[0].protocol = ProtocolChannel(99),
        }
        let mut candidate = setup.clone();
        assert!(candidate.verify(record).is_err(), "change {change}");
        assert!(candidate.require_production().is_err());
    }
}

#[test]
fn configuration_hash_covers_every_recorded_field() {
    let id = DeviceSetupId::new();
    let base = DeviceSetup::new(id, details(), SetupOrigin::Recorded).unwrap();
    assert_eq!(
        base.content_hash(),
        DeviceSetup::new(id, details(), SetupOrigin::Recorded)
            .unwrap()
            .content_hash()
    );
    for change in 0..17 {
        let mut d = details();
        match change {
            0 => d.identity.model.push('x'),
            1 => d.identity.serial_or_local_id = None,
            2 => d.firmware = Some("observed".into()),
            3 => d.connection.push('x'),
            4 => d.operating_system.push('x'),
            5 => d.conversion_supplier.push('x'),
            6 => d.ink.push('x'),
            7 => d.film.push('x'),
            8 => d.feed_path.push('x'),
            9 => d.installation_date = Some("unknown".into()),
            10 => d.installation_notes.push('x'),
            11 => d.reference_settings.push('x'),
            12 => d.physical_slots[0].tank_label.push('x'),
            13 => d.physical_slots[0].head_label.push('x'),
            14 => d.physical_slots[0].ink.push('x'),
            15 => d.channel_map = LogicalChannelMap::new(2, bindings()).unwrap(),
            _ => {
                let mut b = bindings();
                b[4].fraction = 0.25;
                d.channel_map = LogicalChannelMap::new(1, b).unwrap();
            }
        }
        assert_ne!(
            base.content_hash(),
            DeviceSetup::new(id, d, SetupOrigin::Recorded)
                .unwrap()
                .content_hash(),
            "field {change}"
        );
    }
}

#[test]
fn physical_inventory_must_match_map_exactly() {
    for change in 0..4 {
        let mut d = details();
        match change {
            0 => {
                d.physical_slots.pop();
            }
            1 => d.physical_slots[1].id = d.physical_slots[0].id,
            2 => d.physical_slots[0].id = PhysicalSlotId(99),
            _ => d.identity.model.clear(),
        }
        assert!(DeviceSetup::new(DeviceSetupId::new(), d, SetupOrigin::Recorded).is_err());
    }
}

#[test]
fn only_complete_current_matched_evidence_can_verify() {
    let mut setup =
        DeviceSetup::new(DeviceSetupId::new(), details(), SetupOrigin::Recorded).unwrap();
    let base = evidence_for(&setup);
    assert!(setup.verify(base.clone()).is_ok());
    assert!(matches!(
        setup.verification(),
        VerificationState::Verified { .. }
    ));
    let mut incomplete = base.clone();
    incomplete.channels.pop();
    assert!(setup.verify(incomplete).is_err());
    let mut wrong = base;
    wrong.channels[0].observed_logical = LogicalChannel::White;
    assert!(setup.verify(wrong).is_err());
}

#[test]
fn revision_change_resets_verification_and_changes_hash() {
    let mut setup =
        DeviceSetup::new(DeviceSetupId::new(), details(), SetupOrigin::Recorded).unwrap();
    let old_hash = setup.content_hash();
    setup.verify(evidence_for(&setup)).unwrap();
    let revised = setup.revise(details()).unwrap();
    assert_eq!(setup.revision(), 1);
    assert_eq!(setup.content_hash(), old_hash);
    assert!(setup.require_production().is_ok());
    assert_eq!(revised.revision(), 2);
    assert_ne!(revised.content_hash(), old_hash);
    assert!(matches!(
        revised.verification(),
        VerificationState::Unverified
    ));
}

// Synthetic test data only. Never an observation of real hardware.
fn evidence_for(setup: &DeviceSetup) -> EvidenceRecord {
    let channels = setup
        .details()
        .channel_map
        .bindings()
        .iter()
        .map(|b| ChannelEvidence {
            target_label: format!("synthetic patch {}", b.slot.0),
            slot: b.slot,
            protocol: b.protocol,
            expected_logical: b.logical,
            observed_logical: b.logical,
        })
        .collect();
    EvidenceRecord {
        operator: "synthetic test observer".into(),
        setup_id: setup.id(),
        revision: setup.revision(),
        setup_hash: setup.content_hash(),
        map_version: setup.details().channel_map.version(),
        channels,
        target_id: "operator-labeled-channel-target-001".into(),
        observed_at: "2026-11-08T19:20:00Z".into(),
        artifact_hash: veylora_domain::provenance::ContentHash::digest(b"physical-photo-artifact"),
        evidence_hash: veylora_domain::provenance::ContentHash::digest(b"operator-evidence-record"),
    }
}

fn bindings() -> Vec<ChannelBinding> {
    [
        LogicalChannel::Cyan,
        LogicalChannel::Magenta,
        LogicalChannel::Yellow,
        LogicalChannel::Black,
        LogicalChannel::White,
        LogicalChannel::White,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, logical)| ChannelBinding {
        logical,
        slot: PhysicalSlotId(index as u8),
        protocol: ProtocolChannel(index as u8),
        fraction: if logical == LogicalChannel::White {
            0.5
        } else {
            1.0
        },
    })
    .collect()
}

#[test]
fn duplicate_white_slots_share_logical_coverage_without_duplication() {
    let map = LogicalChannelMap::new(1, bindings()).unwrap();
    for (input, expected) in [(0.25, 0.125), (0.5, 0.25), (0.75, 0.375), (1.0, 0.5)] {
        let allocated = map.allocate(LogicalChannel::White, input).unwrap();
        assert_eq!(
            allocated,
            vec![(PhysicalSlotId(4), expected), (PhysicalSlotId(5), expected)]
        );
    }
}

#[test]
fn maps_reject_duplicates_missing_channels_and_invalid_allocations() {
    assert!(LogicalChannelMap::new(0, bindings()).is_err());
    for mutation in 0..9 {
        let mut b = bindings();
        match mutation {
            0 => b[1].slot = b[0].slot,
            1 => b[1].protocol = b[0].protocol,
            2 => {
                b.remove(0);
            }
            3 => b[4].fraction = f64::NAN,
            4 => b[4].fraction = f64::INFINITY,
            5 => b[4].fraction = -0.1,
            6 => b[4].fraction = 0.0,
            7 => b[4].fraction = 1.0,
            _ => b[0].fraction = 1.01,
        }
        assert!(LogicalChannelMap::new(1, b).is_err(), "mutation {mutation}");
    }
}

#[test]
fn allocation_rejects_invalid_coverage_without_clamping() {
    let map = LogicalChannelMap::new(1, bindings()).unwrap();
    for value in [f64::NAN, f64::INFINITY, -0.01, 1.01] {
        assert!(map.allocate(LogicalChannel::White, value).is_err());
    }
}

#[test]
fn experimental_requires_reason_and_never_passes_production() {
    let mut setup =
        DeviceSetup::new(DeviceSetupId::new(), details(), SetupOrigin::Recorded).unwrap();
    assert!(setup.mark_experimental(" ").is_err());
    setup
        .mark_experimental("native channel behavior is not physically established")
        .unwrap();
    assert!(matches!(
        setup.verification(),
        VerificationState::Experimental { .. }
    ));
    assert!(setup.require_production().is_err());
}
