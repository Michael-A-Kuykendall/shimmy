//! Focused unit tests for vision_license.rs core functions
//!
//! Testing the specific functions required:
//! - VisionLicenseError methods (public)
//! - VisionLicenseManager public methods

#[cfg(feature = "vision")]
mod tests {
    use serial_test::serial;
    use shimmy::vision_license::{VisionLicenseError, VisionLicenseManager};

    // Test VisionLicenseError::to_status_code() - all 5 variants
    #[test]
    fn test_all_error_status_codes() {
        assert_eq!(
            VisionLicenseError::MissingLicense.to_status_code(),
            axum::http::StatusCode::PAYMENT_REQUIRED
        );

        assert_eq!(
            VisionLicenseError::ValidationFailed("test".to_string()).to_status_code(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );

        assert_eq!(
            VisionLicenseError::InvalidLicense.to_status_code(),
            axum::http::StatusCode::FORBIDDEN
        );

        assert_eq!(
            VisionLicenseError::FeatureNotEnabled.to_status_code(),
            axum::http::StatusCode::FORBIDDEN
        );

        assert_eq!(
            VisionLicenseError::UsageLimitExceeded.to_status_code(),
            axum::http::StatusCode::PAYMENT_REQUIRED
        );
    }

    // Test VisionLicenseError::to_json_error() - all 5 variants
    #[test]
    fn test_all_error_json_serialization() {
        let missing = VisionLicenseError::MissingLicense.to_json_error();
        assert!(missing["error"].is_object());
        assert_eq!(missing["error"]["code"], "MISSING_LICENSE");
        assert_eq!(missing["error"]["message"], "No license key provided");

        let validation =
            VisionLicenseError::ValidationFailed("timeout".to_string()).to_json_error();
        assert!(validation["error"].is_object());
        assert_eq!(validation["error"]["code"], "VALIDATION_ERROR");
        assert_eq!(
            validation["error"]["message"],
            "License validation failed: timeout"
        );

        let invalid = VisionLicenseError::InvalidLicense.to_json_error();
        assert!(invalid["error"].is_object());
        assert_eq!(invalid["error"]["code"], "INVALID_LICENSE");
        assert_eq!(invalid["error"]["message"], "Invalid or expired license");

        let disabled = VisionLicenseError::FeatureNotEnabled.to_json_error();
        assert!(disabled["error"].is_object());
        assert_eq!(disabled["error"]["code"], "FEATURE_DISABLED");
        assert_eq!(
            disabled["error"]["message"],
            "Vision feature not enabled for this license"
        );

        let exceeded = VisionLicenseError::UsageLimitExceeded.to_json_error();
        assert!(exceeded["error"].is_object());
        assert_eq!(exceeded["error"]["code"], "USAGE_LIMIT_EXCEEDED");
        assert_eq!(exceeded["error"]["message"], "Monthly usage limit exceeded");
    }

    // Simple manager creation test
    #[tokio::test]
    async fn test_manager_creation() {
        let _manager = VisionLicenseManager::new();
        // We can't access private fields, but we can test that new() doesn't panic
        // Manager created successfully if we got here
    }

    // Test default implementation
    #[test]
    fn test_manager_default() {
        let _manager = VisionLicenseManager::default();
        // Test that default doesn't panic
        // Manager created successfully if we got here
    }

    // Test missing license error
    #[tokio::test]
    #[serial]
    async fn test_missing_license() {
        let manager = VisionLicenseManager::new();
        let result = manager.check_vision_access(None).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VisionLicenseError::MissingLicense
        ));
    }

    // Test record usage
    #[tokio::test]
    async fn test_record_usage() {
        let manager = VisionLicenseManager::new();

        // Should not panic or return error
        let result = manager.record_usage().await;
        assert!(result.is_ok());
    }

    // Test loading cache (should not fail even if no cache exists)
    #[tokio::test]
    async fn test_load_cache() {
        let manager = VisionLicenseManager::new();

        // Should not fail even if no cache file exists
        let result = manager.load_cache().await;
        assert!(result.is_ok());
    }
}

#[cfg(not(feature = "vision"))]
mod disabled_tests {
    #[test]
    fn test_vision_disabled() {
        let result = shimmy::vision_license::check_vision_license(Some("test-key"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Vision feature not enabled");
    }
}
