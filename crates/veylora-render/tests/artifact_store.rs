mod common;
use common::TempDir;
use veylora_domain::provenance::ContentHash;
use veylora_render::{ArtifactError, ArtifactStore, DirectoryStore};

#[test]
fn names_must_be_a_single_safe_path_component() {
    let dir = TempDir::new();
    let mut store = DirectoryStore::new(dir.path(), None);
    for bad in [
        "", ".", "..", "../x", "a/b", r"a\b", ".hidden", "sp ace", "c:x", "x\0y",
    ] {
        assert!(
            matches!(store.create(bad), Err(ArtifactError::InvalidName)),
            "{bad:?}"
        );
    }
    assert!(matches!(
        store.create(&"a".repeat(129)),
        Err(ArtifactError::InvalidName)
    ));
    assert!(store.create("ok-name_1.f32").is_ok());
    assert_eq!(dir.files(), vec!["ok-name_1.f32".to_string()]);
}
#[test]
fn files_are_hashed_as_written_and_never_overwritten() {
    let dir = TempDir::new();
    let mut store = DirectoryStore::new(dir.path(), None);
    let mut sink = store.create("a.bin").unwrap();
    sink.write(b"hello ").unwrap();
    sink.write(b"world").unwrap();
    let file = sink.finish().unwrap();
    assert_eq!(
        (file.bytes, file.hash),
        (11, ContentHash::digest(b"hello world"))
    );
    assert_eq!(dir.read("a.bin"), b"hello world");
    assert!(matches!(store.create("a.bin"), Err(ArtifactError::Exists)));
    assert_eq!(dir.read("a.bin"), b"hello world");
}
#[test]
fn quota_counts_every_byte_and_rollback_frees_it() {
    let dir = TempDir::new();
    let mut store = DirectoryStore::new(dir.path(), Some(10));
    assert_eq!(store.remaining_bytes(), Some(10));
    let mut sink = store.create("a.bin").unwrap();
    sink.write(b"123456").unwrap();
    assert_eq!(store.remaining_bytes(), Some(4));
    // A write that does not fit is refused whole and does not count.
    assert!(matches!(sink.write(b"12345"), Err(ArtifactError::NoSpace)));
    assert_eq!(store.remaining_bytes(), Some(4));
    sink.write(b"1234").unwrap();
    drop(sink);
    store.rollback();
    assert!(dir.files().is_empty());
    assert_eq!(store.remaining_bytes(), Some(10));
    assert_eq!(
        DirectoryStore::new(dir.path(), None).remaining_bytes(),
        None
    );
}
