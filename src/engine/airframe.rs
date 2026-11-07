use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::resident::{load_once, Resident};
use super::{GenOptions, InferenceEngine, LoadedModel, ModelSpec};

use super::stop_buffer::StopBuffer;
use airframe::runtime::gpu::{GpuRuntime, SamplingParams};
use anyhow::Context;

/// Airframe GPU inference engine — in-process, zero-HTTP.
pub struct AirframeEngine {
    runtime: Arc<Mutex<Option<Resident<GpuRuntime>>>>,
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
        let path = spec
            .base_path
            .canonicalize()
            .map_err(|_| anyhow::anyhow!("Airframe model file is unavailable"))?;
        load_once(&self.runtime, &path, || async {
            GpuRuntime::load_with_context(
                &path,
                Some(
                    spec.ctx_len.min(
                        airframe::execution::admission::inspect_model(&path)?
                            .native_context
                            .unwrap_or(spec.ctx_len),
                    ),
                ),
            )
            .await
            .map_err(crate::inference_error::from_boxed)
            .context("Airframe GPU load failed")
        })
        .await?;
        Ok(Box::new(AirframeModel {
            runtime: self.runtime.clone(),
        }))
    }
}

struct RequestControl {
    cancel: Option<crate::prepared_prompt::Cancellation>,
    stopped: std::sync::Arc<std::sync::atomic::AtomicBool>,
    inner: Option<Box<dyn airframe::control::InferenceControl + Send + Sync>>,
}
impl airframe::control::InferenceControl for RequestControl {
    fn intervene(
        &self,
        event: &airframe::control::InferenceEvent<'_>,
    ) -> airframe::control::ControlDecision {
        if self.stopped.load(std::sync::atomic::Ordering::Acquire)
            || self
                .cancel
                .as_ref()
                .is_some_and(|c| c.load(std::sync::atomic::Ordering::Acquire))
        {
            return airframe::control::ControlDecision::EarlyExit;
        }
        self.inner
            .as_ref()
            .map(|c| c.intervene(event))
            .unwrap_or(airframe::control::ControlDecision::Allow)
    }
}

