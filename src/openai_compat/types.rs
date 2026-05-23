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
    /// OpenAI-compatible penalty fields. Values in [0, 2].
    /// We map the larger of the two onto `repeat_penalty` using the formula:
    /// `repeat_penalty = 1.0 + max(frequency_penalty, presence_penalty) * 0.5`
    #[serde(default)]
    pub frequency_penalty: Option<f32>,
    #[serde(default)]
    pub presence_penalty: Option<f32>,
}

/// Request body for POST /v1/completions (legacy text completion).
#[derive(Debug, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub prompt: String,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub stream: Option<bool>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_request_deserializes() {
        let json = r#"{"model":"test","prompt":"hello"}"#;
        let req: CompletionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.model, "test");
        assert_eq!(req.prompt, "hello");
        assert!(req.temperature.is_none());
        assert!(req.max_tokens.is_none());
    }

    #[test]
    fn test_chat_request_accepts_penalty_fields() {
        let json = r#"{"model":"m","messages":[],"frequency_penalty":0.5,"presence_penalty":0.3}"#;
        let req: ChatCompletionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.frequency_penalty, Some(0.5));
        assert_eq!(req.presence_penalty, Some(0.3));
    }

    #[test]
    fn test_chat_request_penalty_fields_default_to_none() {
        let json = r#"{"model":"m","messages":[]}"#;
        let req: ChatCompletionRequest = serde_json::from_str(json).unwrap();
        assert!(req.frequency_penalty.is_none());
        assert!(req.presence_penalty.is_none());
    }

    #[test]
    fn test_stop_tokens_single() {
        let json = r#"{"model":"m","messages":[],"stop":"</s>"}"#;
        let req: ChatCompletionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.stop.unwrap().into_vec(), vec!["</s>"]);
    }

    #[test]
    fn test_stop_tokens_multiple() {
        let json = r#"{"model":"m","messages":[],"stop":["</s>","<|eot_id|>"]}"#;
        let req: ChatCompletionRequest = serde_json::from_str(json).unwrap();
        let v = req.stop.unwrap().into_vec();
        assert_eq!(v.len(), 2);
        assert!(v.contains(&"</s>".to_string()));
        assert!(v.contains(&"<|eot_id|>".to_string()));
    }
}
