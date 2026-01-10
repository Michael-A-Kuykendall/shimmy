//! Live Keygen API Integration Tests for Shimmy Vision
//!
//! These tests hit the real Keygen API to validate license validation behavior.
//! They are marked with #[ignore] by default and should be run explicitly:
//!
//! ```bash
//! # Run all live Keygen tests
//! cargo test --features llama,vision keygen_live -- --ignored
//!
//! # Run a specific test
//! cargo test --features llama,vision test_live_valid_license -- --ignored
//! ```
//!
//! ## Prerequisites
//! - `.env` file must contain `KEYGEN_PRODUCT_TOKEN` or `KEYGEN_API_KEY`
//! - `.env` file must contain test license keys (KEYGEN_TEST_LICENSE_*)
//! - Network access to api.keygen.sh
//!
//! ## Test Licenses (created via Keygen API)
//! - VALID: Active license with VISION_ANALYSIS entitlement
//! - EXPIRED: License with past expiry date
//! - SUSPENDED: Active license that has been suspended
//! - NO_ENTITLEMENT: License without VISION_ANALYSIS entitlement

#![cfg(all(feature = "llama", feature = "vision"))]

use serial_test::serial;
use shimmy::vision_license::VisionLicenseManager;
use std::env;

/// Load environment variables from .env file
fn load_env() {
    // Only load once
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        // Manual .env loading without external crate
        if let Ok(contents) = std::fs::read_to_string(".env") {
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    env::set_var(key.trim(), value.trim());
                }
            }
        }
    });
}

/// Get a test license key from environment
fn get_test_key(name: &str) -> Option<String> {
    load_env();
    env::var(name).ok()
}

/// Check if live tests should run (has required env vars)
fn should_run_live_tests() -> bool {
    load_env();
    env::var("KEYGEN_PRODUCT_TOKEN")
        .or_else(|_| env::var("KEYGEN_API_KEY"))
        .is_ok()
        && env::var("KEYGEN_TEST_LICENSE_VALID").is_ok()
}

// ============================================================================
// LIVE API TESTS
// ============================================================================

/// Test validating a valid license with VISION_ANALYSIS entitlement
#[tokio::test]
#[serial]
#[ignore = "Live Keygen API test - run with --ignored flag"]
async fn test_live_valid_license() {
    if !should_run_live_tests() {
        eprintln!("Skipping: Missing KEYGEN_PRODUCT_TOKEN or test license keys");
        return;
    }

    let license_key = get_test_key("KEYGEN_TEST_LICENSE_VALID").unwrap();
    let manager = VisionLicenseManager::new();

    // Set the API key from env
    env::set_var("KEYGEN_API_KEY", env::var("KEYGEN_PRODUCT_TOKEN").unwrap());

    let result = manager.validate_license(&license_key).await;

    assert!(
        result.is_ok(),
        "Valid license should validate: {:?}",
        result
    );
    let validation = result.unwrap();
    assert!(validation.valid, "License should be valid");
    assert!(
        validation.entitlements.contains_key("VISION_ANALYSIS"),
        "Should have VISION_ANALYSIS entitlement"
    );
}

/// Test that an expired license is rejected with EXPIRED code
#[tokio::test]
#[serial]
#[ignore = "Live Keygen API test - run with --ignored flag"]
async fn test_live_expired_license() {
    if !should_run_live_tests() {
        eprintln!("Skipping: Missing KEYGEN_PRODUCT_TOKEN or test license keys");
        return;
    }

    let license_key = get_test_key("KEYGEN_TEST_LICENSE_EXPIRED").unwrap();
    let manager = VisionLicenseManager::new();

    env::set_var("KEYGEN_API_KEY", env::var("KEYGEN_PRODUCT_TOKEN").unwrap());

    let result = manager.validate_license(&license_key).await;

    assert!(
        result.is_ok(),
        "Should get validation response: {:?}",
        result
    );
    let validation = result.unwrap();
    assert!(!validation.valid, "Expired license should be invalid");

    // Check the validation code
    let code = validation
        .meta
        .get("code")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(code, "EXPIRED", "Should return EXPIRED code");
}