struct AirframeModel {
    runtime: Arc<Mutex<Option<Resident<GpuRuntime>>>>,
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
    async fn special_tokens(&self) -> Result<Option<(String, String)>> {
        let guard = self.runtime.lock().await;
        let resident = guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Airframe runtime is unavailable"))?;
        let tok = resident.value.tokenizer_arc();
        Ok(Some((
            tok.token_to_piece(tok.bos_token())?,
            tok.token_to_piece(tok.eos_token())?,
        )))
    }
    async fn generate_prepared(
        &self,
        prepared: crate::prepared_prompt::PreparedPrompt,
        opts: GenOptions,
        cancel: Option<crate::prepared_prompt::Cancellation>,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<crate::engine::GenerationOutput> {
        if prepared.stops() != opts.stop_tokens.as_slice() {
            return Err(crate::prepared_prompt::PromptError::StopPolicy.into());
        }
        let params = Self::bridge_params(&opts);
        let runtime = self.runtime.clone();
        let result=tokio::task::spawn_blocking(move ||->Result<crate::engine::GenerationOutput,Box<dyn std::error::Error+Send+Sync>>{
        if cancel.as_ref().is_some_and(|c|c.load(std::sync::atomic::Ordering::Acquire)){return Err(crate::prepared_prompt::PromptError::Cancelled.into());}
        let guard=runtime.blocking_lock();let resident=guard.as_ref().ok_or_else(||std::io::Error::other("Airframe runtime unavailable"))?;let rt=&resident.value;
        let expected=rt.model_identity();let id=prepared.identity().ok_or(crate::prepared_prompt::PromptError::Identity)?;
        if id.model_sha256!=expected.model_sha256||id.plan_sha256!=expected.plan_sha256{return Err(crate::prepared_prompt::PromptError::Identity.into());}
        let ids=prepared.token_ids().ok_or(crate::prepared_prompt::PromptError::MissingIds)?;
        let buffer=std::sync::Arc::new(std::sync::Mutex::new(StopBuffer::new(&opts.stop_tokens)?));
        let stopped=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let callback_sink=std::sync::Arc::new(std::sync::Mutex::new(on_token));
        let callback_for_tokens=callback_sink.clone();
        let buffer_for_tokens=buffer.clone();let stopped_for_tokens=stopped.clone();
        let callback:Box<dyn FnMut(&str)+Send>=Box::new(move |piece|{
            let mut b=buffer_for_tokens.lock().unwrap();let emitted=b.push(piece);
            if b.stopped(){stopped_for_tokens.store(true,std::sync::atomic::Ordering::Release);}
            drop(b);if !emitted.is_empty(){if let Some(cb)=callback_for_tokens.lock().unwrap().as_mut(){cb(emitted);}}
        });
        let control=Some(Box::new(RequestControl{cancel,stopped:stopped.clone(),inner:build_control(&opts)}) as Box<dyn airframe::control::InferenceControl+Send+Sync>);
        let modify_logits=build_modify_logits(rt,&opts);let trace_cb=build_trace(&opts);
        let result=rt.generate_token_ids_with_metadata(ids,&params,Some(callback),control,modify_logits,trace_cb)?;
        let mut buffer=buffer.lock().unwrap();let tail=buffer.finish();
        let output_text=buffer.output().to_owned();
        drop(buffer);
        if !tail.is_empty(){if let Some(cb)=callback_sink.lock().unwrap().as_mut(){cb(tail);}}
        Ok(crate::engine::GenerationOutput{text:output_text,usage:Some(crate::engine::TokenUsage{prompt_tokens:result.prompt_tokens,completion_tokens:result.completion_tokens,total_tokens:result.prompt_tokens+result.completion_tokens}),finish_reason:Some(if stopped.load(std::sync::atomic::Ordering::Acquire){"stop"}else{match result.finish_reason{airframe::runtime::generation::FinishReason::Stop=>"stop",airframe::runtime::generation::FinishReason::Length=>"length"}}.into())})
    }).await.map_err(|e|anyhow::anyhow!("Airframe task failed: {e}"))?.map_err(crate::inference_error::from_boxed)?;
        Ok(result)
    }
    async fn prepare_prompt(
        &self,
        text: &str,
        source: crate::prompt_render::RenderSource,
        template: Option<&str>,
        stops: &[String],
    ) -> Result<crate::prepared_prompt::PreparedPrompt> {
        if source == crate::prompt_render::RenderSource::FamilyFallback {
            return Err(crate::prepared_prompt::PromptError::Template(
                "uncertified family fallback".into(),
            )
            .into());
        }
        let _ = StopBuffer::new(stops)?;
        let guard = self.runtime.lock().await;
        let resident = guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Airframe runtime is unavailable"))?;
        let rt = &resident.value;
        let ids = rt.encode_prompt(text)?;
        let identity = rt.model_identity();
        Ok(crate::prepared_prompt::PreparedPrompt::model_owned(
            text,
            source,
            template,
            stops,
            ids,
            crate::prepared_prompt::PromptIdentity {
                model_sha256: identity.model_sha256.clone(),
                plan_sha256: identity.plan_sha256.clone(),
            },
        ))
    }
    async fn thinking_policy(&self) -> Result<Option<bool>> {
        let g = self.runtime.lock().await;
        let r = g
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Airframe runtime unavailable"))?;
        Ok(
            if r.value.spec().arch == airframe::core::spec::ModelArch::Qwen3 {
                Some(false)
            } else {
                None
            },
        )
    }
    async fn chat_template(&self) -> Result<Option<String>> {
        let g = self.runtime.lock().await;
        let r = g
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Airframe runtime unavailable"))?;
        Ok(r.value.chat_template().map(str::to_string))
    }
    fn strict_prompt_contract(&self) -> bool {
        true
    }
    async fn warmup(&self) -> Result<()> {
        let runtime = self.runtime.clone();
        let task = tokio::task::spawn_blocking(move || -> Result<()> {
            let guard = runtime.blocking_lock();
            let resident = guard
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Airframe runtime is unavailable"))?;
            let rt = &resident.value;
            let params = SamplingParams {
                max_tokens: 1,
                temperature: 0.0,
                top_p: 1.0,
                repetition_penalty: 1.0,
                seed: 0,
                extra_stop_tokens: vec![],
            };
            let output = rt.generate_with_metadata("warmup", &params, None, None, None, None);
            let output = output
                .map_err(crate::inference_error::from_boxed)
                .context("Airframe warm-up forward failed")?;
            rt.try_reset()
                .map_err(crate::inference_error::from_boxed)
                .context("Airframe warm-up reset failed")?;
            anyhow::ensure!(
                output.prompt_tokens > 0 && output.completion_tokens <= 1,
                "Invalid warm-up accounting"
            );
            tracing::info!(
                prompt_tokens = output.prompt_tokens,
                completion_tokens = output.completion_tokens,
                "warmup_forward_complete"
            );
            Ok(())
        });
        let result = task
            .await
            .map_err(|_| anyhow::anyhow!("Airframe warm-up task failed"))
            .and_then(|r| r);
        if result.is_err() {
            *self.runtime.lock().await = None;
        }
        result
    }

    async fn generate(
        &self,
        prompt: &str,
        opts: GenOptions,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<String> {
        Ok(self
            .generate_with_metadata(prompt, opts, on_token)
            .await?
            .text)
    }

    async fn generate_with_metadata(
        &self,
        prompt: &str,
        opts: GenOptions,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<crate::engine::GenerationOutput> {
        let template = <Self as LoadedModel>::chat_template(self).await?;
        let mut extras =
            crate::prompt_render::extras_for_loaded_model("validated model", self).await?;
        if let Some(policy) = <Self as LoadedModel>::thinking_policy(self).await? {
            extras.enable_thinking = Some(policy);
        }
        let rendered = crate::prompt_render::render_completion_prompt_strict(
            prompt,
            template.as_deref(),
            opts.raw_prompt,
            &extras,
        )?;
        let prepared = self
            .prepare_prompt(
                &rendered.text,
                rendered.source,
                template.as_deref(),
                &opts.stop_tokens,
            )
            .await?;
        self.generate_prepared(prepared, opts, None, on_token).await
    }
}

fn build_control(
    opts: &GenOptions,
) -> Option<Box<dyn airframe::control::InferenceControl + Send + Sync>> {
    airframe::runtime::gpu::fse_control_from_patterns(&opts.fse_reject_patterns)
}

/// Pre-sample logits mask hook type shared by grammar and FSE control paths.
type LogitsMask = Box<dyn Fn(&mut [f32]) + Send + Sync>;
/// Post-sample trace callback type (step, logits, elapsed).
type TraceHook = Box<dyn FnMut(usize, &[f32], f64) + Send>;

fn build_modify_logits(rt: &GpuRuntime, opts: &GenOptions) -> Option<LogitsMask> {
    airframe::grammar::grammar_hooks(
        &opts.grammar_mode,
        rt.tokenizer_arc(),
        rt.spec().n_vocab,
        rt.eos_token(),
        rt.im_end_token(),
    )
    .map(|(mask, _control)| mask)
}

fn build_trace(opts: &GenOptions) -> Option<TraceHook> {
    airframe::runtime::gpu::trace_callback(&opts.trace_path)
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
            grammar_mode: "none".to_string(),
            fse_reject_patterns: String::new(),
            math_bypass: false,
            trace_path: String::new(),
            session_id: String::new(),
            raw_prompt: false,
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
            grammar_mode: "none".to_string(),
            fse_reject_patterns: String::new(),
            math_bypass: false,
            trace_path: String::new(),
            session_id: String::new(),
            raw_prompt: false,
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
            grammar_mode: "none".to_string(),
            fse_reject_patterns: String::new(),
            math_bypass: false,
            trace_path: String::new(),
            session_id: String::new(),
            raw_prompt: false,
            ..GenOptions::default()
        };
        let p = AirframeModel::bridge_params(&opts);
        assert_eq!(p.seed, 42u64);
    }
}
