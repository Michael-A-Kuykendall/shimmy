/// Advanced Rust Features Applied to Shimmy
/// 
/// This file demonstrates the advanced Rust features that have been applied
/// based on the punch discovery analysis recommendations.

use crate::{
    builders::{ModelSpecBuilder, GenOptionsBuilder},
    engine::{ValidatedGenOptions, GenOptions},
    streaming::{TokenStream, AsyncTokenCallback},
    error::ShimmyError,
    model_config, gen_options, render_template,
};

#[cfg(test)]
mod advanced_features_demo {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn demo_builder_pattern() {
        // Fluent API with builder pattern
        let spec = ModelSpecBuilder::new()
            .name("phi3-demo")
            .llama_backend("./models/phi3.gguf")
            .lora_adapter("./adapters/phi3-lora.gguf")
            .template("ChatML")
            .context_length(8192)
            .device("cuda")
            .threads(8)
            .build()
            .unwrap();
        
        assert_eq!(spec.name, "phi3-demo");
        assert_eq!(spec.ctx_len, 8192);
        assert_eq!(spec.device, "cuda");
    }

    #[test]
    fn demo_const_generics_validation() {
        let opts = GenOptions {
            max_tokens: 1024,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            seed: Some(42),
            stream: false,
        };
        
        // Compile-time validation with const generics
        let validated = ValidatedGenOptions::<2048>::new(opts).unwrap();
        let inner = validated.into_inner();
        assert_eq!(inner.max_tokens, 1024);
        
        // This would fail at compile time with a smaller limit:
        // let invalid = ValidatedGenOptions::<512>::new(opts); // Would fail
    }

    #[test]
    fn demo_macro_usage() {
        // Declarative macro for model configuration
        let config = model_config! {
            name: "test-model",
            backend: LlamaGGUF {
                base_path: "./test.gguf",
                lora_path: Some("./test-lora.gguf"),
            },
            template: "ChatML",
            ctx_len: 4096,
            device: "cpu",
            generation: {
                max_tokens: 512,
                temperature: 0.8,
                top_p: 0.95,
                top_k: 30,
            }
        };
        
        assert_eq!(config.name, "test-model");
        assert_eq!(config.ctx_len, 4096);
    }

    #[test]
    fn demo_generation_options_macro() {
        let opts = gen_options! {
            max_tokens: 1024,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.2,
            seed: 42,
            stream: true,
        };
        
        assert_eq!(opts.max_tokens, 1024);
        assert_eq!(opts.temperature, 0.7);
        assert_eq!(opts.seed, Some(42));
        assert!(opts.stream);
    }

    #[test]
    fn demo_template_macro() {
        let messages = vec![
            ("user".to_string(), "Hello".to_string()),
            ("assistant".to_string(), "Hi there!".to_string()),
        ];
        
        let rendered = render_template!(ChatML, 
            system: "You are a helpful assistant",
            messages: &messages
        );
        
        assert!(rendered.contains("<|im_start|>system"));
        assert!(rendered.contains("You are a helpful assistant"));
        assert!(rendered.contains("<|im_start|>user"));
        assert!(rendered.contains("Hello"));
    }

