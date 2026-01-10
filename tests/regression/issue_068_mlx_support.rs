/// MLX support regression test for Issue #68
///
/// This test ensures that MLX support is properly compiled into macOS builds
/// and prevents regression of the issue where macOS binaries were missing MLX features.
use std::process::Command;

#[test]
fn test_mlx_feature_compilation() {
    // Test that MLX feature can be compiled
    let output = Command::new("cargo")
        .args(["check", "--no-default-features", "--features", "mlx"])
        .output()
        .expect("Failed to run cargo check with mlx feature");

    assert!(
        output.status.success(),
        "MLX feature should compile successfully: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    println!("✅ MLX feature compilation test passed");
}

#[test]
fn test_apple_feature_set_compilation() {
    // Test that the 'apple' feature set (which includes MLX) compiles

    // Skip on non-macOS platforms since MLX is Apple-specific
    if !cfg!(target_os = "macos") {
        println!("ℹ️ Skipping apple feature compilation test on non-macOS platform");
        return;
    }

    let output = Command::new("cargo")
        .args(["check", "--no-default-features", "--features", "apple"])
        .output()
        .expect("Failed to run cargo check with apple feature set");

    assert!(
        output.status.success(),
        "Apple feature set should compile successfully: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    println!("✅ Apple feature set compilation test passed");
}

#[test]
fn test_gpu_info_with_mlx_compiled() {
    // Skip on non-macOS platforms since MLX is Apple-specific
    if !cfg!(target_os = "macos") {
        println!("ℹ️ Skipping MLX GPU info test on non-macOS platform");
        return;
    }

    // Build with apple features and test gpu-info output (debug build for speed)
    let build_output = Command::new("cargo")
        .args(["build", "--no-default-features", "--features", "apple"])
        .output()
        .expect("Failed to build with apple features");

    assert!(
        build_output.status.success(),
        "Build with apple features should succeed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    // Test gpu-info command
    let gpu_info_output = Command::new("./target/debug/shimmy")
        .arg("gpu-info")
        .output()
        .expect("Failed to run shimmy gpu-info");

    assert!(
        gpu_info_output.status.success(),
        "gpu-info command should succeed: {}",
        String::from_utf8_lossy(&gpu_info_output.stderr)
    );

    let output_text = String::from_utf8_lossy(&gpu_info_output.stdout);

    // MLX should be compiled in (not showing "Disabled (compile with --features mlx)")
    assert!(
        !output_text.contains("MLX Backend: Disabled"),
        "MLX should not show as disabled when compiled with apple features"
    );

    // Should show either "Not available (requires Apple Silicon)" or actual MLX info
    assert!(
        output_text.contains("MLX Backend:"),
        "MLX Backend information should be present in gpu-info output"
    );

    // Should NOT show the old error message
    assert!(
        !output_text.contains("compile with --features mlx"),
        "Should not suggest compiling with MLX when it's already compiled in"
    );

    println!("✅ GPU info with MLX test passed");
    println!("MLX output: {}", output_text);
}

#[test]
fn test_mlx_feature_availability() {
    // Test that MLX code is available when feature is enabled
    #[cfg(feature = "mlx")]
    {
        // This code should only compile when MLX feature is enabled
        use shimmy::engine::mlx::MLXEngine;

        // Test that MLX engine type exists and can be referenced
        let _engine_check = std::marker::PhantomData::<MLXEngine>;

        println!("✅ MLX engine code is available when feature is enabled");
    }

    #[cfg(not(feature = "mlx"))]
    {
        println!("ℹ️  MLX feature not enabled in this test build");
    }
}

#[test]
fn test_regression_issue_68_scenarios() {
    // Test the specific scenarios reported in Issue #68

    // Scenario 1: cargo install with MLX should work (template compilation)
    // We test compilation rather than full install for speed
    let mlx_compile_test = Command::new("cargo")
        .args(["check", "--features", "mlx"])
        .output()
        .expect("Failed to check MLX compilation");

    assert!(
        mlx_compile_test.status.success(),
        "Issue #68 regression: MLX compilation should work (template files must be included)"
    );

    // Scenario 2: macOS binaries should include MLX support
    // Test that apple feature set includes MLX
    let apple_features_test = Command::new("cargo")
        .args(["check", "--no-default-features", "--features", "apple"])
        .output()
        .expect("Failed to check apple features");

    assert!(
        apple_features_test.status.success(),
        "Issue #68 regression: Apple feature set should compile with MLX support"
    );

    println!("✅ Issue #68 regression tests passed");
}

#[test]
fn test_ci_build_matrix_features() {
    // Test the feature combinations used in the CI build matrix

    // Test Linux features
    let linux_test = Command::new("cargo")
        .args([
            "check",
            "--no-default-features",
            "--features",
            "huggingface,llama",
        ])
        .output()
        .expect("Failed to check Linux features");

    assert!(
        linux_test.status.success(),
        "Linux feature set should compile"
    );

    // Test macOS features (the fix for Issue #68)
    let macos_test = Command::new("cargo")
        .args(["check", "--no-default-features", "--features", "apple"])
        .output()
        .expect("Failed to check macOS features");

    assert!(
        macos_test.status.success(),
        "macOS feature set should compile"
    );

    println!("✅ CI build matrix feature combinations test passed");
}

#[test]
fn test_mlx_status_messages_comprehensive() {
    // Test the comprehensive MLX status message behavior

    // Test apple feature set includes MLX
    let apple_test = Command::new("cargo")
        .args(["check", "--no-default-features", "--features", "apple"])
        .output()
        .expect("Failed to check apple features");

    assert!(
        apple_test.status.success(),
        "Apple feature set should compile with MLX support"
    );

    // Test that mlx feature alone works
    let mlx_only_test = Command::new("cargo")
        .args(["check", "--no-default-features", "--features", "mlx"])
        .output()
        .expect("Failed to check mlx feature alone");

    assert!(
        mlx_only_test.status.success(),
        "MLX feature should compile standalone"
    );

    println!("✅ MLX feature compilation tests comprehensive check passed");
}

#[test]
fn test_mlx_binary_status_messages() {
    // Skip on non-macOS platforms since MLX is Apple-specific
    if !cfg!(target_os = "macos") {
        println!("ℹ️ Skipping MLX binary status test on non-macOS platform");
        return;
    }

    // Build binary with apple features (includes MLX) - debug build for speed
    let build_output = Command::new("cargo")
        .args(["build", "--no-default-features", "--features", "apple"])
        .output()
        .expect("Failed to build with apple features");

    assert!(
        build_output.status.success(),
        "Build with apple features should succeed"
    );

    // Test the gpu-info command output for specific MLX status messages
    let gpu_info_output = Command::new("./target/debug/shimmy")
        .arg("gpu-info")
        .output()
        .expect("Failed to run shimmy gpu-info");

    assert!(
        gpu_info_output.status.success(),
        "gpu-info command should succeed"
    );

    let output_text = String::from_utf8_lossy(&gpu_info_output.stdout);

    // MLX should be compiled in (not showing "Disabled")
    assert!(
        !output_text.contains("MLX Backend: Disabled"),
        "MLX should not show as disabled when compiled with apple features: {}",
        output_text
    );

    // Should show either "Not available (requires Apple Silicon)" or actual MLX info
    assert!(
        output_text.contains("MLX Backend:"),
        "MLX Backend information should be present in gpu-info output: {}",
        output_text
    );

    // Should NOT show the old error message
    assert!(
        !output_text.contains("compile with --features mlx"),
        "Should not suggest compiling with MLX when it's already compiled in: {}",
        output_text
    );

    // Verify the specific expected message for non-Apple Silicon
    if !cfg!(target_arch = "aarch64") || !cfg!(target_os = "macos") {
        assert!(
            output_text.contains("Not supported (requires Apple Silicon macOS)")
                || output_text.contains("MLX Backend: Available"),
            "Should show proper Apple Silicon requirement message: {}",
            output_text
        );
    }

    println!("✅ MLX binary status messages test passed");
}

#[test]
fn test_mlx_regression_prevention() {
    // Comprehensive test to prevent MLX regression issues

    // 1. Test that apple feature set is correctly defined
    let cargo_toml =
        std::fs::read_to_string("Cargo.toml").expect("Should be able to read Cargo.toml");

    assert!(
        cargo_toml.contains("apple = [\"huggingface\", \"mlx\"]"),
        "Apple feature set should include MLX in Cargo.toml"
    );

    // 2. Test release workflow includes macOS targets
    let workflow_file = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Should be able to read release workflow");

    assert!(
        workflow_file.contains("aarch64-apple-darwin")
            && workflow_file.contains("x86_64-apple-darwin"),
        "Release workflow should include macOS targets (both Intel and ARM64)"
    );

    // 3. Test that MLX feature compiles without errors
    let mlx_check = Command::new("cargo")
        .args(["check", "--features", "mlx"])
        .output()
        .expect("Failed to run cargo check with mlx");

    assert!(
        mlx_check.status.success(),
        "MLX feature should compile successfully: {}",
        String::from_utf8_lossy(&mlx_check.stderr)
    );

    println!("✅ MLX regression prevention test passed");
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_apple_feature_build_and_run() {
        // Skip on non-macOS platforms since MLX is Apple-specific
        if !cfg!(target_os = "macos") {
            println!("ℹ️ Skipping full Apple feature test on non-macOS platform");
            return;
        }

        // Full integration test: build and run with apple features (debug for speed)
        let build_result = Command::new("cargo")
            .args(["build", "--no-default-features", "--features", "apple"])
            .output()
            .expect("Failed to build with apple features");

        assert!(
            build_result.status.success(),
            "Full apple feature build should succeed"
        );

        // Test that the binary works
        let version_result = Command::new("./target/debug/shimmy")
            .arg("--version")
            .output()
            .expect("Failed to run shimmy --version");

        assert!(
            version_result.status.success(),
            "shimmy --version should work with apple features"
        );

        let version_output = String::from_utf8_lossy(&version_result.stdout);
        assert!(
            version_output.contains("shimmy"),
            "Version output should contain 'shimmy': {}",
            version_output
        );

        println!("✅ Full apple feature integration test passed");
    }
}
