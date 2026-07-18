use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{GenOptions, InferenceEngine, LoadedModel, ModelSpec};

use airframe::control::{ControlDecision, InferenceControl, InferenceEvent};
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

/// Composite InferenceControl that advances grammar state post-sample,
/// then delegates to an optional inner control (FSE).
struct GrammarAdvancer {
    grammar: Arc<std::sync::Mutex<schoolmarm::GrammarState>>,
    prev_text_len: std::sync::Mutex<usize>,
}

impl InferenceControl for GrammarAdvancer {
    fn intervene(&self, event: &InferenceEvent<'_>) -> ControlDecision {
        let text = event.text;
        let mut prev = self.prev_text_len.lock().unwrap();
        if text.len() > *prev {
            if let Ok(mut gs) = self.grammar.lock() {
                gs.advance_token(&text[*prev..]);
            }
            *prev = text.len();
        }
        ControlDecision::Allow
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

            // Build shared grammar state for both modify_logits + composite control
            let grammar_state = build_grammar_state(&opts, rt);
            let modify_fn = grammar_state.as_ref().map(|gs| {
                let gs_arc = gs.clone();
                let vocab: Vec<String> = (0..rt.spec().n_vocab)
                    .map(|i| rt.tokenizer().decode_single(i as u32, true).unwrap_or_default())
                    .collect();
                let eos = rt.tokenizer().eos_token();
                let im_end = rt.tokenizer().encode("<|im_end|>", false).ok().and_then(|v| {
                    if v.len() == 1 { Some(v[0]) } else { None }
                });
                Box::new(move |logits: &mut [f32]| {
                    if let Ok(state) = gs_arc.lock() {
                        apply_grammar_mask(logits, &state, &vocab, eos, im_end);
                    }
                }) as Box<dyn Fn(&mut [f32]) + Send + Sync>
            });

            // Build composite control: grammar advance + FSE
            let fse_ctrl = airframe::runtime::gpu::fse_control_from_patterns(&opts.fse_reject_patterns);
            let control: Option<Box<dyn InferenceControl + Send + Sync>> = if let Some(ga) = grammar_state {
                let advancer = GrammarAdvancer {
                    grammar: ga,
                    prev_text_len: std::sync::Mutex::new(0),
                };
                if let Some(fse) = fse_ctrl {
                    let fse_arc = std::sync::Arc::new(fse);
                    struct Both {
                        adv: GrammarAdvancer,
                        fse: std::sync::Arc<Box<dyn InferenceControl + Send + Sync>>,
                    }
                    impl InferenceControl for Both {
                        fn intervene(&self, event: &InferenceEvent<'_>) -> ControlDecision {
                            self.adv.intervene(event);
                            self.fse.intervene(event)
                        }
                    }
                    Some(Box::new(Both { adv: advancer, fse: std::sync::Arc::new(fse) }) as Box<dyn InferenceControl + Send + Sync>)
                } else {
                    Some(Box::new(advancer) as Box<dyn InferenceControl + Send + Sync>)
                }
            } else {
                fse_ctrl
            };

            let trace_cb = airframe::runtime::gpu::trace_callback(&opts.trace_path);

            let callback: Option<Box<dyn FnMut(&str) + Send>> = on_token.map(|mut cb| {
                let wrapper: Box<dyn FnMut(&str) + Send> = Box::new(move |piece: &str| {
                    cb(piece.to_string());
                });
                wrapper
            });

            rt.generate(&prompt, &params, callback, control, modify_fn, trace_cb)
        })
        .await
        .map_err(|e| anyhow::anyhow!("Airframe task panicked: {}", e))?
        .map_err(|e| anyhow::anyhow!("Airframe generation failed: {}", e))?;

        Ok(result)
    }
}

fn build_grammar_state(opts: &GenOptions, rt: &GpuRuntime) -> Option<Arc<std::sync::Mutex<schoolmarm::GrammarState>>> {
    if opts.grammar_mode != "developer" {
        return None;
    }
    let grammar_text = r###"
root ::= rust-file
rust-file ::= "// BEGIN_RUST_FILE\n" rust-code "// END_RUST_FILE\n"
rust-code ::= (!("// END_RUST_FILE" | "#include" | "```" | "#"))+
"###;
    let grammar = schoolmarm::Grammar::new(grammar_text).ok()?;
    let gs = schoolmarm::GrammarState::new(grammar).ok()?;
    Some(Arc::new(std::sync::Mutex::new(gs)))
}

fn apply_grammar_mask(
    logits: &mut [f32],
    grammar_state: &schoolmarm::GrammarState,
    vocab_texts: &[String],
    eos_token: u32,
    im_end_token: Option<u32>,
) {
    let vocab_refs: Vec<&str> = vocab_texts.iter().map(|s| s.as_str()).collect();
    let allowed = grammar_state.allowed_tokens(&vocab_refs);
    for (idx, logit) in logits.iter_mut().enumerate() {
        if idx >= allowed.len() || !allowed[idx] {
            *logit = f32::NEG_INFINITY;
        }
    }
    if grammar_state.is_accepting() {
        let eos_idx = eos_token as usize;
        if eos_idx < logits.len() {
            logits[eos_idx] = 0.0;
        }
        if let Some(im_end) = im_end_token {
            let im_end_idx = im_end as usize;
            if im_end_idx < logits.len() {
                logits[im_end_idx] = 0.0;
            }
        }
    }
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