    #[test]
    fn demo_typed_errors() {
        let error = ShimmyError::ModelNotFound { 
            name: "nonexistent-model".to_string() 
        };
        
        match error {
            ShimmyError::ModelNotFound { name } => {
                assert_eq!(name, "nonexistent-model");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[tokio::test]
    async fn demo_async_streaming() {
        let (sender, mut stream) = TokenStream::new();
        let callback = AsyncTokenCallback::new(sender.clone());
        
        // Simulate token streaming
        tokio::spawn(async move {
            sender.send_token("Hello".to_string()).unwrap();
            sender.send_token(" world".to_string()).unwrap();
            sender.send_done().unwrap();
        });
        
        // Collect tokens from stream
        use futures_util::StreamExt;
        let mut tokens = Vec::new();
        
        while let Some(event) = stream.next().await {
            match event {
                crate::streaming::TokenEvent::Token(token) => tokens.push(token),
                crate::streaming::TokenEvent::Done => break,
                crate::streaming::TokenEvent::Error(_) => panic!("Unexpected error"),
            }
        }
        
        assert_eq!(tokens, vec!["Hello", " world"]);
    }
}

/// Example usage of all advanced features together
#[allow(dead_code)]
async fn advanced_usage_example() -> Result<(), ShimmyError> {
    // 1. Use builder pattern for configuration
    let model_spec = ModelSpecBuilder::new()
        .name("production-model")
        .llama_backend("./models/production.gguf")
        .lora_adapter("./adapters/production-lora.gguf")
        .template("ChatML")
        .context_length(16384)
        .device("cuda")
        .threads(16)
        .build()?;
    
    // 2. Create validated generation options
    let gen_opts = GenOptionsBuilder::new()
        .max_tokens(2048)
        .temperature(0.7)
        .top_p(0.9)
        .top_k(40)
        .stream(true)
        .build();
    
    // 3. Set up async streaming
    let (sender, stream) = TokenStream::new();
    let callback = AsyncTokenCallback::new(sender.clone()).into_callback();
    
    // 4. Use macro for quick template rendering
    let prompt = render_template!(ChatML,
        system: "You are an expert Rust programmer",
        messages: &[("user".to_string(), "Explain async/await".to_string())]
    );
    
    println!("Model: {}", model_spec.name);
    println!("Prompt: {}", prompt);
    println!("Generation options: {:?}", gen_opts);
    
    // In a real implementation, this would start generation with the engine
    // engine.generate(&prompt, gen_opts, Some(callback)).await?;
    
    Ok(())
}

/// Performance optimizations using advanced Rust features
pub mod performance_features {
    use rayon::prelude::*;
    use std::sync::Arc;
    
    /// Parallel model analysis using Rayon
    pub fn analyze_models_parallel(model_paths: Vec<std::path::PathBuf>) -> Vec<ModelAnalysis> {
        model_paths
            .into_par_iter()
            .map(|path| analyze_single_model(path))
            .collect()
    }
    
    #[derive(Debug)]
    pub struct ModelAnalysis {
        pub path: std::path::PathBuf,
        pub size_mb: f64,
        pub estimated_parameters: Option<u64>,
    }
    
    fn analyze_single_model(path: std::path::PathBuf) -> ModelAnalysis {
        let size_mb = std::fs::metadata(&path)
            .map(|m| m.len() as f64 / (1024.0 * 1024.0))
            .unwrap_or(0.0);
        
        // Estimate parameters based on file size (rough heuristic)
        let estimated_parameters = if size_mb > 100.0 {
            Some((size_mb * 1_000_000.0) as u64)
        } else {
            None
        };
        
        ModelAnalysis {
            path,
            size_mb,
            estimated_parameters,
        }
    }
}

/// Memory-safe smart pointer patterns
pub mod memory_patterns {
    use std::sync::{Arc, Weak};
    use std::collections::HashMap;
    use tokio::sync::RwLock;
    
    /// Smart pointer-based caching to prevent memory leaks
    pub struct SmartModelCache {
        // Strong references for active models
        active: Arc<RwLock<HashMap<String, Arc<CachedModel>>>>,
        // Weak references for recently used models
        recent: Arc<RwLock<HashMap<String, Weak<CachedModel>>>>,
    }
    
    pub struct CachedModel {
        pub name: String,
        pub data: Vec<u8>,
        pub last_accessed: std::time::Instant,
    }
    
    impl SmartModelCache {
        pub fn new() -> Self {
            Self {
                active: Arc::new(RwLock::new(HashMap::new())),
                recent: Arc::new(RwLock::new(HashMap::new())),
            }
        }
        
        pub async fn get_or_load(&self, name: &str) -> Option<Arc<CachedModel>> {
            // Try active cache first
            {
                let active = self.active.read().await;
                if let Some(model) = active.get(name) {
                    return Some(Arc::clone(model));
                }
            }
            
            // Try recent cache with weak references
            {
                let recent = self.recent.read().await;
                if let Some(weak_ref) = recent.get(name) {
                    if let Some(model) = weak_ref.upgrade() {
                        // Move back to active cache
                        drop(recent);
                        let mut active = self.active.write().await;
                        active.insert(name.to_string(), Arc::clone(&model));
                        return Some(model);
                    }
                }
            }
            
            // Would load from disk in real implementation
            None
        }
    }
}
