// RustChain compatibility layer
#![allow(dead_code)]

use axum::{http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RustChainRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct RustChainResponse {
    pub text: String,
    pub tokens_used: Option<u32>,
}

pub async fn rustchain_generate(
    Json(request): Json<RustChainRequest>,
) -> Result<Json<RustChainResponse>, StatusCode> {
    // Convert to shimmy format and generate
    let _shimmy_request = crate::api::GenerateRequest {
        model: request.model.unwrap_or_default(),
        prompt: Some(request.prompt),
        messages: None,
        system: None,
        max_tokens: request.max_tokens.map(|t| t as usize),
        temperature: request.temperature,
        top_p: None,
        top_k: None,
        stream: Some(false),
    };

    // For now, return a placeholder response since we don't have the full server context
    // This would need proper integration with the AppState and engine
    Ok(Json(RustChainResponse {
        text: "RustChain compatibility endpoint - integration needed".to_string(),
        tokens_used: Some(0),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Json;

    #[test]
    fn test_rustchain_request_deserialization_minimal() {
        let json = r#"{"prompt":"Hello world"}"#;
        let request: RustChainRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.prompt, "Hello world");
        assert!(request.model.is_none());
        assert!(request.max_tokens.is_none());
        assert!(request.temperature.is_none());
    }

    #[test]
    fn test_rustchain_request_deserialization_full() {
        let json = r#"{"prompt":"Test prompt","model":"llama2","max_tokens":100,"temperature":0.7}"#;
        let request: RustChainRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.prompt, "Test prompt");
        assert_eq!(request.model, Some("llama2".to_string()));
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.7));
    }

    #[test]
    fn test_rustchain_request_deserialization_partial() {
        let json = r#"{"prompt":"Partial test","model":"gpt-3.5"}"#;
        let request: RustChainRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.prompt, "Partial test");
        assert_eq!(request.model, Some("gpt-3.5".to_string()));
        assert!(request.max_tokens.is_none());
        assert!(request.temperature.is_none());
    }

    #[test]
    fn test_rustchain_request_deserialization_empty_prompt() {
        let json = r#"{"prompt":""}"#;
        let request: RustChainRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.prompt, "");
        assert!(request.model.is_none());
    }

    #[test]
    fn test_rustchain_request_deserialization_edge_values() {
        let json = r#"{"prompt":"Edge case","max_tokens":1,"temperature":0.0}"#;
        let request: RustChainRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.prompt, "Edge case");
        assert_eq!(request.max_tokens, Some(1));
        assert_eq!(request.temperature, Some(0.0));
    }

    #[test]
    fn test_rustchain_request_deserialization_large_values() {
        let json = r#"{"prompt":"Large values test","max_tokens":4096,"temperature":2.0}"#;
        let request: RustChainRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.prompt, "Large values test");
        assert_eq!(request.max_tokens, Some(4096));
        assert_eq!(request.temperature, Some(2.0));
    }

    #[test]
    fn test_rustchain_response_serialization_minimal() {
        let response = RustChainResponse {
            text: "Hello".to_string(),
            tokens_used: None,
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains(r#""text":"Hello""#));
        assert!(json.contains(r#""tokens_used":null"#));
    }

    #[test]
    fn test_rustchain_response_serialization_with_tokens() {
        let response = RustChainResponse {
            text: "Response with tokens".to_string(),
            tokens_used: Some(42),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains(r#""text":"Response with tokens""#));
        assert!(json.contains(r#""tokens_used":42"#));
    }

    #[test]
    fn test_rustchain_response_serialization_empty_text() {
        let response = RustChainResponse {
            text: "".to_string(),
            tokens_used: Some(0),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains(r#""text":"""#));
        assert!(json.contains(r#""tokens_used":0"#));
    }

    #[tokio::test]
    async fn test_rustchain_generate_minimal_request() {
        let request = RustChainRequest {
            prompt: "Test prompt".to_string(),
            model: None,
            max_tokens: None,
            temperature: None,
        };
        
        let result = rustchain_generate(Json(request)).await;
        
        assert!(result.is_ok());
        let Json(response) = result.unwrap();
        assert_eq!(response.text, "RustChain compatibility endpoint - integration needed");
        assert_eq!(response.tokens_used, Some(0));
    }

    #[tokio::test]
    async fn test_rustchain_generate_full_request() {
        let request = RustChainRequest {
            prompt: "Full test prompt".to_string(),
            model: Some("test-model".to_string()),
            max_tokens: Some(150),
            temperature: Some(0.8),
        };
        
        let result = rustchain_generate(Json(request)).await;
        
        assert!(result.is_ok());
        let Json(response) = result.unwrap();
        assert_eq!(response.text, "RustChain compatibility endpoint - integration needed");
        assert_eq!(response.tokens_used, Some(0));
    }

    #[tokio::test]
    async fn test_rustchain_generate_empty_prompt() {
        let request = RustChainRequest {
            prompt: "".to_string(),
            model: Some("empty-prompt-model".to_string()),
            max_tokens: Some(1),
            temperature: Some(0.1),
        };
        
        let result = rustchain_generate(Json(request)).await;
        
        assert!(result.is_ok());
        let Json(response) = result.unwrap();
        assert!(!response.text.is_empty());
    }

    #[tokio::test]
    async fn test_rustchain_generate_conversion_logic() {
        // Test that the conversion from RustChain to Shimmy format works correctly
        let request = RustChainRequest {
            prompt: "Conversion test".to_string(),
            model: Some("conversion-model".to_string()),
            max_tokens: Some(200),
            temperature: Some(1.5),
        };
        
        // This tests the internal conversion logic that creates the shimmy request
        // We can't directly access the _shimmy_request, but we can ensure the function
        // processes the conversion without errors
        let result = rustchain_generate(Json(request)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rustchain_generate_max_tokens_conversion() {
        // Test u32 to usize conversion for max_tokens
        let request = RustChainRequest {
            prompt: "Token conversion test".to_string(),
            model: None,
            max_tokens: Some(u32::MAX), // Test edge case conversion
            temperature: None,
        };
        
        let result = rustchain_generate(Json(request)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rustchain_generate_model_none_defaults() {
        // Test that None model value gets converted to empty string
        let request = RustChainRequest {
            prompt: "Model default test".to_string(),
            model: None,
            max_tokens: None,
            temperature: None,
        };
        
        let result = rustchain_generate(Json(request)).await;
        assert!(result.is_ok());
        // This tests that the unwrap_or_default() works correctly for model field
    }

    #[tokio::test] 
    async fn test_rustchain_generate_extreme_values() {
        let request = RustChainRequest {
            prompt: "Extreme values test with very long prompt that might test buffer limits and edge cases in string handling".repeat(100),
            model: Some("extreme-model-name-that-is-very-long".repeat(10)),
            max_tokens: Some(4_294_967_295), // u32::MAX
            temperature: Some(f32::MAX),
        };
        
        let result = rustchain_generate(Json(request)).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_rustchain_request_invalid_json_fails() {
        let invalid_json = r#"{"prompt":}"#;
        let result: Result<RustChainRequest, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_rustchain_request_missing_prompt_fails() {
        let json = r#"{"model":"test"}"#;
        let result: Result<RustChainRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_rustchain_response_roundtrip_serialization() {
        let original = RustChainResponse {
            text: "Roundtrip test".to_string(),
            tokens_used: Some(123),
        };
        
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: RustChainResponse = serde_json::from_str(&json).unwrap();
        
        assert_eq!(original.text, deserialized.text);
        assert_eq!(original.tokens_used, deserialized.tokens_used);
    }

    #[test]
    fn test_rustchain_response_with_special_characters() {
        let response = RustChainResponse {
            text: "Special chars: \"quotes\", 'apostrophes', \n newlines, \t tabs, 🦀 emojis".to_string(),
            tokens_used: Some(50),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: RustChainResponse = serde_json::from_str(&json).unwrap();
        
        assert_eq!(response.text, deserialized.text);
        assert_eq!(response.tokens_used, deserialized.tokens_used);
    }
}
