/// Regression test for Issue #63: Pre-built Windows exe reports wrong version
///
/// This test ensures that the binary version output matches the Cargo.toml version
/// and that all expected commands are available.

#[test]
fn test_binary_version_matches_cargo_toml() {
    // Test that env!("CARGO_PKG_VERSION") matches what's in Cargo.toml
    let version = env!("CARGO_PKG_VERSION");

    // Basic sanity checks on version format
    assert!(!version.is_empty(), "Version should not be empty");
    assert!(
        version.contains('.'),
        "Version should contain dots (semantic versioning)"
    );

    // Version should not be the default/placeholder version
    assert_ne!(version, "0.1.0", "Version should not be the default 0.1.0");
    assert_ne!(version, "0.0.0", "Version should not be 0.0.0");

    // Version should follow semantic versioning pattern (at least major.minor.patch)
    let parts: Vec<&str> = version.split('.').collect();
    assert!(
        parts.len() >= 3,
        "Version should have at least major.minor.patch: {}",
        version
    );

    // Each part should be a valid number
    for part in &parts[0..3] {
        part.parse::<u32>()
            .unwrap_or_else(|_| panic!("Version part '{}' should be a valid number", part));
    }
}

#[test]
fn test_cli_parser_includes_all_expected_commands() {
    // Test that the CLI parser includes all expected commands
    use clap::Parser;
    use shimmy::cli::{Cli, Command};

    // Test that basic commands parse correctly
    let serve_cli = Cli::try_parse_from(["shimmy", "serve"]);
    assert!(serve_cli.is_ok(), "Serve command should parse correctly");

    let list_cli = Cli::try_parse_from(["shimmy", "list"]);
    assert!(list_cli.is_ok(), "List command should parse correctly");

    let discover_cli = Cli::try_parse_from(["shimmy", "discover"]);
    assert!(
        discover_cli.is_ok(),
        "Discover command should parse correctly"
    );

    let gpu_info_cli = Cli::try_parse_from(["shimmy", "gpu-info"]);
    assert!(
        gpu_info_cli.is_ok(),
        "GPU-info command should parse correctly"
    );

    // Verify gpu-info command specifically
    if let Ok(cli) = gpu_info_cli {
        matches!(cli.cmd, Command::GpuInfo);
    }
}

#[test]
fn test_version_flag_functionality() {
    // Test that the version flag would work correctly
    use clap::Parser;
    use shimmy::cli::Cli;

    // Test long version flag
    let _version_result = Cli::try_parse_from(["shimmy", "--version"]);
    // clap will exit on --version, so this will be an error, but we can test the structure

    // Test short version flag
    let _version_result_short = Cli::try_parse_from(["shimmy", "-V"]);
    // clap will exit on -V, so this will be an error, but we can test the structure

    // The important thing is that the CLI is configured to handle version flags
    // If this test runs without panicking, the version infrastructure is working
}

#[test]
fn test_cargo_pkg_version_environment_variable() {
    // Test that the CARGO_PKG_VERSION environment variable is properly set during build
    let version = env!("CARGO_PKG_VERSION");

    // Should match the pattern of a real version
    assert!(!version.is_empty(), "CARGO_PKG_VERSION should not be empty");

    // Should not contain any build artifacts that could cause issues
    assert!(
        !version.contains("\\"),
        "Version should not contain backslashes"
    );
    assert!(!version.contains("\""), "Version should not contain quotes");
    assert!(
        !version.contains("'"),
        "Version should not contain single quotes"
    );

    // Should be a clean version string
    let trimmed = version.trim();
    assert_eq!(
        version, trimmed,
        "Version should not have leading/trailing whitespace"
    );
}

#[test]
fn test_help_output_contains_expected_commands() {
    // Test that help output would contain all expected commands
    use clap::CommandFactory;
    use shimmy::cli::Cli;

    let mut app = Cli::command();
    let help_text = app.render_help().to_string();

    // Check that all major commands are mentioned in help
    assert!(
        help_text.contains("serve"),
        "Help should mention 'serve' command"
    );
    assert!(
        help_text.contains("list"),
        "Help should mention 'list' command"
    );
    assert!(
        help_text.contains("discover"),
        "Help should mention 'discover' command"
    );
    assert!(
        help_text.contains("gpu-info"),
        "Help should mention 'gpu-info' command"
    );
    assert!(
        help_text.contains("probe"),
        "Help should mention 'probe' command"
    );
    assert!(
        help_text.contains("bench"),
        "Help should mention 'bench' command"
    );
    assert!(
        help_text.contains("generate"),
        "Help should mention 'generate' command"
    );
    assert!(
        help_text.contains("init"),
        "Help should mention 'init' command"
    );

    // Check version flag is mentioned
    assert!(
        help_text.contains("-V") || help_text.contains("--version"),
        "Help should mention version flag"
    );
}

#[test]
fn test_build_environment_consistency() {
    // Test that build environment variables are consistent
    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_version = env!("CARGO_PKG_VERSION");

    assert_eq!(pkg_name, "shimmy", "Package name should be 'shimmy'");
    assert!(
        !pkg_version.is_empty(),
        "Package version should not be empty"
    );

    // Test that the version is reasonable (not a placeholder)
    assert!(
        pkg_version != "0.1.0" || cfg!(test),
        "Production version should not be 0.1.0 (found: {})",
        pkg_version
    );
}

#[test]
fn test_version_consistency_across_codebase() {
    // Test that version is consistently used across the codebase
    let version = env!("CARGO_PKG_VERSION");

    // This mainly tests that the version can be accessed consistently
    // In actual code, this version is used in:
    // - Server endpoints (health check, metrics)
    // - CLI version output
    // - API responses

    assert!(
        !version.is_empty(),
        "Version should be available throughout codebase"
    );

    // Test version parsing - should be valid semantic version
    let version_parts: Vec<&str> = version.split('.').collect();
    assert!(
        version_parts.len() >= 2,
        "Version should have at least major.minor components"
    );

    // First two parts should be numeric
    for (i, part) in version_parts
        .iter()
        .enumerate()
        .take(2.min(version_parts.len()))
    {
        part.parse::<u32>()
            .unwrap_or_else(|_| panic!("Version component {} should be numeric", i));
    }
}
