use std::collections::HashSet;
use veylora_domain::ids::{AssetId, AttemptId, DeviceSetupId, JobId, PrintModeId, RecipeId};

#[test]
fn typed_ids_generate_unique_uuids_and_roundtrip_through_public_formats() {
    macro_rules! check_id {
        ($id:ty) => {{
            let ids: HashSet<_> = (0..128).map(|_| <$id>::new()).collect();
            assert_eq!(ids.len(), 128);
            for id in ids {
                let text = id.to_string();
                assert_eq!(text.len(), 36);
                assert_eq!(text.as_bytes()[14], b'4');
                assert_eq!(text.parse::<$id>().unwrap(), id);
                let json = serde_json::to_string(&id).unwrap();
                assert_eq!(json, format!("\"{text}\""));
                assert_eq!(serde_json::from_str::<$id>(&json).unwrap(), id);
            }
            assert!("not-a-uuid".parse::<$id>().is_err());
            assert!(serde_json::from_str::<$id>("\"not-a-uuid\"").is_err());
        }};
    }
    check_id!(JobId);
    check_id!(AssetId);
    check_id!(DeviceSetupId);
    check_id!(PrintModeId);
    check_id!(RecipeId);
    check_id!(AttemptId);
}
