use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{GenOptions, InferenceEngine, LoadedModel, ModelSpec};

use airframe::runtime::gpu::{GpuRuntime, SamplingParams};

/// Airframe GPU inference engine — in-process, zero-HTTP.
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
        drop(guard);

        Ok(Box::new(AirframeModel {
            runtime: self.runtime.clone(),
        }))
    }
}

struct AirframeModel {
    runtime: Arc<Mutex<Option<GpuRuntime>>>,
}

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
    async fn generate(
        &self,
        prompt: &str,
        opts: GenOptions,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<String> {
        let params = Self::bridge_params(&opts);
        let runtime = self.runtime.clone();
        let prompt = prompt.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let guard = runtime.blocking_lock();
            let rt = guard
                .as_ref()
                .expect("AirframeModel used before engine loaded");
            rt.reset();

            // Build hooks from opts. Grammar mask + control come from
            // airframe::grammar::grammar_hooks and share one GrammarState
            // (the mask reads it, the control advances it), so when a
            // grammar mode is active its control takes precedence over the
            // FSE reject-pattern control.
            let fse_control = build_control(&opts);
            let (modify_logits, grammar_control) = match airframe::grammar::grammar_hooks(
                &opts.grammar_mode,
                rt.tokenizer_arc(),
                rt.spec().n_vocab,
                rt.eos_token(),
                rt.im_end_token(),
            ) {
                Some((mask, control)) => (Some(mask), Some(control)),
                None => (None, None),
            };
            let control = grammar_control.or(fse_control);
            let trace_cb = build_trace(&opts);

            let callback: Option<Box<dyn FnMut(&str) + Send>> = on_token.map(|mut cb| {
                let wrapper: Box<dyn FnMut(&str) + Send> = Box::new(move |piece: &str| {
                    cb(piece.to_string());
                });
                wrapper
            });

            rt.generate(&prompt, &params, callback, control, modify_logits, trace_cb)
        })
        .await
        .map_err(|e| anyhow::anyhow!("Airframe task panicked: {}", e))?
        .map_err(|e| anyhow::anyhow!("Airframe generation failed: {}", e))?;

        Ok(result)
    }
}

fn build_control(opts: &GenOptions) -> Option<Box<dyn airframe::control::InferenceControl + Send + Sync>> {
    airframe::runtime::gpu::fse_control_from_patterns(&opts.fse_reject_patterns)
}

fn build_trace(opts: &GenOptions) -> Option<Box<dyn FnMut(usize, &[f32], f64) + Send>> {
    airframe::runtime::gpu::trace_callback(&opts.trace_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::GenOptions;

    #[test]
    fn test_bridge_params_maps_basic_fields() {
        let opts = GenOptions {
            max_tokens: 128, temperature: 0.5, top_p: 0.85, top_k: 40,
            repeat_penalty: 1.2, seed: Some(7), stream: false,
            stop_tokens: Vec::new(),
            grammar_mode: "none".to_string(), fse_reject_patterns: String::new(),
            math_bypass: false, trace_path: String::new(), session_id: String::new(),
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
            grammar_mode: "none".to_string(), fse_reject_patterns: String::new(),
            math_bypass: false, trace_path: String::new(), session_id: String::new(),
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
            grammar_mode: "none".to_string(), fse_reject_patterns: String::new(),
            math_bypass: false, trace_path: String::new(), session_id: String::new(),
            ..GenOptions::default()
        };
        let p = AirframeModel::bridge_params(&opts);
        assert_eq!(p.seed, 42u64);
    }
}
