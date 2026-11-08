use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodeLimits {
    pub max_pixels: u64,
    pub max_profile_bytes: u64,
    pub max_dimension: u32,
}

impl Default for DecodeLimits {
    fn default() -> Self {
        Self {
            max_pixels: 100_000_000,
            max_profile_bytes: 16 * 1024 * 1024,
            max_dimension: 32_768,
        }
    }
}
