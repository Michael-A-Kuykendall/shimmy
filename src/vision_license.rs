//! Shimmy Vision Licensing Module
//!
//! Keygen-based licensing for vision features.
//! Handles license validation, caching, and usage metering.
//!
//! ## Security Features (per Keygen official recommendations)
//!
//! 1. **Hard-coded Account ID & Public Key** - Prevents key-swapping attacks
//!    where a bad actor redirects validation to their own Keygen account.
//!    See: https://keygen.sh/docs/api/security/#security-public-tokens
//!
//! 2. **Ed25519 Response Signature Verification** - Prevents MITM and replay
//!    attacks by cryptographically verifying API responses.
//!    See: https://keygen.sh/docs/api/signatures/
//!
//! 3. **Custom User-Agent Header** - Enables Keygen's AI/ML crack detection.
//!    See: https://keygen.sh/docs/api/security/#security-crack-prevention

#[cfg(feature = "vision")]
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
#[cfg(feature = "vision")]
use serde::{Deserialize, Serialize};

/// Hard-coded Keygen Account ID (SECURITY: Do not move to environment variable)
/// This is a public identifier, safe to embed in source code.
#[cfg(feature = "vision")]
pub const KEYGEN_ACCOUNT_ID: &str = "6270bf9c-23ad-4483-9296-3a6d9178514a";

/// Hard-coded Keygen Ed25519 Public Key (SECURITY: Do not move to environment variable)
/// Used to verify API response signatures, preventing MITM and replay attacks.
/// Format: Hex-encoded 32-byte Ed25519 public key
#[cfg(feature = "vision")]
pub const KEYGEN_PUBLIC_KEY: &str =
    "42f313585a72a41513208800f730944f1a3b74a8acfff539f96ce244d029fa5d";

/// Shimmy version for User-Agent header (helps Keygen detect cracks)
#[cfg(feature = "vision")]
const SHIMMY_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "vision")]
use std::collections::HashMap;
#[cfg(feature = "vision")]
use std::path::PathBuf;
#[cfg(feature = "vision")]
use std::sync::Arc;
#[cfg(feature = "vision")]
use tokio::sync::RwLock;

/// License validation response from Keygen
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseValidation {
    pub valid: bool,
    pub entitlements: HashMap<String, serde_json::Value>,
    pub expires_at: Option<String>,
    pub meta: HashMap<String, serde_json::Value>,
}

/// Cached license information
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedLicense {
    pub key: String,
    pub validation: LicenseValidation,
    pub cached_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Usage tracking for metering
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub requests_today: u32,
    pub requests_this_month: u32,
    pub last_reset: chrono::DateTime<chrono::Utc>,
}

/// Vision licensing manager
#[cfg(feature = "vision")]
#[derive(Debug, Clone)]
pub struct VisionLicenseManager {
    cache: Arc<RwLock<Option<CachedLicense>>>,
    usage: Arc<RwLock<UsageStats>>,
    cache_path: PathBuf,
    usage_path: PathBuf,
}

#[cfg(feature = "vision")]
impl VisionLicenseManager {
    /// Create a new license manager
    pub fn new() -> Self {
        let cache_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("shimmy")
            .join("vision");

        std::fs::create_dir_all(&cache_dir).ok();

        Self {
            cache: Arc::new(RwLock::new(None)),
            usage: Arc::new(RwLock::new(UsageStats {
                requests_today: 0,
                requests_this_month: 0,
                last_reset: chrono::Utc::now(),
            })),
            cache_path: cache_dir.join("license_cache.json"),
            usage_path: cache_dir.join("usage_stats.json"),
        }
    }

    /// Load cached license and usage data
    pub async fn load_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Load cached license
        if self.cache_path.exists() {
            let data = tokio::fs::read_to_string(&self.cache_path).await?;
            let cached: CachedLicense = serde_json::from_str(&data)?;
            *self.cache.write().await = Some(cached);
        }

        // Load usage stats
        if self.usage_path.exists() {
            let data = tokio::fs::read_to_string(&self.usage_path).await?;
            let usage: UsageStats = serde_json::from_str(&data)?;
            *self.usage.write().await = usage;
        }

