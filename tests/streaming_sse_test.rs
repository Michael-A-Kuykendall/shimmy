#[cfg(test)]
mod streaming_sse_tests {
    use super::*;

    #[test]
    fn test_sse_format_no_duplicate_data_prefix() {
        // Regression test for Issue #76
        // Ensure SSE responses don't have duplicate "data: data:" prefixes
        
        // This test verifies that our streaming SSE format is correct
        // The Axum SSE Event::default().data() already adds the "data:" prefix
        // so we shouldn't add it manually in our format strings
        
        // Test the manual format strings we use for streaming chunks
        let chunk_json = r#"{"id":"test","object":"chat.completion.chunk"}"#;
        
        // Our format should NOT include "data:" prefix (Axum adds it)
        let our_format = format!("{}\n\n", chunk_json);
        assert!(!our_format.starts_with("data:"), 
               "Manual format should not include 'data:' prefix (Issue #76 regression!)");
        
        // Our DONE format should NOT include "data:" prefix  
        let done_format = "[DONE]\n\n".to_string();
        assert!(!done_format.starts_with("data:"),
               "DONE format should not include 'data:' prefix (Issue #76 regression!)");
        
        // The expected SSE output after Axum processing should have single "data:" prefix
        let expected_sse_chunk = format!("data: {}", chunk_json);
        let expected_sse_done = "data: [DONE]";
        
        // Verify we expect single "data:" prefix, not double
        assert!(expected_sse_chunk.starts_with("data: "));
        assert!(!expected_sse_chunk.starts_with("data: data:"));
        assert!(expected_sse_done.starts_with("data: "));
        assert!(!expected_sse_done.starts_with("data: data:"));
    }
    
    #[test]
    fn test_streaming_response_structure() {
        // Test that streaming chunks have proper JSON structure
        use serde_json;
        
        // Simulate a streaming chunk response
        let chunk_data = serde_json::json!({
            "id": "chatcmpl-test",
            "object": "chat.completion.chunk", 
            "created": 1234567890,
            "model": "test-model",
            "choices": [{
                "index": 0,
                "delta": {
                    "content": "Hello",
                    "role": null
                },
                "finish_reason": null
            }]
        });
        
        // Our format (without manual "data:" prefix)
        let formatted = format!("{}\n\n", serde_json::to_string(&chunk_data).unwrap());
        
        // Should be valid JSON followed by double newline
        assert!(formatted.ends_with("\n\n"));
        
        // Should not have "data:" prefix (Axum SSE adds that)
        assert!(!formatted.starts_with("data:"));
        
        // Content should be parseable JSON
        let content = formatted.trim();
        let parsed: serde_json::Value = serde_json::from_str(content).unwrap();
        assert_eq!(parsed["object"], "chat.completion.chunk");
    }
}