use veylora_device::channels::{
    ChannelBinding, LogicalChannel, LogicalChannelMap, PhysicalSlotId, ProtocolChannel,
};
use veylora_device::mode::{
    DepositionSpec, MediaSpec, ModeRecord, ModeStatus, PrintMode, ProtocolSpec, QualitySpec,
    RasterSpec,
};
use veylora_device::setup::{
    ChannelEvidence, DeviceIdentity, DeviceSetup, EvidenceRecord, PhysicalSlot, SetupDetails,
    SetupOrigin, SetupRecord, VerificationState,
};
use veylora_domain::{
    ids::{DeviceSetupId, PrintModeId},
    provenance::ContentHash,
};

fn bindings() -> Vec<ChannelBinding> {
    use LogicalChannel::*;
    [Cyan, Magenta, Yellow, Black, White, White]
        .into_iter()
        .enumerate()
        .map(|(i, logical)| ChannelBinding {
            logical,
            slot: PhysicalSlotId(i as u8),
            protocol: ProtocolChannel(10 + i as u8),
            fraction: if logical == White { 0.5 } else { 1.0 },
        })
        .collect()
}

fn details() -> SetupDetails {
    SetupDetails {
        identity: DeviceIdentity {
            model: "synthetic".into(),
            serial_or_local_id: None,
        },
        firmware: Some("fw".into()),
        connection: "fixture".into(),
        operating_system: "fixture".into(),
        conversion_supplier: "fixture".into(),
        ink: "fixture".into(),
        film: "fixture".into(),
        feed_path: "feed".into(),
        installation_date: None,
        installation_notes: String::new(),
        reference_settings: "fixture".into(),
        physical_slots: (0..6)
            .map(|id| PhysicalSlot {
                id: PhysicalSlotId(id),
                tank_label: format!("t{id}"),
                head_label: format!("h{id}"),
                ink: "fixture".into(),
            })
            .collect(),
        channel_map: LogicalChannelMap::new(1, bindings()).unwrap(),
    }
}

fn evidence(setup: &DeviceSetup) -> EvidenceRecord {
    EvidenceRecord {
        operator: "op".into(),
        setup_id: setup.id(),
        revision: setup.revision(),
        setup_hash: setup.content_hash(),
        map_version: 1,
        channels: bindings()
            .iter()
            .map(|b| ChannelEvidence {
                target_label: format!("L{}", b.slot.0),
                slot: b.slot,
                protocol: b.protocol,
                expected_logical: b.logical,
                observed_logical: b.logical,
            })
            .collect(),
        target_id: "target".into(),
        observed_at: "2026-11-15T00:00:00Z".into(),
        artifact_hash: ContentHash::digest(b"artifact"),
        evidence_hash: ContentHash::digest(b"evidence"),
    }
}

type SetupEdit = Box<dyn Fn(&mut SetupRecord)>;
type ModeEdit = Box<dyn Fn(&mut ModeRecord)>;

fn json<T: serde::Serialize + serde::de::DeserializeOwned>(v: &T) -> T {
    serde_json::from_str(&serde_json::to_string(v).unwrap()).unwrap()
}

#[test]
fn an_unverified_setup_round_trips_through_json() {
    let s = DeviceSetup::new(DeviceSetupId::new(), details(), SetupOrigin::Recorded).unwrap();
    let back = DeviceSetup::from_record(json(&s.to_record())).unwrap();
    assert_eq!(back.content_hash(), s.content_hash());
    assert_eq!(back.details(), s.details());
    assert_eq!(back.verification(), &VerificationState::Unverified);
}

#[test]
fn experimental_and_verified_states_round_trip_and_replay_their_evidence() {
    let mut s = DeviceSetup::new(DeviceSetupId::new(), details(), SetupOrigin::Imported).unwrap();
    s.mark_experimental("bench trial").unwrap();
    let back = DeviceSetup::from_record(json(&s.to_record())).unwrap();
    assert!(
        matches!(back.verification(), VerificationState::Experimental { reason } if reason == "bench trial")
    );

    let mut v = DeviceSetup::new(DeviceSetupId::new(), details(), SetupOrigin::Recorded).unwrap();
    let e = evidence(&v);
    v.verify(e.clone()).unwrap();
    let back = DeviceSetup::from_record(json(&v.to_record())).unwrap();
    assert!(back.require_production().is_ok());
    assert_eq!(back.evidence(), Some(&e));
}

#[test]
fn revised_setups_keep_their_revision_and_hash() {
    let s = DeviceSetup::new(DeviceSetupId::new(), details(), SetupOrigin::Recorded).unwrap();
    let mut d = details();
    d.firmware = Some("fw2".into());
    let r = s.revise(d).unwrap();
    assert_eq!(r.revision(), 2);
    let back = DeviceSetup::from_record(json(&r.to_record())).unwrap();
    assert_eq!(
        (back.revision(), back.content_hash()),
        (2, r.content_hash())
    );
}

