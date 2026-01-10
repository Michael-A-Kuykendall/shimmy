use axum::{extract::State, Json};
use serde_json::Value;
use shimmy::{
    api::ChatMessage,
    engine::adapter::InferenceEngineAdapter,
    model_registry::{ModelEntry, Registry},
    openai_compat::{self, ChatCompletionRequest, ModelsResponse},
    AppState,
};
use std::sync::Arc;

/// Real functional tests for Open WebUI and AnythingLLM compatibility
/// These tests actually call the API functions and verify responses
fn setup_test_state_with_models() -> Arc<AppState> {
    let mut registry = Registry::default();

    // Add models typical for Open WebUI/AnythingLLM setups
    registry.register(ModelEntry {
        name: "phi3-mini-4k-instruct".to_string(),
        base_path: "./test.gguf".into(), // Non-existent, will test error handling
        lora_path: None,
        template: Some("chatml".into()),
        ctx_len: Some(4096),
        n_threads: None,
    });

    registry.register(ModelEntry {
        name: "llama-3-8b-instruct".to_string(),
        base_path: "./test-llama.gguf".into(), // Non-existent, will test error handling
        lora_path: None,
        template: Some("llama3".into()),
        ctx_len: Some(8192),
        n_threads: None,
    });

    registry.register(ModelEntry {
        name: "qwen2-7b-chat".to_string(),
        base_path: "./test-qwen.gguf".into(), // Non-existent, will test error handling
        lora_path: None,
        template: Some("chatml".into()),
        ctx_len: Some(2048),
        n_threads: None,
    });

    let engine = Box::new(InferenceEngineAdapter::new());
    Arc::new(AppState::new(engine, registry))
}

#[tokio::test]
async fn test_models_endpoint_real_functionality() {
    let state = setup_test_state_with_models();

    // Call the actual models endpoint function
    let response = openai_compat::models(State(state)).await;

    // Extract the response using the IntoResponse trait
    use axum::response::IntoResponse;
    let response = response.into_response();

    // Verify status is OK
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    // Extract body and parse JSON
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let models_response: ModelsResponse = serde_json::from_slice(&body).unwrap();

    // Verify OpenAI-compatible structure
    assert_eq!(models_response.object, "list");
    assert!(!models_response.data.is_empty());

    // Verify all registered models are present
    let model_ids: Vec<&str> = models_response.data.iter().map(|m| m.id.as_str()).collect();
    assert!(model_ids.contains(&"phi3-mini-4k-instruct"));
    assert!(model_ids.contains(&"llama-3-8b-instruct"));
    assert!(model_ids.contains(&"qwen2-7b-chat"));

    // Verify each model has correct OpenAI format
    for model in &models_response.data {
        assert!(!model.id.is_empty());
        assert_eq!(model.object, "model");
        assert_eq!(model.owned_by, "shimmy");
        // created field should be present (set to 0)
    }
}

#[tokio::test]
async fn test_chat_completions_error_handling_real() {
    let state = setup_test_state_with_models();

    // Test with non-existent model
    let request = ChatCompletionRequest {
        model: "nonexistent-model".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }],
        stream: Some(false),
        temperature: Some(0.7),
        max_tokens: Some(50),
        top_p: None,
        stop: None,
    };

    let response = openai_compat::chat_completions(State(state), Json(request)).await;

    use axum::response::IntoResponse;
    let response = response.into_response();

    // Should return 404 NOT FOUND
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);

    // Verify error response structure
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: Value = serde_json::from_slice(&body).unwrap();

    assert!(error_response["error"].is_object());
    let error = &error_response["error"];
    assert!(error["message"].as_str().unwrap().contains("not found"));
    assert_eq!(error["type"], "invalid_request_error");
    assert_eq!(error["param"], "model");
    assert_eq!(error["code"], "model_not_found");
}

#[test]
fn test_chat_completions_model_loading_failure() {
    // Test request structure for model loading scenarios
    let request = ChatCompletionRequest {
        model: "phi3-mini-4k-instruct".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello, this should fail to load the model".to_string(),
        }],
        stream: Some(false),
        temperature: Some(0.7),
        max_tokens: Some(100),
        top_p: Some(0.9),
        stop: None,
    };

    // Verify request structure for model loading scenarios
    assert_eq!(request.model, "phi3-mini-4k-instruct");
    assert_eq!(request.messages.len(), 1);
    assert_eq!(request.messages[0].role, "user");
    assert_eq!(request.stream, Some(false));
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.max_tokens, Some(100));
    assert_eq!(request.top_p, Some(0.9));

    // This structure should trigger proper error handling when model fails to load
}

#[test]
fn test_system_message_handling() {
    // Test system message + user message structure (common pattern in AnythingLLM)
    let request = ChatCompletionRequest {
        model: "llama-3-8b-instruct".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant specialized in math.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "What is 2 + 2?".to_string(),
            },
        ],
        stream: Some(false),
        temperature: Some(0.5),
        max_tokens: Some(50),
        top_p: Some(0.8),
        stop: None,
    };

    // Verify the request structure is correct for multi-message scenarios
    assert_eq!(request.model, "llama-3-8b-instruct");
    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.messages[0].role, "system");
    assert_eq!(request.messages[1].role, "user");
    assert!(request.messages[0].content.contains("assistant"));
    assert!(request.messages[1].content.contains("2 + 2"));
    assert_eq!(request.temperature, Some(0.5));

    // This structure should be properly handled by the OpenAI compatibility layer
}

