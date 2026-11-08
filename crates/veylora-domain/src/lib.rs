//! Platform-independent domain contracts for Veylora RIP.

pub mod errors;
pub mod ids;
pub mod job;
pub mod layout;
pub mod lifecycle;
pub mod preflight;
pub mod provenance;
pub mod recipe;

/// Product display name shared by headless and desktop adapters.
pub const PRODUCT_NAME: &str = "Veylora RIP";

/// Software version inherited from the workspace package metadata.
pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    #[test]
    fn reports_product_identity_and_version() {
        assert_eq!(super::PRODUCT_NAME, "Veylora RIP");
        assert_eq!(super::version(), "0.0.0");
    }
}
