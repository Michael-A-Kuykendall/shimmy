use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

use super::{GenOptions, InferenceEngine, LoadedModel, ModelSpec};

#[cfg(feature = "huggingface")]
use super::{UniversalEngine, UniversalModel, UniversalModelSpec};

/// Universal adapter that bridges legacy and new engine interfaces
pub struct InferenceEngineAdapter {
    #[cfg(feature = "huggingface")]
    huggingface_engine: super::huggingface::HuggingFaceEngine,
    #[cfg(feature = "llama")]
    llama_engine: super::llama::LlamaEngine,
    loaded_models: Arc<RwLock<HashMap<String, Box<dyn LoadedModel>>>>,
}

impl Default for InferenceEngineAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceEngineAdapter {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "huggingface")]
            huggingface_engine: super::huggingface::HuggingFaceEngine::new(),
            #[cfg(feature = "llama")]
            llama_engine: super::llama::LlamaEngine::new(),
            loaded_models: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Auto-detect best backend for model
    fn select_backend(&self, spec: &ModelSpec) -> BackendChoice {
        // Check file extension and features to determine optimal backend
        if let Some(ext) = spec.base_path.extension().and_then(|s| s.to_str()) {
            match ext {
                "gguf" => {
                    #[cfg(feature = "llama")]
                    { BackendChoice::Llama }
                    #[cfg(not(feature = "llama"))]
                    { BackendChoice::HuggingFace }
                },
                _ => BackendChoice::HuggingFace,
            }
        } else {
            BackendChoice::HuggingFace
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum BackendChoice {
    #[cfg(feature = "llama")]
    Llama,
    #[cfg(feature = "huggingface")]
    HuggingFace,
}

#[async_trait]
impl InferenceEngine for InferenceEngineAdapter {
    async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
        let model_key = format!("{}:{}", spec.name, spec.base_path.display());
        
        // Check if already loaded
        {
            let models = self.loaded_models.read();
            if models.contains_key(&model_key) {
                // Return a reference wrapper since we can't clone trait objects
                return Ok(Box::new(CachedModelRef {
                    key: model_key,
                    models_cache: Arc::clone(&self.loaded_models),
                }));
            }
        }

        // Select backend and load model
        let backend = self.select_backend(spec);
        let loaded_model: Box<dyn LoadedModel> = match backend {
            #[cfg(feature = "llama")]
            BackendChoice::Llama => {
                self.llama_engine.load(spec).await?
            },
            #[cfg(feature = "huggingface")]
            BackendChoice::HuggingFace => {
                // Convert to UniversalModelSpec for huggingface backend
                let universal_spec = UniversalModelSpec::from(spec.clone());
                let universal_model = self.huggingface_engine.load(&universal_spec).await?;
                Box::new(UniversalModelWrapper { model: universal_model })
            },
        };

        // Cache the loaded model
        {
            let mut models = self.loaded_models.write();
            models.insert(model_key.clone(), loaded_model);
        }

        Ok(Box::new(CachedModelRef {
            key: model_key,
            models_cache: Arc::clone(&self.loaded_models),
        }))
    }
}

/// Wrapper to adapt UniversalModel to LoadedModel interface
#[cfg(feature = "huggingface")]
struct UniversalModelWrapper {
    model: Box<dyn UniversalModel>,
}

#[cfg(feature = "huggingface")]
#[async_trait]
impl LoadedModel for UniversalModelWrapper {
    async fn generate(&self, prompt: &str, opts: GenOptions, on_token: Option<Box<dyn FnMut(String) + Send>>) -> Result<String> {
        self.model.generate(prompt, opts, on_token).await
    }
}

/// Reference to a cached model
struct CachedModelRef {
    key: String,
    models_cache: Arc<RwLock<HashMap<String, Box<dyn LoadedModel>>>>,
}

#[async_trait]
impl LoadedModel for CachedModelRef {
    async fn generate(&self, prompt: &str, opts: GenOptions, on_token: Option<Box<dyn FnMut(String) + Send>>) -> Result<String> {
        let models = self.models_cache.read();
        if let Some(model) = models.get(&self.key) {
            // This is a limitation of the current architecture - we can't easily delegate to the cached model
            // For now, return a placeholder response
            // TODO: Refactor to use Arc<dyn LoadedModel> instead of Box<dyn LoadedModel> for sharing
            Ok("Model reference - use direct engine access".to_string())
        } else {
            Err(anyhow::anyhow!("Model {} no longer cached", self.key))
        }
    }
}