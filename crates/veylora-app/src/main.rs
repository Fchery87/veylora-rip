//! Veylora RIP desktop shell (Tauri 2).
//!
//! Thin adapter: it exposes typed commands that return read models. All
//! authoritative behavior stays in Rust application services (ADR 0001).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod state;

use state::CapabilityState;

#[tauri::command]
fn get_capabilities() -> CapabilityState {
    CapabilityState::fixture()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_capabilities])
        .run(tauri::generate_context!())
        .expect("error while running Veylora RIP");
}
