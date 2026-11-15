use std::path::PathBuf;
use veylora_device::channels::{
    ChannelBinding, LogicalChannel, LogicalChannelMap, PhysicalSlotId, ProtocolChannel,
};
use veylora_device::mode::{
    DepositionSpec, MediaSpec, ModeStatus, PrintMode, ProtocolSpec, QualitySpec, RasterSpec,
};
use veylora_device::setup::{
    ChannelEvidence, DeviceIdentity, DeviceSetup, EvidenceRecord, PhysicalSlot, SetupDetails,
    SetupOrigin, VerificationState,
};
use veylora_domain::{
    ids::{DeviceSetupId, PrintModeId},
    provenance::ContentHash,
};
use veylora_persistence::{FileDeviceStore, PersistenceError};

fn dir(label: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let p = std::env::temp_dir().join(format!(
        "veylora-devices-{label}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&p);
    p
}

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

fn setup() -> DeviceSetup {
    DeviceSetup::new(
        DeviceSetupId::new(),
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
        },
        SetupOrigin::Recorded,
    )
    .unwrap()
}

fn evidence(s: &DeviceSetup) -> EvidenceRecord {
    EvidenceRecord {
        operator: "op".into(),
        setup_id: s.id(),
        revision: s.revision(),
        setup_hash: s.content_hash(),
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
        target_id: "t".into(),
        observed_at: "now".into(),
        artifact_hash: ContentHash::digest(b"a"),
        evidence_hash: ContentHash::digest(b"e"),
    }
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
            channel_sequence: vec![0, 1],
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
fn setups_and_modes_survive_a_restart() {
    let root = dir("restart");
    let s = setup();
    let m = mode(s.id());
    let (sh, mh) = {
        let st = FileDeviceStore::open(&root).unwrap();
        (st.put_setup(&s).unwrap(), st.put_mode(&m).unwrap())
    };
    assert_eq!((sh, mh), (s.content_hash(), m.content_hash()));
    let st = FileDeviceStore::open(&root).unwrap();
    assert_eq!(st.get_setup(&sh).unwrap().details(), s.details());
    assert_eq!(st.get_mode(&mh).unwrap().content_hash(), m.content_hash());
    assert_eq!(st.list_setups().unwrap(), vec![sh]);
    assert_eq!(st.list_modes().unwrap(), vec![mh]);
}

#[test]
fn a_verification_change_is_kept_as_a_new_state_and_the_latest_wins() {
    let root = dir("states");
    let st = FileDeviceStore::open(&root).unwrap();
    let mut s = setup();
    let h = st.put_setup(&s).unwrap();
    assert_eq!(
        st.put_setup(&s).unwrap(),
        h,
        "unchanged state is not rewritten"
    );
    s.mark_experimental("trial").unwrap();
    st.put_setup(&s).unwrap();
    let e = evidence(&s);
    s.verify(e).unwrap();
    st.put_setup(&s).unwrap();
    assert!(matches!(
        st.get_setup(&h).unwrap().verification(),
        VerificationState::Verified { .. }
    ));
    // Earlier states stay on disk as history.
    let states = std::fs::read_dir(root.join("setups").join(h.to_string()))
        .unwrap()
        .count();
    assert_eq!(states, 3);
    assert_eq!(st.setup_states(&h).unwrap().len(), 3);
    assert_eq!(st.list_setups().unwrap(), vec![h]);
}

#[test]
fn a_promoted_mode_is_a_different_record_and_the_old_one_stays_readable() {
    let st = FileDeviceStore::open(dir("promote")).unwrap();
    let m = mode(DeviceSetupId::new());
    let mut v = m.clone();
    v.mark_validated().unwrap();
    let (a, b) = (st.put_mode(&m).unwrap(), st.put_mode(&v).unwrap());
    assert_ne!(a, b);
    assert_eq!(st.get_mode(&a).unwrap().status(), ModeStatus::Experimental);
    assert_eq!(st.get_mode(&b).unwrap().status(), ModeStatus::Validated);
}

#[test]
fn missing_damaged_and_misfiled_records_are_reported() {
    let root = dir("damage");
    let st = FileDeviceStore::open(&root).unwrap();
    let h = ContentHash::digest(b"nothing");
    assert!(matches!(
        st.get_setup(&h),
        Err(PersistenceError::NotFound { .. })
    ));
    assert!(matches!(
        st.get_mode(&h),
        Err(PersistenceError::NotFound { .. })
    ));

    let m = mode(DeviceSetupId::new());
    let mh = st.put_mode(&m).unwrap();
    let path = root.join("modes").join(format!("{mh}.json"));
    std::fs::write(&path, b"{broken").unwrap();
    assert!(matches!(
        st.get_mode(&mh),
        Err(PersistenceError::Corrupt { .. })
    ));
    // A valid record filed under the wrong name is corrupt too.
    std::fs::write(
        &path,
        serde_json::to_vec(&mode(DeviceSetupId::new()).to_record()).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        st.get_mode(&mh),
        Err(PersistenceError::Corrupt { .. })
    ));
    // Writing over a damaged file is refused, not silently replaced.
    assert!(matches!(
        st.put_mode(&m),
        Err(PersistenceError::Corrupt { .. })
    ));

    let s = setup();
    let sh = st.put_setup(&s).unwrap();
    let file = root
        .join("setups")
        .join(sh.to_string())
        .join("00000001.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    value["details"]["firmware"] = serde_json::json!("edited");
    std::fs::write(&file, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(matches!(
        st.get_setup(&sh),
        Err(PersistenceError::Corrupt { .. })
    ));
}

#[test]
fn newer_schema_versions_are_refused() {
    let root = dir("version");
    FileDeviceStore::open(&root).unwrap();
    std::fs::write(root.join("VERSION"), b"9").unwrap();
    assert!(matches!(
        FileDeviceStore::open(&root),
        Err(PersistenceError::SchemaOutOfRange { found: 9, .. })
    ));
}

#[test]
fn a_setup_filed_under_another_setups_name_is_corrupt() {
    let root = dir("misfiled");
    let st = FileDeviceStore::open(&root).unwrap();
    let (a, b) = (setup(), setup());
    let (ha, hb) = (st.put_setup(&a).unwrap(), st.put_setup(&b).unwrap());
    let from = root
        .join("setups")
        .join(ha.to_string())
        .join("00000001.json");
    let to = root
        .join("setups")
        .join(hb.to_string())
        .join("00000001.json");
    std::fs::copy(from, to).unwrap();
    assert!(matches!(
        st.get_setup(&hb),
        Err(PersistenceError::Corrupt { .. })
    ));
}
