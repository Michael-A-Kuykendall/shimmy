/// Regression Test: Issue #130 - GPU layer offloading not working
///
/// **User Report**: @D0wn10ad (Windows, Intel Graphics)
/// Built with `--features llama-vulkan` but layers assigned to CPU instead of GPU
///
/// **Root Cause**: GpuBackend::gpu_layers() returned 999 for ALL backends including CPU
/// This caused llama.cpp to not properly offload layers to GPU even when compiled with GPU features
///
/// **Fix**: Match on backend type - CPU returns 0, GPU backends (CUDA/Vulkan/OpenCL) return 999
///
/// **This test validates**:
/// - CPU backend returns 0 GPU layers (no offloading)
/// - CUDA backend returns 999 GPU layers (full offload) when feature enabled
/// - Vulkan backend returns 999 GPU layers when feature enabled
/// - OpenCL backend returns 999 GPU layers when feature enabled
///
/// **Related Issues**: #126 (MoE GPU detection), #129 (precompiled binaries missing GPU)
#[cfg(test)]
mod tests {
    use shimmy::engine::llama::GpuBackend;

    #[test]
    fn test_cpu_backend_returns_zero_layers() {
        let backend = GpuBackend::Cpu;
        assert_eq!(
            backend.gpu_layers(),
            0,
            "CPU backend should return 0 GPU layers (no offloading)"
        );
    }

    #[test]
    #[cfg(feature = "llama-cuda")]
    fn test_cuda_backend_returns_999_layers() {
        let backend = GpuBackend::Cuda;
        assert_eq!(
            backend.gpu_layers(),
            999,
            "CUDA backend should return 999 (offload all layers)"
        );
    }

    #[test]
    #[cfg(feature = "llama-vulkan")]
    fn test_vulkan_backend_returns_999_layers() {
        let backend = GpuBackend::Vulkan;
        assert_eq!(
            backend.gpu_layers(),
            999,
            "Vulkan backend should return 999 (offload all layers)"
        );
    }

    #[test]
    #[cfg(feature = "llama-opencl")]
    fn test_opencl_backend_returns_999_layers() {
        let backend = GpuBackend::OpenCL;
        assert_eq!(
            backend.gpu_layers(),
            999,
            "OpenCL backend should return 999 (offload all layers)"
        );
    }

    /// Verify all backends return consistent layer counts
    #[test]
    #[cfg(feature = "llama")]
    fn test_all_backends_layer_consistency() {
        // CPU should always return 0
        assert_eq!(GpuBackend::Cpu.gpu_layers(), 0);

        // All GPU backends should return 999 (full offload)
        #[cfg(feature = "llama-cuda")]
        assert_eq!(GpuBackend::Cuda.gpu_layers(), 999);

        #[cfg(feature = "llama-vulkan")]
        assert_eq!(GpuBackend::Vulkan.gpu_layers(), 999);

        #[cfg(feature = "llama-opencl")]
        assert_eq!(GpuBackend::OpenCL.gpu_layers(), 999);
    }
}
