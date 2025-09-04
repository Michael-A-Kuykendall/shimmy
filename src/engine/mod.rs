use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenOptions {
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub repeat_penalty: f32,
    pub seed: Option<u32>,
    pub stream: bool,
}

impl Default for GenOptions {
    fn default() -> Self {
        Self { max_tokens: 256, temperature: 0.7, top_p: 0.9, top_k: 40, repeat_penalty: 1.1, seed: None, stream: true }
    }
}

// Universal backend support - true shim architecture
#[derive(Debug, Clone)]
#[cfg(feature = "huggingface")]
pub enum ModelBackend {
    // GGUF via llama.cpp (existing)
    LlamaGGUF {
        base_path: PathBuf,
        lora_path: Option<PathBuf>,
    },
    
    // HuggingFace + PEFT (your personal models!)
    HuggingFace {
        base_model_id: String,           // "microsoft/Phi-3-mini-4k-instruct"
        peft_path: Option<PathBuf>,      // "./phi3-personal-h100-cloud"
        use_local: bool,                 // Use cached vs download
    },
    
    // Pure Rust Candle (future)
    Candle {
        model_path: PathBuf,
        adapter_path: Option<PathBuf>,
    },
}

#[derive(Debug, Clone)]  
#[cfg(feature = "huggingface")]
pub struct UniversalModelSpec {
    pub name: String,
    pub backend: ModelBackend,
    pub template: Option<String>,
    pub ctx_len: usize,
    pub device: String,                  // "cpu", "cuda", "metal"
    pub n_threads: Option<i32>,
}

// Legacy ModelSpec for backward compatibility
#[derive(Debug, Clone)]
pub struct ModelSpec {
    pub name: String,
    pub base_path: PathBuf,
    pub lora_path: Option<PathBuf>,
    pub template: Option<String>,
    pub ctx_len: usize,
    pub n_threads: Option<i32>,
}

#[cfg(feature = "huggingface")]
impl From<ModelSpec> for UniversalModelSpec {
    fn from(spec: ModelSpec) -> Self {
        UniversalModelSpec {
            name: spec.name,
            backend: ModelBackend::LlamaGGUF {
                base_path: spec.base_path,
                lora_path: spec.lora_path,
            },
            template: spec.template,
            ctx_len: spec.ctx_len,
            device: "cpu".to_string(),
            n_threads: spec.n_threads,
        }
    }
}

// Universal Engine trait - supports any backend
#[async_trait]
#[cfg(feature = "huggingface")]
pub trait UniversalEngine: Send + Sync {
    async fn load(&self, spec: &UniversalModelSpec) -> Result<Box<dyn UniversalModel>>;
}

#[async_trait]
#[cfg(feature = "huggingface")]
pub trait UniversalModel: Send + Sync {
    async fn generate(&self, prompt: &str, opts: GenOptions, on_token: Option<Box<dyn FnMut(String) + Send>>) -> Result<String>;
}

// Legacy trait for backward compatibility
#[async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>>;
}

#[async_trait]
pub trait LoadedModel: Send + Sync {
    async fn generate(&self, prompt: &str, opts: GenOptions, on_token: Option<Box<dyn FnMut(String) + Send>>) -> Result<String>;
}

pub mod llama;

#[cfg(feature = "huggingface")]
pub mod huggingface;

#[cfg(feature = "huggingface")]  
pub mod universal;

pub mod adapter;
