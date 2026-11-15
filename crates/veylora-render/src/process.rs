//! The seam where artwork becomes C, M, Y and K coverage.
//!
//! A real separation needs a licensed printer/ink profile and the color engine, neither of which
//! exists yet, so this crate does not guess one. Callers supply a [`ProcessSeparator`]; the only
//! built-in is [`WhiteOnly`], the spec's white-only diagnostic mode.
use std::ops::Range;
use veylora_image::SourceAsset;
use veylora_separation::Plane;

/// Process coverage for a band of source rows, before alpha is applied.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessBand {
    pub c: Plane,
    pub m: Plane,
    pub y: Plane,
    pub k: Plane,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeparatorError(pub String);

/// Must be pointwise: the planes for rows `rows` may depend only on those source rows, so any
/// banding gives the same result. Each plane must be `width x rows.len()`. The pipeline multiplies
/// the result by alpha, so hidden color under transparent pixels can never print.
pub trait ProcessSeparator {
    fn separate(
        &self,
        source: &SourceAsset,
        rows: Range<usize>,
    ) -> Result<ProcessBand, SeparatorError>;
}
/// No process ink; only white (and the clear mask) can print.
#[derive(Debug, Clone, Copy, Default)]
pub struct WhiteOnly;
impl ProcessSeparator for WhiteOnly {
    fn separate(
        &self,
        source: &SourceAsset,
        rows: Range<usize>,
    ) -> Result<ProcessBand, SeparatorError> {
        let blank = || Plane::new(source.metadata.width as usize, rows.len(), 0.0);
        Ok(ProcessBand {
            c: blank(),
            m: blank(),
            y: blank(),
            k: blank(),
        })
    }
}
