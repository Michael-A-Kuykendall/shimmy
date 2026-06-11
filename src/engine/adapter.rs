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
        assert_eq!(
            a.select_backend(&spec("m", "model.safetensors")),
            BackendChoice::SafeTensors
        );
    }

    #[test]
    #[cfg(feature = "airframe")]
    fn test_gguf_routes_to_airframe() {
        let a = InferenceEngineAdapter::new();
        assert_eq!(
            a.select_backend(&spec("m", "model.gguf")),
            BackendChoice::Airframe
        );
    }

    #[test]
    fn test_safetensors_priority() {
        let a = InferenceEngineAdapter::new();
        assert_eq!(
            a.select_backend(&spec("m", "/models/org/pytorch_model.safetensors")),
            BackendChoice::SafeTensors
        );
    }
}
