use crate::api::ChatMessage;
use serde::{Deserialize, Serialize};

/// OpenAI spec allows `content` to be either a plain string or an array of content parts
/// (used by Zed, Cursor, Continue, GitHub Copilot when attaching file context).
/// We flatten arrays to a newline-joined string before passing to the engine.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

impl MessageContent {
    pub fn into_text(self) -> String {
        match self {
            MessageContent::Text(s) => s,
            MessageContent::Parts(parts) => parts
                .into_iter()
                .filter_map(|p| p.text)
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    pub fn as_text(&self) -> String {
        match self {
            MessageContent::Text(s) => s.clone(),
            MessageContent::Parts(parts) => parts
                .iter()
                .filter_map(|p| p.text.as_deref().map(str::to_owned))
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ContentPart {
    #[serde(rename = "type")]
    pub part_type: String,
    pub text: Option<String>,
}

/// OpenAI-compatible message for incoming requests — supports both string and
/// multi-part content arrays per the OpenAI Chat Completions spec.
#[derive(Debug, Deserialize)]
pub struct OAIMessage {
    pub role: String,
    pub content: MessageContent,
}

impl OAIMessage {
    pub fn content_text(&self) -> String {
        self.content.as_text()
    }
    pub fn into_chat_message(self) -> ChatMessage {
        ChatMessage {
            role: self.role,
            content: self.content.into_text(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<OAIMessage>,
    #[serde(default)]
    pub stream: Option<bool>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub stop: Option<StopTokens>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum StopTokens {
    Single(String),
    Multiple(Vec<String>),
}

impl StopTokens {
    pub(super) fn into_vec(self) -> Vec<String> {
        match self {
            StopTokens::Single(s) => vec![s],
            StopTokens::Multiple(v) => v,
        }
    }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkChoice {
    pub index: usize,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Delta {
    pub content: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub object: String,
    pub data: Vec<ListModel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListModel {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

/// Extended OpenAI model representation with optional permission/lineage fields.
/// Defined for API completeness; not yet surfaced by any active route.
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}