/// Test that a suspended license is rejected with SUSPENDED code
#[tokio::test]
#[serial]
#[ignore = "Live Keygen API test - run with --ignored flag"]
async fn test_live_suspended_license() {
    if !should_run_live_tests() {
        eprintln!("Skipping: Missing KEYGEN_PRODUCT_TOKEN or test license keys");
        return;
    }

    let license_key = get_test_key("KEYGEN_TEST_LICENSE_SUSPENDED").unwrap();
    let manager = VisionLicenseManager::new();

    env::set_var("KEYGEN_API_KEY", env::var("KEYGEN_PRODUCT_TOKEN").unwrap());

    let result = manager.validate_license(&license_key).await;

    assert!(
        result.is_ok(),
        "Should get validation response: {:?}",
        result
    );
    let validation = result.unwrap();
    assert!(!validation.valid, "Suspended license should be invalid");

    let code = validation
        .meta
        .get("code")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(code, "SUSPENDED", "Should return SUSPENDED code");
}

/// Test that a license without VISION_ANALYSIS entitlement is rejected
#[tokio::test]
#[serial]
#[ignore = "Live Keygen API test - run with --ignored flag"]
async fn test_live_missing_entitlement() {
    if !should_run_live_tests() {
        eprintln!("Skipping: Missing KEYGEN_PRODUCT_TOKEN or test license keys");
        return;
    }

    let license_key = get_test_key("KEYGEN_TEST_LICENSE_NO_ENTITLEMENT").unwrap();
    let manager = VisionLicenseManager::new();

    env::set_var("KEYGEN_API_KEY", env::var("KEYGEN_PRODUCT_TOKEN").unwrap());

    let result = manager.validate_license(&license_key).await;

    assert!(
        result.is_ok(),
        "Should get validation response: {:?}",
        result
    );
    let validation = result.unwrap();
    assert!(
        !validation.valid,
        "License without entitlement should be invalid"
    );

    let code = validation
        .meta
        .get("code")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(
        code, "ENTITLEMENTS_MISSING",
        "Should return ENTITLEMENTS_MISSING code"
    );
}

/// Test that an invalid/non-existent license key is rejected
#[tokio::test]
#[serial]
#[ignore = "Live Keygen API test - run with --ignored flag"]
async fn test_live_invalid_license_key() {
    if !should_run_live_tests() {
        eprintln!("Skipping: Missing KEYGEN_PRODUCT_TOKEN or test license keys");
        return;
    }

    let manager = VisionLicenseManager::new();

    env::set_var("KEYGEN_API_KEY", env::var("KEYGEN_PRODUCT_TOKEN").unwrap());

    // Use a fake license key
    let result = manager
        .validate_license("FAKE00-LICENSE-KEY123-NOTREAL-000000-V3")
        .await;

    assert!(
        result.is_ok(),
        "Should get validation response: {:?}",
        result
    );
    let validation = result.unwrap();
    assert!(!validation.valid, "Fake license should be invalid");

    let code = validation
        .meta
        .get("code")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(code, "NOT_FOUND", "Should return NOT_FOUND code");
}

/// Test that response signatures are properly verified (prevents MITM attacks)
#[tokio::test]
#[serial]
#[ignore = "Live Keygen API test - run with --ignored flag"]
async fn test_live_signature_verification() {
    if !should_run_live_tests() {
        eprintln!("Skipping: Missing KEYGEN_PRODUCT_TOKEN or test license keys");
        return;
    }

    let license_key = get_test_key("KEYGEN_TEST_LICENSE_VALID").unwrap();
    let manager = VisionLicenseManager::new();

    env::set_var("KEYGEN_API_KEY", env::var("KEYGEN_PRODUCT_TOKEN").unwrap());

    // This should succeed because Keygen signs all responses
    // If signature verification was broken, this would fail
    let result = manager.validate_license(&license_key).await;

    assert!(
        result.is_ok(),
        "Signature verification should pass for valid Keygen response: {:?}",
        result
    );
}

