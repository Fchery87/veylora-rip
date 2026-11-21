//! Deterministic support archive container (POSIX ustar, regular files only).
//!
//! The writer and reader share one path policy so an archive this crate produces can be
//! extracted anywhere without writing outside the destination, and a hostile archive is
//! refused before any byte is used. Nothing here touches the network or the filesystem.

use std::collections::BTreeSet;

const BLOCK: usize = 512;
const NAME_LEN: usize = 100;
/// Largest single file the reader will accept.
pub const MAX_ENTRY_BYTES: u64 = 1 << 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleEntry {
    pub path: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleError {
    /// The path could escape the destination or is not portable.
    UnsafePath(String),
    Duplicate(String),
    /// Links, directories, devices and extended headers are never produced or accepted.
    UnsupportedEntry(String),
    TooLarge,
    Malformed(&'static str),
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsafePath(p) => write!(f, "unsafe archive path {p:?}"),
            Self::Duplicate(p) => write!(f, "duplicate archive path {p:?}"),
            Self::UnsupportedEntry(p) => write!(f, "archive entry {p:?} is not a regular file"),
            Self::TooLarge => write!(f, "archive entry is too large"),
            Self::Malformed(why) => write!(f, "malformed archive: {why}"),
        }
    }
}

impl std::error::Error for BundleError {}

const RESERVED: [&str; 4] = ["con", "prn", "aux", "nul"];

fn reserved_device(component: &str) -> bool {
    let stem = component
        .split('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if RESERVED.contains(&stem.as_str()) {
        return true;
    }
    let numbered = |prefix: &str| {
        stem.strip_prefix(prefix)
            .is_some_and(|n| matches!(n.as_bytes(), [b'1'..=b'9']))
    };
    numbered("com") || numbered("lpt")
}

/// The single path policy: relative, `/`-separated, short, portable to Windows.
pub fn check_path(path: &str) -> Result<(), BundleError> {
    let bad = || Err(BundleError::UnsafePath(path.to_string()));
    if path.is_empty() || path.len() > NAME_LEN {
        return bad();
    }
    if path
        .chars()
        .any(|c| c.is_control() || matches!(c, '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*'))
    {
        return bad();
    }
    for component in path.split('/') {
        if component.is_empty()
            || component == "."
            || component == ".."
            || component.ends_with('.')
            || component.ends_with(' ')
            || reserved_device(component)
        {
            return bad();
        }
    }
    Ok(())
}

fn octal(field: &mut [u8], value: u64) {
    let digits = field.len() - 1;
    field[..digits].copy_from_slice(format!("{value:0digits$o}").as_bytes());
}

fn header(path: &str, size: u64) -> [u8; BLOCK] {
    let mut h = [0u8; BLOCK];
    h[..path.len()].copy_from_slice(path.as_bytes());
    octal(&mut h[100..108], 0o644);
    octal(&mut h[108..116], 0);
    octal(&mut h[116..124], 0);
    octal(&mut h[124..136], size);
    octal(&mut h[136..148], 0);
    h[156] = b'0';
    h[257..263].copy_from_slice(b"ustar\0");
    h[263..265].copy_from_slice(b"00");
    h[148..156].copy_from_slice(b"        ");
    let sum: u32 = h.iter().map(|b| u32::from(*b)).sum();
    h[148..154].copy_from_slice(format!("{sum:06o}").as_bytes());
    h[154] = 0;
    h[155] = b' ';
    h
}

/// Entries are sorted by path and carry fixed metadata, so equal content gives equal bytes.
pub fn write_tar(entries: &[BundleEntry]) -> Result<Vec<u8>, BundleError> {
    let mut sorted: Vec<&BundleEntry> = entries.iter().collect();
    sorted.sort_by(|a, b| a.path.cmp(&b.path));
    let mut seen = BTreeSet::new();
    for e in &sorted {
        check_path(&e.path)?;
        if !seen.insert(e.path.to_ascii_lowercase()) {
            return Err(BundleError::Duplicate(e.path.clone()));
        }
    }
    let mut out = Vec::new();
    for e in sorted {
        out.extend_from_slice(&header(&e.path, e.bytes.len() as u64));
        out.extend_from_slice(&e.bytes);
        out.resize(out.len().div_ceil(BLOCK) * BLOCK, 0);
    }
    out.extend(std::iter::repeat_n(0u8, 2 * BLOCK));
    Ok(out)
}

fn parse_octal(field: &[u8]) -> Result<u64, BundleError> {
    let text = std::str::from_utf8(field).map_err(|_| BundleError::Malformed("number field"))?;
    let trimmed = text.trim_matches(|c| c == '\0' || c == ' ');
    if trimmed.is_empty() {
        return Ok(0);
    }
    u64::from_str_radix(trimmed, 8).map_err(|_| BundleError::Malformed("number field"))
}

pub fn read_tar(bytes: &[u8]) -> Result<Vec<BundleEntry>, BundleError> {
    if !bytes.len().is_multiple_of(BLOCK) {
        return Err(BundleError::Malformed(
            "length is not a whole number of blocks",
        ));
    }
    let mut entries = Vec::new();
    let mut seen = BTreeSet::new();
    let mut at = 0;
    loop {
        let Some(block) = bytes.get(at..at + BLOCK) else {
            return Err(BundleError::Malformed("missing end-of-archive marker"));
        };
        if block.iter().all(|b| *b == 0) {
            return match bytes.get(at + BLOCK..at + 2 * BLOCK) {
                Some(next) if next.iter().all(|b| *b == 0) && at + 2 * BLOCK == bytes.len() => {
                    Ok(entries)
                }
                _ => Err(BundleError::Malformed("bad end-of-archive marker")),
            };
        }
        let mut probe = block.to_vec();
        probe[148..156].fill(b' ');
        let sum: u64 = probe.iter().map(|b| u64::from(*b)).sum();
        if parse_octal(&block[148..156])? != sum {
            return Err(BundleError::Malformed("header checksum"));
        }
        if &block[257..262] != b"ustar" || block[345..500].iter().any(|b| *b != 0) {
            return Err(BundleError::Malformed("not a plain ustar header"));
        }
        let name_end = block[..NAME_LEN]
            .iter()
            .position(|b| *b == 0)
            .unwrap_or(NAME_LEN);
        let name = std::str::from_utf8(&block[..name_end])
            .map_err(|_| BundleError::Malformed("entry name is not UTF-8"))?
            .to_string();
        check_path(&name)?;
        if !matches!(block[156], b'0' | 0) {
            return Err(BundleError::UnsupportedEntry(name));
        }
        let size = parse_octal(&block[124..136])?;
        if size > MAX_ENTRY_BYTES {
            return Err(BundleError::TooLarge);
        }
        let size = size as usize;
        let start = at + BLOCK;
        let Some(body) = bytes.get(start..start + size) else {
            return Err(BundleError::Malformed("entry body is truncated"));
        };
        let padded = size.div_ceil(BLOCK) * BLOCK;
        if start + padded > bytes.len() {
            return Err(BundleError::Malformed("entry padding is truncated"));
        }
        if !seen.insert(name.to_ascii_lowercase()) {
            return Err(BundleError::Duplicate(name));
        }
        entries.push(BundleEntry {
            path: name,
            bytes: body.to_vec(),
        });
        at = start + padded;
    }
}
