use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{GenOptions, InferenceEngine, LoadedModel, ModelSpec};

use airframe::runtime::gpu::{GpuRuntime, SamplingParams};

/// Airframe GPU inference engine — in-process, zero-HTTP.
/// The GPU runtime is loaded once on first `load()` and reused for all subsequent requests.
pub struct AirframeEngine {
    runtime: Arc<Mutex<Option<GpuRuntime>>>,
}

impl AirframeEngine {
    pub fn new() -> Self {
        Self {
            runtime: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for AirframeEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InferenceEngine for AirframeEngine {
    async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
        let mut guard = self.runtime.lock().await;
        if guard.is_none() {
            let rt = GpuRuntime::load(&spec.base_path)
                .await
                .map_err(|e| anyhow::anyhow!("Airframe GPU load failed: {}", e))?;
            *guard = Some(rt);
        }
        // Extract chat_template from the loaded runtime
        let chat_template = guard
            .as_ref()
            .and_then(|rt| rt.chat_template())
            .map(|s| s.to_string());
        drop(guard);

        Ok(Box::new(AirframeModel {
            runtime: self.runtime.clone(),
            chat_template,
        }))
    }
}

struct AirframeModel {
    runtime: Arc<Mutex<Option<GpuRuntime>>>,
    chat_template: Option<String>,
}

// Safety: GpuRuntime is behind Arc<Mutex> and only accessed from spawn_blocking
unsafe impl Send for AirframeModel {}
unsafe impl Sync for AirframeModel {}

impl AirframeModel {
    fn bridge_params(opts: &GenOptions) -> SamplingParams {
        SamplingParams {
            max_tokens: opts.max_tokens,
            temperature: opts.temperature,
            top_p: opts.top_p,
            repetition_penalty: opts.repeat_penalty,
            seed: opts.seed.unwrap_or(42) as u64,
            extra_stop_tokens: opts.stop_tokens.clone(),
        }
    }
}

#[async_trait]
impl LoadedModel for AirframeModel {
    fn chat_template(&self) -> Option<&str> {
        self.chat_template.as_deref()
    }

    async fn generate(
        &self,
        prompt: &str,
        opts: GenOptions,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<String> {
        let params = Self::bridge_params(&opts);
        let runtime = self.runtime.clone();
        let prompt = prompt.to_string();

        // GPU compute is synchronous — run on the blocking threadpool
        let result = tokio::task::spawn_blocking(move || {
            let guard = runtime.blocking_lock();
            let rt = guard
                .as_ref()
                .expect("AirframeModel used before engine loaded");

            // Reset KV cache for each generation (stateless per-request)
            rt.reset();

            let callback: Option<Box<dyn FnMut(&str) + Send>> = on_token.map(|mut cb| {
                let wrapper: Box<dyn FnMut(&str) + Send> = Box::new(move |piece: &str| {
                    cb(piece.to_string());
                });
                wrapper
            });

            rt.generate_isf(&prompt, &params, callback)
        })
        .await
        .map_err(|e| anyhow::anyhow!("Airframe task panicked: {}", e))?
        .map_err(|e| anyhow::anyhow!("Airframe generation failed: {}", e))?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::GenOptions;

    #[test]
    fn test_bridge_params_maps_basic_fields() {
        let opts = GenOptions {
            max_tokens: 128,
            temperature: 0.5,
            top_p: 0.85,
            top_k: 40,
            repeat_penalty: 1.2,
            seed: Some(7),
            stream: false,
            stop_tokens: Vec::new(),
        };
        let p = AirframeModel::bridge_params(&opts);
        assert_eq!(p.max_tokens, 128);
        assert!((p.temperature - 0.5).abs() < 1e-6);
        assert!((p.top_p - 0.85).abs() < 1e-6);
        assert!((p.repetition_penalty - 1.2).abs() < 1e-6);
        assert_eq!(p.seed, 7u64);
        assert!(p.extra_stop_tokens.is_empty());
    }

    #[test]
    fn test_bridge_params_propagates_stop_tokens() {
        let opts = GenOptions {
            stop_tokens: vec!["<|eot_id|>".to_string(), "<|im_end|>".to_string()],
            ..GenOptions::default()
        };
        let p = AirframeModel::bridge_params(&opts);
        assert_eq!(p.extra_stop_tokens.len(), 2);
        assert!(p.extra_stop_tokens.contains(&"<|eot_id|>".to_string()));
        assert!(p.extra_stop_tokens.contains(&"<|im_end|>".to_string()));
    }

    #[test]
    fn test_bridge_params_seed_default_when_none() {
        let opts = GenOptions {
            seed: None,
            ..GenOptions::default()
        };
        let p = AirframeModel::bridge_params(&opts);
        assert_eq!(p.seed, 42u64);
    }
}
