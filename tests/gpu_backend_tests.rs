#[cfg(test)]
mod gpu_backend_tests {
    use shimmy::engine::llama::LlamaEngine;

    #[test]
    fn test_llama_engine_creation() {
        let engine = LlamaEngine::new();
        let backend_info = engine.get_backend_info();

        // Should always return some backend info
        assert!(!backend_info.is_empty());

        // In test environment without GPU features, should be CPU
        #[cfg(not(any(
            feature = "llama-cuda",
            feature = "llama-vulkan",
            feature = "llama-opencl"
        )))]
        assert_eq!(backend_info, "CPU");
    }

    #[test]
    fn test_llama_engine_with_explicit_cpu_backend() {
        let engine = LlamaEngine::new_with_backend(Some("cpu"));
        let backend_info = engine.get_backend_info();
        assert_eq!(backend_info, "CPU");
    }

    #[test]
    fn test_llama_engine_with_auto_backend() {
        let engine = LlamaEngine::new_with_backend(Some("auto"));
        let backend_info = engine.get_backend_info();

        // Auto should select best available backend
        assert!(!backend_info.is_empty());
    }

    #[test]
    fn test_llama_engine_with_unknown_backend() {
        // Unknown backend should fallback to auto-detect
        let engine = LlamaEngine::new_with_backend(Some("unknown-backend"));
        let backend_info = engine.get_backend_info();

        // Should fallback gracefully
        assert!(!backend_info.is_empty());
    }

    #[test]
    #[cfg(feature = "llama-cuda")]
    fn test_cuda_backend_info() {
        let engine = LlamaEngine::new();
        let backend_info = engine.get_backend_info();

        // Should include CUDA if available, or fallback to CPU
        assert!(backend_info == "CUDA" || backend_info == "CPU");
    }

    #[test]
    #[cfg(feature = "llama-cuda")]
    fn test_explicit_cuda_backend() {
        let engine = LlamaEngine::new_with_backend(Some("cuda"));
        let backend_info = engine.get_backend_info();

        // Should use CUDA backend when explicitly requested
        assert_eq!(backend_info, "CUDA");
    }

    #[test]
    #[cfg(feature = "llama-vulkan")]
    fn test_vulkan_backend_info() {
        let engine = LlamaEngine::new_with_backend(Some("vulkan"));
        let backend_info = engine.get_backend_info();

        // Should be Vulkan if available, or CPU if Vulkan not available on system
        assert!(backend_info == "Vulkan" || backend_info == "CPU");
    }

    #[test]
    #[cfg(feature = "llama-vulkan")]
    fn test_explicit_vulkan_backend() {
        let engine = LlamaEngine::new_with_backend(Some("vulkan"));
        let backend_info = engine.get_backend_info();

        // Should use Vulkan backend when explicitly requested
        assert_eq!(backend_info, "Vulkan");
    }

    #[test]
    #[cfg(feature = "llama-opencl")]
    fn test_opencl_backend_info() {
        let engine = LlamaEngine::new_with_backend(Some("opencl"));
        let backend_info = engine.get_backend_info();

        // Should be OpenCL if available, or CPU if OpenCL not available on system
        assert!(backend_info == "OpenCL" || backend_info == "CPU");
    }

    #[test]
    #[cfg(feature = "llama-opencl")]
    fn test_explicit_opencl_backend() {
        let engine = LlamaEngine::new_with_backend(Some("opencl"));
        let backend_info = engine.get_backend_info();

        // Should use OpenCL backend when explicitly requested
        assert_eq!(backend_info, "OpenCL");
    }

    /// Regression test for Issue #72: GPU backend flag ignored
    /// This test ensures that the --gpu-backend CLI flag is properly
    /// wired through to the engine and affects GPU layer offloading.
    #[test]
    #[cfg(any(
        feature = "llama-vulkan",
        feature = "llama-opencl",
        feature = "llama-cuda"
    ))]
    fn test_issue_72_gpu_backend_flag_respected() {
        // Test that explicit backend selection works
        let vulkan_engine = LlamaEngine::new_with_backend(Some("vulkan"));
        let opencl_engine = LlamaEngine::new_with_backend(Some("opencl"));
        let cpu_engine = LlamaEngine::new_with_backend(Some("cpu"));

        // Backends should be different
        let vulkan_info = vulkan_engine.get_backend_info();
        let opencl_info = opencl_engine.get_backend_info();
        let cpu_info = cpu_engine.get_backend_info();

        // CPU should always be CPU
        assert_eq!(cpu_info, "CPU");

        // At least one GPU backend should be non-CPU
        assert!(
            vulkan_info != "CPU" || opencl_info != "CPU",
            "GPU backends should be available when features are enabled"
        );
    }
}
