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
    #[cfg(feature = "mlx")]
    mlx_engine: super::mlx::MLXEngine,
    safetensors_engine: super::safetensors_native::SafeTensorsEngine,
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
            #[cfg(feature = "mlx")]
            mlx_engine: super::mlx::MLXEngine::new(),
            safetensors_engine: super::safetensors_native::SafeTensorsEngine::new(),
        }
    }

    /// Auto-detect best backend for model
    fn select_backend(&self, spec: &ModelSpec) -> BackendChoice {
        // Check file extension and path patterns to determine optimal backend
        let path_str = spec.base_path.to_string_lossy();

        // FIRST: Check for HuggingFace model IDs (format: "org/model-name")
        #[cfg(feature = "huggingface")]
        {
            if path_str.contains('/') && !path_str.contains('\\') && !path_str.contains('.') {
                // Looks like a HuggingFace model ID (has slash, no backslash, no file extension)
                return BackendChoice::HuggingFace;
            }
        }

        // Check for MLX files FIRST on Apple Silicon - best performance
        #[cfg(feature = "mlx")]
        {
            if let Some(ext) = spec.base_path.extension().and_then(|s| s.to_str()) {
                if ext == "npz" || ext == "mlx" {
                    // MLX native format
                    return BackendChoice::MLX;
                }
            }
            
            // Check if we're on Apple Silicon and model is MLX-compatible
            if cfg!(target_os = "macos") && std::env::consts::ARCH == "aarch64" {
                let model_name = spec.name.to_lowercase();
                if model_name.contains("llama") || model_name.contains("mistral") 
                    || model_name.contains("phi") || model_name.contains("qwen") {
                    // Prefer MLX for known compatible models on Apple Silicon
                    return BackendChoice::MLX;
                }
            }
        }

        // Check for SafeTensors files SECOND - native Rust implementation
        if let Some(ext) = spec.base_path.extension().and_then(|s| s.to_str()) {
            if ext == "safetensors" {
                return BackendChoice::SafeTensors;
            }
        }

        // Check for GGUF files by extension - these should use LlamaEngine
        if let Some(ext) = spec.base_path.extension().and_then(|s| s.to_str()) {
            if ext == "gguf" {
                #[cfg(feature = "llama")]
                {
                    return BackendChoice::Llama;
                }
                #[cfg(not(feature = "llama"))]
                {
                    // This shouldn't happen with default features, but handle gracefully
                    panic!("GGUF file detected but llama feature not enabled. Please install with --features llama");
                }
            }
        }

        // Check for Ollama blob files (GGUF files without extension)
        if path_str.contains("ollama") && path_str.contains("blobs") && path_str.contains("sha256-")
        {
            #[cfg(feature = "llama")]
            {
                return BackendChoice::Llama;
            }
            #[cfg(not(feature = "llama"))]
            {
                #[cfg(feature = "huggingface")]
                {
                    return BackendChoice::HuggingFace;
                }
                #[cfg(not(feature = "huggingface"))]
                {
                    panic!("Ollama blob detected but no backend enabled");
                }
            }
        }

        // Check for other patterns that indicate GGUF files
        if path_str.contains(".gguf")
            || spec.name.contains("llama")
            || spec.name.contains("phi")
            || spec.name.contains("qwen")
            || spec.name.contains("gemma")
            || spec.name.contains("mistral")
        {
            #[cfg(feature = "llama")]
            {
                return BackendChoice::Llama;
            }
            #[cfg(not(feature = "llama"))]
            {
                #[cfg(feature = "huggingface")]
                {
                    return BackendChoice::HuggingFace;
                }
                #[cfg(not(feature = "huggingface"))]
                {
                    panic!("GGUF model detected but no backend enabled");
                }
            }
        }

        // Default to HuggingFace for other models
        #[cfg(feature = "huggingface")]
        {
            BackendChoice::HuggingFace
        }
        #[cfg(not(feature = "huggingface"))]
        {
            #[cfg(feature = "llama")]
            {
                BackendChoice::Llama
            }
            #[cfg(not(feature = "llama"))]
            {
                panic!("No backend features enabled. Please compile with --features llama or --features huggingface");
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum BackendChoice {
    #[cfg(feature = "llama")]
    Llama,
    #[cfg(feature = "huggingface")]
    HuggingFace,
    #[cfg(feature = "mlx")]
    MLX,
    SafeTensors,
}

#[async_trait]
impl InferenceEngine for InferenceEngineAdapter {
    async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
        // Select backend and load model directly (no caching for now to avoid complexity)
        let backend = self.select_backend(spec);
        match backend {
            BackendChoice::SafeTensors => {
                // Use native SafeTensors engine - NO Python dependency!
                self.safetensors_engine.load(spec).await
            }
            #[cfg(feature = "mlx")]
            BackendChoice::MLX => {
                // Use MLX engine for Apple Silicon Metal GPU acceleration
                self.mlx_engine.load(spec).await
            }
            #[cfg(feature = "llama")]
            BackendChoice::Llama => self.llama_engine.load(spec).await,
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
                Ok(Box::new(UniversalModelWrapper {
                    model: universal_model,
                }))
            }
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
    async fn generate(
        &self,
        prompt: &str,
        opts: GenOptions,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<String> {
        self.model.generate(prompt, opts, on_token).await
    }
}

// Note: Cached model references removed as they were unused placeholder code.
// Future implementation should use Arc<dyn LoadedModel> for proper model sharing.

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_spec(name: &str, path: &str) -> ModelSpec {
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
    fn test_huggingface_model_id_detection() {
        let adapter = InferenceEngineAdapter::new();
        
        // Test HuggingFace model IDs
        let hf_spec = create_test_spec("qwen", "Qwen/Qwen3-Next-80B-A3B-Instruct");
        let backend = adapter.select_backend(&hf_spec);
        #[cfg(feature = "huggingface")]
        assert_eq!(backend, BackendChoice::HuggingFace);
        
        let hf_spec2 = create_test_spec("llama", "meta-llama/Llama-2-7b-chat-hf");
        let backend2 = adapter.select_backend(&hf_spec2);
        #[cfg(feature = "huggingface")]
        assert_eq!(backend2, BackendChoice::HuggingFace);
    }
    
    #[test]
    fn test_local_file_detection() {
        let adapter = InferenceEngineAdapter::new();
        
        // Test local files still work
        let gguf_spec = create_test_spec("local", "model.gguf");
        let backend = adapter.select_backend(&gguf_spec);
        #[cfg(feature = "llama")]
        assert_eq!(backend, BackendChoice::Llama);
        
        let safetensors_spec = create_test_spec("local", "model.safetensors");
        let backend2 = adapter.select_backend(&safetensors_spec);
        assert_eq!(backend2, BackendChoice::SafeTensors);
        
        // Test Windows paths (should not be treated as HF model IDs)
        let windows_spec = create_test_spec("local", "C:\\path\\to\\model.gguf");
        let backend3 = adapter.select_backend(&windows_spec);
        #[cfg(feature = "llama")]
        assert_eq!(backend3, BackendChoice::Llama);
    }
}