#[test]
fn tampered_setup_records_are_refused() {
    let s = DeviceSetup::new(DeviceSetupId::new(), details(), SetupOrigin::Recorded).unwrap();
    let base = s.to_record();
    let cases: Vec<(&str, SetupEdit)> = vec![
        (
            "firmware changed without rehash",
            Box::new(|r| r.details.firmware = Some("other".into())),
        ),
        ("revision changed", Box::new(|r| r.revision = 7)),
        ("schema", Box::new(|r| r.schema_version = 99)),
        (
            "hash",
            Box::new(|r| r.content_hash = ContentHash::digest(b"x")),
        ),
        (
            "verified without evidence",
            Box::new(|r| {
                r.verification = VerificationState::Verified {
                    evidence_hash: ContentHash::digest(b"e"),
                }
            }),
        ),
        (
            "empty model",
            Box::new(|r| r.details.identity.model = " ".into()),
        ),
    ];
    for (name, edit) in cases {
        let mut r = base.clone();
        edit(&mut r);
        assert!(DeviceSetup::from_record(r).is_err(), "{name}");
    }
    // Evidence that no longer matches the setup is refused too.
    let mut v = DeviceSetup::new(DeviceSetupId::new(), details(), SetupOrigin::Recorded).unwrap();
    v.verify(evidence(&v)).unwrap();
    let mut r = v.to_record();
    r.evidence.as_mut().unwrap().channels[0].observed_logical = LogicalChannel::Black;
    assert!(DeviceSetup::from_record(r).is_err());
    let mut r = v.to_record();
    r.evidence.as_mut().unwrap().evidence_hash = ContentHash::digest(b"changed");
    assert!(DeviceSetup::from_record(r).is_err());
}

#[test]
fn json_with_an_invalid_channel_map_does_not_load() {
    let s = DeviceSetup::new(DeviceSetupId::new(), details(), SetupOrigin::Recorded).unwrap();
    let mut value = serde_json::to_value(s.to_record()).unwrap();
    value["details"]["channel_map"]["bindings"][0]["fraction"] = serde_json::json!(0.5);
    assert!(serde_json::from_value::<SetupRecord>(value).is_err());
    let mut value = serde_json::to_value(s.to_record()).unwrap();
    value["details"]["surprise"] = serde_json::json!(1);
    assert!(serde_json::from_value::<SetupRecord>(value).is_err());
}

fn mode(setup: DeviceSetupId) -> PrintMode {
    PrintMode::new(
        PrintModeId::new(),
        setup,
        RasterSpec {
            dpi_x: 720,
            dpi_y: 720,
            printable_width_px: 2400,
            printable_height_px: 3600,
            band_height_px: 128,
        },
        ProtocolSpec {
            label: "p".into(),
            channel_sequence: vec![0, 1, 2],
            firmware_fingerprint: "fw".into(),
        },
        MediaSpec {
            media_label: "m".into(),
            feed_path: "feed".into(),
            film_fingerprint: "f".into(),
        },
        DepositionSpec {
            passes: 4,
            overlap_percent: 50,
            coverage_caps: vec![("cyan".into(), 0.9)],
        },
        QualitySpec {
            target_label: "t".into(),
            notes: String::new(),
            tolerance_percent: 2.5,
        },
    )
    .unwrap()
}

#[test]
fn modes_round_trip_in_every_status_with_stable_hashes() {
    let m = mode(DeviceSetupId::new());
    let back = PrintMode::from_record(json(&m.to_record())).unwrap();
    assert_eq!(
        (back.content_hash(), back.status()),
        (m.content_hash(), ModeStatus::Experimental)
    );
    let mut v = m.clone();
    v.mark_validated().unwrap();
    let back = PrintMode::from_record(json(&v.to_record())).unwrap();
    assert_eq!(
        (back.status(), back.content_hash()),
        (ModeStatus::Validated, v.content_hash())
    );
    assert_ne!(v.content_hash(), m.content_hash());
    let revised = m
        .revise(
            m.raster().clone(),
            m.protocol().clone(),
            m.media().clone(),
            m.deposition().clone(),
            QualitySpec {
                tolerance_percent: 3.0,
                ..m.quality().clone()
            },
        )
        .unwrap();
    assert_eq!(
        PrintMode::from_record(json(&revised.to_record()))
            .unwrap()
            .revision(),
        2
    );
}

#[test]
fn tampered_mode_records_are_refused() {
    let base = mode(DeviceSetupId::new()).to_record();
    let cases: Vec<(&str, ModeEdit)> = vec![
        ("dpi", Box::new(|r| r.raster.dpi_x = 1440)),
        (
            "status flipped to validated",
            Box::new(|r| r.status = ModeStatus::Validated),
        ),
        (
            "hash",
            Box::new(|r| r.content_hash = ContentHash::digest(b"x")),
        ),
        ("schema", Box::new(|r| r.schema_version = 2)),
        ("invalid band", Box::new(|r| r.raster.band_height_px = 0)),
        (
            "cap out of range",
            Box::new(|r| r.deposition.coverage_caps[0].1 = 1.5),
        ),
    ];
    for (name, edit) in cases {
        let mut r = base.clone();
        edit(&mut r);
        assert!(PrintMode::from_record(r).is_err(), "{name}");
    }
}
