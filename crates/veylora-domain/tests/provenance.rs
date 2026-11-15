use veylora_domain::provenance::{ContentHash, Provenance};

#[test]
fn sha256_matches_known_vectors_and_roundtrips_canonically() {
    for (input, expected) in [
        (
            "",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            "abc",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
    ] {
        let hash = ContentHash::digest(input.as_bytes());
        assert_eq!(hash.to_string(), expected);
        assert_eq!(expected.parse::<ContentHash>().unwrap(), hash);
        let json = serde_json::to_string(&hash).unwrap();
        assert_eq!(json, format!("\"{expected}\""));
        assert_eq!(serde_json::from_str::<ContentHash>(&json).unwrap(), hash);
    }
    assert_ne!(ContentHash::digest(b"abc"), ContentHash::digest(b"abd"));
}

#[test]
fn hash_parsing_rejects_noncanonical_values() {
    for value in [
        String::new(),
        "a".repeat(63),
        "a".repeat(65),
        "g".repeat(64),
        "A".repeat(64),
    ] {
        assert!(value.parse::<ContentHash>().is_err());
        assert!(
            serde_json::from_str::<ContentHash>(&serde_json::to_string(&value).unwrap()).is_err()
        );
    }
}

#[test]
fn provenance_serializes_and_hashes_stably() {
    let provenance = Provenance {
        software_version: "0.0.0".into(),
        backend_version: "native-test/1".into(),
        content_hashes: vec![("artwork".into(), ContentHash::digest(b"artwork"))],
    };
    let json = serde_json::to_string(&provenance).unwrap();
    assert_eq!(
        serde_json::from_str::<Provenance>(&json).unwrap(),
        provenance
    );
    assert_eq!(provenance.stable_hash(), provenance.stable_hash());
    assert_ne!(
        provenance.stable_hash(),
        (Provenance {
            software_version: "0.0.1".into(),
            ..provenance.clone()
        })
        .stable_hash()
    );
    assert_ne!(
        provenance.stable_hash(),
        (Provenance {
            content_hashes: vec![("artwork".into(), ContentHash::digest(b"changed"))],
            ..provenance.clone()
        })
        .stable_hash()
    );
}

#[test]
fn provenance_hash_distinguishes_entry_boundaries_and_order() {
    let first = Provenance {
        software_version: "s".into(),
        backend_version: "b".into(),
        content_hashes: vec![
            ("a".into(), ContentHash::digest(b"one")),
            ("b".into(), ContentHash::digest(b"two")),
        ],
    };
    let second = Provenance {
        content_hashes: vec![
            ("b".into(), ContentHash::digest(b"two")),
            ("a".into(), ContentHash::digest(b"one")),
        ],
        ..first.clone()
    };
    assert_ne!(first.stable_hash(), second.stable_hash());
}

#[test]
fn incremental_hashing_matches_one_shot_digest_for_any_chunking() {
    use veylora_domain::provenance::ContentHasher;
    let data: Vec<u8> = (0..1000u32).map(|i| (i * 31 % 251) as u8).collect();
    let expected = ContentHash::digest(&data);
    for chunk in [1, 2, 7, 64, 999, 1000, 4096] {
        let mut hasher = ContentHasher::new();
        for part in data.chunks(chunk) {
            hasher.update(part);
        }
        assert_eq!(hasher.finalize(), expected, "chunk {chunk}");
    }
    assert_eq!(ContentHasher::new().finalize(), ContentHash::digest(b""));
}
