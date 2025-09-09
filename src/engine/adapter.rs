use anyhow::Result;
use async_trait::async_trait;

use super::{GenOptions, InferenceEngine, LoadedModel, ModelSpec};

#[cfg(feature = "huggingface")]
use super::{UniversalEngine, UniversalModel, UniversalModelSpec};

/// Universal adapter that bridges legacy and new engine interfaces
pub struct InferenceEngineAdapter {
    #[cfg(feature = "huggingface")]
    huggingface_engine: super::huggingface::HuggingFaceEngine,
    #[cfg(feature = "llama")]
    llama_engine: super::llama::LlamaEngine,
    // Note: loaded_models removed as caching is not currently implemented
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
        }
    }

    /// Auto-detect best backend for model
    fn select_backend(&self, spec: &ModelSpec) -> BackendChoice {
        // Check file extension and path patterns to determine optimal backend
        let path_str = spec.base_path.to_string_lossy();
        
        // Check for GGUF files by extension - these should ALWAYS use LlamaEngine
        if let Some(ext) = spec.base_path.extension().and_then(|s| s.to_str()) {
            if ext == "gguf" {
                #[cfg(feature = "llama")]
                { return BackendChoice::Llama; }
                #[cfg(not(feature = "llama"))]
                { 
                    // This shouldn't happen with default features, but handle gracefully
                    panic!("GGUF file detected but llama feature not enabled. Please install with --features llama");
                }
            }
        }
        
        // Check for Ollama blob files (GGUF files without extension)
        if path_str.contains("ollama") && path_str.contains("blobs") && path_str.contains("sha256-") {
            #[cfg(feature = "llama")]
            { return BackendChoice::Llama; }
            #[cfg(not(feature = "llama"))]
            { 
                #[cfg(feature = "huggingface")]
                { return BackendChoice::HuggingFace; }
                #[cfg(not(feature = "huggingface"))]
                { panic!("Ollama blob detected but no backend enabled"); }
            }
        }
        
        // Check for other patterns that indicate GGUF files
        if path_str.contains(".gguf") || spec.name.contains("llama") || spec.name.contains("phi") || spec.name.contains("qwen") || spec.name.contains("gemma") || spec.name.contains("mistral") {
            #[cfg(feature = "llama")]
            { return BackendChoice::Llama; }
            #[cfg(not(feature = "llama"))]
            { 
                #[cfg(feature = "huggingface")]
                { return BackendChoice::HuggingFace; }
                #[cfg(not(feature = "huggingface"))]
                { panic!("GGUF model detected but no backend enabled"); }
            }
        }
        
        // Default to HuggingFace for other models
        #[cfg(feature = "huggingface")]
        { BackendChoice::HuggingFace }
        #[cfg(not(feature = "huggingface"))]
        { 
            #[cfg(feature = "llama")]
            { BackendChoice::Llama }
            #[cfg(not(feature = "llama"))]
            { panic!("No backend features enabled. Please compile with --features llama or --features huggingface"); }
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
        // Select backend and load model directly (no caching for now to avoid complexity)
        let backend = self.select_backend(spec);
        match backend {
            #[cfg(feature = "llama")]
            BackendChoice::Llama => {
                self.llama_engine.load(spec).await
            },
            #[cfg(feature = "huggingface")]
            BackendChoice::HuggingFace => {
                // Convert to UniversalModelSpec for huggingface backend (for HF model IDs)
                let universal_spec = UniversalModelSpec {
                    name: spec.name.clone(),
                    backend: super::ModelBackend::HuggingFace {
                        base_model_id: spec.base_path.to_string_lossy().to_string(),
                        peft_path: spec.lora_path.as_ref().map(|p| p.to_path_buf()),
                        use_local: true,
                    },
                    template: spec.template.clone(),
                    ctx_len: spec.ctx_len,
                    device: "cpu".to_string(),
                    n_threads: spec.n_threads,
                };
                let universal_model = self.huggingface_engine.load(&universal_spec).await?;
                Ok(Box::new(UniversalModelWrapper { model: universal_model }))
            },
        }
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

// Note: Cached model references removed as they were unused placeholder code.
// Future implementation should use Arc<dyn LoadedModel> for proper model sharing.