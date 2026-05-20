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

        // v2.0: GPU support via Airframe/wgpu. No CUDA/Vulkan/MLX in CI release workflow.
        // Airframe is compiled locally and uploaded as release binaries.
        // Validate that Windows and macOS targets are still present.
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

        // Validate Airframe is referenced (v2.0 GPU engine)
        assert!(
            workflow_content.contains("airframe"),
            "Release workflow should reference airframe (v2.0 GPU engine)"
        );
    }

    #[test]
    fn test_release_workflow_platform_specific_features() {
        let workflow_path = ".github/workflows/release.yml";
        let workflow_content =
            fs::read_to_string(workflow_path).expect("Failed to read release workflow file");

        // v2.0: Release workflow builds for multiple platforms. Validate cross-platform targets.
        let has_multi_platform = workflow_content.contains("ubuntu-latest")
            && workflow_content.contains("windows-latest")
            && workflow_content.contains("macos-latest");

        assert!(
            has_multi_platform,
            "Release workflow should build for Linux, Windows, and macOS"
        );
    }

    #[test]
    fn test_readme_documents_gpu_support_in_releases() {
        let readme = fs::read_to_string("README.md").expect("Failed to read README.md");

        // v2.0: GPU support via Airframe/wgpu. Check for wgpu/WebGPU mentions.
        let mentions_gpu = readme.to_lowercase().contains("gpu")
            && (readme.to_lowercase().contains("wgpu")
                || readme.to_lowercase().contains("webgpu")
                || readme.to_lowercase().contains("airframe"));

        assert!(
            mentions_gpu,
            "README should document GPU support (Airframe/wgpu in v2.0)"
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
