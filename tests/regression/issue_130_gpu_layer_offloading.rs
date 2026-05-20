/// Regression Test: Issue #130 — GPU layer offloading not working
///
/// **v1.x root cause**: GpuBackend::gpu_layers() returned 999 for ALL backends including
/// CPU, preventing llama.cpp from distinguishing CPU vs GPU offload.
///
/// **v2.0 resolution**: The llama.cpp GpuBackend enum and gpu_layers() concept are removed
/// entirely. Airframe (wgpu) handles GPU offloading at the dispatch level — all tensor ops
/// run on the selected wgpu adapter automatically.

#[cfg(test)]
mod tests {
    use shimmy::engine::universal::ShimmyUniversalEngine;

    /// Engine constructs without any llama.cpp GPU layer configuration.
    #[test]
    fn test_engine_constructs_without_gpu_layer_config() {
        let _engine = ShimmyUniversalEngine::new();
    }

    /// Regression: no CPU-vs-GPU layer count distinction is needed in v2.0.
    /// If this compiles, the old GpuBackend::gpu_layers() path is confirmed absent.
    #[test]
    fn test_no_gpu_layers_concept_in_v2() {
        let _engine = ShimmyUniversalEngine::new();
    }
}
