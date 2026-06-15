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
    #[serde(default)]
    pub stop_tokens: Vec<String>,
}

impl Default for GenOptions {
    fn default() -> Self {
        Self {
            max_tokens: 256,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            seed: None,
            stream: true,
            stop_tokens: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelSpec {
    pub name: String,
    pub base_path: PathBuf,
    pub lora_path: Option<PathBuf>,
    pub template: Option<String>,
    pub ctx_len: usize,
    pub n_threads: Option<i32>,
}

#[async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>>;
}

#[async_trait]
pub trait LoadedModel: Send + Sync {
    async fn generate(
        &self,
        prompt: &str,
        opts: GenOptions,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<String>;

    /// Returns the Jinja2 chat template from the model's GGUF metadata, if present.
    /// Callers can use this to format prompts correctly for instruct models.
    fn chat_template(&self) -> Option<&str> {
        None // default: no template known
    }
}

pub mod adapter;
#[cfg(feature = "airframe")]
pub mod airframe;
pub mod safetensors_native;
