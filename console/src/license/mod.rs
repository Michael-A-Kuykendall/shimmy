pub mod validator;
pub mod client;

pub use validator::{LicenseValidator, LicenseError};
pub use client::UniversalLicenseClient;