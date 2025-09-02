#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use axum::{extract::State, Json, response::IntoResponse};
use std::sync::Arc;
use crate::{api::ChatMessage, AppState};

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

#[derive(Debug, Serialize)]
pub struct ModelsResponse {
    pub object: String,
    pub data: Vec<Model>,
}

#[derive(Debug, Serialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

pub async fn models(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let models = state.registry.list_all_available()
        .into_iter()
        .map(|name| Model {
            id: name,
            object: "model".to_string(),
            created: 0, // TODO: Add creation timestamp to model spec
            owned_by: "shimmy".to_string(),
        })
        .collect();
    
    Json(ModelsResponse {
        object: "list".to_string(),
        data: models,
    })
}

pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatCompletionRequest>
) -> impl IntoResponse {
    use axum::http::StatusCode;
    
    // Load and validate model
    let Some(spec) = state.registry.to_spec(&req.model) else { 
        return StatusCode::NOT_FOUND.into_response(); 
    };
    let engine = &state.engine;
    let Ok(loaded) = engine.load(&spec).await else { 
        return StatusCode::BAD_GATEWAY.into_response(); 
    };

    // Construct prompt from messages
    let fam = match spec.template.as_deref() {
        Some("chatml") => crate::templates::TemplateFamily::ChatML,
        Some("llama3") | Some("llama-3") => crate::templates::TemplateFamily::Llama3,
        _ => crate::templates::TemplateFamily::OpenChat,
    };
    let pairs = req.messages.iter()
        .map(|m| (m.role.clone(), m.content.clone()))
        .collect::<Vec<_>>();
    let prompt = fam.render(None, &pairs, None);

    // Set generation options
    let mut opts = crate::engine::GenOptions::default();
    if let Some(t) = req.temperature { opts.temperature = t; }
    if let Some(p) = req.top_p { opts.top_p = p; }
    if let Some(m) = req.max_tokens { opts.max_tokens = m; }
    if let Some(s) = req.stream { opts.stream = s; }

    if opts.stream {
        // Handle streaming response with proper OpenAI format
        use axum::response::sse::{Event, Sse};
        use tokio_stream::wrappers::UnboundedReceiverStream;
        use tokio_stream::StreamExt;
        
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let mut opts_clone = opts.clone(); 
        opts_clone.stream = false;
        let prompt_clone = prompt.clone();
        let model_clone = req.model.clone();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let id = format!("chatcmpl-{}", uuid::Uuid::new_v4().simple());
        
        tokio::spawn(async move {
            let tx_tokens = tx.clone();
            let id_for_tokens = id.clone();
            let model_for_tokens = model_clone.clone();
            let id_for_final = id.clone();
            let model_for_final = model_clone.clone();
            
            // Send initial chunk with role
            let initial_chunk = ChatCompletionChunk {
                id: id_for_tokens.clone(),
                object: "chat.completion.chunk".to_string(),
                created: timestamp,
                model: model_for_tokens.clone(),
                choices: vec![ChunkChoice {
                    index: 0,
                    delta: Delta {
                        role: Some("assistant".to_string()),
                        content: None,
                    },
                    finish_reason: None,
                }],
            };
            let _ = tx_tokens.send(format!("data: {}\n\n", serde_json::to_string(&initial_chunk).unwrap()));
            
            // Generate and stream tokens
            let _ = loaded.generate(&prompt_clone, opts_clone, Some(Box::new(move |tok| {
                let chunk = ChatCompletionChunk {
                    id: id_for_tokens.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created: timestamp,
                    model: model_for_tokens.clone(),
                    choices: vec![ChunkChoice {
                        index: 0,
                        delta: Delta {
                            role: None,
                            content: Some(tok),
                        },
                        finish_reason: None,
                    }],
                };
                let _ = tx_tokens.send(format!("data: {}\n\n", serde_json::to_string(&chunk).unwrap()));
            }))).await;
            
            // Send final chunk
            let final_chunk = ChatCompletionChunk {
                id: id_for_final,
                object: "chat.completion.chunk".to_string(),
                created: timestamp,
                model: model_for_final,
                choices: vec![ChunkChoice {
                    index: 0,
                    delta: Delta {
                        role: None,
                        content: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
            };
            let _ = tx.send(format!("data: {}\n\n", serde_json::to_string(&final_chunk).unwrap()));
            let _ = tx.send("data: [DONE]\n\n".to_string());
        });
        
        let stream = UnboundedReceiverStream::new(rx)
            .map(|s| Ok::<Event, std::convert::Infallible>(Event::default().data(s)));
        Sse::new(stream).into_response()
    } else {
        // Handle non-streaming response
        match loaded.generate(&prompt, opts, None).await {
            Ok(content) => {
                let response = ChatCompletionResponse {
                    id: format!("chatcmpl-{}", uuid::Uuid::new_v4().simple()),
                    object: "chat.completion".to_string(),
                    created: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    model: req.model,
                    choices: vec![Choice {
                        index: 0,
                        message: ChatMessage {
                            role: "assistant".to_string(),
                            content,
                        },
                        finish_reason: Some("stop".to_string()),
                    }],
                    usage: Usage {
                        prompt_tokens: 0, // TODO: Implement token counting
                        completion_tokens: 0,
                        total_tokens: 0,
                    },
                };
                Json(response).into_response()
            }
            Err(_) => StatusCode::BAD_GATEWAY.into_response(),
        }
    }
}
