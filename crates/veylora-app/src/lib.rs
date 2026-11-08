//! Veylora RIP desktop shell library: read models and presentation view models.
//!
//! Everything here is presentation over application-service data (ADR 0001).
//! The Tauri binary in `main.rs` only exposes these as typed commands.

pub mod jobs;
pub mod screens;
pub mod state;
pub mod view_models;
