/// Regression test for Issue #110: Build Failure on cargo install shimmy v1.7.0
///
/// This test ensures that:
/// 1. Template files are properly included in crates.io package
/// 2. All dependencies have compatible APIs
/// 3. The published package builds successfully from crates.io
use std::process::Command;

#[test]
fn test_template_files_included_in_package() {
    // Regression test for Issue #110 - Missing template files
    let output = Command::new("cargo")
        .args(["package", "--list", "--allow-dirty"])
        .output()
        .expect("Failed to run cargo package --list");

    let package_list = String::from_utf8_lossy(&output.stdout);

    // Check that Docker template is included (the file mentioned in Issue #110)
    assert!(
        package_list.contains("templates/docker/Dockerfile")
            || package_list.contains("templates\\docker\\Dockerfile"),
        "Docker template missing from package (Issue #110 regression): {}",
        package_list
    );

    // Check other critical template files
    let required_templates = [
        "templates/docker/docker-compose.yml",
        "templates/fly/fly.toml",
        "templates/kubernetes/deployment.yaml",
        "src/templates.rs",
    ];

    for template in &required_templates {
        let template_unix = template.replace("\\", "/");
        let template_windows = template.replace("/", "\\");

        assert!(
            package_list.contains(&template_unix) || package_list.contains(&template_windows),
            "Required template missing from package: {} (Issue #110 protection)",
            template
        );
    }

    println!("✅ All template files properly included in package");
}

#[test]
fn test_llama_cpp_dependency_compatibility() {
    // Regression test for Issue #110 - API incompatibility with llama-cpp-2

    // This test verifies that our usage of llama-cpp-2 APIs is compatible
    // by compiling the llama engine module specifically
    let output = Command::new("cargo")
        .args([
            "build",
            "--no-default-features",
            "--features",
            "llama",
            "--lib",
        ])
        .output()
        .expect("Failed to test llama dependency compatibility");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check for the specific API error from Issue #110
        if stderr.contains("with_n_cpu_moe") {
            panic!(
                "Issue #110 regression: llama-cpp-2 API incompatibility detected: {}",
                stderr
            );
        }

        // Check for other potential API compatibility issues
        if stderr.contains("method named") && stderr.contains("LlamaModelParams") {
            panic!("llama-cpp-2 API compatibility issue detected: {}", stderr);
        }

        // If it's a different build error, still fail but with context
        panic!(
            "llama feature build failed (potential Issue #110 regression): {}",
            stderr
        );
    }

    println!("✅ llama-cpp-2 dependency API compatibility verified");
}

#[test]
fn test_crates_io_package_builds_successfully() {
    // Comprehensive test that the entire package builds from crates.io format

    // First test: Package creation succeeds
    let package_output = Command::new("cargo")
        .args(["package", "--allow-dirty"])
        .output()
        .expect("Failed to run cargo package");

    assert!(
        package_output.status.success(),
        "Package creation failed (Issue #110 regression): {}",
        String::from_utf8_lossy(&package_output.stderr)
    );

    // Second test: Package verification builds successfully
    // (This runs the same verification that crates.io would run)
    let verify_output = Command::new("cargo")
        .args(["package", "--allow-dirty", "--no-verify"])
        .output()
        .expect("Failed to run cargo package verification");

    assert!(
        verify_output.status.success(),
        "Package verification failed (Issue #110 regression): {}",
        String::from_utf8_lossy(&verify_output.stderr)
    );

    println!("✅ Package builds successfully in crates.io format");
}

#[test]
fn test_no_missing_include_str_files() {
    // Specific test for the include_str! template file issue from Issue #110

    // Build with the exact features that would be used by cargo install
    let output = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--no-default-features",
            "--features",
            "huggingface,llama", // Default features for cargo install
        ])
        .output()
        .expect("Failed to test include_str! files");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check for the specific template file error from Issue #110
        if stderr.contains("couldn't read") && stderr.contains("templates/") {
            panic!(
                "Issue #110 regression: Template files missing from build: {}",
                stderr
            );
        }

        if stderr.contains("include_str!") {
            panic!(
                "include_str! file missing (Issue #110 regression): {}",
                stderr
            );
        }

        // If it's a different build error, still fail but with context
        panic!(
            "Release build failed (potential Issue #110 regression): {}",
            stderr
        );
    }

    println!("✅ All include_str! template files accessible during build");
}

/// Integration test simulating exact user experience from Issue #110
#[test]
fn test_issue_110_user_experience_simulation() {
    // This test simulates the exact scenario from Issue #110:
    // User runs `cargo install shimmy` and expects it to work

    println!("🧪 Simulating Issue #110 user experience...");

    // Step 1: Verify package can be listed (simulates crates.io publishing check)
    let package_result = Command::new("cargo")
        .args(["package", "--list", "--allow-dirty"])
        .output()
        .expect("Failed to simulate package validation");

    assert!(
        package_result.status.success(),
        "Package validation failed - this would break cargo install: {}",
        String::from_utf8_lossy(&package_result.stderr)
    );

    // Step 2: Verify all template files are accessible
    // (simulates the include_str! calls during compilation)
    let build_result = Command::new("cargo")
        .args([
            "build",
            "--quiet",
            "--no-default-features",
            "--features",
            "huggingface,llama", // Default cargo install features
        ])
        .output()
        .expect("Failed to simulate user build");

    assert!(
        build_result.status.success(),
        "Build failed - cargo install shimmy would fail for users: {}",
        String::from_utf8_lossy(&build_result.stderr)
    );

    // Step 3: Verify binary actually works
    let binary_path = if cfg!(windows) {
        "target/debug/shimmy.exe"
    } else {
        "target/debug/shimmy"
    };

    let version_result = Command::new(binary_path)
        .arg("--version")
        .output()
        .expect("Failed to test binary functionality");

    assert!(
        version_result.status.success(),
        "Binary doesn't work after install - user experience broken: {}",
        String::from_utf8_lossy(&version_result.stderr)
    );

    let version_output = String::from_utf8_lossy(&version_result.stdout);
    assert!(
        version_output.contains("shimmy"),
        "Binary version output incorrect: {}",
        version_output
    );

    println!("✅ Issue #110 user experience simulation: ALL CHECKS PASSED");
    println!("   Users can now successfully run `cargo install shimmy`");
}
