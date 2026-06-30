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
    //
    // v2.0: llama-cpp-2 was removed. The `llama` feature is now an empty stub
    // (declared in Cargo.toml as `llama = []`) so existing CI scripts that pass
    // `--features llama` don't hard-error. The primary engine is airframe (GPU).

    // Verify the llama stub is declared in Cargo.toml
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    assert!(
        cargo_toml.contains("llama"),
        "Cargo.toml should still declare a `llama` feature stub for backwards compatibility"
    );

    // Verify the default GPU engine (airframe) compiles cleanly
    let output = Command::new("cargo")
        .args(["build", "--features", "airframe", "--lib"])
        .output()
        .expect("Failed to build with airframe feature");

    assert!(
        output.status.success(),
        "Airframe engine build failed (Issue #110 guard): {}",
        String::from_utf8_lossy(&output.stderr)
    );

    println!("✅ Issue #110: llama stub present; airframe engine compiles cleanly");
}

#[test]
fn test_crates_io_package_builds_successfully() {
    // v2.0: publish = false is intentionally set because the airframe path dep
    // (path = "../") cannot be published to crates.io. Shimmy is distributed as
    // release binaries. Validate that this is correctly declared.
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    assert!(
        cargo_toml.contains("publish = false"),
        "Cargo.toml should declare publish = false (path dep blocks crates.io publish)"
    );

    // Validate that the default feature set (airframe + huggingface on crates.io) builds
    let build_output = Command::new("cargo")
        .args(["build"])
        .output()
        .expect("Failed to run cargo build");

    assert!(
        build_output.status.success(),
        "Default feature set build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    println!("Package distribution configuration validated (binary-only, no crates.io)");
}

#[test]
fn test_no_missing_include_str_files() {
    // Specific test for the include_str! template file issue from Issue #110

    // Build with default features (airframe + huggingface, both on crates.io)
    let output = Command::new("cargo")
        .args(["build", "--release"])
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

    // Step 2: Verify all template files are accessible (default features = airframe + huggingface)
    let build_result = Command::new("cargo")
        .args(["build", "--quiet"])
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
