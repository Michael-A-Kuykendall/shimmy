/// Anthropic Claude API compatibility layer
///
/// This module provides compatibility with the Anthropic Claude API format,
/// allowing tools like Claude Code to work with shimmy in local networks.
///
/// Reference: https://docs.claude.com/claude/reference/messages_post
use crate::{api::ChatMessage, AppState};
use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Anthropic Messages API request format
#[derive(Debug, Deserialize)]
pub struct AnthropicMessageRequest {
    pub model: String,
    pub max_tokens: usize, // Required in Anthropic API
    pub messages: Vec<AnthropicMessage>,
    #[serde(default)]
    pub system: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub top_k: Option<i32>,
    #[serde(default)]
    pub stream: Option<bool>,
}

/// Anthropic message format - supports complex content blocks
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnthropicMessage {
    pub role: String, // "user" or "assistant"
    pub content: AnthropicContent,
}

/// Anthropic content can be either a string or array of content blocks
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum AnthropicContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

/// Content block for complex messages (text, images, etc.)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
    pub source: Option<ImageSource>,
}

/// Image source for image content blocks
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

/// Anthropic Messages API response format
#[derive(Debug, Serialize)]
pub struct AnthropicMessageResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String, // "message"
    pub role: String, // "assistant"
    pub content: Vec<AnthropicContentBlock>,
    pub model: String,
    pub stop_reason: String,
    pub stop_sequence: Option<String>,
    pub usage: AnthropicUsage,
}

/// Response content block
#[derive(Debug, Serialize)]
pub struct AnthropicContentBlock {
    #[serde(rename = "type")]
    pub content_type: String, // "text"
    pub text: String,
}

/// Token usage information in Anthropic format
#[derive(Debug, Serialize)]
pub struct AnthropicUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

