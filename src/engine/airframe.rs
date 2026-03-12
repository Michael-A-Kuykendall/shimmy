use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::header::CONNECTION;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::{GenOptions, InferenceEngine, LoadedModel, ModelSpec};

const DEFAULT_AIRFRAME_BASE_URL: &str = "http://127.0.0.1:8080";
const STATUS_POLL_INTERVAL_MS: u64 = 250;

#[derive(Clone)]
pub struct AirframeEngine {
    base_url: String,
    client: Client,
}

impl AirframeEngine {
    pub fn new() -> Self {
        let base_url = std::env::var("SHIMMY_AIRFRAME_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_AIRFRAME_BASE_URL.to_string());

        Self {
            base_url,
            client: Client::new(),
        }
    }
}

#[derive(Clone)]
struct AirframeLoadedModel {
    base_url: String,
    client: Client,
    model_name: String,
    session_id: String,
}

#[derive(Deserialize)]
struct SubmitResponse {
    job_id: String,
}

#[derive(Deserialize)]
struct JobResult {
    text: String,
}

#[derive(Deserialize)]
struct JobStatus {
    status: String,
    result: Option<JobResult>,
    error: Option<String>,
}

#[async_trait]
impl InferenceEngine for AirframeEngine {
    async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
        Ok(Box::new(AirframeLoadedModel {
            base_url: self.base_url.clone(),
            client: self.client.clone(),
            model_name: spec.name.clone(),
            session_id: Uuid::new_v4().to_string(),
        }))
    }
}

#[async_trait]
impl LoadedModel for AirframeLoadedModel {
    async fn generate(
        &self,
        prompt: &str,
        opts: GenOptions,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<String> {
        if let Some(callback) = on_token {
            self.generate_streaming(prompt, opts, callback).await
        } else {
            self.generate_non_streaming(prompt, opts).await
        }
    }
}

impl AirframeLoadedModel {
    async fn submit_job(&self, prompt: &str, opts: &GenOptions, stream: bool) -> Result<String> {
        let url = format!("{}/api/repro/inference", self.base_url);
        let body = json!({
            "task": "chat",
            "prompt": prompt,
            "prompt_mode": "raw",
            "max_tokens": opts.max_tokens,
            "temperature": opts.temperature,
            "top_p": opts.top_p,
            "repetition_penalty": opts.repeat_penalty,
            "session_id": self.session_id,
            "stream": stream
        });

        let response = self
            .client
            .post(&url)
            .header(CONNECTION, "close")
            .json(&body)
            .send()
            .await
            .with_context(|| format!("failed to submit Airframe job for model '{}'", self.model_name))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Airframe submit failed for model '{}': {}",
                self.model_name,
                error_text
            ));
        }

        let submit: SubmitResponse = response
            .json()
            .await
            .context("failed to decode Airframe job submission response")?;
        Ok(submit.job_id)
    }

    async fn generate_non_streaming(&self, prompt: &str, opts: GenOptions) -> Result<String> {
        let job_id = self.submit_job(prompt, &opts, false).await?;
        let status_url = format!("{}/api/repro/job-status?job_id={}", self.base_url, job_id);

        loop {
            tokio::time::sleep(std::time::Duration::from_millis(STATUS_POLL_INTERVAL_MS)).await;

            let response = self
                .client
                .get(&status_url)
                .header(CONNECTION, "close")
                .send()
                .await
                .with_context(|| {
                    format!("failed to poll Airframe job status for '{}'", self.model_name)
                })?;

            if !response.status().is_success() {
                continue;
            }

            let status: JobStatus = response
                .json()
                .await
                .context("failed to decode Airframe job status response")?;

            match status.status.to_lowercase().as_str() {
                "completed" => {
                    let text = status.result.map(|result| result.text).unwrap_or_default();
                    return Ok(text);
                }
                "failed" => {
                    return Err(anyhow!(
                        "Airframe generation failed for model '{}': {}",
                        self.model_name,
                        status.error.unwrap_or_else(|| "unknown error".to_string())
                    ));
                }
                _ => {}
            }
        }
    }

    async fn generate_streaming(
        &self,
        prompt: &str,
        opts: GenOptions,
        mut on_token: Box<dyn FnMut(String) + Send>,
    ) -> Result<String> {
        use futures_util::StreamExt;

        let job_id = self.submit_job(prompt, &opts, true).await?;
        let stream_url = format!("{}/api/repro/job-stream?job_id={}", self.base_url, job_id);

        let response = self
            .client
            .get(&stream_url)
            .header(CONNECTION, "close")
            .send()
            .await
            .with_context(|| format!("failed to open Airframe stream for '{}'", self.model_name))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Airframe stream failed for model '{}': {}",
                self.model_name,
                error_text
            ));
        }

        let mut full_text = String::new();
        let mut byte_stream = response.bytes_stream();
        while let Some(item) = byte_stream.next().await {
            let bytes = item.context("failed to read Airframe stream chunk")?;
            if bytes.is_empty() {
                continue;
            }

            let piece = String::from_utf8_lossy(&bytes).to_string();
            if piece.is_empty() {
                continue;
            }

            full_text.push_str(&piece);
            on_token(piece);
        }

        Ok(full_text)
    }
}