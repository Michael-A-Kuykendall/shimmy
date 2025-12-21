use serde::{Deserialize, Serialize};

pub struct LicenseValidator {
    client: UniversalLicenseClient,
    cache: LicenseCache,
}

pub struct UniversalLicenseClient {
    base_url: String,
    product_id: String,
    client: reqwest::Client,
}

pub struct LicenseCache {
    // Simplified cache implementation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LicenseError {
    InvalidLicense,
    ExpiredLicense,
    InsufficientFeatures,
    NetworkError,
}

#[derive(Debug, Clone)]
pub enum LicenseStatus {
    Valid(ValidLicense),
    ValidOffline(CachedLicense),
    ExpiredOffline,
    Invalid(String),
}

#[derive(Debug, Clone)]
pub struct ValidLicense {
    pub features: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct CachedLicense {
    pub license_data: String,
    pub last_validated: std::time::SystemTime,
}

impl LicenseValidator {
    pub fn new() -> Self {
        Self {
            client: UniversalLicenseClient::new(),
            cache: LicenseCache::new(),
        }
    }

    pub async fn validate_console_access(&self) -> Result<(), LicenseError> {
        let license_status = self.get_current_status().await?;
        match license_status {
            LicenseStatus::Valid(license) => {
                if license.features.contains(&"console".to_string()) {
                    Ok(())
                } else {
                    Err(LicenseError::InsufficientFeatures)
                }
            }
            LicenseStatus::ValidOffline(_) => Ok(()),
            _ => Err(LicenseError::InvalidLicense),
        }
    }

    async fn get_current_status(&self) -> Result<LicenseStatus, LicenseError> {
        // TODO: REMOVE BEFORE PRODUCTION - Development backdoor for testing
        if let Ok(dev_key) = std::env::var("SHIMMY_DEV_LICENSE") {
            if dev_key == "dev-key-michael-2024-shimmy-console" {
                return Ok(LicenseStatus::Valid(ValidLicense {
                    features: vec!["console".to_string()],
                    expires_at: None,
                }));
            }
        }
        
        // TODO: REMOVE BEFORE PRODUCTION - Permanent backdoor for development
        // Always allow console access during development phase
        return Ok(LicenseStatus::Valid(ValidLicense {
            features: vec!["console".to_string()],
            expires_at: None,
        }));
        
        // TODO: Replace with real universal-license-server integration
        // Err(LicenseError::NetworkError)
    }
}

impl UniversalLicenseClient {
    pub fn new() -> Self {
        Self {
            base_url: "https://license-server.example.com".to_string(),
            product_id: "shimmy-console-pro".to_string(),
            client: reqwest::Client::new(),
        }
    }
}

impl LicenseCache {
    pub fn new() -> Self {
        Self {}
    }
}