/// Convert Anthropic message format to our internal ChatMessage format
impl From<AnthropicMessage> for ChatMessage {
    fn from(msg: AnthropicMessage) -> Self {
        let content = match msg.content {
            AnthropicContent::Text(text) => text,
            AnthropicContent::Blocks(blocks) => {
                // Extract text from content blocks, ignore images for now
                blocks
                    .iter()
                    .filter_map(|block| {
                        if block.content_type == "text" {
                            block.text.clone()
                        } else {
                            // For non-text blocks, provide a placeholder
                            Some(format!("[{} content]", block.content_type))
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        };

        ChatMessage {
            role: msg.role,
            content,
        }
    }
}

/// Anthropic Messages API endpoint: POST /v1/messages
pub async fn messages(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AnthropicMessageRequest>,
) -> impl IntoResponse {
    // Convert Anthropic format to our internal format
    let internal_messages: Vec<ChatMessage> =
        req.messages.into_iter().map(|msg| msg.into()).collect();

    // Find the model
    let Some(spec) = state.registry.to_spec(&req.model) else {
        tracing::error!("Model '{}' not found in registry", req.model);
        return axum::http::StatusCode::NOT_FOUND.into_response();
    };

    // Extract system message if present
    let system_message = req.system.clone();

    // Build generation options using default values and override with request params
    let mut options = crate::engine::GenOptions {
        max_tokens: req.max_tokens,
        stream: req.stream.unwrap_or(false),
        ..Default::default()
    };

    if let Some(temp) = req.temperature {
        options.temperature = temp;
    }
    if let Some(p) = req.top_p {
        options.top_p = p;
    }
    if let Some(k) = req.top_k {
        options.top_k = k;
    }

    // Prepare the prompt using the same logic as OpenAI compatibility
    let (system_prompt, conversation_pairs) =
        extract_system_and_pairs(&internal_messages, system_message);

    let mut prompt = String::new();
    if let Some(system) = system_prompt {
        prompt.push_str(&format!("System: {}\n\n", system));
    }

    // Add conversation pairs
    for (user_msg, assistant_msg) in conversation_pairs {
        prompt.push_str(&format!("Human: {}\n", user_msg));
        if let Some(assistant) = assistant_msg {
            prompt.push_str(&format!("Assistant: {}\n", assistant));
        } else {
            prompt.push_str("Assistant: ");
        }
    }

    // Load the model and generate response
    let Ok(loaded_model) = state.engine.load(&spec).await else {
        tracing::error!("Failed to load model '{}'", req.model);
        return axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    match loaded_model.generate(&prompt, options, None).await {
        Ok(response) => {
            let anthropic_response = AnthropicMessageResponse {
                id: format!("msg_{}", Uuid::new_v4()),
                response_type: "message".to_string(),
                role: "assistant".to_string(),
                content: vec![AnthropicContentBlock {
                    content_type: "text".to_string(),
                    text: response.clone(),
                }],
                model: req.model,
                stop_reason: "end_turn".to_string(),
                stop_sequence: None,
                usage: AnthropicUsage {
                    input_tokens: estimate_tokens(&prompt),
                    output_tokens: estimate_tokens(&response),
                },
            };

            Json(anthropic_response).into_response()
        }
        Err(e) => {
            tracing::error!("Generation failed: {}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Extract system message and conversation pairs from messages
/// This mimics the logic from openai_compat.rs but adapted for Anthropic format
fn extract_system_and_pairs(
    messages: &[ChatMessage],
    explicit_system: Option<String>,
) -> (Option<String>, Vec<(&str, Option<&str>)>) {
    let mut pairs = Vec::new();
    let mut system_message = explicit_system;

    // Handle system message in first position (common pattern)
    let start_idx = if let Some(first) = messages.first() {
        if first.role == "system" {
            system_message = Some(first.content.clone());
            1
        } else {
            0
        }
    } else {
        0
    };

    // Extract user/assistant pairs
    let mut i = start_idx;
    while i < messages.len() {
        if messages[i].role == "user" {
            let user_msg = &messages[i].content;
            let assistant_msg = if i + 1 < messages.len() && messages[i + 1].role == "assistant" {
                Some(messages[i + 1].content.as_str())
            } else {
                None
            };

            pairs.push((user_msg.as_str(), assistant_msg));

            // Skip the assistant message if we found one
            if assistant_msg.is_some() {
                i += 2;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    (system_message, pairs)
}

/// Simple token estimation (rough approximation)
fn estimate_tokens(text: &str) -> usize {
    // Very rough estimate: ~1 token per 4 characters
    (text.len() as f32 / 4.0).ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ChatMessage;

    #[test]
    fn test_anthropic_message_conversion() {
        let anthropic_msg = AnthropicMessage {
            role: "user".to_string(),
            content: AnthropicContent::Text("Hello, world!".to_string()),
        };

        let chat_msg: ChatMessage = anthropic_msg.into();
        assert_eq!(chat_msg.role, "user");
        assert_eq!(chat_msg.content, "Hello, world!");
    }

    #[test]
    fn test_anthropic_content_blocks_conversion() {
        let anthropic_msg = AnthropicMessage {
            role: "user".to_string(),
            content: AnthropicContent::Blocks(vec![
                ContentBlock {
                    content_type: "text".to_string(),
                    text: Some("Hello".to_string()),
                    source: None,
                },
                ContentBlock {
                    content_type: "text".to_string(),
                    text: Some("World".to_string()),
                    source: None,
                },
            ]),
        };

        let chat_msg: ChatMessage = anthropic_msg.into();
        assert_eq!(chat_msg.content, "Hello\nWorld");
    }

    #[test]
    fn test_extract_system_and_pairs() {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Hi there!".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "How are you?".to_string(),
            },
        ];

        let (system, pairs) = extract_system_and_pairs(&messages, None);

        assert_eq!(system, Some("You are a helpful assistant".to_string()));
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], ("Hello", Some("Hi there!")));
        assert_eq!(pairs[1], ("How are you?", None));
    }

    #[test]
    fn test_explicit_system_message() {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];

        let (system, pairs) =
            extract_system_and_pairs(&messages, Some("Custom system".to_string()));

        assert_eq!(system, Some("Custom system".to_string()));
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("Hello", None));
    }

    #[test]
    fn test_token_estimation() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("test"), 1); // 4 chars = 1 token
        assert_eq!(estimate_tokens("hello world"), 3); // 11 chars = 2.75 -> 3 tokens
    }
}
