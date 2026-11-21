use crate::error::ReleaseError;

/// `MAJOR.MINOR.PATCH`, numbers only. Pre-release and build tags are not supported, so ordering
/// is never ambiguous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u64, u64, u64);

impl Version {
    pub fn parse(text: &str) -> Result<Self, ReleaseError> {
        let bad = || ReleaseError::Version(text.to_string());
        let parts: Vec<&str> = text.split('.').collect();
        if parts.len() != 3 {
            return Err(bad());
        }
        let mut n = [0u64; 3];
        for (slot, part) in n.iter_mut().zip(parts) {
            let plain = !part.is_empty()
                && part.bytes().all(|b| b.is_ascii_digit())
                && (part == "0" || !part.starts_with('0'));
            if !plain {
                return Err(bad());
            }
            *slot = part.parse().map_err(|_| bad())?;
        }
        Ok(Self(n[0], n[1], n[2]))
    }
}