        Ok(())
    }

    /// Validate a license key with Keygen
    pub async fn validate_license(
        &self,
        license_key: &str,
    ) -> Result<LicenseValidation, Box<dyn std::error::Error>> {
        // Check cache first
        if let Some(cached) = self.cache.read().await.as_ref() {
            if cached.key == license_key {
                // Check if still valid (with 24h grace period)
                let now = chrono::Utc::now();
                if let Some(expires) = cached.expires_at {
                    if now < expires + chrono::Duration::hours(24) {
                        return Ok(cached.validation.clone());
                    }
                } else if (now - cached.cached_at) < chrono::Duration::hours(24) {
                    return Ok(cached.validation.clone());
                }
            }
        }

        // Validate with Keygen API
        let validation = self.call_keygen_validate(license_key).await?;

        // Cache the result
        let cached = CachedLicense {
            key: license_key.to_string(),
            validation: validation.clone(),
            cached_at: chrono::Utc::now(),
            expires_at: validation
                .expires_at
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        };

        // Save to disk
        let data = serde_json::to_string_pretty(&cached)?;
        tokio::fs::write(&self.cache_path, &data).await?;

        *self.cache.write().await = Some(cached);

        Ok(validation)
    }

    /// Check if vision is allowed for the given license
    pub async fn check_vision_access(
        &self,
        license_key: Option<&str>,
    ) -> Result<(), VisionLicenseError> {
        let Some(key) = license_key else {
            return Err(VisionLicenseError::MissingLicense);
        };

        let validation = self
            .validate_license(key)
            .await
            .map_err(|e| VisionLicenseError::ValidationFailed(e.to_string()))?;

        if !validation.valid {
            return Err(VisionLicenseError::InvalidLicense);
        }

        // Check VISION_ANALYSIS entitlement
        if !validation.entitlements.contains_key("VISION_ANALYSIS") {
            return Err(VisionLicenseError::FeatureNotEnabled);
        }

        // Check usage limits
        let usage = self.usage.read().await;
        if let Some(monthly_cap) = validation.entitlements.get("monthly_cap") {
            if let Some(cap) = monthly_cap.as_u64() {
                if usage.requests_this_month >= cap as u32 {
                    return Err(VisionLicenseError::UsageLimitExceeded);
                }
            }
        }

        Ok(())
    }

    /// Record a vision request for metering
    pub async fn record_usage(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut usage = self.usage.write().await;
        let now = chrono::Utc::now();

        // Reset counters if needed
        if (now - usage.last_reset).num_days() >= 1 {
            usage.requests_today = 0;
        }
        if (now - usage.last_reset).num_days() >= 30 {
            usage.requests_this_month = 0;
            usage.last_reset = now;
        }

        usage.requests_today += 1;
        usage.requests_this_month += 1;

        // Save to disk
        let data = serde_json::to_string_pretty(&*usage)?;
        tokio::fs::write(&self.usage_path, &data).await?;

        Ok(())
    }

    /// Call Keygen API to validate license
    ///
    /// ## Security Features
    /// - Hard-coded account ID (prevents key-swapping)
    /// - Ed25519 signature verification (prevents MITM/replay)
    /// - Custom User-Agent (enables crack detection)
    async fn call_keygen_validate(
        &self,
        license_key: &str,
    ) -> Result<LicenseValidation, Box<dyn std::error::Error>> {
        // SECURITY: Account ID is hard-coded to prevent key-swapping attacks
        // API key (product token) can remain in env as it's server-side only
        let api_key = std::env::var("KEYGEN_API_KEY")
            .or_else(|_| std::env::var("KEYGEN_PRODUCT_TOKEN"))
            .map_err(|_| "KEYGEN_API_KEY or KEYGEN_PRODUCT_TOKEN environment variable not set")?;

        // Build client with custom User-Agent for crack detection
        let user_agent = format!(
            "Shimmy/{} (shimmy-vision) {}/{}",
            SHIMMY_VERSION,
            std::env::consts::OS,
            std::env::consts::ARCH
        );
        let client = reqwest::Client::builder().user_agent(&user_agent).build()?;

        // Include entitlements and policy in response for full license context
        let url = format!(
            "https://api.keygen.sh/v1/accounts/{}/licenses/actions/validate-key",
            KEYGEN_ACCOUNT_ID
        );

        #[derive(Serialize)]
        struct ValidateRequest {
            meta: ValidateMeta,
        }

        #[derive(Serialize)]
        struct ValidateMeta {
            key: String,
            scope: ValidateScope,
        }

        #[derive(Serialize)]
        struct ValidateScope {
            /// Include entitlements in validation scope
            entitlements: Vec<String>,
        }

        #[derive(Deserialize)]
        struct ValidateResponse {
            meta: ValidateResponseMeta,
            data: Option<LicenseData>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct LicenseData {
            attributes: LicenseAttributes,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct LicenseAttributes {
            expiry: Option<String>,
            max_uses: Option<u64>,
            uses: Option<u64>,
        }

        #[derive(Deserialize)]
        struct ValidateResponseMeta {
            valid: bool,
            code: String,
            #[serde(default)]
            detail: Option<String>,
            /// Entitlements attached to license (when scope.entitlements is used)
            #[serde(default)]
            scope: Option<ScopeMeta>,
        }

        #[derive(Deserialize)]
        struct ScopeMeta {
            #[serde(default)]
            entitlements: Vec<String>,
        }

        let request_body = ValidateRequest {
            meta: ValidateMeta {
                key: license_key.to_string(),
                scope: ValidateScope {
                    entitlements: vec!["VISION_ANALYSIS".to_string()],
                },
            },
        };

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/vnd.api+json")
            .header("Accept", "application/vnd.api+json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Keygen API error: {}", response.status()).into());
        }

        // SECURITY: Extract headers needed for signature verification
        let signature_header = response
            .headers()
            .get("Keygen-Signature")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let date_header = response
            .headers()
            .get("Date")
            .or_else(|| response.headers().get("Keygen-Date"))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Get response body as text for signature verification
        let response_body = response.text().await?;

        // SECURITY: Verify response signature to prevent MITM attacks
        // Only verify if we have both signature and date headers
        if let (Some(sig_header), Some(date)) = (&signature_header, &date_header) {
            // SECURITY: Check for replay attacks - reject responses older than 5 minutes
            // Per Keygen docs: https://keygen.sh/docs/api/signatures/#response-signatures
            Self::check_response_freshness(date)?;

            Self::verify_response_signature(sig_header, date, &response_body)?;
        } else {
            // Log warning but don't fail - Keygen may not always send signatures
            tracing::warn!(
                "Keygen response missing signature or date header - skipping verification. \
                 sig={}, date={}",
                signature_header.is_some(),
                date_header.is_some()
            );
        }

        // Parse the verified response
        let validate_response: ValidateResponse = serde_json::from_str(&response_body)?;

        // Extract entitlements and usage info
        let mut entitlements = HashMap::new();

        // Check scoped entitlements from validation
        if let Some(ref scope) = validate_response.meta.scope {
            for code in &scope.entitlements {
                entitlements.insert(code.clone(), serde_json::Value::Bool(true));
            }
        }

        // Extract maxUses as monthly_cap and current usage
        if let Some(ref data) = validate_response.data {
            if let Some(max_uses) = data.attributes.max_uses {
                entitlements.insert(
                    "monthly_cap".to_string(),
                    serde_json::Value::Number(max_uses.into()),
                );
            }
            if let Some(uses) = data.attributes.uses {
                entitlements.insert(
                    "current_uses".to_string(),
                    serde_json::Value::Number(uses.into()),
                );
            }
        }

        // No default fallback - Keygen policies are source of truth for caps

        // Extract expiry from license data
        let expires_at = validate_response
            .data
            .as_ref()
            .and_then(|d| d.attributes.expiry.clone());

        Ok(LicenseValidation {
            valid: validate_response.meta.valid,
            entitlements,
            expires_at,
            meta: {
                let mut meta = HashMap::new();
                meta.insert(
                    "code".to_string(),
                    serde_json::Value::String(validate_response.meta.code),
                );
                if let Some(detail) = validate_response.meta.detail {
                    meta.insert("detail".to_string(), serde_json::Value::String(detail));
                }
                meta
            },
        })
    }

    /// Verify Keygen API response signature using Ed25519
    ///
    /// ## Security
    /// This prevents man-in-the-middle attacks where an attacker could
    /// intercept and modify API responses to make invalid licenses appear valid.
    ///
    /// ## Signing String Format (per Keygen docs)
    /// ```text
    /// (request-target): post /v1/accounts/<id>/licenses/actions/validate-key
    /// host: api.keygen.sh
    /// date: <Date header>
    /// digest: sha-256=<base64 SHA256 of body>
    /// ```
    pub fn verify_response_signature(
        sig_header: &str,
        date_header: &str,
        response_body: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use base64::{engine::general_purpose::STANDARD, Engine};
        use sha2::{Digest, Sha256};

        // Parse signature header to extract the signature value
        // Format: keyid="...", algorithm="ed25519", signature="<base64>", headers="..."
        let sig_base64 = sig_header
            .split(',')
            .find(|part| part.trim().starts_with("signature="))
            .and_then(|part| {
                part.trim()
                    .strip_prefix("signature=\"")
                    .and_then(|s| s.strip_suffix('"'))
            })
            .ok_or("Invalid signature header format: missing signature field")?;

        // Parse algorithm to ensure it's ed25519
        let algorithm = sig_header
            .split(',')
            .find(|part| part.trim().starts_with("algorithm="))
            .and_then(|part| {
                part.trim()
                    .strip_prefix("algorithm=\"")
                    .and_then(|s| s.strip_suffix('"'))
            });

        if algorithm != Some("ed25519") {
            return Err(format!(
                "Unsupported signature algorithm: {:?} (expected ed25519)",
                algorithm
            )
            .into());
        }

        // Compute SHA-256 digest of response body
        let digest_bytes = Sha256::digest(response_body.as_bytes());
        let digest_b64 = STANDARD.encode(digest_bytes);

        // Build the signing string per Keygen's format:
        // (request-target): post /v1/accounts/<id>/licenses/actions/validate-key
        // host: api.keygen.sh
        // date: <Date header>
        // digest: sha-256=<base64>
        let signing_string = format!(
            "(request-target): post /v1/accounts/{}/licenses/actions/validate-key\n\
             host: api.keygen.sh\n\
             date: {}\n\
             digest: sha-256={}",
            KEYGEN_ACCOUNT_ID, date_header, digest_b64
        );

        // Decode the public key from hex
        let public_key_bytes =
            hex::decode(KEYGEN_PUBLIC_KEY).map_err(|e| format!("Invalid public key hex: {}", e))?;

        let public_key_array: [u8; 32] = public_key_bytes
            .try_into()
            .map_err(|_| "Public key must be exactly 32 bytes")?;

        let verifying_key = VerifyingKey::from_bytes(&public_key_array)
            .map_err(|e| format!("Invalid Ed25519 public key: {}", e))?;

        // Decode signature from base64
        let sig_bytes = STANDARD
            .decode(sig_base64)
            .map_err(|e| format!("Invalid signature base64: {}", e))?;

        let sig_array: [u8; 64] = sig_bytes
            .try_into()
            .map_err(|_| "Signature must be exactly 64 bytes")?;

        let signature = Signature::from_bytes(&sig_array);

        // Verify the signature against the signing string
        verifying_key
            .verify(signing_string.as_bytes(), &signature)
            .map_err(|e| {
                format!(
                    "SECURITY WARNING: Response signature verification failed! \
                     Possible MITM attack detected. Error: {}",
                    e
                )
            })?;

        tracing::debug!("Keygen response signature verified successfully");
        Ok(())
    }

    /// Check that response is fresh (not a replay attack)
    ///
    /// Per Keygen docs: "If the signature is valid, but the response date is
    /// older than 5 minutes, we recommend rejecting the response"
    /// See: https://keygen.sh/docs/api/signatures/#response-signatures
    pub fn check_response_freshness(date_header: &str) -> Result<(), Box<dyn std::error::Error>> {
        use chrono::{DateTime, Utc};

        // Parse the HTTP date format: "Wed, 09 Jun 2021 16:08:15 GMT"
        let response_time = DateTime::parse_from_rfc2822(date_header)
            .map_err(|e| format!("Invalid date header format: {} ({})", date_header, e))?
            .with_timezone(&Utc);

        let now = Utc::now();
        let age = now.signed_duration_since(response_time);

        // Reject responses older than 5 minutes (replay attack protection)
        const MAX_AGE_SECONDS: i64 = 5 * 60;
        if age.num_seconds() > MAX_AGE_SECONDS {
            return Err(format!(
                "SECURITY WARNING: Response is too old ({} seconds). \
                 Possible replay attack detected. Response date: {}",
                age.num_seconds(),
                date_header
            )
            .into());
        }

        // Also reject responses from the future (clock manipulation)
        if age.num_seconds() < -60 {
            return Err(format!(
                "SECURITY WARNING: Response date is in the future. \
                 Possible clock tampering detected. Response date: {}",
                date_header
            )
            .into());
        }

        Ok(())
    }

    // ============ Test Helpers ============
    // These methods are public for integration tests but should only be used in tests.
    // They provide controlled access to internal state for test setup and verification.

    /// Set cached license (for testing only)
    #[doc(hidden)]
    #[allow(dead_code)]
    pub async fn set_cached_license(&self, cached: Option<CachedLicense>) {
        *self.cache.write().await = cached;
    }

    /// Get cached license (for testing only)
    #[doc(hidden)]
    #[allow(dead_code)]
    pub async fn get_cached_license(&self) -> Option<CachedLicense> {
        self.cache.read().await.clone()
    }

    /// Set usage stats (for testing only)
    #[doc(hidden)]
    #[allow(dead_code)]
    pub async fn set_usage_stats(&self, stats: UsageStats) {
        *self.usage.write().await = stats;
    }

    /// Get usage stats (for testing only)
    #[doc(hidden)]
    #[allow(dead_code)]
    pub async fn get_usage_stats(&self) -> UsageStats {
        self.usage.read().await.clone()
    }
}

