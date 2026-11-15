//! Strict PNG container preflight. This pass does not decompress or allocate.
use crate::{DecodeLimits, ImageError};

pub(crate) fn u32be(b: &[u8]) -> u32 {
    u32::from_be_bytes(b[..4].try_into().unwrap())
}

pub(crate) struct Chunks<'a> {
    bytes: &'a [u8],
}
impl<'a> Chunks<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes: &bytes[8..] }
    }
}
impl<'a> Iterator for Chunks<'a> {
    type Item = (&'a [u8], &'a [u8]);
    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            return None;
        }
        let n = u32be(self.bytes) as usize;
        let item = (&self.bytes[4..8], &self.bytes[8..8 + n]);
        self.bytes = &self.bytes[n + 12..];
        Some(item)
    }
}
pub(crate) struct Header {
    pub width: u32,
    pub height: u32,
    pub color: u8,
    pub decoded: usize,
    pub filtered: u64,
}

pub(crate) fn preflight(bytes: &[u8], limits: DecodeLimits) -> Result<Header, ImageError> {
    use ImageError::*;
    if bytes.len() as u64 > limits.max_file_bytes {
        return Err(ResourceLimit);
    }
    if bytes.get(..8) != Some(b"\x89PNG\r\n\x1a\n") {
        return Err(Malformed);
    }
    let mut pos = 8usize;
    let mut metadata = 0u64;
    let mut header = None;
    let mut idat = false;
    let mut after_idat = false;
    let mut seen = 0u32;
    while pos < bytes.len() {
        let prefix = bytes
            .get(pos..pos.checked_add(8).ok_or(Malformed)?)
            .ok_or(Malformed)?;
        let n = u32be(prefix) as usize;
        let end = pos
            .checked_add(12)
            .and_then(|v| v.checked_add(n))
            .ok_or(Malformed)?;
        let whole = bytes.get(pos..end).ok_or(Malformed)?;
        let kind = &prefix[4..8];
        if !kind.iter().all(u8::is_ascii_alphabetic) || !kind[2].is_ascii_uppercase() {
            return Err(Malformed);
        }
        if crc32fast::hash(&whole[4..n + 8]) != u32be(&whole[n + 8..]) {
            return Err(Malformed);
        }
        let d = &whole[8..n + 8];
        if kind != b"IDAT" {
            metadata = metadata.checked_add(n as u64 + 12).ok_or(ResourceLimit)?;
            if metadata > limits.max_metadata_bytes {
                return Err(ResourceLimit);
            }
        }
        if header.is_none() && kind != b"IHDR" {
            return Err(Malformed);
        }
        let singleton = match kind {
            b"IHDR" => 1,
            b"iCCP" => 2,
            b"sRGB" => 4,
            b"gAMA" => 8,
            b"cHRM" => 16,
            b"cICP" => 32,
            b"pHYs" => 64,
            b"PLTE" => 128,
            b"tIME" => 256,
            b"bKGD" => 512,
            _ => 0,
        };
        if singleton != 0 && (seen & singleton != 0 || (idat && kind != b"tIME")) {
            return Err(Malformed);
        }
        seen |= singleton;
        match kind {
            b"IHDR" => {
                if n != 13 {
                    return Err(Malformed);
                }
                let width = u32be(d);
                let height = u32be(&d[4..]);
                if width == 0 || height == 0 {
                    return Err(Malformed);
                }
                let pixels = u64::from(width) * u64::from(height);
                if width > limits.max_dimension
                    || height > limits.max_dimension
                    || pixels > limits.max_pixels
                {
                    return Err(ResourceLimit);
                }
                let channels = match d[9] {
                    0 => 1,
                    2 => 3,
                    4 => 2,
                    6 => 4,
                    _ => return Err(UnsupportedFormat),
                };
                if d[8] != 8 || d[10] != 0 || d[11] != 0 || d[12] != 0 {
                    return Err(UnsupportedFormat);
                }
                let decoded = pixels.checked_mul(channels).ok_or(ResourceLimit)?;
                if decoded > limits.max_decoded_bytes {
                    return Err(ResourceLimit);
                }
                header = Some(Header {
                    width,
                    height,
                    color: d[9],
                    decoded: usize::try_from(decoded).map_err(|_| ResourceLimit)?,
                    filtered: decoded
                        .checked_add(u64::from(height))
                        .ok_or(ResourceLimit)?,
                });
            }
            b"IDAT" => {
                if after_idat {
                    return Err(Malformed);
                }
                idat = true;
            }
            b"IEND" => {
                if n != 0 || !idat || end != bytes.len() {
                    return Err(Malformed);
                }
                return header.ok_or(Malformed);
            }
            b"acTL" | b"fcTL" | b"fdAT" => return Err(Animation),
            b"cICP" => {
                if n != 4 {
                    return Err(Malformed);
                }
                if matches!(d[1], 16 | 18) {
                    return Err(Hdr);
                }
                if d != [1, 13, 0, 1] {
                    return Err(UnsupportedColor);
                }
            }
            b"mDCv" | b"cLLi" => return Err(Hdr),
            b"sRGB" => {
                if n != 1 || d[0] > 3 {
                    return Err(Malformed);
                }
            }
            b"gAMA" => {
                if n != 4 || u32be(d) == 0 {
                    return Err(Malformed);
                }
            }
            b"cHRM" => {
                if n != 32 {
                    return Err(Malformed);
                }
                for pair in d.as_chunks::<8>().0 {
                    let x = u32be(pair);
                    let y = u32be(&pair[4..]);
                    if y == 0 || u64::from(x) + u64::from(y) > 100_000 {
                        return Err(Malformed);
                    }
                }
            }
            b"pHYs" => {
                if n != 9 || d[8] > 1 {
                    return Err(Malformed);
                }
            }
            b"iCCP" => {}
            b"PLTE" => {
                if n == 0
                    || n > 768
                    || !n.is_multiple_of(3)
                    || matches!(header.as_ref().unwrap().color, 0 | 4)
                {
                    return Err(Malformed);
                }
            }
            // Reject rather than implicitly discard output-affecting or compressed metadata.
            b"tRNS" | b"sBIT" | b"eXIf" | b"zTXt" | b"iTXt" => return Err(UnsupportedMetadata),
            b"tEXt" => {
                let separator = d.iter().position(|b| *b == 0).ok_or(Malformed)?;
                let name = &d[..separator];
                if !(1..=79).contains(&separator)
                    || name.iter().any(|b| !matches!(b, 32..=126 | 161..=255))
                    || name.first() == Some(&32)
                    || name.last() == Some(&32)
                    || name.windows(2).any(|w| w == b"  ")
                    || d[separator + 1..].contains(&0)
                {
                    return Err(Malformed);
                }
            }
            b"tIME" => {
                if n != 7
                    || !(1..=12).contains(&d[2])
                    || !(1..=31).contains(&d[3])
                    || d[4] > 23
                    || d[5] > 59
                    || d[6] > 60
                {
                    return Err(Malformed);
                }
            }
            b"bKGD" => {
                let expected = if matches!(header.as_ref().unwrap().color, 0 | 4) {
                    2
                } else {
                    6
                };
                if n != expected || d.as_chunks::<2>().0.iter().any(|v| v[0] != 0) {
                    return Err(Malformed);
                }
            }
            _ => return Err(UnsupportedMetadata),
        }
        if idat && kind != b"IDAT" {
            after_idat = true;
        }
        pos = end;
    }
    Err(Malformed)
}
