use anyhow::Result;
use async_trait::async_trait;

use super::{InferenceEngine, LoadedModel, ModelSpec};

#[cfg(feature = "huggingface")]
use super::{UniversalEngine, UniversalModel};

#[cfg(feature = "huggingface")]
use super::universal::ShimmyUniversalEngine;

/// Adapter that makes UniversalEngine compatible with legacy InferenceEngine trait
pub struct InferenceEngineAdapter {
    #[cfg(feature = "huggingface")]
    universal_engine: ShimmyUniversalEngine,
    
    #[cfg(not(feature = "huggingface"))]
    llama_engine: super::llama::LlamaEngine,
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
            universal_engine: ShimmyUniversalEngine::new(),
            
            #[cfg(not(feature = "huggingface"))]
            llama_engine: super::llama::LlamaEngine::new(),
        }
    }
}

#[async_trait]
impl InferenceEngine for InferenceEngineAdapter {
    async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
        #[cfg(feature = "huggingface")]
        {
            // Convert ModelSpec to UniversalModelSpec
            let universal_spec = spec.clone().into();
            let universal_model = self.universal_engine.load(&universal_spec).await?;
            Ok(Box::new(LoadedModelAdapter { inner: universal_model }))
        }
        
        #[cfg(not(feature = "huggingface"))]
        {
            // Use LLAMA engine directly
            self.llama_engine.load(spec).await
        }
    }
}

/// Adapter that makes UniversalModel compatible with legacy LoadedModel trait
#[cfg(feature = "huggingface")]
struct LoadedModelAdapter {
    inner: Box<dyn UniversalModel>,
}

#[cfg(feature = "huggingface")]
#[async_trait]
impl LoadedModel for LoadedModelAdapter {
    async fn generate(
        &self,
        prompt: &str,
        opts: super::GenOptions,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<String> {
        self.inner.generate(prompt, opts, on_token).await
    }
}
