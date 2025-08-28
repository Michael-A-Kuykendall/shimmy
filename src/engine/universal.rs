use anyhow::{anyhow, Result};
use async_trait::async_trait;

use super::{
    huggingface::HuggingFaceEngine, llama::LlamaEngine, ModelBackend, UniversalEngine,
    UniversalModel, UniversalModelSpec, InferenceEngine,
};

/// Universal engine that routes to appropriate backend
pub struct ShimmyUniversalEngine {
    llama_engine: LlamaEngine,
    huggingface_engine: HuggingFaceEngine,
}

impl ShimmyUniversalEngine {
    pub fn new() -> Self {
        Self {
            llama_engine: LlamaEngine::new(),
            huggingface_engine: HuggingFaceEngine::new(),
        }
    }

}

impl Default for ShimmyUniversalEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UniversalEngine for ShimmyUniversalEngine {
    async fn load(&self, spec: &UniversalModelSpec) -> Result<Box<dyn UniversalModel>> {
        match &spec.backend {
            ModelBackend::LlamaGGUF { .. } => {
                // Convert to legacy ModelSpec and use LlamaEngine
                let legacy_spec = spec.clone().try_into()?;
                let loaded = self.llama_engine.load(&legacy_spec).await?;
                Ok(Box::new(UniversalModelAdapter { model: loaded }))
            }
            ModelBackend::HuggingFace { .. } => {
                self.huggingface_engine.load(spec).await
            }
            ModelBackend::Candle { .. } => {
                Err(anyhow!("Candle backend not yet implemented"))
            }
        }
    }
}

/// Adapter to make legacy LoadedModel work with UniversalModel
struct UniversalModelAdapter {
    model: Box<dyn super::LoadedModel>,
}

#[async_trait]
impl UniversalModel for UniversalModelAdapter {
    async fn generate(
        &self,
        prompt: &str,
        opts: super::GenOptions,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<String> {
        self.model.generate(prompt, opts, on_token).await
    }
}

/// Convert UniversalModelSpec to legacy ModelSpec for LlamaEngine compatibility
impl TryFrom<UniversalModelSpec> for super::ModelSpec {
    type Error = anyhow::Error;

    fn try_from(spec: UniversalModelSpec) -> Result<Self> {
        match spec.backend {
            ModelBackend::LlamaGGUF { base_path, lora_path } => Ok(super::ModelSpec {
                name: spec.name,
                base_path,
                lora_path,
                template: spec.template,
                ctx_len: spec.ctx_len,
                n_threads: spec.n_threads,
            }),
            _ => Err(anyhow!("Cannot convert non-GGUF backend to legacy ModelSpec")),
        }
    }
}