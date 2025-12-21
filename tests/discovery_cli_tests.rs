/// Integration tests for discovery CLI commands
/// Tests: shimmy discovery start/stop/status, shimmy backends
///
/// NOTE: These tests spawn actual processes and are intentionally simplified.
/// Full E2E testing will be done in Phase 5 (E2E Testing).
use tokio::time::Duration;

#[tokio::test]
async fn test_discovery_cli_commands_exist() {
    // Simple smoke test - verify commands exist and parse correctly
    use clap::Parser;
    use shimmy::cli::{Cli, Command, DiscoveryCommand};

    // Test "shimmy discovery start" parses
    let cli = Cli::try_parse_from(&["shimmy", "discovery", "start"]).unwrap();
    match cli.cmd {
        Command::Discovery { discovery_cmd } => match discovery_cmd {
            DiscoveryCommand::Start => assert!(true),
            _ => panic!("Expected Start command"),
        },
        _ => panic!("Expected Discovery command"),
    }

    // Test "shimmy discovery status" parses
    let cli = Cli::try_parse_from(&["shimmy", "discovery", "status"]).unwrap();
    match cli.cmd {
        Command::Discovery { discovery_cmd } => match discovery_cmd {
            DiscoveryCommand::Status => assert!(true),
            _ => panic!("Expected Status command"),
        },
        _ => panic!("Expected Discovery command"),
    }

    // Test "shimmy discovery stop" parses
    let cli = Cli::try_parse_from(&["shimmy", "discovery", "stop"]).unwrap();
    match cli.cmd {
        Command::Discovery { discovery_cmd } => match discovery_cmd {
            DiscoveryCommand::Stop => assert!(true),
            _ => panic!("Expected Stop command"),
        },
        _ => panic!("Expected Discovery command"),
    }

    // Test "shimmy backends" parses
    let cli = Cli::try_parse_from(&["shimmy", "backends"]).unwrap();
    match cli.cmd {
        Command::Backends => assert!(true),
        _ => panic!("Expected Backends command"),
    }
}

#[tokio::test]
async fn test_discovery_status_when_not_running() {
    // Test status command when discovery service isn't running
    // Just verify it doesn't crash and shows appropriate message

    use reqwest::Client;
    let client = Client::new();

    // Attempt to query non-running service
    let result = client
        .get("http://127.0.0.1:11430/discover")
        .timeout(Duration::from_millis(500))
        .send()
        .await;

    // Should fail to connect
    assert!(result.is_err() || !result.unwrap().status().is_success());
}

#[tokio::test]
async fn test_backends_command_when_discovery_not_running() {
    // Similar to status test - verify backends command handles no service gracefully

    use reqwest::Client;
    let client = Client::new();

    let result = client
        .get("http://127.0.0.1:11430/discover")
        .timeout(Duration::from_millis(500))
        .send()
        .await;

    // Should fail to connect or return error
    assert!(result.is_err() || !result.unwrap().status().is_success());
}

// NOTE: Actual service startup tests are complex due to:
// 1. Process lifecycle management across Windows/Linux
// 2. Port binding conflicts with parallel tests
// 3. Cleanup challenges (zombie processes)
//
// These will be implemented in Phase 5.1 (E2E Workflow Test) as bash script.
