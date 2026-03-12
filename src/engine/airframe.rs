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

            rt.generate(&prompt, &params, callback)
        })
        .await
        .map_err(|e| anyhow::anyhow!("Airframe task panicked: {}", e))?
        .map_err(|e| anyhow::anyhow!("Airframe generation failed: {}", e))?;

        Ok(result)
    }
}