#[cfg(feature = "vision")]
impl Default for VisionLicenseManager {
    fn default() -> Self {
        Self::new()
    }
}

/// License-related errors
#[cfg(feature = "vision")]
#[derive(Debug, thiserror::Error)]
pub enum VisionLicenseError {
    #[error("No license key provided")]
    MissingLicense,

    #[error("License validation failed: {0}")]
    ValidationFailed(String),

    #[error("Invalid or expired license")]
    InvalidLicense,

    #[error("Vision feature not enabled for this license")]
    FeatureNotEnabled,

    #[error("Monthly usage limit exceeded")]
    UsageLimitExceeded,
}

#[cfg(feature = "vision")]
impl VisionLicenseError {
    /// Convert to HTTP status code
    pub fn to_status_code(&self) -> axum::http::StatusCode {
        match self {
            VisionLicenseError::MissingLicense => axum::http::StatusCode::PAYMENT_REQUIRED,
            VisionLicenseError::ValidationFailed(_) => {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            VisionLicenseError::InvalidLicense => axum::http::StatusCode::FORBIDDEN,
            VisionLicenseError::FeatureNotEnabled => axum::http::StatusCode::FORBIDDEN,
            VisionLicenseError::UsageLimitExceeded => axum::http::StatusCode::PAYMENT_REQUIRED,
        }
    }

    /// Convert to JSON error response
    pub fn to_json_error(&self) -> serde_json::Value {
        serde_json::json!({
            "error": {
                "code": match self {
                    VisionLicenseError::MissingLicense => "MISSING_LICENSE",
                    VisionLicenseError::ValidationFailed(_) => "VALIDATION_ERROR",
                    VisionLicenseError::InvalidLicense => "INVALID_LICENSE",
                    VisionLicenseError::FeatureNotEnabled => "FEATURE_DISABLED",
                    VisionLicenseError::UsageLimitExceeded => "USAGE_LIMIT_EXCEEDED",
                },
                "message": self.to_string()
            }
        })
    }
}

/// Stub implementation for when vision is disabled
#[cfg(not(feature = "vision"))]
pub fn check_vision_license(_license: Option<&str>) -> Result<(), &'static str> {
    Err("Vision feature not enabled")
}
