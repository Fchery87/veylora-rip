use veylora_domain::provenance::ContentHash;
use veylora_persistence::{FileAssetStore, PersistenceError};

fn dir(label: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("veylora-assets-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    p
}

#[test]
fn stores_by_content_hash_once_and_reads_back_verified() {
    let root = dir("basic");
    let s = FileAssetStore::open(&root).unwrap();
    let h = s.put(b"png bytes").unwrap();
    assert_eq!(h, ContentHash::digest(b"png bytes"));
    assert_eq!(s.put(b"png bytes").unwrap(), h);
    assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
    assert_eq!(
        FileAssetStore::open(&root).unwrap().get(&h).unwrap(),
        b"png bytes"
    );
}

#[test]
fn missing_and_altered_assets_are_reported() {
    let root = dir("bad");
    let s = FileAssetStore::open(&root).unwrap();
    assert!(matches!(
        s.get(&ContentHash::digest(b"x")),
        Err(PersistenceError::NotFound { .. })
    ));
    let h = s.put(b"original").unwrap();
    std::fs::write(root.join(h.to_string()), b"altered").unwrap();
    assert!(matches!(s.get(&h), Err(PersistenceError::Corrupt { .. })));
    assert!(matches!(
        s.put(b"original"),
        Err(PersistenceError::Corrupt { .. })
    ));
}
