#![allow(clippy::too_many_arguments)]
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use parking_lot::Mutex;
use tracing::info;

use super::{GenOptions, InferenceEngine, LoadedModel, ModelSpec};

pub struct LlamaEngine;
impl LlamaEngine { pub fn new() -> Self { Self } }

#[async_trait]
impl InferenceEngine for LlamaEngine {
    async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
        #[cfg(feature = "llama")]
        {
            use std::num::NonZeroU32;
            use llama_cpp_2 as llama;
            let be = llama::llama_backend::LlamaBackend::init()?;
            let model = llama::model::LlamaModel::load_from_file(&be, &spec.base_path, &Default::default())?;
            let ctx_params = llama::context::params::LlamaContextParams::default()
                .with_n_ctx(NonZeroU32::new(spec.ctx_len as u32))
                .with_n_batch(2048)
                .with_n_ubatch(512)
                .with_n_threads(spec.n_threads.unwrap_or(std::thread::available_parallelism().map(|n| n.get() as i32).unwrap_or(4)))
                .with_n_threads_batch(spec.n_threads.unwrap_or(std::thread::available_parallelism().map(|n| n.get() as i32).unwrap_or(4)));
            let mut ctx = model.new_context(&be, ctx_params)?;
            if let Some(ref lora) = spec.lora_path {
                let adapter = model.lora_adapter_init(lora)?;
                ctx.lora_adapter_set(&adapter, 1.0).map_err(|e| anyhow!("lora set: {e:?}"))?;
                info!(adapter=%lora.display(), "LoRA adapter attached");
            }
            Ok(Box::new(LlamaLoaded { _be: be, model, ctx: Mutex::new(ctx) }))
        }
        #[cfg(not(feature = "llama"))]
        {
            let _ = spec; // silence unused warning
            Err(anyhow!("binary built without `llama` feature; recompile with --features llama"))
        }
    }
}

#[cfg(feature = "llama")]
struct LlamaLoaded {
    _be: llama_cpp_2::llama_backend::LlamaBackend,
    model: llama_cpp_2::model::LlamaModel,
    ctx: Mutex<llama_cpp_2::context::LlamaContext<'static>>,
}

#[cfg(feature = "llama")]
#[async_trait]
impl LoadedModel for LlamaLoaded {
    async fn generate(&self, prompt: &str, opts: GenOptions, mut on_token: Option<Box<dyn FnMut(&str) + Send>>) -> Result<String> {
        use llama_cpp_2::{llama_batch::LlamaBatch, model::{AddBos, Special}, sampling::LlamaSampler};
        let mut ctx = self.ctx.lock();
        let mut tokens = self.model.str_to_token(prompt, AddBos::Always)?;
        let mut batch = LlamaBatch::get_one(&tokens)?; // logits on last token
        ctx.decode(&mut batch)?;
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(opts.temperature),
            LlamaSampler::top_p(opts.top_p, 1),
            LlamaSampler::top_k(opts.top_k),
            LlamaSampler::penalties(opts.repeat_penalty, 0.0, 0.0, 64),
            LlamaSampler::greedy(),
        ]).with_tokens(tokens.iter().copied());
        let mut out = String::new();
        for _ in 0..opts.max_tokens {
            let token = sampler.sample(&ctx, 0);
            if self.model.is_eog_token(token) { break; }
            let piece = self.model.token_to_str(token, Special::TokenOnly)?;
            if let Some(cb) = on_token.as_mut() { cb(&piece); }
            out.push_str(&piece);
            let mut step = LlamaBatch::new(1, 1);
            step.add(token, (tokens.len() as i32).try_into().unwrap(), &[0], true)?;
            ctx.decode(&mut step)?;
            tokens.push(token);
        }
        Ok(out)
    }
}

#[cfg(not(feature = "llama"))]
struct LlamaLoadedStub;
#[cfg(not(feature = "llama"))]
#[async_trait]
impl LoadedModel for LlamaLoadedStub {
    async fn generate(&self, prompt: &str, _opts: GenOptions, mut on_token: Option<Box<dyn FnMut(&str) + Send>>) -> Result<String> {
        if let Some(cb) = on_token.as_mut() { cb("(stub)"); }
        Ok(format!("(shimmy stub — build with --features llama) {}", prompt))
    }
}