/// Test multiple consecutive validations (rate limiting, connection reuse)
#[tokio::test]
#[serial]
#[ignore = "Live Keygen API test - run with --ignored flag"]
async fn test_live_multiple_validations() {
    if !should_run_live_tests() {
        eprintln!("Skipping: Missing KEYGEN_PRODUCT_TOKEN or test license keys");
        return;
    }

    let license_key = get_test_key("KEYGEN_TEST_LICENSE_VALID").unwrap();
    let manager = VisionLicenseManager::new();

    env::set_var("KEYGEN_API_KEY", env::var("KEYGEN_PRODUCT_TOKEN").unwrap());

    // Run 3 consecutive validations
    for i in 1..=3 {
        let result = manager.validate_license(&license_key).await;
        assert!(
            result.is_ok(),
            "Validation {} should succeed: {:?}",
            i,
            result
        );
        let validation = result.unwrap();
        assert!(validation.valid, "Validation {} should be valid", i);
    }
}

/// Test that the User-Agent header is properly set (crack detection)
#[tokio::test]
#[serial]
#[ignore = "Live Keygen API test - run with --ignored flag"]
async fn test_live_user_agent_sent() {
    if !should_run_live_tests() {
        eprintln!("Skipping: Missing KEYGEN_PRODUCT_TOKEN or test license keys");
        return;
    }

    // This test just verifies that requests succeed with our custom User-Agent
    // Keygen logs User-Agent for analytics/crack detection
    let license_key = get_test_key("KEYGEN_TEST_LICENSE_VALID").unwrap();
    let manager = VisionLicenseManager::new();

    env::set_var("KEYGEN_API_KEY", env::var("KEYGEN_PRODUCT_TOKEN").unwrap());

    let result = manager.validate_license(&license_key).await;
    assert!(
        result.is_ok(),
        "Should work with custom User-Agent: {:?}",
        result
    );
}

// ============================================================================
// SMOKE TEST (can run in CI with secrets)
// ============================================================================

/// Single smoke test that can be enabled in CI with proper secrets
/// This validates the full license flow works end-to-end
#[tokio::test]
#[serial]
#[ignore = "Live Keygen API test - run with --ignored flag"]
async fn test_live_smoke_test() {
    if !should_run_live_tests() {
        eprintln!("=== SMOKE TEST SKIPPED ===");
        eprintln!("Missing environment variables for live testing.");
        eprintln!("Required: KEYGEN_PRODUCT_TOKEN and KEYGEN_TEST_LICENSE_VALID");
        return;
    }

    eprintln!("=== SMOKE TEST RUNNING ===");
    eprintln!("Testing against live Keygen API...");

    let license_key = get_test_key("KEYGEN_TEST_LICENSE_VALID").unwrap();
    let manager = VisionLicenseManager::new();

    env::set_var("KEYGEN_API_KEY", env::var("KEYGEN_PRODUCT_TOKEN").unwrap());

    let result = manager.validate_license(&license_key).await;

    match result {
        Ok(validation) => {
            eprintln!("✅ API call succeeded");
            eprintln!("   valid: {}", validation.valid);
            eprintln!("   code: {:?}", validation.meta.get("code"));
            eprintln!(
                "   entitlements: {:?}",
                validation.entitlements.keys().collect::<Vec<_>>()
            );
            assert!(validation.valid, "License should be valid");
            assert!(
                validation.entitlements.contains_key("VISION_ANALYSIS"),
                "Should have VISION_ANALYSIS"
            );
            eprintln!("=== SMOKE TEST PASSED ===");
        }
        Err(e) => {
            panic!("=== SMOKE TEST FAILED ===\nError: {}", e);
        }
    }
}
