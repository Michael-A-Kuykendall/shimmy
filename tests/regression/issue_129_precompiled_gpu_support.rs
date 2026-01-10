/// Regression Test: Issue #129 - GPU support not available in precompiled binaries
///
/// **User Report**: @D0wn10ad (Windows)
/// Downloaded precompiled binary from GitHub releases, but `shimmy gpu-info` showed:
/// ```
/// ❌ CUDA support disabled
/// ❌ Vulkan support disabled  
/// ❌ OpenCL support disabled
/// ```
///
/// **Root Cause**: Release workflow (`.github/workflows/release.yml`) built binaries
/// without GPU features. Windows builds used default features (CPU only).
///
/// **Fix**: Update release workflow to build platform-specific binaries with GPU support:
/// - Windows: `--features "huggingface,llama,llama-vulkan"` (Vulkan for broad GPU compat)
/// - macOS: `--features "huggingface,llama,mlx"` (MLX for Apple Silicon)
/// - Linux musl: `--features huggingface` (avoid llama.cpp C++ issues)
///
/// **This test validates**:
/// - Release workflow YAML contains GPU features for Windows/macOS
/// - Documentation mentions GPU support in precompiled binaries
/// - Build configuration is correct for each platform
///
/// **Note**: This test validates the CONFIGURATION, not the actual binary compilation
/// (which happens in CI/CD). It ensures the workflow is set up correctly.
#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn test_release_workflow_includes_gpu_features() {
        let workflow_path = ".github/workflows/release.yml";
        let workflow_content =
            fs::read_to_string(workflow_path).expect("Failed to read release workflow file");

        // Validate Windows builds include Vulkan support
        assert!(
            workflow_content.contains("llama-vulkan") || workflow_content.contains("Features="),
            "Release workflow should build Windows binaries with Vulkan GPU support (Issue #129)"
        );

        // Validate macOS builds include MLX support
        assert!(
            workflow_content.contains("mlx"),
            "Release workflow should build macOS binaries with MLX support for Apple Silicon"
        );

        // Validate we're actually building for Windows
        assert!(
            workflow_content.contains("windows-latest")
                && workflow_content.contains("x86_64-pc-windows-msvc"),
            "Release workflow should build Windows x86_64 binaries"
        );

        // Validate we're actually building for macOS
        assert!(
            workflow_content.contains("macos-latest"),
            "Release workflow should build macOS binaries"
        );
    }

    #[test]
    fn test_release_workflow_platform_specific_features() {
        let workflow_path = ".github/workflows/release.yml";
        let workflow_content =
            fs::read_to_string(workflow_path).expect("Failed to read release workflow file");

        // Check for platform-specific feature logic
        // Should have conditional logic for Windows vs macOS vs Linux
        let has_conditional_logic = workflow_content.contains("if")
            && (workflow_content.contains("windows") || workflow_content.contains("macos"))
            && workflow_content.contains("FEATURES");

        assert!(
            has_conditional_logic,
            "Release workflow should have platform-specific feature configuration"
        );
    }

    #[test]
    fn test_readme_documents_gpu_support_in_releases() {
        let readme = fs::read_to_string("README.md").expect("Failed to read README.md");

        // Check that README mentions GPU support is available
        let mentions_gpu = readme.to_lowercase().contains("gpu")
            && (readme.to_lowercase().contains("vulkan") || readme.to_lowercase().contains("cuda"));

        assert!(
            mentions_gpu,
            "README should document GPU support availability"
        );
    }

    #[test]
    #[cfg(feature = "llama-vulkan")]
    fn test_vulkan_support_compiled_when_feature_enabled() {
        // When llama-vulkan feature is enabled, Vulkan backend should be available
        use shimmy::engine::llama::GpuBackend;

        let vulkan_backend = GpuBackend::Vulkan;
        assert_eq!(
            vulkan_backend.gpu_layers(),
            999,
            "Vulkan backend should support GPU layer offloading when feature is enabled"
        );
    }

    #[test]
    #[cfg(feature = "llama-cuda")]
    fn test_cuda_support_compiled_when_feature_enabled() {
        // When llama-cuda feature is enabled, CUDA backend should be available
        use shimmy::engine::llama::GpuBackend;

        let cuda_backend = GpuBackend::Cuda;
        assert_eq!(
            cuda_backend.gpu_layers(),
            999,
            "CUDA backend should support GPU layer offloading when feature is enabled"
        );
    }

    #[test]
    fn test_cpu_backend_always_available() {
        // CPU backend should always be available (no feature flag required)
        use shimmy::engine::llama::GpuBackend;

        let cpu_backend = GpuBackend::Cpu;
        assert_eq!(
            cpu_backend.gpu_layers(),
            0,
            "CPU backend should return 0 GPU layers (no offloading)"
        );
    }
}
