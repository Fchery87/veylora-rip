//! Calibration of logical process planes and deterministic ordered-threshold screening.
//!
//! Pipeline order: separation -> clear mask -> calibration (curves, channel limits, combined
//! limit) -> slot allocation -> screening. Calibration never creates ink from empty coverage and
//! limits only reduce it, so a pixel cleared before calibration stays clear through screening.
#![forbid(unsafe_code)]

mod calibration;
mod halftone;

pub use calibration::{apply_calibration, CalibrationError};
pub use halftone::{
    screen_band, PageOrigin, ScreenError, ScreenSpec, ScreenedBand, MAX_SCREEN_CELL_PIXELS,
};
