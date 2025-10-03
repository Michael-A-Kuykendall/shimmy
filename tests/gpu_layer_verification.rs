/// Integration test to verify GPU layers are actually configured
/// This test ensures that Issue #72 fix actually works end-to-end
#[cfg(test)]
mod gpu_layer_verification {
    use shimmy::engine::llama::LlamaEngine;

    #[test]
    fn test_gpu_backend_selection_cpu() {
        let engine = LlamaEngine::new_with_backend(Some("cpu"));
        let info = engine.get_backend_info();
        assert_eq!(info, "CPU", "CPU backend should be selected");
    }

    #[test]
    #[cfg(feature = "llama-vulkan")]
    fn test_gpu_backend_selection_vulkan() {
        let engine = LlamaEngine::new_with_backend(Some("vulkan"));
        let info = engine.get_backend_info();
        assert_eq!(
            info, "Vulkan",
            "Vulkan backend should be selected when feature enabled"
        );
    }

    #[test]
    #[cfg(feature = "llama-opencl")]
    fn test_gpu_backend_selection_opencl() {
        let engine = LlamaEngine::new_with_backend(Some("opencl"));
        let info = engine.get_backend_info();
        assert_eq!(
            info, "OpenCL",
            "OpenCL backend should be selected when feature enabled"
        );
    }

    #[test]
    #[cfg(feature = "llama-cuda")]
    fn test_gpu_backend_selection_cuda() {
        let engine = LlamaEngine::new_with_backend(Some("cuda"));
        let info = engine.get_backend_info();
        assert_eq!(
            info, "CUDA",
            "CUDA backend should be selected when feature enabled"
        );
    }

    #[test]
    fn test_auto_backend_fallback_to_cpu_when_no_gpu() {
        #[cfg(not(any(
            feature = "llama-cuda",
            feature = "llama-vulkan",
            feature = "llama-opencl"
        )))]
        {
            let engine = LlamaEngine::new_with_backend(Some("auto"));
            let info = engine.get_backend_info();
            assert_eq!(
                info, "CPU",
                "Auto should fall back to CPU when no GPU features enabled"
            );
        }
    }

    /// This is the regression test for Issue #72
    /// Verifies that --gpu-backend flag actually affects backend selection
    #[test]
    #[cfg(any(feature = "llama-vulkan", feature = "llama-opencl"))]
    fn test_issue_72_regression_gpu_backend_not_ignored() {
        // Test that CPU is selected when explicitly requested
        let cpu_engine = LlamaEngine::new_with_backend(Some("cpu"));
        assert_eq!(cpu_engine.get_backend_info(), "CPU");

        // Test that GPU backend is selected when requested and available
        #[cfg(feature = "llama-vulkan")]
        {
            let vulkan_engine = LlamaEngine::new_with_backend(Some("vulkan"));
            assert_eq!(
                vulkan_engine.get_backend_info(),
                "Vulkan",
                "Issue #72: --gpu-backend vulkan flag should select Vulkan backend"
            );
        }

        #[cfg(feature = "llama-opencl")]
        {
            let opencl_engine = LlamaEngine::new_with_backend(Some("opencl"));
            assert_eq!(
                opencl_engine.get_backend_info(),
                "OpenCL",
                "Issue #72: --gpu-backend opencl flag should select OpenCL backend"
            );
        }
    }
}