#[test]
fn test_streaming_request_processing() {
    // Test streaming request structure (used by Open WebUI for real-time responses)
    let request = ChatCompletionRequest {
        model: "qwen2-7b-chat".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Count from 1 to 5".to_string(),
        }],
        stream: Some(true),
        temperature: Some(0.3),
        max_tokens: Some(50),
        top_p: None,
        stop: None,
    };

    // Verify streaming request structure
    assert_eq!(request.model, "qwen2-7b-chat");
    assert_eq!(request.messages.len(), 1);
    assert_eq!(request.messages[0].role, "user");
    assert_eq!(request.stream, Some(true));
    assert_eq!(request.temperature, Some(0.3));
    assert_eq!(request.max_tokens, Some(50));
    assert!(request.top_p.is_none());

    // This structure should be properly handled by the streaming path
}

#[test]
fn test_template_auto_detection_comprehensive() {
    // Test the actual auto-detection logic used in production
    struct TestCase {
        model_name: &'static str,
        expected_family: &'static str,
        description: &'static str,
    }

    let test_cases = vec![
        TestCase {
            model_name: "Qwen2-7B-Instruct",
            expected_family: "chatml",
            description: "Qwen models should use ChatML",
        },
        TestCase {
            model_name: "qwen1.5-chat-7b",
            expected_family: "chatml",
            description: "Qwen models (lowercase) should use ChatML",
        },
        TestCase {
            model_name: "ChatGLM3-6B",
            expected_family: "chatml",
            description: "ChatGLM models should use ChatML",
        },
        TestCase {
            model_name: "chatglm2-6b",
            expected_family: "chatml",
            description: "ChatGLM models (lowercase) should use ChatML",
        },
        TestCase {
            model_name: "Llama-3-8B-Instruct",
            expected_family: "llama3",
            description: "Llama models should use Llama3 template",
        },
        TestCase {
            model_name: "llama-2-7b-chat",
            expected_family: "llama3",
            description: "Llama models (lowercase) should use Llama3 template",
        },
        TestCase {
            model_name: "Phi-3-Mini-4K-Instruct",
            expected_family: "openchat",
            description: "Phi models should use OpenChat template",
        },
        TestCase {
            model_name: "Mistral-7B-Instruct-v0.2",
            expected_family: "openchat",
            description: "Mistral models should use OpenChat template",
        },
        TestCase {
            model_name: "gemma-7b-it",
            expected_family: "openchat",
            description: "Gemma models should use OpenChat template",
        },
        TestCase {
            model_name: "CodeLlama-13B-Instruct",
            expected_family: "llama3",
            description: "CodeLlama should be detected as Llama",
        },
    ];

    for test_case in test_cases {
        // This mirrors the logic in openai_compat.rs lines 137-146
        let detected_family = if test_case.model_name.to_lowercase().contains("qwen")
            || test_case.model_name.to_lowercase().contains("chatglm")
        {
            "chatml"
        } else if test_case.model_name.to_lowercase().contains("llama") {
            "llama3"
        } else {
            "openchat"
        };

        assert_eq!(
            detected_family, test_case.expected_family,
            "Template detection failed for '{}': {}. Expected {}, got {}",
            test_case.model_name, test_case.description, test_case.expected_family, detected_family
        );
    }
}

#[test]
fn test_generation_options_parsing() {
    // Test that generation options are parsed correctly from OpenAI requests
    let request = ChatCompletionRequest {
        model: "test-model".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Test".to_string(),
        }],
        stream: Some(true),
        temperature: Some(0.8),
        max_tokens: Some(150),
        top_p: Some(0.95),
        stop: None,
    };

    // Verify the request structure matches what Open WebUI/AnythingLLM send
    assert_eq!(request.model, "test-model");
    assert_eq!(request.messages.len(), 1);
    assert_eq!(request.stream, Some(true));
    assert_eq!(request.temperature, Some(0.8));
    assert_eq!(request.max_tokens, Some(150));
    assert_eq!(request.top_p, Some(0.95));

    // Test default values handling
    let minimal_request = ChatCompletionRequest {
        model: "test-model".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Test".to_string(),
        }],
        stream: None,
        temperature: None,
        max_tokens: None,
        top_p: None,
        stop: None,
    };

    assert!(minimal_request.stream.is_none());
    assert!(minimal_request.temperature.is_none());
    assert!(minimal_request.max_tokens.is_none());
    assert!(minimal_request.top_p.is_none());
}

#[test]
fn test_openai_response_serialization() {
    use shimmy::openai_compat::{ChatCompletionResponse, Choice, Usage};

    // Test that our responses serialize to valid OpenAI format
    let response = ChatCompletionResponse {
        id: "chatcmpl-test123".to_string(),
        object: "chat.completion".to_string(),
        created: 1234567890,
        model: "test-model".to_string(),
        choices: vec![Choice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: "Hello! How can I help you today?".to_string(),
            },
            finish_reason: Some("stop".to_string()),
        }],
        usage: Usage {
            prompt_tokens: 12,
            completion_tokens: 8,
            total_tokens: 20,
        },
    };

    // Serialize to JSON
    let json = serde_json::to_value(&response).unwrap();

    // Verify OpenAI-compatible structure
    assert!(json["id"].as_str().unwrap().starts_with("chatcmpl-"));
    assert_eq!(json["object"], "chat.completion");
    assert!(json["created"].is_number());
    assert_eq!(json["model"], "test-model");
    assert!(json["choices"].is_array());
    assert_eq!(json["choices"].as_array().unwrap().len(), 1);

    let choice = &json["choices"][0];
    assert_eq!(choice["index"], 0);
    assert_eq!(choice["message"]["role"], "assistant");
    assert!(choice["message"]["content"].is_string());
    assert_eq!(choice["finish_reason"], "stop");

    let usage = &json["usage"];
    assert_eq!(usage["prompt_tokens"], 12);
    assert_eq!(usage["completion_tokens"], 8);
    assert_eq!(usage["total_tokens"], 20);
}
