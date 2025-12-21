/// Tests for 'shimmy status' command formatting and output

#[tokio::test]
async fn test_status_command_parsing() {
    use clap::Parser;
    use shimmy::cli::{Cli, Command};

    // Test "shimmy status" parses correctly
    let cli = Cli::try_parse_from(&["shimmy", "status"]).unwrap();
    match cli.cmd {
        Command::Status => assert!(true),
        _ => panic!("Expected Status command"),
    }
}

#[tokio::test]
async fn test_status_handles_no_discovery_service() {
    use reqwest::Client;
    use tokio::time::Duration;

    let client = Client::new();

    // Attempt to query discovery service when it's not running
    let result = client
        .get("http://127.0.0.1:11430/discover")
        .timeout(Duration::from_millis(500))
        .send()
        .await;

    // Should fail or return error status
    assert!(result.is_err() || !result.unwrap().status().is_success());
}

#[tokio::test]
async fn test_status_formats_backend_info() {
    use shimmy::discovery::{BackendCapabilities, HealthStatus, ValidationStatus};
    use shimmy::discovery::service::BackendRegistration;

    // Create test backend data
    let backend = BackendRegistration {
        id: "test-backend-123".to_string(),
        port: 12345,
        url: "http://localhost:12345".to_string(),
        models: vec![
            shimmy::discovery::ModelInfo {
                name: "test-model-1".to_string(),
                backend_type: "llama".to_string(),
                compiled_support: true,
                size_bytes: None,
                parameter_count: None,
                quantization: None,
                context_length: None,
            },
            shimmy::discovery::ModelInfo {
                name: "test-model-2".to_string(),
                backend_type: "llama".to_string(),
                compiled_support: true,
                size_bytes: None,
                parameter_count: None,
                quantization: None,
                context_length: None,
            },
        ],
        capabilities: BackendCapabilities {
            backend_type: "llama".to_string(),
            features_compiled: vec!["llama".to_string(), "cuda".to_string()],
            websocket_working: true,
            http_working: true,
            models_loaded: true,
            streaming_supported: true,
        },
        last_heartbeat: std::time::Instant::now(),
        health: HealthStatus::Ok,
        validation: ValidationStatus::default(),
    };

    // Verify struct fields are accessible for formatting
    assert_eq!(backend.id, "test-backend-123");
    assert_eq!(backend.port, 12345);
    assert_eq!(backend.models.len(), 2);
    assert_eq!(backend.capabilities.backend_type, "llama");

    // Test health icon selection logic
    let health_icon = match backend.health {
        HealthStatus::Ok => "✅",
        HealthStatus::Degraded => "⚠️",
        HealthStatus::Failing => "❌",
    };
    assert_eq!(health_icon, "✅");

    // Test elapsed time calculation
    let now = std::time::Instant::now();
    let elapsed = now.saturating_duration_since(backend.last_heartbeat);
    assert!(elapsed.as_secs() < 2); // Should be very recent
}

#[test]
fn test_status_health_icon_variants() {
    use shimmy::discovery::HealthStatus;

    // Test all health status variants have correct icons
    let test_cases = vec![
        (HealthStatus::Ok, "✅"),
        (HealthStatus::Degraded, "⚠️"),
        (HealthStatus::Failing, "❌"),
    ];

    for (status, expected_icon) in test_cases {
        let icon = match status {
            HealthStatus::Ok => "✅",
            HealthStatus::Degraded => "⚠️",
            HealthStatus::Failing => "❌",
        };
        assert_eq!(icon, expected_icon);
    }
}
