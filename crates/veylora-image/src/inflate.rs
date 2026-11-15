//! Bounded zlib validation shared by IDAT and ICC. Never inflate into a growable buffer.
use crate::ImageError;
use flate2::{Decompress, FlushDecompress, Status};

pub(crate) fn inflate<'a>(
    chunks: impl IntoIterator<Item = &'a [u8]>,
    limit: u64,
    mut consume: impl FnMut(&[u8]) -> Result<(), ImageError>,
) -> Result<u64, ImageError> {
    let mut z = Decompress::new(true);
    let mut ended = false;
    let mut output = [0u8; 8192];
    for mut input in chunks {
        if ended && !input.is_empty() {
            return Err(ImageError::Malformed);
        }
        loop {
            if ended {
                break;
            }
            let before_in = z.total_in();
            let before_out = z.total_out();
            // At the cap, a one-byte sentinel detects overflow without allocating it.
            let available = (limit.saturating_sub(before_out)).min(8191) as usize + 1;
            let status = z
                .decompress(input, &mut output[..available], FlushDecompress::None)
                .map_err(|_| ImageError::Malformed)?;
            let used = (z.total_in() - before_in) as usize;
            let produced = (z.total_out() - before_out) as usize;
            if z.total_out() > limit {
                return Err(ImageError::ResourceLimit);
            }
            consume(&output[..produced])?;
            input = &input[used..];
            if status == Status::StreamEnd {
                ended = true;
                if !input.is_empty() {
                    return Err(ImageError::Malformed);
                }
                break;
            }
            if used == 0 && produced == 0 {
                if !input.is_empty() {
                    return Err(ImageError::Malformed);
                }
                break;
            }
        }
    }
    if !ended {
        return Err(ImageError::Malformed);
    }
    Ok(z.total_out())
}
