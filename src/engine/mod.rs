use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
#[error("Airframe supports one resident model per engine; restart with the requested model")]
pub struct ResidentModelConflict;

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
    /// Force raw completion (bypass chat template). Serde default false.
    #[serde(default)]
    pub raw_prompt: bool,
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
            raw_prompt: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelSpec {
    pub name: String,
    pub base_path: PathBuf,
    pub lora_path: Option<PathBuf>,
    pub template: Option<String>,
    /// The real GGUF `tokenizer.chat_template` (Jinja) for this model, if any.
    /// Populated at registry-load from the GGUF header (see model_registry).
    pub chat_template: Option<String>,
    pub ctx_len: usize,
    pub n_threads: Option<i32>,
}

#[async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>>;
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone)]
pub struct GenerationOutput {
    pub text: String,
    pub usage: Option<TokenUsage>,
    pub finish_reason: Option<String>,
}

#[async_trait]
pub trait LoadedModel: Send + Sync {
    fn strict_prompt_contract(&self) -> bool {
        false
    }
    async fn chat_template(&self) -> Result<Option<String>> {
        Ok(None)
    }
    async fn thinking_policy(&self) -> Result<Option<bool>> {
        Ok(None)
    }
    async fn prepare_prompt(
        &self,
        text: &str,
        source: crate::prompt_render::RenderSource,
        template: Option<&str>,
        stops: &[String],
    ) -> Result<crate::prepared_prompt::PreparedPrompt> {
        Ok(crate::prepared_prompt::PreparedPrompt::backend_owned(
            text, source, template, stops,
        ))
    }
    async fn generate_prepared(
        &self,
        prepared: crate::prepared_prompt::PreparedPrompt,
        mut opts: GenOptions,
        _cancel: Option<crate::prepared_prompt::Cancellation>,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<GenerationOutput> {
        opts.raw_prompt = true;
        self.generate_with_metadata(prepared.text(), opts, on_token)
            .await
    }

    /// Model-owned BOS/EOS strings; unknown backends retain template fallbacks.
    async fn special_tokens(&self) -> Result<Option<(String, String)>> {
        Ok(None)
    }
    /// Initialize a loaded backend without accepting a user request.
    async fn warmup(&self) -> Result<()> {
        anyhow::bail!("Startup warm-up is unsupported by this backend")
    }

    async fn generate(
        &self,
        prompt: &str,
        opts: GenOptions,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<String>;

    /// Backends that cannot supply exact accounting leave metadata unavailable.
    async fn generate_with_metadata(
        &self,
        prompt: &str,
        opts: GenOptions,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<GenerationOutput> {
        Ok(GenerationOutput {
            text: self.generate(prompt, opts, on_token).await?,
            usage: None,
            finish_reason: None,
        })
    }
}

pub mod adapter;
#[cfg(feature = "airframe")]
pub mod airframe;
#[cfg(feature = "airframe")]
mod resident;
pub mod safetensors_native;
#[cfg(any(feature = "airframe", test))]
mod stop_buffer;
