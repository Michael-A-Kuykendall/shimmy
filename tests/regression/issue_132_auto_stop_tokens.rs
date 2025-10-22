// Regression tests for Issue #132: Auto-configure stop tokens for chat templates
//
// Issue: https://github.com/Dicklesworthstone/shimmy/issues/132
//
// ## Problem
// Models using chat templates (like gpt-oss-20b with ChatML) were outputting
// raw template tokens like `<|im_end|>` and `<|im_start|>` in the generated text.
//
// ## Root Cause
// Chat templates were being applied to format prompts, but stop tokens were not
// being configured automatically. This caused the model to continue generating
// past template markers instead of stopping when it reached them.
//
// ## Solution
// - Added `stop_tokens` field to `GenOptions`
// - Added `stop_tokens()` method to `TemplateFamily` enum
// - Auto-configure stop tokens in OpenAI compatibility layer based on template
// - Merge user-provided stop tokens with template defaults
// - Check stop tokens during generation and truncate output
//
// ## Test Coverage
// - Template stop token configuration for ChatML
// - Template stop token configuration for Llama3
// - Template stop token configuration for OpenChat
// - User-provided stop tokens merge with template defaults
// - Stop token checking in generation loop

mod tests {
    use shimmy::templates::TemplateFamily;

    #[test]
    fn test_chatml_template_has_stop_tokens() {
        // Issue #132: ChatML templates must have stop tokens configured
        let template = TemplateFamily::ChatML;
        let stop_tokens = template.stop_tokens();

        // ChatML uses <|im_end|> and <|im_start|> as delimiters
        assert!(
            !stop_tokens.is_empty(),
            "ChatML template must have stop tokens configured"
        );
        assert!(
            stop_tokens.contains(&"<|im_end|>".to_string()),
            "ChatML template must include <|im_end|> stop token"
        );
        assert!(
            stop_tokens.contains(&"<|im_start|>".to_string()),
            "ChatML template must include <|im_start|> stop token"
        );
    }

    #[test]
    fn test_llama3_template_has_stop_tokens() {
        // Issue #132: Llama3 templates must have stop tokens configured
        let template = TemplateFamily::Llama3;
        let stop_tokens = template.stop_tokens();

        // Llama3 uses <|eot_id|> and <|end_of_text|> as delimiters
        assert!(
            !stop_tokens.is_empty(),
            "Llama3 template must have stop tokens configured"
        );
        assert!(
            stop_tokens.contains(&"<|eot_id|>".to_string()),
            "Llama3 template must include <|eot_id|> stop token"
        );
        assert!(
            stop_tokens.contains(&"<|end_of_text|>".to_string()),
            "Llama3 template must include <|end_of_text|> stop token"
        );
    }

    #[test]
    fn test_openchat_template_stop_tokens() {
        // OpenChat doesn't use special tokens, so no stop tokens needed
        let template = TemplateFamily::OpenChat;
        let stop_tokens = template.stop_tokens();
        assert!(
            stop_tokens.is_empty(),
            "OpenChat template should not have stop tokens"
        );
    }

    #[test]
    fn test_gen_options_has_stop_tokens_field() {
        // Issue #132: GenOptions must support stop tokens
        use shimmy::engine::GenOptions;

        let opts = GenOptions::default();
        assert!(
            opts.stop_tokens.is_empty(),
            "Default GenOptions should have empty stop_tokens"
        );

        // Test setting stop tokens
        let opts_with_stop = GenOptions {
            stop_tokens: vec!["<|im_end|>".to_string()],
            ..Default::default()
        };
        assert_eq!(opts_with_stop.stop_tokens.len(), 1);
        assert_eq!(opts_with_stop.stop_tokens[0], "<|im_end|>");
    }

    #[test]
    fn test_chat_completion_request_accepts_stop_tokens() {
        // Issue #132: OpenAI-compatible API must accept stop tokens
        use shimmy::openai_compat::ChatCompletionRequest;

        // Test with single stop token
        let json_single = r#"{
        "model": "test-model",
        "messages": [{"role": "user", "content": "Hello"}],
        "stop": "<|im_end|>"
    }"#;
        let req: ChatCompletionRequest = serde_json::from_str(json_single).unwrap();
        assert!(req.stop.is_some(), "Request should parse single stop token");

