//! Comprehensive unit tests for vision_license.rs with 90%+ coverage
//!
//! Tests cover:
//! - check_response_freshness() with valid, stale, and future dates
//! - verify_response_signature() with various signature scenarios
//! - VisionLicenseError status code and JSON serialization
//! - VisionLicenseManager functionality with mocking

#[cfg(feature = "vision")]
mod vision_license_tests {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use chrono::{Duration, Utc};
    use serial_test::serial;
    use shimmy::vision_license::*;
    use std::collections::HashMap;

    // Helper to access private methods through a wrapper
    struct VisionLicenseManagerTestWrapper;

    impl VisionLicenseManagerTestWrapper {
        /// Test wrapper for check_response_freshness
        pub fn check_response_freshness(
            date_header: &str,
        ) -> Result<(), Box<dyn std::error::Error>> {
            VisionLicenseManager::check_response_freshness(date_header)
        }

        /// Test wrapper for verify_response_signature
        pub fn verify_response_signature(
            sig_header: &str,
            date_header: &str,
            response_body: &str,
        ) -> Result<(), Box<dyn std::error::Error>> {
            VisionLicenseManager::verify_response_signature(sig_header, date_header, response_body)
        }
    }

    #[tokio::test]
    async fn test_check_response_freshness_valid_current_time() {
        // Test with current time (should be valid)
        let current_time = Utc::now();
        let date_header = current_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let result = VisionLicenseManagerTestWrapper::check_response_freshness(&date_header);
        assert!(result.is_ok(), "Current time should be valid");
    }

    #[tokio::test]
    async fn test_check_response_freshness_valid_recent() {
        // Test with time 1 minute ago (should be valid)
        let recent_time = Utc::now() - Duration::minutes(1);
        let date_header = recent_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let result = VisionLicenseManagerTestWrapper::check_response_freshness(&date_header);
        assert!(result.is_ok(), "Recent time (1 min ago) should be valid");
    }

    #[tokio::test]
    async fn test_check_response_freshness_valid_edge_case() {
        // Test with time exactly 5 minutes ago (should be valid - right at boundary)
        let edge_time = Utc::now() - Duration::minutes(5);
        let date_header = edge_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let result = VisionLicenseManagerTestWrapper::check_response_freshness(&date_header);
        assert!(result.is_ok(), "Time exactly 5 minutes ago should be valid");
    }

