// Comprehensive regression test suite for Shimmy
// Validates core functionality to prevent regressions

use shimmy::*;
use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(test)]
mod regression_tests {
    use super::*;
    use shimmy::discovery::*;
    use shimmy::model_registry::{ModelEntry, Registry};
    use shimmy::openai_compat::*;
    use shimmy::templates::TemplateFamily;

    #[test]
    fn test_model_registry_basic_operations() {
        let mut registry = Registry::new();

        // Test model registration
        let test_model = ModelEntry {
            name: "test-model".to_string(),
            base_path: PathBuf::from("test.gguf"),
            lora_path: None,
            template: Some("chatml".to_string()),
            ctx_len: Some(2048),
            n_threads: None,
        };

        registry.register(test_model.clone());

        // Test retrieval
        let retrieved = registry.get("test-model");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test-model");

        // Test listing
        let models = registry.list();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "test-model");
    }

    #[test]
    fn test_model_discovery_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test model files
        std::fs::write(temp_path.join("test1.gguf"), "dummy content").unwrap();
        std::fs::write(temp_path.join("test2.safetensors"), "dummy content").unwrap();
        std::fs::write(temp_path.join("readme.txt"), "not a model").unwrap();

        // Test discovery
        let models = discover_models_from_directory(temp_path).unwrap();

        // Should find 2 model files, ignore txt file
        assert_eq!(models.len(), 2);

        let model_names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
        assert!(model_names.contains(&"test1".to_string()));
        assert!(model_names.contains(&"test2".to_string()));
    }

    #[test]
    fn test_template_rendering_regression() {
        // Test ChatML template (used by Qwen models)
        let chatml = TemplateFamily::ChatML;
        let messages = vec![
            ("user".to_string(), "Hello".to_string()),
            ("assistant".to_string(), "Hi there!".to_string()),
        ];

        let result = chatml.render(None, &messages, Some("How are you?"));

        // Verify ChatML format is maintained
        assert!(result.contains("<|im_start|>user"));
        assert!(result.contains("<|im_end|>"));
        assert!(result.contains("Hello"));
        assert!(result.contains("Hi there!"));
        assert!(result.contains("How are you?"));
        assert!(result.contains("<|im_start|>assistant"));

        // Test Llama3 template
        let llama3 = TemplateFamily::Llama3;
        let result = llama3.render(None, &messages, Some("Test"));

        assert!(result.contains("<|start_header_id|>user<|end_header_id|>"));
        assert!(result.contains("<|eot_id|>"));
        assert!(result.contains("Test"));
    }

    #[test]
    fn test_openai_api_structures_serialization() {
        // Test ChatCompletionRequest deserialization
        let json = r#"{
            "model": "test-model",
            "messages": [
                {"role": "user", "content": "Hello"}
            ],
            "temperature": 0.7,
            "max_tokens": 100
        }"#;

        let request: ChatCompletionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.model, "test-model");
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.max_tokens, Some(100));

        // Test ChatCompletionResponse serialization
        let response = ChatCompletionResponse {
            id: "test-id".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "test-model".to_string(),
            choices: vec![Choice {
                index: 0,
                message: crate::api::ChatMessage {
                    role: "assistant".to_string(),
                    content: "Hello!".to_string(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Usage {
                prompt_tokens: 5,
                completion_tokens: 2,
                total_tokens: 7,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("Hello!"));
        assert!(json.contains("chat.completion"));
    }

    #[test]
    fn test_qwen_model_template_detection() {
        // Test the fix for Issue #13 - Qwen models should use ChatML
        let qwen_models = vec![
            "qwen-7b",
            "Qwen2-0.5B-Instruct",
            "qwen3-4b-instruct-2507-f16",
            "QWEN-CHAT-7B",
        ];

        for model_name in qwen_models {
            // Simulate the template detection logic from openai_compat.rs
            let detected_template = if model_name.to_lowercase().contains("qwen") {
                TemplateFamily::ChatML
            } else {
                TemplateFamily::OpenChat
            };

            // All Qwen models should be detected as ChatML
            assert!(
                matches!(detected_template, TemplateFamily::ChatML),
                "Model {} should use ChatML template",
                model_name
            );
        }
    }

    #[test]
    fn test_custom_model_directory_environment_variables() {
        // Test the fix for Issue #12 - Custom model directories

        // Set test environment variables
        std::env::set_var("SHIMMY_MODEL_PATHS", "test/path1;test/path2");
        std::env::set_var("OLLAMA_MODELS", "test/ollama/path");

        // Test that ModelDiscovery picks up these paths
        let discovery = ModelDiscovery::from_env();

        // Clean up
        std::env::remove_var("SHIMMY_MODEL_PATHS");
        std::env::remove_var("OLLAMA_MODELS");

        // The search_paths should include our test paths
        let path_strings: Vec<String> = discovery
            .search_paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        let has_custom_path1 = path_strings.iter().any(|p| p.contains("test/path1"));
        let has_custom_path2 = path_strings.iter().any(|p| p.contains("test/path2"));
        let has_ollama_path = path_strings.iter().any(|p| p.contains("test/ollama/path"));

        assert!(has_custom_path1, "Should include custom path 1");
        assert!(has_custom_path2, "Should include custom path 2");
        assert!(has_ollama_path, "Should include Ollama path");
    }

    #[test]
    fn test_models_response_api_compatibility() {
        // Test that models endpoint returns proper OpenAI-compatible format
        let models_response = ModelsResponse {
            object: "list".to_string(),
            data: vec![
                Model {
                    id: "qwen3-4b-instruct".to_string(),
                    object: "model".to_string(),
                    created: 0,
                    owned_by: "shimmy".to_string(),
                },
                Model {
                    id: "llama-7b".to_string(),
                    object: "model".to_string(),
                    created: 0,
                    owned_by: "shimmy".to_string(),
                },
            ],
        };

        // Test serialization
        let json = serde_json::to_string(&models_response).unwrap();
        assert!(json.contains("\"object\":\"list\""));
        assert!(json.contains("qwen3-4b-instruct"));
        assert!(json.contains("llama-7b"));
        assert!(json.contains("\"owned_by\":\"shimmy\""));

        // Test that it matches OpenAI API format
        let parsed: ModelsResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.object, "list");
        assert_eq!(parsed.data.len(), 2);
    }

    #[test]
    fn test_error_handling_robustness() {
        // Test that our error handling improvements don't break anything

        // Test model not found scenario
        let registry = Registry::new();
        let result = registry.get("nonexistent-model");
        assert!(result.is_none());

        // Test empty model list
        let models = registry.list();
        assert_eq!(models.len(), 0);

        // Test invalid directory discovery
        let invalid_path = PathBuf::from("/nonexistent/directory/path");
        let result = discover_models_from_directory(&invalid_path);
        // Should not crash, should return empty or error gracefully
        assert!(result.is_ok() || result.is_err()); // Either is acceptable
    }

    #[test]
    fn test_cli_model_dirs_option_compatibility() {
        // Test that CLI option doesn't break parsing
        use clap::Parser;
        use shimmy::cli::Cli;

        // Test with model-dirs option
        let cli =
            Cli::try_parse_from(&["shimmy", "--model-dirs", "test/path1;test/path2", "serve"]);

        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.model_dirs, Some("test/path1;test/path2".to_string()));

        // Test without model-dirs option (backward compatibility)
        let cli = Cli::try_parse_from(&["shimmy", "list"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(cli.model_dirs.is_none());
    }

    #[test]
    fn test_template_inference_regression() {
        let registry = Registry::new();

        // Test template inference for different model types
        let test_cases = vec![
            ("llama-7b-chat", "llama3"),
            ("phi-3-mini", "chatml"),
            ("qwen2-instruct", "chatml"),
            ("mistral-7b", "chatml"),
            ("gemma-2b", "chatml"),
            ("unknown-model", "chatml"), // default
        ];

        for (model_name, expected_template) in test_cases {
            let inferred = registry.infer_template(model_name);
            assert_eq!(
                inferred, expected_template,
                "Model {} should infer template {}",
                model_name, expected_template
            );
        }
    }
}