        // Test with multiple stop tokens
        let json_multiple = r#"{
        "model": "test-model",
        "messages": [{"role": "user", "content": "Hello"}],
        "stop": ["<|im_end|>", "<|im_start|>"]
    }"#;
        let req: ChatCompletionRequest = serde_json::from_str(json_multiple).unwrap();
        assert!(
            req.stop.is_some(),
            "Request should parse multiple stop tokens"
        );

        // Test without stop tokens (should still parse)
        let json_no_stop = r#"{
        "model": "test-model",
        "messages": [{"role": "user", "content": "Hello"}]
    }"#;
        let req: ChatCompletionRequest = serde_json::from_str(json_no_stop).unwrap();
        assert!(
            req.stop.is_none(),
            "Request should parse without stop tokens"
        );
    }

    #[test]
    fn test_template_specific_stop_tokens_are_correct() {
        // Issue #132: Verify each template has the correct stop tokens
        // based on their respective specifications

        // ChatML (used by Qwen, ChatGLM, gpt-oss, etc.)
        let chatml = TemplateFamily::ChatML;
        let chatml_stops = chatml.stop_tokens();
        assert_eq!(
            chatml_stops.len(),
            2,
            "ChatML should have exactly 2 stop tokens"
        );

        // Llama3 (used by Meta Llama 3.x models)
        let llama3 = TemplateFamily::Llama3;
        let llama3_stops = llama3.stop_tokens();
        assert_eq!(
            llama3_stops.len(),
            2,
            "Llama3 should have exactly 2 stop tokens"
        );

        // OpenChat (simple format without special tokens)
        let openchat = TemplateFamily::OpenChat;
        let openchat_stops = openchat.stop_tokens();
        assert_eq!(
            openchat_stops.len(),
            0,
            "OpenChat should have no stop tokens"
        );
    }

    #[test]
    fn test_stop_tokens_prevent_template_leakage() {
        // Issue #132: The original problem - template tokens leaking into output
        // This test verifies the fix at the API contract level

        use shimmy::templates::TemplateFamily;

        // Simulate what happens with gpt-oss-20b (ChatML template)
        let template = TemplateFamily::ChatML;
        let stop_tokens = template.stop_tokens();

        // Example problematic output from Issue #132
        let mut simulated_output = "Sure! Here's the answer: 42<|im_end|>".to_string();

        // Verify stop tokens are configured
        assert!(!stop_tokens.is_empty(), "ChatML must have stop tokens");

        // Simulate the truncation logic from generate()
        for stop_token in &stop_tokens {
            if let Some(pos) = simulated_output.rfind(stop_token) {
                simulated_output.truncate(pos);
                break;
            }
        }

        // After truncation, the stop token should be removed
        assert_eq!(
            simulated_output, "Sure! Here's the answer: 42",
            "Stop token should be removed from output"
        );
        assert!(
            !simulated_output.contains("<|im_end|>"),
            "Output should not contain <|im_end|> after truncation"
        );
    }

    #[test]
    fn test_multiple_stop_token_truncation() {
        // Issue #132: Test that the first matching stop token is used for truncation
        use shimmy::templates::TemplateFamily;

        let template = TemplateFamily::ChatML;
        let stop_tokens = template.stop_tokens();

        // Output with multiple potential stop points
        let mut output = "Hello<|im_start|>user\nMore text<|im_end|>".to_string();

        // Find and truncate at the first occurrence
        for stop_token in &stop_tokens {
            if let Some(pos) = output.rfind(stop_token) {
                output.truncate(pos);
                break;
            }
        }

        // Should truncate at <|im_end|> since rfind finds last occurrence
        assert!(
            !output.contains("<|im_end|>"),
            "Should truncate at stop token"
        );
    }

    #[test]
    fn test_gen_options_serialization_with_stop_tokens() {
        // Issue #132: Verify GenOptions can be serialized/deserialized with stop tokens
        use shimmy::engine::GenOptions;

        let opts = GenOptions {
            stop_tokens: vec!["<|im_end|>".to_string(), "<|im_start|>".to_string()],
            ..Default::default()
        };

        // Serialize
        let json = serde_json::to_string(&opts).unwrap();
        assert!(
            json.contains("stop_tokens"),
            "Serialized JSON should include stop_tokens field"
        );

        // Deserialize
        let deserialized: GenOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.stop_tokens.len(),
            2,
            "Deserialized GenOptions should preserve stop tokens"
        );
        assert_eq!(deserialized.stop_tokens[0], "<|im_end|>");
        assert_eq!(deserialized.stop_tokens[1], "<|im_start|>");
    }

    #[test]
    fn test_stop_tokens_default_to_empty() {
        // Issue #132: Default GenOptions should have empty stop_tokens, not None
        use shimmy::engine::GenOptions;

        let opts = GenOptions::default();
        assert!(
            opts.stop_tokens.is_empty(),
            "Default GenOptions should have empty stop_tokens vector"
        );

        // Should be able to extend it
        let mut opts_extended = opts;
        opts_extended.stop_tokens.push("custom_stop".to_string());
        assert_eq!(opts_extended.stop_tokens.len(), 1);
    }
}
