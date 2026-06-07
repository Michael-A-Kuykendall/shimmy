use anyhow::Result;

pub struct LicenseClient;

impl LicenseClient {
    pub fn new(_base_url: &str) -> Self {
        Self
    }

    pub async fn validate(&self, _key: &str) -> Result<LicenseStatus> {
        Ok(LicenseStatus::Valid)
    }
}

pub enum LicenseStatus {
    Valid,
    Invalid(String),
    Expired,
}
