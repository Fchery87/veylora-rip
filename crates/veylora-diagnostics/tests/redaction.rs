use veylora_diagnostics::bundle::{read_tar, write_tar, BundleEntry, BundleError};
use veylora_diagnostics::redact::Redactor;

fn entry(path: &str, bytes: &[u8]) -> BundleEntry {
    BundleEntry {
        path: path.to_string(),
        bytes: bytes.to_vec(),
    }
}

#[test]
fn known_roots_are_replaced_in_both_separator_styles() {
    let r = Redactor::new(&["C:\\Work\\ws".to_string()]);
    assert_eq!(
        r.redact("open C:\\Work\\ws\\jobs\\a.json failed"),
        "open <workspace>\\jobs\\a.json failed"
    );
    assert_eq!(
        r.redact("open C:/Work/ws/jobs/a.json failed"),
        "open <workspace>/jobs/a.json failed"
    );
}

#[test]
fn user_profile_names_are_removed_wherever_they_appear() {
    let r = Redactor::new(&[]);
    assert_eq!(
        r.redact("at C:\\Users\\Frantz\\Documents\\x.png: denied"),
        "at C:\\Users\\<user>\\Documents\\x.png: denied"
    );
    assert_eq!(
        r.redact("at c:/users/frantz/x.png"),
        "at c:/users/<user>/x.png"
    );
    assert_eq!(r.redact("see /home/frantz/x"), "see /home/<user>/x");
    assert_eq!(r.redact("see /Users/frantz/x"), "see /Users/<user>/x");
}

#[test]
fn text_without_paths_is_unchanged_and_longer_roots_win() {
    let r = Redactor::new(&["C:\\a".to_string(), "C:\\a\\b".to_string()]);
    assert_eq!(r.redact("nothing here"), "nothing here");
    assert_eq!(r.redact("C:\\a\\b\\c"), "<workspace>\\c");
}

#[test]
fn empty_roots_are_ignored() {
    let r = Redactor::new(&[String::new()]);
    assert_eq!(r.redact("abc"), "abc");
}

#[test]
fn tar_round_trips_and_is_deterministic() {
    let entries = vec![entry("b/two.json", b"{}"), entry("a.txt", b"hello")];
    let one = write_tar(&entries).unwrap();
    let two = write_tar(&entries).unwrap();
    assert_eq!(one, two);
    let back = read_tar(&one).unwrap();
    // Entries come back sorted by path regardless of input order.
    assert_eq!(
        back.iter().map(|e| e.path.as_str()).collect::<Vec<_>>(),
        ["a.txt", "b/two.json"]
    );
    assert_eq!(back[0].bytes, b"hello");
    assert_eq!(back[1].bytes, b"{}");
    assert_eq!(one.len() % 512, 0);
}

#[test]
fn unsafe_entry_paths_are_refused_when_writing() {
    for bad in [
        "../x", "a/../x", "/abs", "\\abs", "C:\\x", "C:x", "a\\b", "", "a//b", "./x", "a/",
        "con\0", "CON", "nul.txt", "a/com1", "lpt9.log", "a./b", "a /b", "..", ".", "a/..", "a?b",
    ] {
        assert!(
            matches!(
                write_tar(&[entry(bad, b"x")]),
                Err(BundleError::UnsafePath(_))
            ),
            "{bad:?}"
        );
    }
}

#[test]
fn duplicate_paths_are_refused() {
    let err = write_tar(&[entry("a", b"1"), entry("a", b"2")]).unwrap_err();
    assert!(matches!(err, BundleError::Duplicate(_)));
}

#[test]
fn long_paths_are_refused_not_truncated() {
    let long = "d/".repeat(60) + "x";
    assert!(matches!(
        write_tar(&[entry(&long, b"x")]),
        Err(BundleError::UnsafePath(_))
    ));
}

