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
    #[serde(default)]
    pub grammar_mode: String,
    #[serde(default)]
    pub fse_reject_patterns: String,
    #[serde(default)]
    pub math_bypass: bool,
    #[serde(default)]
    pub trace_path: String,
    #[serde(default)]
    pub session_id: String,
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
            grammar_mode: "none".to_string(),
            fse_reject_patterns: String::new(),
            math_bypass: false,
            trace_path: String::new(),
            session_id: String::new(),
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
}

pub mod adapter;
#[cfg(feature = "airframe")]
pub mod airframe;
pub mod safetensors_native;
