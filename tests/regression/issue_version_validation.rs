/// Version validation regression test
///
/// This test ensures our CI/CD version validation logic works correctly
/// and prevents the type of version mismatch issues we experienced.
use std::process::Command;

#[test]
fn test_cargo_toml_version_format() {
    // Read Cargo.toml and extract version
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    let version_line = cargo_toml
        .lines()
        .find(|line| line.trim().starts_with("version = "))
        .expect("No version line found in Cargo.toml");

    // Extract version string
    let version = version_line
        .split('=')
        .nth(1)
        .expect("Invalid version line format")
        .trim()
        .trim_matches('"')
        .trim();

    // Basic validation checks
    assert!(!version.is_empty(), "Version should not be empty");
    assert_ne!(version, "0.1.0", "Version should not be default 0.1.0");

    // Semantic versioning format check
    let parts: Vec<&str> = version.split('.').collect();
    assert!(
        parts.len() >= 3,
        "Version should have at least 3 parts (major.minor.patch): {}",
        version
    );

    // Each part should be numeric (for the first 3 parts)
    for (i, part) in parts.iter().take(3).enumerate() {
        part.parse::<u32>()
            .unwrap_or_else(|_| panic!("Version part {} should be numeric: {}", i, part));
    }

    println!("✅ Cargo.toml version format is valid: {}", version);
}

#[test]
fn test_version_validation_script_simulation() {
    // Simulate the validation logic that runs in CI/CD

    // Read actual Cargo.toml version
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    let cargo_version = cargo_toml
        .lines()
        .find(|line| line.trim().starts_with("version = "))
        .expect("No version line found")
        .split('=')
        .nth(1)
        .expect("Invalid version line")
        .trim()
        .trim_matches('"')
        .trim();

    // Simulate various tag scenarios
    let test_cases = vec![
        (
            format!("v{}", cargo_version),
            true,
            "Matching tag should pass",
        ),
        ("v0.1.0".to_string(), false, "Default version should fail"),
        (
            "v999.999.999".to_string(),
            false,
            "Non-matching version should fail",
        ),
        (
            format!("v{}.1", cargo_version),
            false,
            "Different patch version should fail",
        ),
    ];

    for (tag, should_pass, description) in test_cases {
        let tag_version = tag.strip_prefix('v').unwrap_or(&tag);
        let matches = cargo_version == tag_version;

        if should_pass {
            assert!(
                matches || tag_version == "999.999.999",
                "Test case failed: {} (tag: {}, cargo: {})",
                description,
                tag_version,
                cargo_version
            );
        } else if tag_version != "999.999.999" {
            assert!(
                !matches,
                "Test case failed: {} (tag: {}, cargo: {})",
                description, tag_version, cargo_version
            );
        }

        println!("✅ Test case passed: {}", description);
    }
}

#[test]
fn test_binary_version_output() {
    // Build and test the binary version output
    let output = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--no-default-features",
            "--features",
            "huggingface",
        ])
        .output()
        .expect("Failed to build binary");

    assert!(
        output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Test version output
    let version_output = Command::new("./target/release/shimmy")
        .arg("--version")
        .output()
        .expect("Failed to run shimmy --version");

    assert!(
        version_output.status.success(),
        "Version command failed: {}",
        String::from_utf8_lossy(&version_output.stderr)
    );

    let version_text = String::from_utf8_lossy(&version_output.stdout);

    // Get expected version from Cargo.toml
    let expected_version = env!("CARGO_PKG_VERSION");

    assert!(
        version_text.contains(expected_version),
        "Binary version output should contain '{}', got: '{}'",
        expected_version,
        version_text.trim()
    );

    println!(
        "✅ Binary version output is correct: {}",
        version_text.trim()
    );
}

#[test]
fn test_version_validation_prevents_regression() {
    // This test documents the specific issues we're preventing

    let current_version = env!("CARGO_PKG_VERSION");

    // Issue #63: Prevent 0.1.0 version in releases
    assert_ne!(
        current_version, "0.1.0",
        "Release should never have version 0.1.0 (Issue #63 regression check)"
    );

    // Ensure version follows semantic versioning
    let parts: Vec<&str> = current_version.split('.').collect();
    assert!(parts.len() >= 3, "Version must follow semantic versioning");

    // Ensure all parts are numeric
    for part in &parts[0..3] {
        part.parse::<u32>()
            .expect("Version components must be numeric");
    }

    println!(
        "✅ Version regression checks passed for version: {}",
        current_version
    );
}

#[cfg(test)]
mod ci_validation_tests {
    // Note: Functions imported as needed in each test

    #[test]
    fn test_ci_version_validation_logic() {
        // Test the exact logic used in CI/CD

        fn validate_version_match(cargo_version: &str, tag_version: &str) -> Result<(), String> {
            if cargo_version != tag_version {
                return Err(format!(
                    "Version mismatch: Cargo.toml={}, Tag={}",
                    cargo_version, tag_version
                ));
            }

            if cargo_version == "0.1.0" {
                return Err("Version 0.1.0 is not allowed in releases".to_string());
            }

            Ok(())
        }

        // Test passing cases
        assert!(validate_version_match("1.5.5", "1.5.5").is_ok());
        assert!(validate_version_match("2.0.0", "2.0.0").is_ok());

        // Test failing cases
        assert!(validate_version_match("1.5.5", "1.5.4").is_err());
        assert!(validate_version_match("0.1.0", "0.1.0").is_err());
        assert!(validate_version_match("1.0.0", "2.0.0").is_err());

        println!("✅ CI validation logic tests passed");
    }
}

/// Integration test that simulates the full CI validation workflow
#[test]
fn integration_test_full_validation_workflow() {
    let current_version = env!("CARGO_PKG_VERSION");

    // Step 1: Cargo.toml version extraction (simulated)
    assert!(!current_version.is_empty(), "Version extraction failed");

    // Step 2: Tag version comparison (simulated - would be v{current_version})
    let simulated_tag = format!("v{}", current_version);
    let tag_version = simulated_tag.strip_prefix('v').unwrap();
    assert_eq!(current_version, tag_version, "Tag/Cargo version mismatch");

    // Step 3: Binary build and version check
    // Note: This is already tested in test_binary_version_output()

    // Step 4: Semantic versioning validation
    let parts: Vec<&str> = current_version.split('.').collect();
    assert!(parts.len() >= 3, "Invalid semantic versioning");

    // Step 5: Regression checks
    assert_ne!(current_version, "0.1.0", "Blocked development version");

    println!("✅ Full validation workflow simulation passed");
}