fn raw_header(name: &str, typeflag: u8, size: usize) -> Vec<u8> {
    let mut h = vec![0u8; 512];
    h[..name.len()].copy_from_slice(name.as_bytes());
    h[100..107].copy_from_slice(b"0000644");
    h[124..135].copy_from_slice(format!("{size:011o}").as_bytes());
    h[136..147].copy_from_slice(b"00000000000");
    h[156] = typeflag;
    h[257..263].copy_from_slice(b"ustar\0");
    h[263..265].copy_from_slice(b"00");
    h[148..156].copy_from_slice(b"        ");
    let sum: u32 = h.iter().map(|b| *b as u32).sum();
    h[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
    h
}

fn archive(header: Vec<u8>, body: &[u8]) -> Vec<u8> {
    let mut out = header;
    out.extend_from_slice(body);
    out.resize(out.len().div_ceil(512) * 512, 0);
    out.extend(std::iter::repeat_n(0u8, 1024));
    out
}

#[test]
fn reading_refuses_path_traversal_and_special_entries() {
    for name in ["../evil", "/etc/passwd", "a/../../b", "C:\\x", "a\\b"] {
        let tar = archive(raw_header(name, b'0', 1), b"x");
        assert!(
            matches!(read_tar(&tar), Err(BundleError::UnsafePath(_))),
            "{name}"
        );
    }
    // Symlink, hard link, directory and device entries are not regular files.
    for flag in *b"12345" {
        let tar = archive(raw_header("a", flag, 0), b"");
        assert!(
            matches!(read_tar(&tar), Err(BundleError::UnsupportedEntry(_))),
            "{flag}"
        );
    }
}

#[test]
fn reading_refuses_damage() {
    let good = write_tar(&[entry("a", b"hello")]).unwrap();
    // Bad checksum.
    let mut bad = good.clone();
    bad[0] ^= 1;
    assert!(matches!(read_tar(&bad), Err(BundleError::Malformed(_))));
    // Truncated body.
    assert!(matches!(
        read_tar(&good[..512 + 2]),
        Err(BundleError::Malformed(_))
    ));
    // Missing end-of-archive marker.
    assert!(matches!(
        read_tar(&good[..1024]),
        Err(BundleError::Malformed(_))
    ));
    // Not a multiple of the block size.
    assert!(matches!(
        read_tar(&good[..good.len() - 1]),
        Err(BundleError::Malformed(_))
    ));
}

#[test]
fn reading_refuses_oversized_declared_entries() {
    // Declares 2 GiB but carries nothing: must not allocate or loop on it.
    let mut tar = raw_header("a", b'0', 0);
    tar[124..135].copy_from_slice(b"20000000000");
    let sum_fix = {
        tar[148..156].copy_from_slice(b"        ");
        tar.iter().map(|b| *b as u32).sum::<u32>()
    };
    tar[148..156].copy_from_slice(format!("{sum_fix:06o}\0 ").as_bytes());
    tar.extend(std::iter::repeat_n(0u8, 1024));
    assert_eq!(read_tar(&tar), Err(BundleError::TooLarge));
}

#[test]
fn reading_refuses_duplicates() {
    let one = write_tar(&[entry("a", b"1")]).unwrap();
    let mut two = one[..512 + 512].to_vec();
    two.extend_from_slice(&one);
    assert!(matches!(read_tar(&two), Err(BundleError::Duplicate(_))));
}

#[test]
fn ordinary_names_that_resemble_reserved_ones_are_accepted() {
    for ok in [
        "console.json",
        "com10",
        "lpt0",
        "a b/c.d",
        "render-manifests/x.json",
    ] {
        write_tar(&[entry(ok, b"x")]).unwrap_or_else(|e| panic!("{ok}: {e}"));
    }
}

#[test]
fn names_that_differ_only_by_case_collide() {
    assert!(matches!(
        write_tar(&[entry("A", b"1"), entry("a", b"2")]),
        Err(BundleError::Duplicate(_))
    ));
    let one = write_tar(&[entry("A", b"1")]).unwrap();
    let two = write_tar(&[entry("a", b"2")]).unwrap();
    let mut both = one[..1024].to_vec();
    both.extend_from_slice(&two);
    assert!(matches!(read_tar(&both), Err(BundleError::Duplicate(_))));
}

#[test]
fn data_after_the_end_marker_is_refused() {
    let mut tar = write_tar(&[entry("a", b"1")]).unwrap();
    tar.extend(std::iter::repeat_n(0u8, 512));
    assert!(matches!(read_tar(&tar), Err(BundleError::Malformed(_))));
}

#[test]
fn a_ustar_prefix_field_is_refused_because_it_could_hide_a_path() {
    let mut h = raw_header("a", b'0', 0);
    h[345..350].copy_from_slice(b"../..");
    h[148..156].copy_from_slice(b"        ");
    let sum: u32 = h.iter().map(|b| *b as u32).sum();
    h[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
    let tar = archive(h, b"");
    assert!(matches!(read_tar(&tar), Err(BundleError::Malformed(_))));
}

#[test]
fn redaction_is_idempotent_and_tolerates_edges() {
    let r = Redactor::new(&["C:\\Work\\ws\\".to_string()]);
    let once = r.redact("C:\\Work\\ws\\x and C:\\Users\\frantz\\y and /home/");
    assert_eq!(once, "<workspace>\\x and C:\\Users\\<user>\\y and /home/");
    assert_eq!(r.redact(&once), once);
}
