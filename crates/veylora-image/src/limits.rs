//! Resource budgets checked before decoding or allocating raster/profile storage.
#[derive(Debug, Clone, Copy)]
pub struct DecodeLimits {
    pub max_file_bytes: u64,
    pub max_dimension: u32,
    pub max_pixels: u64,
    pub max_decoded_bytes: u64,
    pub max_profile_bytes: u64,
    /// Sum of all non-IDAT chunk payloads and framing, including unknown chunks.
    pub max_metadata_bytes: u64,
    /// Separate budget passed to the PNG decoder for its intermediate buffers.
    pub max_decoder_bytes: usize,
}
impl Default for DecodeLimits {
    fn default() -> Self {
        Self {
            max_file_bytes: 64 * 1024 * 1024,
            max_dimension: 16_384,
            max_pixels: 64 * 1024 * 1024,
            max_decoded_bytes: 256 * 1024 * 1024,
            max_profile_bytes: 4 * 1024 * 1024,
            max_metadata_bytes: 8 * 1024 * 1024,
            max_decoder_bytes: 16 * 1024 * 1024,
        }
    }
}
