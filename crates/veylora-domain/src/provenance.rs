//! Content identity and reproducibility metadata, independent of platforms and storage.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::{fmt, str::FromStr};

/// SHA-256 of exact input bytes, with no text normalization or metadata added.
/// Text and serde representations are exactly 64 lowercase ASCII hex digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    pub fn digest(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }
}

/// The input was not a canonical SHA-256 hex string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseContentHashError;

impl fmt::Display for ParseContentHashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expected exactly 64 lowercase ASCII hexadecimal digits")
    }
}

impl std::error::Error for ParseContentHashError {}

impl FromStr for ContentHash {
    type Err = ParseContentHashError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != 64
            || !value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(ParseContentHashError);
        }
        let mut bytes = [0; 32];
        for (index, out) in bytes.iter_mut().enumerate() {
            let pair = &value.as_bytes()[index * 2..index * 2 + 2];
            let digit = |b: u8| if b <= b'9' { b - b'0' } else { b - b'a' + 10 };
            *out = digit(pair[0]) * 16 + digit(pair[1]);
        }
        Ok(Self(bytes))
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl Serialize for ContentHash {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub software_version: String,
    pub backend_version: String,
    pub content_hashes: Vec<(String, ContentHash)>,
}

impl Provenance {
    /// Hash a versioned, length-delimited representation whose ordering is significant.
    pub fn stable_hash(&self) -> ContentHash {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"veylora-provenance-v1\0");
        append_string(&mut bytes, &self.software_version);
        append_string(&mut bytes, &self.backend_version);
        bytes.extend_from_slice(&(self.content_hashes.len() as u64).to_le_bytes());
        for (label, hash) in &self.content_hashes {
            append_string(&mut bytes, label);
            bytes.extend_from_slice(&hash.0);
        }
        ContentHash::digest(&bytes)
    }
}

fn append_string(output: &mut Vec<u8>, value: &str) {
    output.extend_from_slice(&(value.len() as u64).to_le_bytes());
    output.extend_from_slice(value.as_bytes());
}
