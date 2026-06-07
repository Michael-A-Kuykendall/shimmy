use anyhow::Result;
use async_trait::async_trait;

use super::{InferenceEngine, LoadedModel, ModelSpec};

pub struct InferenceEngineAdapter {
    #[cfg(feature = "airframe")]
    airframe_engine: super::airframe::AirframeEngine,
    safetensors_engine: super::safetensors_native::SafeTensorsEngine,
}

impl Default for InferenceEngineAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceEngineAdapter {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "airframe")]
            airframe_engine: super::airframe::AirframeEngine::new(),
            safetensors_engine: super::safetensors_native::SafeTensorsEngine::new(),
        }
    }

    /// Backend arg retained for CLI compatibility; ignored in v2 (airframe auto-detects GPU)
    pub fn new_with_backend(_gpu_backend: Option<&str>) -> Self {
        Self::new()
    }

    fn select_backend(&self, spec: &ModelSpec) -> BackendChoice {
        let path_str = spec.base_path.to_string_lossy();

        if let Some(ext) = spec.base_path.extension().and_then(|s| s.to_str()) {
            match ext {
                "safetensors" => return BackendChoice::SafeTensors,
                "gguf" => {
                    #[cfg(feature = "airframe")]
                    return BackendChoice::Airframe;
                    #[cfg(not(feature = "airframe"))]
                    return BackendChoice::SafeTensors;
                }
                _ => {}
            }
        }

        // Ollama blob files (extensionless GGUF)
        if path_str.contains("ollama") && path_str.contains("blobs") {
            #[cfg(feature = "airframe")]
            return BackendChoice::Airframe;
        }

        // Name/path heuristic for GGUF
        if path_str.contains(".gguf") {
            #[cfg(feature = "airframe")]
            return BackendChoice::Airframe;
        }

        BackendChoice::SafeTensors
    }
}

#[derive(Debug, Clone, PartialEq)]
enum BackendChoice {
    #[cfg(feature = "airframe")]
    Airframe,
    SafeTensors,
}

#[async_trait]
impl InferenceEngine for InferenceEngineAdapter {
    async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
        match self.select_backend(spec) {
            BackendChoice::SafeTensors => self.safetensors_engine.load(spec).await,
            #[cfg(feature = "airframe")]
            BackendChoice::Airframe => self.airframe_engine.load(spec).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn spec(name: &str, path: &str) -> ModelSpec {
        ModelSpec {
            name: name.to_string(),
            base_path: PathBuf::from(path),
            lora_path: None,
            template: None,
            ctx_len: 2048,
            n_threads: None,
        }
    }

    #[test]
    fn test_safetensors_routes_to_safetensors() {
        let a = InferenceEngineAdapter::new();
        assert_eq!(a.select_backend(&spec("m", "model.safetensors")), BackendChoice::SafeTensors);
    }

    #[test]
<<<<<<< HEAD
    fn test_local_file_detection() {
        let adapter = InferenceEngineAdapter::new();

        // Test local files still work
        let safetensors_spec = create_test_spec("local", "model.safetensors");
        let backend = adapter.select_backend(&safetensors_spec);
        assert_eq!(backend, BackendChoice::SafeTensors);

        // Test Windows paths (should not be treated as HF model IDs)
        let windows_spec = create_test_spec("local", "C:\\path\\to\\model.safetensors");
        let backend2 = adapter.select_backend(&windows_spec);
        assert_eq!(backend2, BackendChoice::SafeTensors);
=======
    #[cfg(feature = "airframe")]
    fn test_gguf_routes_to_airframe() {
        let a = InferenceEngineAdapter::new();
        assert_eq!(a.select_backend(&spec("m", "model.gguf")), BackendChoice::Airframe);
>>>>>>> 1b661ad (fix: strip legacy engine code, clean feature flags for v2.2 release)
    }

    #[test]
    fn test_safetensors_priority() {
        let a = InferenceEngineAdapter::new();
        assert_eq!(
            a.select_backend(&spec("m", "/models/org/pytorch_model.safetensors")),
            BackendChoice::SafeTensors
        );
<<<<<<< HEAD
        let backend2 = adapter.select_backend(&safetensors_complex);
        assert_eq!(backend2, BackendChoice::SafeTensors);

        // Windows paths with safetensors
        let safetensors_windows =
            create_test_spec("model", "C:\\models\\org\\model\\model.safetensors");
        let backend3 = adapter.select_backend(&safetensors_windows);
        assert_eq!(backend3, BackendChoice::SafeTensors);
    }

    #[test]
    fn test_file_extension_priority() {
        let adapter = InferenceEngineAdapter::new();

        // File extensions should take priority over everything else
        let safetensors_spec = create_test_spec("llama-model", "path/to/llama.safetensors");
        let backend = adapter.select_backend(&safetensors_spec);
        assert_eq!(backend, BackendChoice::SafeTensors);

        #[cfg(feature = "mlx")]
        {
            let mlx_spec = create_test_spec("qwen-model", "path/to/qwen.mlx");
            let backend2 = adapter.select_backend(&mlx_spec);
            assert_eq!(backend2, BackendChoice::MLX);
        }
=======
>>>>>>> 1b661ad (fix: strip legacy engine code, clean feature flags for v2.2 release)
    }
}
