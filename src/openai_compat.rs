use serde::{Deserialize, Serialize};
use axum::{extract::State, Json, response::IntoResponse};
use std::sync::Arc;
use crate::{api::{ChatMessage, GenerateRequest}, AppState};

#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub stream: Option<bool>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub top_p: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Serialize)]
pub struct Choice {
    pub index: usize,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Serialize)]
pub struct ChunkChoice {
    pub index: usize,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Delta {
    pub content: Option<String>,
    pub role: Option<String>,
}

pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatCompletionRequest>
) -> impl IntoResponse {
    // Convert OpenAI request to shimmy format
    let shimmy_req = GenerateRequest {
        model: req.model.clone(),
        prompt: None,
        messages: Some(req.messages),
        system: None,
        temperature: req.temperature,
        top_p: req.top_p,
        top_k: None,
        max_tokens: req.max_tokens,
        stream: req.stream,
    };
    
    // For now, delegate to existing generate endpoint
    // TODO: Convert response to proper OpenAI format
    crate::api::generate(State(state), Json(shimmy_req)).await
}
