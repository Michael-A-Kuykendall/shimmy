pub mod client;
pub mod validation;

pub use client::{LicenseClient, LicenseStatus};

pub async fn validate_console_access() -> anyhow::Result<()> {
    // Check license
    Ok(())
}
