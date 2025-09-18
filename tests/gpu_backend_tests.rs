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
        #[cfg(not(any(feature = "llama-cuda", feature = "llama-vulkan", feature = "llama-opencl")))]
        assert_eq!(backend_info, "CPU");
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
    #[cfg(feature = "llama-vulkan")]
    fn test_vulkan_backend_info() {
        let engine = LlamaEngine::new();
        let backend_info = engine.get_backend_info();
        
        // Should include Vulkan if available, or fallback to CPU
        assert!(backend_info == "Vulkan" || backend_info == "CPU");
    }

    #[test]
    #[cfg(feature = "llama-opencl")]
    fn test_opencl_backend_info() {
        let engine = LlamaEngine::new();
        let backend_info = engine.get_backend_info();
        
        // Should include OpenCL if available, or fallback to CPU
        assert!(backend_info == "OpenCL" || backend_info == "CPU");
    }
}