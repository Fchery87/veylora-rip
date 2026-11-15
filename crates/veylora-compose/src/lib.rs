//! Deterministic placement resampling and page compositing.
//!
//! Every placement is drawn into one page raster at the device pixel pitch. The kernel is exact
//! area averaging: a page pixel takes the area-weighted mean of the source pixels it covers, in
//! premultiplied alpha so hidden color under transparency never bleeds into neighbors. Rotation
//! and mirror are quarter-turn flips, so the inverse map stays axis-aligned and the same
//! [`veylora_layout::ResolvedPlacement`] the preview uses decides where every pixel lands.
//! Placements composite source-over in layout order. Placement boxes may sit on fractional
//! pixels; the partly covered edge pixels become partial alpha.
//!
//! Nothing is guessed: a placement outside the page, a crop outside its source, mixed color
//! encodings, or a gray source whose encoding cannot be expanded exactly are refused.
//!
//! The composed page is held in memory at 4 bytes per pixel (bounded by `max_page_pixels`).
//! [`ALGORITHM`] changes whenever the arithmetic does, so retained artifacts can name it.
#![forbid(unsafe_code)]

mod axis;
mod page;

pub use page::{compose_page, ComposeError, ComposeInput, ComposedPage};

/// Identifies the resampling and compositing arithmetic for provenance.
pub const ALGORITHM: &str = "veylora-compose-area-v1";
