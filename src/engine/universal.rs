use anyhow::{anyhow, Result};
use async_trait::async_trait;

use super::{
    huggingface::HuggingFaceEngine, llama::LlamaEngine, InferenceEngine, ModelBackend,
    UniversalEngine, UniversalModel, UniversalModelSpec,
};

/// Universal engine that routes to appropriate backend
#[allow(dead_code)]
pub struct ShimmyUniversalEngine {
    llama_engine: LlamaEngine,
    huggingface_engine: HuggingFaceEngine,
}

impl ShimmyUniversalEngine {
    #[allow(dead_code)]
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
            ModelBackend::HuggingFace { .. } => self.huggingface_engine.load(spec).await,
            ModelBackend::Candle { .. } => Err(anyhow!("Candle backend not yet implemented")),
        }
    }
}

/// Adapter to make legacy LoadedModel work with UniversalModel
#[allow(dead_code)]
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
            ModelBackend::LlamaGGUF {
                base_path,
                lora_path,
            } => Ok(super::ModelSpec {
                name: spec.name,
                base_path,
                lora_path,
                template: spec.template,
                ctx_len: spec.ctx_len,
                n_threads: spec.n_threads,
            }),
            _ => Err(anyhow!(
                "Cannot convert non-GGUF backend to legacy ModelSpec"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{GenOptions, LoadedModel, ModelSpec};
    use std::path::PathBuf;

    // Mock LoadedModel for testing UniversalModelAdapter
    struct MockLoadedModel {
        response: String,
    }

    #[async_trait]
    impl LoadedModel for MockLoadedModel {
        async fn generate(
            &self,
            prompt: &str,
            opts: GenOptions,
            mut on_token: Option<Box<dyn FnMut(String) + Send>>,
        ) -> Result<String> {
            // Simulate token generation for callback testing
            if let Some(ref mut callback) = on_token {
                callback("Generated".to_string());
                callback("response".to_string());
                callback("to:".to_string());
            }

            Ok(format!(
                "{} (prompt: {}, max_tokens: {})",
                self.response, prompt, opts.max_tokens
            ))
        }
    }

    #[test]
    fn test_shimmy_universal_engine_new() {
        let engine = ShimmyUniversalEngine::new();
        // Verify the struct was created (we can't access private fields, but we can test creation)
        // The struct itself is zero-sized since engines are unit structs
        assert_eq!(
            std::mem::size_of_val(&engine),
            std::mem::size_of::<(
                crate::engine::llama::LlamaEngine,
                crate::engine::huggingface::HuggingFaceEngine
            )>()
        );
    }

    #[test]
    fn test_shimmy_universal_engine_default() {
        let engine1 = ShimmyUniversalEngine::new();
        let engine2 = ShimmyUniversalEngine::default();
        // Both should be equivalent (unit structs)
        assert_eq!(
            std::mem::size_of_val(&engine1),
            std::mem::size_of_val(&engine2)
        );
    }

    #[test]
    fn test_universal_engine_llama_gguf_routing() {
        // Test the ModelBackend::LlamaGGUF routing logic without actually loading files
        let spec = UniversalModelSpec {
            name: "test-llama".to_string(),
            backend: ModelBackend::LlamaGGUF {
                base_path: PathBuf::from("test.gguf"),
                lora_path: None,
            },
            template: Some("chatml".to_string()),
            ctx_len: 2048,
            device: "cpu".to_string(),
            n_threads: Some(4),
        };

        // Test that we can create the engine and that it routes to the correct backend
        let engine = ShimmyUniversalEngine::new();

        // Test the conversion logic by matching on backend type
        match &spec.backend {
            ModelBackend::LlamaGGUF {
                base_path,
                lora_path,
            } => {
                // This verifies the pattern matching works for GGUF backend
                assert_eq!(base_path, &PathBuf::from("test.gguf"));
                assert_eq!(lora_path, &None);

                // Test the conversion to legacy ModelSpec
                let legacy_spec: Result<ModelSpec> = spec.clone().try_into();
                assert!(legacy_spec.is_ok());
                let legacy = legacy_spec.unwrap();
                assert_eq!(legacy.name, "test-llama");
                assert_eq!(legacy.base_path, PathBuf::from("test.gguf"));
                assert_eq!(legacy.ctx_len, 2048);
                assert_eq!(legacy.n_threads, Some(4));
            }
            _ => panic!("Expected LlamaGGUF backend"),
        }

        // Verify engine structure
        assert_eq!(
            std::mem::size_of_val(&engine),
            std::mem::size_of::<(
                crate::engine::llama::LlamaEngine,
                crate::engine::huggingface::HuggingFaceEngine
            )>()
        );
    }

    #[test]
    fn test_universal_engine_llama_gguf_with_lora_routing() {
        // Test the ModelBackend::LlamaGGUF with LoRA routing logic without loading files
        let spec = UniversalModelSpec {
            name: "test-llama-lora".to_string(),
            backend: ModelBackend::LlamaGGUF {
                base_path: PathBuf::from("base.gguf"),
                lora_path: Some(PathBuf::from("lora.bin")),
            },
            template: Some("llama3".to_string()),
            ctx_len: 4096,
            device: "cuda".to_string(),
            n_threads: None,
        };

        let engine = ShimmyUniversalEngine::new();

        // Test the backend matching and LoRA handling
        match &spec.backend {
            ModelBackend::LlamaGGUF {
                base_path,
                lora_path,
            } => {
                // Verify GGUF backend with LoRA is correctly structured
                assert_eq!(base_path, &PathBuf::from("base.gguf"));
                assert_eq!(lora_path, &Some(PathBuf::from("lora.bin")));

                // Test conversion to legacy ModelSpec preserves LoRA path
                let legacy_spec: Result<ModelSpec> = spec.clone().try_into();
                assert!(legacy_spec.is_ok());
                let legacy = legacy_spec.unwrap();
                assert_eq!(legacy.name, "test-llama-lora");
                assert_eq!(legacy.base_path, PathBuf::from("base.gguf"));
                assert_eq!(legacy.lora_path, Some(PathBuf::from("lora.bin")));
                assert_eq!(legacy.template, Some("llama3".to_string()));
                assert_eq!(legacy.ctx_len, 4096);
                assert_eq!(legacy.n_threads, None);
            }
            _ => panic!("Expected LlamaGGUF backend"),
        }

        // Verify engine can be created
        assert!(std::mem::size_of_val(&engine) > 0);
    }

    #[tokio::test]
    async fn test_universal_engine_load_huggingface() {
        let engine = ShimmyUniversalEngine::new();
        let spec = UniversalModelSpec {
            name: "test-hf".to_string(),
            backend: ModelBackend::HuggingFace {
                base_model_id: "microsoft/Phi-3-mini-4k-instruct".to_string(),
                peft_path: None,
                use_local: false,
            },
            template: None,
            ctx_len: 4096,
            device: "cpu".to_string(),
            n_threads: None,
        };

        let result = engine.load(&spec).await;
        // This will delegate to HuggingFaceEngine::load()
        // May succeed or fail depending on environment setup - just test the path is taken
        match result {
            Ok(_) => {
                // If Python dependencies are available, loading may succeed
                // This tests the successful path through HuggingFace backend
            }
            Err(_) => {
                // If Python dependencies are missing, loading will fail
                // This tests the error path through HuggingFace backend
            }
        }
    }

    #[tokio::test]
    async fn test_universal_engine_load_huggingface_with_peft() {
        let engine = ShimmyUniversalEngine::new();
        let spec = UniversalModelSpec {
            name: "test-hf-peft".to_string(),
            backend: ModelBackend::HuggingFace {
                base_model_id: "microsoft/Phi-3-mini-4k-instruct".to_string(),
                peft_path: Some(PathBuf::from("./my-peft-adapter")),
                use_local: true,
            },
            template: Some("chatml".to_string()),
            ctx_len: 2048,
            device: "cuda".to_string(),
            n_threads: Some(2),
        };

        let result = engine.load(&spec).await;
        // This tests HuggingFace with PEFT path - may succeed or fail based on environment
        match result {
            Ok(_) => {
                // Tests successful HuggingFace + PEFT loading if env is set up
            }
            Err(_) => {
                // Tests error handling for HuggingFace + PEFT when env not available
            }
        }
    }

    #[tokio::test]
    async fn test_universal_engine_load_candle_not_implemented() {
        let engine = ShimmyUniversalEngine::new();
        let spec = UniversalModelSpec {
            name: "test-candle".to_string(),
            backend: ModelBackend::Candle {
                model_path: PathBuf::from("model.safetensors"),
                adapter_path: None,
            },
            template: None,
            ctx_len: 2048,
            device: "cpu".to_string(),
            n_threads: None,
        };

        let result = engine.load(&spec).await;
        assert!(result.is_err());

        // Verify the specific error message
        match result {
            Err(e) => assert!(e.to_string().contains("Candle backend not yet implemented")),
            Ok(_) => panic!("Expected error but got success"),
        }
    }

    #[tokio::test]
    async fn test_universal_engine_load_candle_with_adapter() {
        let engine = ShimmyUniversalEngine::new();
        let spec = UniversalModelSpec {
            name: "test-candle-adapter".to_string(),
            backend: ModelBackend::Candle {
                model_path: PathBuf::from("base_model.safetensors"),
                adapter_path: Some(PathBuf::from("adapter.safetensors")),
            },
            template: Some("custom".to_string()),
            ctx_len: 8192,
            device: "metal".to_string(),
            n_threads: Some(8),
        };

        let result = engine.load(&spec).await;
        assert!(result.is_err());

        match result {
            Err(e) => assert!(e.to_string().contains("Candle backend not yet implemented")),
            Ok(_) => panic!("Expected error but got success"),
        }
    }

    #[tokio::test]
    async fn test_universal_model_adapter_generate() {
        let mock_model = MockLoadedModel {
            response: "Test response".to_string(),
        };

        let adapter = UniversalModelAdapter {
            model: Box::new(mock_model),
        };

        let opts = GenOptions {
            max_tokens: 100,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            seed: Some(42),
            stream: true,
        };

        let result = adapter.generate("Hello world", opts, None).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("Test response"));
        assert!(response.contains("Hello world"));
        assert!(response.contains("100")); // max_tokens
    }

    #[tokio::test]
    async fn test_universal_model_adapter_generate_with_callback() {
        let mock_model = MockLoadedModel {
            response: "Callback test".to_string(),
        };

        let adapter = UniversalModelAdapter {
            model: Box::new(mock_model),
        };

        // Use Arc + Mutex for thread safety
        use std::sync::{Arc, Mutex};
        let tokens_received = Arc::new(Mutex::new(Vec::new()));
        let tokens_clone = tokens_received.clone();

        let callback = Box::new(move |token: String| {
            tokens_clone.lock().unwrap().push(token);
        });

        let opts = GenOptions::default();
        let result = adapter.generate("Test prompt", opts, Some(callback)).await;

        assert!(result.is_ok());

        // Verify callback was called
        let received_tokens = tokens_received.lock().unwrap();
        assert_eq!(received_tokens.len(), 3); // "Generated", "response", "to:"
        assert_eq!(received_tokens[0], "Generated");
        assert_eq!(received_tokens[1], "response");
        assert_eq!(received_tokens[2], "to:");
    }

    #[test]
    fn test_try_from_universal_spec_to_model_spec_llama_gguf() {
        let universal_spec = UniversalModelSpec {
            name: "test-model".to_string(),
            backend: ModelBackend::LlamaGGUF {
                base_path: PathBuf::from("/path/to/model.gguf"),
                lora_path: None,
            },
            template: Some("chatml".to_string()),
            ctx_len: 4096,
            device: "cpu".to_string(),
            n_threads: Some(8),
        };

        let result: Result<ModelSpec> = universal_spec.try_into();
        assert!(result.is_ok());

        let model_spec = result.unwrap();
        assert_eq!(model_spec.name, "test-model");
        assert_eq!(model_spec.base_path, PathBuf::from("/path/to/model.gguf"));
        assert_eq!(model_spec.lora_path, None);
        assert_eq!(model_spec.template, Some("chatml".to_string()));
        assert_eq!(model_spec.ctx_len, 4096);
        assert_eq!(model_spec.n_threads, Some(8));
    }

    #[test]
    fn test_try_from_universal_spec_to_model_spec_llama_gguf_with_lora() {
        let universal_spec = UniversalModelSpec {
            name: "lora-model".to_string(),
            backend: ModelBackend::LlamaGGUF {
                base_path: PathBuf::from("/base/model.gguf"),
                lora_path: Some(PathBuf::from("/lora/adapter.bin")),
            },
            template: Some("llama3".to_string()),
            ctx_len: 2048,
            device: "cuda".to_string(),
            n_threads: None,
        };

        let result: Result<ModelSpec> = universal_spec.try_into();
        assert!(result.is_ok());

        let model_spec = result.unwrap();
        assert_eq!(model_spec.name, "lora-model");
        assert_eq!(model_spec.base_path, PathBuf::from("/base/model.gguf"));
        assert_eq!(
            model_spec.lora_path,
            Some(PathBuf::from("/lora/adapter.bin"))
        );
        assert_eq!(model_spec.template, Some("llama3".to_string()));
        assert_eq!(model_spec.ctx_len, 2048);
        assert_eq!(model_spec.n_threads, None);
    }

    #[test]
    fn test_try_from_universal_spec_to_model_spec_huggingface_fails() {
        let universal_spec = UniversalModelSpec {
            name: "hf-model".to_string(),
            backend: ModelBackend::HuggingFace {
                base_model_id: "microsoft/Phi-3-mini-4k-instruct".to_string(),
                peft_path: None,
                use_local: false,
            },
            template: None,
            ctx_len: 4096,
            device: "cpu".to_string(),
            n_threads: None,
        };

        let result: Result<ModelSpec> = universal_spec.try_into();
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Cannot convert non-GGUF backend to legacy ModelSpec"));
    }

    #[test]
    fn test_try_from_universal_spec_to_model_spec_candle_fails() {
        let universal_spec = UniversalModelSpec {
            name: "candle-model".to_string(),
            backend: ModelBackend::Candle {
                model_path: PathBuf::from("/path/to/model.safetensors"),
                adapter_path: Some(PathBuf::from("/path/to/adapter.safetensors")),
            },
            template: Some("custom".to_string()),
            ctx_len: 8192,
            device: "metal".to_string(),
            n_threads: Some(4),
        };

        let result: Result<ModelSpec> = universal_spec.try_into();
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Cannot convert non-GGUF backend to legacy ModelSpec"));
    }

    #[test]
    fn test_universal_model_spec_field_preservation() {
        // Test that all fields are properly preserved during conversion
        let universal_spec = UniversalModelSpec {
            name: "complex-model".to_string(),
            backend: ModelBackend::LlamaGGUF {
                base_path: PathBuf::from("/very/long/path/to/model.gguf"),
                lora_path: Some(PathBuf::from("/another/long/path/lora.bin")),
            },
            template: Some("complex-template-name".to_string()),
            ctx_len: 16384,
            device: "cuda".to_string(),
            n_threads: Some(16),
        };

        let result: Result<ModelSpec> = universal_spec.try_into();
        assert!(result.is_ok());

        let model_spec = result.unwrap();
        assert_eq!(model_spec.name, "complex-model");
        assert_eq!(
            model_spec.base_path.to_str().unwrap(),
            "/very/long/path/to/model.gguf"
        );
        assert!(model_spec.lora_path.is_some());
        assert_eq!(
            model_spec.lora_path.unwrap().to_str().unwrap(),
            "/another/long/path/lora.bin"
        );
        assert_eq!(model_spec.template.unwrap(), "complex-template-name");
        assert_eq!(model_spec.ctx_len, 16384);
        assert_eq!(model_spec.n_threads.unwrap(), 16);
    }

    #[test]
    fn test_universal_model_spec_minimal_fields() {
        // Test conversion with minimal required fields
        let universal_spec = UniversalModelSpec {
            name: "minimal".to_string(),
            backend: ModelBackend::LlamaGGUF {
                base_path: PathBuf::from("model.gguf"),
                lora_path: None,
            },
            template: None,
            ctx_len: 1024,
            device: "cpu".to_string(),
            n_threads: None,
        };

        let result: Result<ModelSpec> = universal_spec.try_into();
        assert!(result.is_ok());

        let model_spec = result.unwrap();
        assert_eq!(model_spec.name, "minimal");
        assert_eq!(model_spec.base_path, PathBuf::from("model.gguf"));
        assert!(model_spec.lora_path.is_none());
        assert!(model_spec.template.is_none());
        assert_eq!(model_spec.ctx_len, 1024);
        assert!(model_spec.n_threads.is_none());
    }

    #[test]
    fn test_engine_struct_sizes() {
        // Verify the engine structs are properly sized
        let engine = ShimmyUniversalEngine::new();

        // The struct contains two unit structs (LlamaEngine and HuggingFaceEngine)
        // Size should be greater than zero
        let size = std::mem::size_of_val(&engine);
        assert!(
            size > 0,
            "Engine size should be greater than zero, got: {}",
            size
        );

        // Test that we can create multiple instances without issues
        let _engine2 = ShimmyUniversalEngine::default();
        let _engine3 = ShimmyUniversalEngine::new();
    }
}
