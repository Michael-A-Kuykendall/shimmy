/// Test to validate that distribution channels include MLX support
/// This test ensures Issue #114 is resolved

#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn test_cargo_install_mlx_syntax() {
        // Test that the cargo install command syntax is valid
        // This verifies users can run: cargo install shimmy --features mlx

        let output = Command::new("cargo").args(&["help", "install"]).output();

        assert!(output.is_ok(), "cargo install command should be available");

        // The command syntax should be valid (we can't test actual installation in CI)
        // But we can verify the features exist in our Cargo.toml
        let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml should exist");

        assert!(
            cargo_toml.contains("mlx = []"),
            "MLX feature should be defined"
        );
        assert!(
            cargo_toml.contains("llama-cuda"),
            "CUDA feature should be defined"
        );
        assert!(
            cargo_toml.contains("huggingface"),
            "HuggingFace feature should be defined"
        );
    }

    #[test]
    fn test_platform_specific_features() {
        // Test that platform-specific installation recommendations are valid
        let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml should exist");

        // Apple Silicon: MLX should be available
        assert!(
            cargo_toml.contains("mlx = []"),
            "MLX feature for Apple Silicon"
        );

        // NVIDIA: CUDA should be available
        assert!(cargo_toml.contains("llama-cuda"), "CUDA feature for NVIDIA");

        // Cross-platform: Vulkan should be available
        assert!(
            cargo_toml.contains("llama-vulkan"),
            "Vulkan feature for cross-platform"
        );

        // Fallback: HuggingFace should be available
        assert!(
            cargo_toml.contains("huggingface"),
            "HuggingFace fallback feature"
        );
    }

    #[test]
    fn test_release_workflow_includes_mlx() {
        // Test that the release workflow builds macOS binaries with MLX
        let workflow = std::fs::read_to_string(".github/workflows/release.yml")
            .expect("Release workflow should exist");

        // Should have MLX in the macOS build steps
        assert!(
            workflow.contains("mlx"),
            "Release workflow should include MLX features"
        );
        assert!(
            workflow.contains("aarch64-apple-darwin"),
            "Release should build Apple Silicon binaries"
        );
        assert!(
            workflow.contains("x86_64-apple-darwin"),
            "Release should build Intel Mac binaries"
        );
    }

    #[test]
    fn test_documentation_mentions_mlx() {
        // Test that documentation includes MLX installation instructions
        let readme = std::fs::read_to_string("README.md").expect("README.md should exist");

        assert!(
            readme.contains("--features mlx"),
            "README should mention MLX installation"
        );
        assert!(
            readme.contains("Apple Silicon"),
            "README should mention Apple Silicon"
        );
        assert!(
            readme.contains("cargo install shimmy --features mlx"),
            "README should have exact command"
        );
    }

    #[test]
    fn test_homebrew_formula_exists() {
        // Test that Homebrew formula template exists
        let formula_exists = std::path::Path::new("packaging/homebrew/shimmy.rb").exists();
        assert!(formula_exists, "Homebrew formula template should exist");

        if formula_exists {
            let formula = std::fs::read_to_string("packaging/homebrew/shimmy.rb")
                .expect("Should be able to read Homebrew formula");

            // Should reference macOS binaries
            assert!(formula.contains("macos"), "Formula should reference macOS");
            assert!(
                formula.contains("darwin"),
                "Formula should reference Darwin platform"
            );
        }
    }

    #[test]
    fn test_feature_flags_consistency() {
        // Test that feature flags are consistent and don't conflict
        let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml should exist");

        // Default features should be reasonable
        assert!(
            cargo_toml.contains("default = ["),
            "Should have default features"
        );

        // Platform features should be optional
        assert!(
            cargo_toml.contains("mlx = []"),
            "MLX should be optional feature"
        );
        assert!(
            cargo_toml.contains("llama-cuda"),
            "CUDA should be optional feature"
        );

        // Should have convenience feature sets
        assert!(
            cargo_toml.contains("gpu = ["),
            "Should have GPU convenience features"
        );
        assert!(
            cargo_toml.contains("apple = ["),
            "Should have Apple convenience features"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_mlx_feature_compiles_on_macos() {
        // On macOS, test that we can actually build with MLX features
        // This would catch compilation issues specific to MLX

        let output = Command::new("cargo")
            .args(&["check", "--features", "mlx"])
            .output();

        if let Ok(result) = output {
            if !result.status.success() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                panic!("MLX feature should compile on macOS: {}", stderr);
            }
        }
        // If cargo check fails for other reasons (missing dependencies), that's OK
        // This test is just to catch MLX-specific compilation issues
    }

    #[test]
    fn test_issue_114_documentation_fix() {
        // Verify that the fixes for Issue #114 are properly documented
        let readme = std::fs::read_to_string("README.md").expect("README.md should exist");

        // Should have platform-specific installation table
        assert!(
            readme.contains("Platform-Specific Installation"),
            "Should have platform-specific installation section"
        );
        assert!(
            readme.contains("Apple Silicon"),
            "Should mention Apple Silicon"
        );
        assert!(
            readme.contains("MLX Metal acceleration"),
            "Should mention MLX acceleration"
        );

        // Should have verification instructions
        assert!(
            readme.contains("shimmy gpu-info"),
            "Should show how to verify GPU support"
        );
        assert!(
            readme.contains("MLX Backend"),
            "Should mention MLX backend verification"
        );
    }
}