    #[tokio::test]
    async fn test_check_response_freshness_stale_six_minutes() {
        // Test with time 6 minutes ago (should be stale)
        let stale_time = Utc::now() - Duration::minutes(6);
        let date_header = stale_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let result = VisionLicenseManagerTestWrapper::check_response_freshness(&date_header);
        assert!(result.is_err(), "Time 6 minutes ago should be stale");

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("SECURITY WARNING"));
        assert!(error_msg.contains("too old"));
        assert!(error_msg.contains("Possible replay attack"));
    }

    #[tokio::test]
    async fn test_check_response_freshness_stale_one_hour() {
        // Test with time 1 hour ago (should be stale)
        let stale_time = Utc::now() - Duration::hours(1);
        let date_header = stale_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let result = VisionLicenseManagerTestWrapper::check_response_freshness(&date_header);
        assert!(result.is_err(), "Time 1 hour ago should be stale");

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("SECURITY WARNING"));
        assert!(error_msg.contains("too old"));
    }

    #[tokio::test]
    async fn test_check_response_freshness_future_edge_case() {
        // Test with time 30 seconds in future (should be valid - within 60 second tolerance)
        let future_time = Utc::now() + Duration::seconds(30);
        let date_header = future_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let result = VisionLicenseManagerTestWrapper::check_response_freshness(&date_header);
        assert!(
            result.is_ok(),
            "Time 30 seconds in future should be valid (clock skew tolerance)"
        );
    }

    #[tokio::test]
    async fn test_check_response_freshness_future_invalid() {
        // Test with time 2 minutes in future (should be invalid)
        let future_time = Utc::now() + Duration::minutes(2);
        let date_header = future_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let result = VisionLicenseManagerTestWrapper::check_response_freshness(&date_header);
        assert!(
            result.is_err(),
            "Time 2 minutes in future should be invalid"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("SECURITY WARNING"));
        assert!(error_msg.contains("future"));
        assert!(error_msg.contains("clock tampering"));
    }

    #[tokio::test]
    async fn test_check_response_freshness_invalid_format() {
        // Test with invalid date format
        let invalid_date = "invalid-date-format";

        let result = VisionLicenseManagerTestWrapper::check_response_freshness(invalid_date);
        assert!(result.is_err(), "Invalid date format should return error");

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid date header format"));
    }

    #[tokio::test]
    async fn test_verify_response_signature_missing_signature_field() {
        // Test signature header missing the signature field
        let sig_header =
            r#"keyid="test", algorithm="ed25519", headers="(request-target) host date digest""#;
        let date_header = "Wed, 09 Jun 2021 16:08:15 GMT";
        let response_body = r#"{"meta":{"valid":true}}"#;

        let result = VisionLicenseManagerTestWrapper::verify_response_signature(
            sig_header,
            date_header,
            response_body,
        );
        assert!(
            result.is_err(),
            "Missing signature field should return error"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid signature header format"));
        assert!(error_msg.contains("missing signature field"));
    }

    #[tokio::test]
    async fn test_verify_response_signature_wrong_algorithm() {
        // Test signature with wrong algorithm
        let sig_header = r#"keyid="test", algorithm="rsa-sha256", signature="dGVzdHNpZw==", headers="(request-target) host date digest""#;
        let date_header = "Wed, 09 Jun 2021 16:08:15 GMT";
        let response_body = r#"{"meta":{"valid":true}}"#;

        let result = VisionLicenseManagerTestWrapper::verify_response_signature(
            sig_header,
            date_header,
            response_body,
        );
        assert!(result.is_err(), "Wrong algorithm should return error");

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unsupported signature algorithm"));
        assert!(error_msg.contains("expected ed25519"));
    }

    #[tokio::test]
    async fn test_verify_response_signature_missing_algorithm() {
        // Test signature header missing algorithm field
        let sig_header = r#"keyid="test", signature="dGVzdHNpZw==", headers="(request-target) host date digest""#;
        let date_header = "Wed, 09 Jun 2021 16:08:15 GMT";
        let response_body = r#"{"meta":{"valid":true}}"#;

        let result = VisionLicenseManagerTestWrapper::verify_response_signature(
            sig_header,
            date_header,
            response_body,
        );
        assert!(result.is_err(), "Missing algorithm should return error");

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unsupported signature algorithm"));
    }

    #[tokio::test]
    async fn test_verify_response_signature_malformed_header_no_quotes() {
        // Test malformed signature header (missing quotes)
        let sig_header = "keyid=test, algorithm=ed25519, signature=dGVzdHNpZw==, headers=(request-target) host date digest";
        let date_header = "Wed, 09 Jun 2021 16:08:15 GMT";
        let response_body = r#"{"meta":{"valid":true}}"#;

        let result = VisionLicenseManagerTestWrapper::verify_response_signature(
            sig_header,
            date_header,
            response_body,
        );
        assert!(
            result.is_err(),
            "Malformed header (no quotes) should return error"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid signature header format"));
    }

    #[tokio::test]
    async fn test_verify_response_signature_invalid_base64() {
        // Test signature with invalid base64
        let sig_header = r#"keyid="test", algorithm="ed25519", signature="invalid-base64!", headers="(request-target) host date digest""#;
        let date_header = "Wed, 09 Jun 2021 16:08:15 GMT";
        let response_body = r#"{"meta":{"valid":true}}"#;

        let result = VisionLicenseManagerTestWrapper::verify_response_signature(
            sig_header,
            date_header,
            response_body,
        );
        assert!(
            result.is_err(),
            "Invalid base64 signature should return error"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid signature base64"));
    }

    #[tokio::test]
    async fn test_verify_response_signature_wrong_signature_length() {
        // Test signature with wrong length (not 64 bytes)
        let sig_header = r#"keyid="test", algorithm="ed25519", signature="dGVzdA==", headers="(request-target) host date digest""#; // "test" in base64, only 4 bytes
        let date_header = "Wed, 09 Jun 2021 16:08:15 GMT";
        let response_body = r#"{"meta":{"valid":true}}"#;

        let result = VisionLicenseManagerTestWrapper::verify_response_signature(
            sig_header,
            date_header,
            response_body,
        );
        assert!(
            result.is_err(),
            "Wrong signature length should return error"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Signature must be exactly 64 bytes"));
    }

    #[tokio::test]
    async fn test_verify_response_signature_invalid_ed25519_signature() {
        // Test with valid format but cryptographically invalid signature
        // This will fail at the verification step since we're using the real public key
        let valid_length_sig = STANDARD.encode(&[0u8; 64]); // 64 bytes of zeros
        let sig_header = format!(
            r#"keyid="test", algorithm="ed25519", signature="{}", headers="(request-target) host date digest""#,
            valid_length_sig
        );
        let date_header = "Wed, 09 Jun 2021 16:08:15 GMT";
        let response_body = r#"{"meta":{"valid":true}}"#;

        let result = VisionLicenseManagerTestWrapper::verify_response_signature(
            &sig_header,
            date_header,
            response_body,
        );
        assert!(
            result.is_err(),
            "Invalid signature should fail verification"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("SECURITY WARNING"));
        assert!(error_msg.contains("signature verification failed"));
        assert!(error_msg.contains("MITM attack"));
    }

    #[test]
    fn test_vision_license_error_to_status_code_missing_license() {
        let error = VisionLicenseError::MissingLicense;
        let status = error.to_status_code();
        assert_eq!(status, axum::http::StatusCode::PAYMENT_REQUIRED);
    }

    #[test]
    fn test_vision_license_error_to_status_code_validation_failed() {
        let error = VisionLicenseError::ValidationFailed("test error".to_string());
        let status = error.to_status_code();
        assert_eq!(status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_vision_license_error_to_status_code_invalid_license() {
        let error = VisionLicenseError::InvalidLicense;
        let status = error.to_status_code();
        assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_vision_license_error_to_status_code_feature_not_enabled() {
        let error = VisionLicenseError::FeatureNotEnabled;
        let status = error.to_status_code();
        assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_vision_license_error_to_status_code_usage_limit_exceeded() {
        let error = VisionLicenseError::UsageLimitExceeded;
        let status = error.to_status_code();
        assert_eq!(status, axum::http::StatusCode::PAYMENT_REQUIRED);
    }

    #[test]
    fn test_vision_license_error_to_json_error_missing_license() {
        let error = VisionLicenseError::MissingLicense;
        let json = error.to_json_error();

        assert!(json["error"].is_object());
        assert_eq!(json["error"]["code"], "MISSING_LICENSE");
        assert_eq!(json["error"]["message"], "No license key provided");
    }

    #[test]
    fn test_vision_license_error_to_json_error_validation_failed() {
        let error = VisionLicenseError::ValidationFailed("network timeout".to_string());
        let json = error.to_json_error();

        assert!(json["error"].is_object());
        assert_eq!(json["error"]["code"], "VALIDATION_ERROR");
        assert_eq!(
            json["error"]["message"],
            "License validation failed: network timeout"
        );
    }

    #[test]
    fn test_vision_license_error_to_json_error_invalid_license() {
        let error = VisionLicenseError::InvalidLicense;
        let json = error.to_json_error();

        assert!(json["error"].is_object());
        assert_eq!(json["error"]["code"], "INVALID_LICENSE");
        assert_eq!(json["error"]["message"], "Invalid or expired license");
    }

    #[test]
    fn test_vision_license_error_to_json_error_feature_not_enabled() {
        let error = VisionLicenseError::FeatureNotEnabled;
        let json = error.to_json_error();

        assert!(json["error"].is_object());
        assert_eq!(json["error"]["code"], "FEATURE_DISABLED");
        assert_eq!(
            json["error"]["message"],
            "Vision feature not enabled for this license"
        );
    }

    #[test]
    fn test_vision_license_error_to_json_error_usage_limit_exceeded() {
        let error = VisionLicenseError::UsageLimitExceeded;
        let json = error.to_json_error();

        assert!(json["error"].is_object());
        assert_eq!(json["error"]["code"], "USAGE_LIMIT_EXCEEDED");
        assert_eq!(json["error"]["message"], "Monthly usage limit exceeded");
    }

    #[tokio::test]
    async fn test_vision_license_manager_new() {
        let manager = VisionLicenseManager::new();

        // Should create manager with empty cache and initial usage stats
        let cache = manager.get_cached_license().await;
        assert!(cache.is_none(), "New manager should have empty cache");

        let usage = manager.get_usage_stats().await;
        assert_eq!(
            usage.requests_today, 0,
            "New manager should have zero daily requests"
        );
        assert_eq!(
            usage.requests_this_month, 0,
            "New manager should have zero monthly requests"
        );
    }

    #[tokio::test]
    async fn test_vision_license_manager_default() {
        let manager = VisionLicenseManager::default();

        // Should behave same as new()
        let cache = manager.get_cached_license().await;
        assert!(cache.is_none(), "Default manager should have empty cache");
    }

    #[tokio::test]
    #[serial]
    async fn test_check_vision_access_missing_license() {
        let manager = VisionLicenseManager::new();
        let result = manager.check_vision_access(None).await;

        assert!(result.is_err(), "Missing license should return error");
        match result.unwrap_err() {
            VisionLicenseError::MissingLicense => {} // Expected
            other => panic!("Expected MissingLicense, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_record_usage_increments_counters() {
        let manager = VisionLicenseManager::new();

        // Record usage
        let result = manager.record_usage().await;
        assert!(result.is_ok(), "Recording usage should succeed");

        // Check that counters were incremented
        let usage = manager.get_usage_stats().await;
        assert_eq!(
            usage.requests_today, 1,
            "Daily counter should be incremented"
        );
        assert_eq!(
            usage.requests_this_month, 1,
            "Monthly counter should be incremented"
        );
    }

    #[tokio::test]
    async fn test_record_usage_multiple_calls() {
        let manager = VisionLicenseManager::new();

        // Record usage multiple times
        for _ in 0..5 {
            let result = manager.record_usage().await;
            assert!(result.is_ok(), "Recording usage should succeed");
        }

        // Check that counters accumulated
        let usage = manager.get_usage_stats().await;
        assert_eq!(usage.requests_today, 5, "Daily counter should accumulate");
        assert_eq!(
            usage.requests_this_month, 5,
            "Monthly counter should accumulate"
        );
    }

    #[test]
    fn test_license_validation_serialization() {
        // Test LicenseValidation struct serialization/deserialization
        let mut entitlements = HashMap::new();
        entitlements.insert("VISION_ANALYSIS".to_string(), serde_json::Value::Bool(true));
        entitlements.insert(
            "monthly_cap".to_string(),
            serde_json::Value::Number(1000.into()),
        );

        let mut meta = HashMap::new();
        meta.insert(
            "code".to_string(),
            serde_json::Value::String("VALID".to_string()),
        );

        let validation = LicenseValidation {
            valid: true,
            entitlements,
            expires_at: Some("2024-12-31T23:59:59Z".to_string()),
            meta,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&validation).unwrap();
        assert!(json.contains("\"valid\":true"));
        assert!(json.contains("VISION_ANALYSIS"));
        assert!(json.contains("monthly_cap"));

        // Deserialize back
        let deserialized: LicenseValidation = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.valid, validation.valid);
        assert_eq!(deserialized.expires_at, validation.expires_at);
    }

    #[test]
    fn test_cached_license_serialization() {
        // Test CachedLicense struct serialization/deserialization
        let validation = LicenseValidation {
            valid: true,
            entitlements: HashMap::new(),
            expires_at: None,
            meta: HashMap::new(),
        };

        let cached = CachedLicense {
            key: "test-key".to_string(),
            validation,
            cached_at: chrono::Utc::now(),
            expires_at: None,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&cached).unwrap();
        assert!(json.contains("test-key"));
        assert!(json.contains("cached_at"));

        // Deserialize back
        let deserialized: CachedLicense = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.key, "test-key");
    }

    #[test]
    fn test_usage_stats_serialization() {
        // Test UsageStats struct serialization/deserialization
        let stats = UsageStats {
            requests_today: 42,
            requests_this_month: 1337,
            last_reset: chrono::Utc::now(),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"requests_today\":42"));
        assert!(json.contains("\"requests_this_month\":1337"));

        // Deserialize back
        let deserialized: UsageStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.requests_today, 42);
        assert_eq!(deserialized.requests_this_month, 1337);
    }

    // Property-based testing for date handling edge cases
    #[tokio::test]
    async fn test_check_response_freshness_property_based_recent_times() {
        // Test various times within the last 5 minutes (should all be valid)
        for seconds_ago in [0, 30, 60, 120, 240, 299] {
            // Up to 4 min 59 sec ago
            let time = Utc::now() - Duration::seconds(seconds_ago);
            let date_header = time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

            let result = VisionLicenseManagerTestWrapper::check_response_freshness(&date_header);
            assert!(
                result.is_ok(),
                "Time {} seconds ago should be valid",
                seconds_ago
            );
        }
    }

    #[tokio::test]
    async fn test_check_response_freshness_property_based_stale_times() {
        // Test various times older than 5 minutes (should all be stale)
        for minutes_ago in [6, 10, 30, 60, 120] {
            let time = Utc::now() - Duration::minutes(minutes_ago);
            let date_header = time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

            let result = VisionLicenseManagerTestWrapper::check_response_freshness(&date_header);
            assert!(
                result.is_err(),
                "Time {} minutes ago should be stale",
                minutes_ago
            );
        }
    }

    #[tokio::test]
    async fn test_check_response_freshness_property_based_future_times() {
        // Test various future times within tolerance (should be valid)
        for seconds_future in [1, 30, 59] {
            // Up to 59 seconds future
            let time = Utc::now() + Duration::seconds(seconds_future);
            let date_header = time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

            let result = VisionLicenseManagerTestWrapper::check_response_freshness(&date_header);
            assert!(
                result.is_ok(),
                "Time {} seconds in future should be valid",
                seconds_future
            );
        }

        // Test times beyond tolerance (should be invalid)
        for minutes_future in [2, 5, 10] {
            let time = Utc::now() + Duration::minutes(minutes_future);
            let date_header = time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

            let result = VisionLicenseManagerTestWrapper::check_response_freshness(&date_header);
            assert!(
                result.is_err(),
                "Time {} minutes in future should be invalid",
                minutes_future
            );
        }
    }

    // Edge case testing for malformed signature headers
    #[tokio::test]
    async fn test_verify_response_signature_malformed_headers_variants() {
        let date_header = "Wed, 09 Jun 2021 16:08:15 GMT";
        let response_body = r#"{"meta":{"valid":true}}"#;

        let malformed_headers = vec![
            "",                                                         // Empty header
            "invalid",                                                  // No structure
            "signature=test",      // Missing quotes and other fields
            r#"signature="test""#, // Only signature field
            r#"keyid="test", algorithm="ed25519""#, // Missing signature
            r#"keyid=test, algorithm="ed25519", signature="dGVzdA==""#, // Mixed quote styles
            "keyid=\"test\", algorithm=\"ed25519\", signature=\"\"", // Empty signature
        ];

        for (i, header) in malformed_headers.iter().enumerate() {
            let result = VisionLicenseManagerTestWrapper::verify_response_signature(
                header,
                date_header,
                response_body,
            );
            assert!(
                result.is_err(),
                "Malformed header {} should return error: {}",
                i,
                header
            );
        }
    }

    // Test error message formats and content
    #[tokio::test]
    async fn test_error_messages_content() {
        // Test that error messages contain expected security warnings
        let stale_time = Utc::now() - Duration::minutes(10);
        let date_header = stale_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let result = VisionLicenseManagerTestWrapper::check_response_freshness(&date_header);
        let error_msg = result.unwrap_err().to_string();

        // Verify security warning content
        assert!(error_msg.contains("SECURITY WARNING"));
        assert!(error_msg.contains("too old"));
        assert!(error_msg.contains("Possible replay attack"));
        assert!(error_msg.contains(&date_header)); // Should include the actual date for debugging
    }

    #[test]
    fn test_vision_license_error_display() {
        // Test that error display formats are correct
        let errors = vec![
            VisionLicenseError::MissingLicense,
            VisionLicenseError::ValidationFailed("test error".to_string()),
            VisionLicenseError::InvalidLicense,
            VisionLicenseError::FeatureNotEnabled,
            VisionLicenseError::UsageLimitExceeded,
        ];

        for error in errors {
            let display = format!("{}", error);
            let debug = format!("{:?}", error);

            // All errors should have non-empty display and debug representations
            assert!(!display.is_empty(), "Error display should not be empty");
            assert!(!debug.is_empty(), "Error debug should not be empty");

            // Display should be user-friendly, debug should be technical
            assert!(display.len() > 5, "Error display should be descriptive");
        }
    }

    // =========================================================================
    // SECURITY REGRESSION TESTS - Attack Scenario Coverage
    // =========================================================================

    /// Regression test: Replay attack with captured old response
    /// An attacker captures a valid response and replays it later to bypass revocation
    #[tokio::test]
    async fn test_security_replay_attack_with_captured_response() {
        // Simulate a response captured 10 minutes ago
        let captured_time = Utc::now() - Duration::minutes(10);
        let date_header = captured_time
            .format("%a, %d %b %Y %H:%M:%S GMT")
            .to_string();

        let result = VisionLicenseManagerTestWrapper::check_response_freshness(&date_header);

        assert!(result.is_err(), "Replayed response should be rejected");
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("replay attack"),
            "Error should mention replay attack"
        );
    }

    /// Regression test: MITM attack modifying response body
    /// An attacker intercepts the response and changes "valid":false to "valid":true
    #[tokio::test]
    async fn test_security_mitm_attack_modified_body() {
        // Create a forged "valid" signature with zero bytes (attacker doesn't have private key)
        let forged_sig = STANDARD.encode(&[0u8; 64]);
        let sig_header = format!(
            r#"keyid="test", algorithm="ed25519", signature="{}", headers="(request-target) host date digest""#,
            forged_sig
        );
        let date_header = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        // Attacker modified body to show valid:true
        let tampered_body = r#"{"meta":{"valid":true}}"#;

        let result = VisionLicenseManagerTestWrapper::verify_response_signature(
            &sig_header,
            &date_header,
            tampered_body,
        );

        assert!(
            result.is_err(),
            "Modified response should fail signature verification"
        );
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("MITM attack"),
            "Error should mention MITM attack"
        );
    }

    /// Regression test: Signature algorithm downgrade attack
    /// Attacker tries to force use of weaker algorithm
    #[tokio::test]
    async fn test_security_algorithm_downgrade_attack() {
        // Attacker tries RSA-SHA256 (easier to forge with stolen cert)
        let weak_algorithms = ["rsa-sha256", "hmac-sha256", "sha256", "md5", "none"];
        let date_header = "Wed, 09 Jun 2021 16:08:15 GMT";
        let response_body = r#"{"meta":{"valid":true}}"#;

        for algo in weak_algorithms {
            let sig_header = format!(
                r#"keyid="test", algorithm="{}", signature="dGVzdHNpZw==", headers="date""#,
                algo
            );

            let result = VisionLicenseManagerTestWrapper::verify_response_signature(
                &sig_header,
                date_header,
                response_body,
            );

            assert!(result.is_err(), "Algorithm {} should be rejected", algo);
            let error = result.unwrap_err().to_string();
            assert!(
                error.contains("Unsupported signature algorithm") || error.contains("ed25519"),
                "Error should reject non-Ed25519 algorithm: {}",
                algo
            );
        }
    }

    /// Regression test: Clock tampering attack
    /// Attacker sets Date header far in the future to extend validity
    #[tokio::test]
    async fn test_security_clock_tampering_attack() {
        // Attacker sets date 1 day in future to make old response "fresh"
        let future_time = Utc::now() + Duration::days(1);
        let date_header = future_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let result = VisionLicenseManagerTestWrapper::check_response_freshness(&date_header);

        assert!(result.is_err(), "Future-dated response should be rejected");
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("clock tampering"),
            "Error should mention clock tampering"
        );
    }

    /// Regression test: Empty or truncated signature attack
    /// Attacker provides empty/partial signature hoping for permissive handling
    #[tokio::test]
    async fn test_security_truncated_signature_attack() {
        let date_header = "Wed, 09 Jun 2021 16:08:15 GMT";
        let response_body = r#"{"meta":{"valid":true}}"#;

        // Pre-encode to avoid temporary lifetime issues
        let sig_32 = STANDARD.encode(&[0u8; 32]);
        let sig_63 = STANDARD.encode(&[0u8; 63]);
        let sig_65 = STANDARD.encode(&[0u8; 65]);

        let truncated_signatures = [
            "",              // Empty
            "YQ==",          // 1 byte (too short)
            sig_32.as_str(), // 32 bytes (half length)
            sig_63.as_str(), // 63 bytes (1 byte short)
            sig_65.as_str(), // 65 bytes (1 byte over)
        ];

        for sig in truncated_signatures {
            let sig_header = format!(
                r#"keyid="test", algorithm="ed25519", signature="{}", headers="date""#,
                sig
            );

            let result = VisionLicenseManagerTestWrapper::verify_response_signature(
                &sig_header,
                date_header,
                response_body,
            );

            assert!(
                result.is_err(),
                "Truncated signature '{}' should be rejected",
                sig
            );
        }
    }

    /// Regression test: Hard-coded account ID prevents key-swapping
    /// Ensures KEYGEN_ACCOUNT_ID constant matches expected value
    #[test]
    fn test_security_hardcoded_account_id() {
        // The account ID should be hard-coded, not from env
        // This prevents attackers from redirecting to their own Keygen account
        let expected_account_id = "6270bf9c-23ad-4483-9296-3a6d9178514a";
        assert_eq!(
            shimmy::vision_license::KEYGEN_ACCOUNT_ID,
            expected_account_id,
            "Account ID must be hard-coded to prevent key-swapping attacks"
        );
    }

    /// Regression test: Hard-coded public key prevents key substitution
    /// Ensures KEYGEN_PUBLIC_KEY constant matches expected value
    #[test]
    fn test_security_hardcoded_public_key() {
        // The Ed25519 public key should be hard-coded for signature verification
        let expected_key = "42f313585a72a41513208800f730944f1a3b74a8acfff539f96ce244d029fa5d";
        assert_eq!(
            shimmy::vision_license::KEYGEN_PUBLIC_KEY,
            expected_key,
            "Public key must be hard-coded to prevent MITM key substitution"
        );
    }
}

// Tests for when vision feature is disabled
#[cfg(not(feature = "vision"))]
mod vision_disabled_tests {
    use shimmy::vision_license::*;

    #[test]
    fn test_check_vision_license_disabled() {
        let result = check_vision_license(Some("test-key"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Vision feature not enabled");

        let result = check_vision_license(None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Vision feature not enabled");
    }
}
