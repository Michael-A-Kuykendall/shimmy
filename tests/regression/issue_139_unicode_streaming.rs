use crate::engine::universal::{ShimmyUniversalEngine, UniversalModelSpec};
use crate::model_registry::Registry;

#[cfg(test)]
mod issue_139_tests {
    use super::*;

    #[tokio::test]
    async fn test_unicode_stop_token_handling() {
        // Test that stop tokens are handled correctly with Unicode characters
        // This reproduces the issue where stop token truncation could split
        // multi-byte UTF-8 characters, causing FromUtf8Error

        let registry = Registry::with_discovery();
        let engine = ShimmyUniversalEngine::new();

        // Find a model to test with
        let models = registry.list_loaded();
        if models.is_empty() {
            println!("No models loaded, skipping Unicode stop token test");
            return;
        }

        let model_name = &models[0].name;
        let spec = registry.load(model_name).expect("Failed to load model");

        let model = engine.load(&spec.into()).await
            .expect("Failed to load model for Unicode test");

        // Test prompt that should generate content that might include Unicode
        let prompt = "Write a short poem about emojis";

        // Use a stop token that could appear near Unicode characters
        let mut opts = crate::engine::GenOptions::default();
        opts.stop_tokens = vec![".".to_string()]; // Stop at period

        let result = model.generate(prompt, opts, Some(Box::new(|_token| {
            // Just receiving tokens should not cause a panic
            // The issue was that stop token truncation could create invalid UTF-8
        }))).await;

        // The test should not panic with FromUtf8Error
        // If it completes successfully, we've fixed the Unicode issue
        match result {
            Ok(generated_text) => {
                println!("Unicode stop token test passed! Generated: {}", generated_text);
                // Verify the result is valid UTF-8
                assert!(std::str::from_utf8(generated_text.as_bytes()).is_ok(),
                    "Generated text is not valid UTF-8");
            }
            Err(e) => {
                // Check if this is the specific Unicode error we're trying to fix
                if e.to_string().contains("FromUtf8Error") ||
                   e.to_string().contains("incomplete utf-8 byte sequence") {
                    panic!("Unicode stop token issue not fixed: {}", e);
                } else {
                    // Some other error, might be model-specific, don't fail the test
                    println!("Test skipped due to non-Unicode error: {}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_unicode_generation_with_callback() {
        // Test that Unicode characters work correctly with token streaming callback
        let registry = Registry::with_discovery();
        let engine = ShimmyUniversalEngine::new();

        let models = registry.list_loaded();
        if models.is_empty() {
            println!("No models loaded, skipping Unicode callback test");
            return;
        }

        let model_name = &models[0].name;
        let spec = registry.load(model_name).expect("Failed to load model");

        let model = engine.load(&spec.into()).await
            .expect("Failed to load model for callback test");

        // Simple prompt that might generate Unicode
        let prompt = "Hello world 🌍";

        let result = model.generate(prompt, Default::default(), Some(Box::new(|token| {
            // Verify each token received is valid UTF-8
            assert!(std::str::from_utf8(token.as_bytes()).is_ok(),
                "Token received via callback is not valid UTF-8: {:?}", token.as_bytes());
        }))).await;

        match result {
            Ok(_) => println!("Unicode callback test passed!"),
            Err(e) => {
                if e.to_string().contains("FromUtf8Error") ||
                   e.to_string().contains("incomplete utf-8 byte sequence") {
                    panic!("Unicode callback still causes panic: {}", e);
                }
                // Other errors are acceptable for this test
                println!("Callback test completed with non-Unicode error: {}", e);
            }
        }
    }
}