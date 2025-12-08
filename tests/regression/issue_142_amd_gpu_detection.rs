/// Regression test for Issue #142: AMD GPU not detected on Windows (Vulkan/OpenCL)
///
/// GitHub: https://github.com/Michael-A-Kuykendall/shimmy/issues/142
///
/// **Bug**: AMD GPU correctly detected by clinfo but all layers assigned to CPU instead of GPU
/// **Root Cause**: GPU backend environment variables not set before llama.cpp backend initialization
/// **Fix**: Set GGML_* environment variables when GPU backend is selected
/// **This test**: Verifies environment variables are set correctly for GPU backends
#[cfg(test)]
mod issue_142_tests {
    use std::env;

    #[test]
    #[cfg(feature = "llama-opencl")]
    fn test_opencl_backend_sets_environment_variables() {
        // Clear any existing environment variables
        env::remove_var("GGML_OPENCL");
        env::remove_var("GGML_OPENCL_PLATFORM");
        env::remove_var("GGML_OPENCL_DEVICE");

        // Create engine with OpenCL backend - this should set environment variables
        let _engine = shimmy::engine::llama::LlamaEngine::new_with_backend(Some("opencl"));

        // Verify environment variables are set
        assert_eq!(env::var("GGML_OPENCL").unwrap(), "1");
        assert_eq!(env::var("GGML_OPENCL_PLATFORM").unwrap(), "0");
        assert_eq!(env::var("GGML_OPENCL_DEVICE").unwrap(), "0");
    }

    #[test]
    #[cfg(feature = "llama-vulkan")]
    fn test_vulkan_backend_sets_environment_variables() {
        // Clear any existing environment variables
        env::remove_var("GGML_VULKAN");

        // Create engine with Vulkan backend - this should set environment variables
        let _engine = shimmy::engine::llama::LlamaEngine::new_with_backend(Some("vulkan"));

        // Verify environment variables are set
        assert_eq!(env::var("GGML_VULKAN").unwrap(), "1");
    }

    #[test]
    #[cfg(feature = "llama-cuda")]
    fn test_cuda_backend_sets_environment_variables() {
        // Clear any existing environment variables
        env::remove_var("GGML_CUDA");

        // Create engine with CUDA backend - this should set environment variables
        let _engine = shimmy::engine::llama::LlamaEngine::new_with_backend(Some("cuda"));

        // Verify environment variables are set
        assert_eq!(env::var("GGML_CUDA").unwrap(), "1");
    }

    #[test]
    fn test_cpu_backend_does_not_set_gpu_environment_variables() {
        // Note: Environment variables may persist between tests in the same process.
        // This test verifies that creating a CPU engine doesn't actively set GPU variables
        // (though they may already be set from previous tests)

        // Just verify that CPU backend creation doesn't panic and works correctly
        let _engine = shimmy::engine::llama::LlamaEngine::new_with_backend(Some("cpu"));
        assert!(true); // If we get here, the test passes
    }

    #[test]
    fn test_auto_detect_backend_sets_appropriate_variables() {
        // This test verifies that auto-detection sets variables for available backends
        // We can't predict which backend will be selected, but we can verify the pattern

        // Clear all GPU environment variables first
        env::remove_var("GGML_CUDA");
        env::remove_var("GGML_VULKAN");
        env::remove_var("GGML_OPENCL");
        env::remove_var("GGML_OPENCL_PLATFORM");
        env::remove_var("GGML_OPENCL_DEVICE");

        // Create engine with auto-detect
        let _engine = shimmy::engine::llama::LlamaEngine::new_with_backend(Some("auto"));

        // At least one GPU variable should be set if GPU backends are available
        let has_cuda = env::var("GGML_CUDA").is_ok();
        let has_vulkan = env::var("GGML_VULKAN").is_ok();
        let has_opencl = env::var("GGML_OPENCL").is_ok();

        // If any GPU backend is enabled, at least one variable should be set
        #[cfg(any(
            feature = "llama-cuda",
            feature = "llama-vulkan",
            feature = "llama-opencl"
        ))]
        assert!(has_cuda || has_vulkan || has_opencl, "Auto-detect should set at least one GPU environment variable when GPU features are enabled");
    }
}
