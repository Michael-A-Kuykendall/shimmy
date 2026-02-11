//! License client for validating shimmy licenses

use serde::{Deserialize, Serialize};

/// Information about a license
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    /// License key
    pub key: String,
    /// Whether the license is valid
    pub valid: bool,
    /// License holder email
    pub email: Option<String>,
    /// Expiration date (ISO 8601)
    pub expires_at: Option<String>,
    /// Features enabled by this license
    pub features: Vec<String>,
}

/// Client for validating licenses
pub struct LicenseClient {
    license_server_url: String,
    cached_info: Option<LicenseInfo>,
}

impl LicenseClient {
    /// Create a new license client
    pub fn new(license_server_url: impl Into<String>) -> Self {
        Self {
            license_server_url: license_server_url.into(),
            cached_info: None,
        }
    }

    /// Create a client for offline/local use
    pub fn offline() -> Self {
        Self {
            license_server_url: String::new(),
            cached_info: None,
        }
    }

    /// Validate a license key
    pub async fn validate(&mut self, key: &str) -> anyhow::Result<bool> {
        // For offline mode or empty URL, accept any non-empty key
        if self.license_server_url.is_empty() {
            let valid = !key.is_empty();
            self.cached_info = Some(LicenseInfo {
                key: key.to_string(),
                valid,
                email: None,
                expires_at: None,
                features: if valid {
                    vec!["basic".to_string()]
                } else {
                    vec![]
                },
            });
            return Ok(valid);
        }

        // Online validation
        let client = reqwest::Client::new();
        let url = format!("{}/validate", self.license_server_url);

        let response = client
            .post(&url)
            .json(&serde_json::json!({ "key": key }))
            .send()
            .await?;

        if response.status().is_success() {
            let info: LicenseInfo = response.json().await?;
            let valid = info.valid;
            self.cached_info = Some(info);
            Ok(valid)
        } else {
            Ok(false)
        }
    }

    /// Get cached license info
    pub fn get_info(&self) -> Option<&LicenseInfo> {
        self.cached_info.as_ref()
    }

    /// Check if a specific feature is enabled
    pub fn has_feature(&self, feature: &str) -> bool {
        self.cached_info
            .as_ref()
            .map(|info| info.valid && info.features.iter().any(|f| f == feature))
            .unwrap_or(false)
    }

    /// Check if any valid license is present
    pub fn is_licensed(&self) -> bool {
        self.cached_info
            .as_ref()
            .map(|info| info.valid)
            .unwrap_or(false)
    }
}

impl Default for LicenseClient {
    fn default() -> Self {
        Self::offline()
    }
}
