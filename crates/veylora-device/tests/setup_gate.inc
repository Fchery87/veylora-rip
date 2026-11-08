use veylora_device::setup::{DeviceIdentity, DeviceSetup, PhysicalSlot, SetupDetails, SetupOrigin, VerificationState};
use veylora_domain::{errors::ErrorCode, ids::DeviceSetupId};

fn details() -> SetupDetails {
    SetupDetails {
        identity: DeviceIdentity { model: "synthetic test device".into(), serial_or_local_id: Some("fixture-only".into()) },
        firmware: None,
        connection: "fixture connection".into(), operating_system: "fixture OS".into(),
        conversion_supplier: "fixture supplier".into(), ink: "fixture ink".into(), film: "fixture film".into(),
        feed_path: "fixture feed".into(), installation_date: None, installation_notes: "synthetic, not physical evidence".into(),
        reference_settings: "fixture settings".into(),
        physical_slots: (0..6).map(|id| PhysicalSlot { id: PhysicalSlotId(id), tank_label: format!("test tank {id}"), head_label: format!("test head {id}"), ink: "fixture ink".into() }).collect(),
        channel_map: LogicalChannelMap::new(1, bindings()).unwrap(),
    }
}

#[test]
fn new_imported_and_inferred_setups_cannot_pass_production_gate() {
    for origin in [SetupOrigin::Recorded, SetupOrigin::Imported, SetupOrigin::Inferred] {
        let setup = DeviceSetup::new(DeviceSetupId::new(), details(), origin).unwrap();
        assert_eq!(setup.verification(), &VerificationState::Unverified);
        assert_eq!(setup.require_production().unwrap_err().code, ErrorCode::UnverifiedDeviceSetup);
    }
}
