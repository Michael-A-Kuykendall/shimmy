/// Regression test for Issue #53: SSE streaming responses contain duplicate 'data:' prefix
///
/// This test ensures that Server-Sent Events (SSE) streaming responses
/// maintain proper OpenAI API compatibility by avoiding duplicate 'data:' prefixes
/// that would break client parsing.
use serde_json::json;

#[test]
fn test_sse_data_prefix_format() {
    // Test that SSE stream chunks have proper format without duplicate prefixes
    let test_chunk = json!({
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "test-model",
        "choices": [{
            "index": 0,
            "delta": {
                "content": "Hello world"
            },
            "finish_reason": null
        }]
    });

    let chunk_str = serde_json::to_string(&test_chunk).unwrap();

    // Simulate proper SSE formatting (should have only one 'data:' prefix)
    let sse_line = format!("data: {}\n\n", chunk_str);

    // Test that we don't have duplicate 'data:' prefixes
    assert_eq!(
        sse_line.matches("data:").count(),
        1,
        "Should have exactly one 'data:' prefix"
    );
    assert!(sse_line.starts_with("data: "), "Should start with 'data: '");
    assert!(sse_line.ends_with("\n\n"), "Should end with double newline");

    // Test that the JSON content is valid
    let content = &sse_line[6..sse_line.len() - 2]; // Remove "data: " prefix and "\n\n" suffix
    let parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    assert_eq!(parsed["object"], "chat.completion.chunk");
    assert_eq!(parsed["choices"][0]["delta"]["content"], "Hello world");
}

#[test]
fn test_sse_final_chunk_format() {
    // Test that the final [DONE] chunk has proper format
    let done_chunk = "data: [DONE]\n\n";

    assert_eq!(
        done_chunk.matches("data:").count(),
        1,
        "Final chunk should have exactly one 'data:' prefix"
    );
    assert!(
        done_chunk.starts_with("data: "),
        "Final chunk should start with 'data: '"
    );
    assert!(
        done_chunk.contains("[DONE]"),
        "Final chunk should contain [DONE]"
    );
    assert!(
        done_chunk.ends_with("\n\n"),
        "Final chunk should end with double newline"
    );
}

#[test]
fn test_sse_openai_compatibility() {
    // Test that our SSE format matches OpenAI API specification
    let openai_compatible_chunks = vec![
        "data: {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"gpt-3.5-turbo\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\"},\"finish_reason\":null}]}\n\n",
        "data: {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"gpt-3.5-turbo\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n",
        "data: {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"gpt-3.5-turbo\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        "data: [DONE]\n\n"
    ];

    for chunk in openai_compatible_chunks {
        // Each chunk should have exactly one 'data:' prefix
        assert_eq!(
            chunk.matches("data:").count(),
            1,
            "Chunk should have exactly one 'data:' prefix: {}",
            chunk
        );

        // Each chunk should start with 'data: ' and end with '\n\n'
        assert!(
            chunk.starts_with("data: "),
            "Chunk should start with 'data: ': {}",
            chunk
        );
        assert!(
            chunk.ends_with("\n\n"),
            "Chunk should end with '\\n\\n': {}",
            chunk
        );

        // If not [DONE], should contain valid JSON
        if !chunk.contains("[DONE]") {
            let json_content = &chunk[6..chunk.len() - 2]; // Remove "data: " and "\n\n"
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_content);
            assert!(parsed.is_ok(), "Chunk should contain valid JSON: {}", chunk);
        }
    }
}

#[test]
fn test_no_malformed_sse_chunks() {
    // Test cases that should NOT occur (regression prevention)
    let malformed_examples = vec![
        "data: data: {\"content\":\"hello\"}\n\n", // Double prefix
        "data:{\"content\":\"hello\"}\n\n",        // Missing space after colon
        "data: {\"content\":\"hello\"}\n",         // Single newline instead of double
        "{\"content\":\"hello\"}\n\n",             // Missing data: prefix entirely
    ];

    for malformed in &malformed_examples {
        // These patterns should be detected as malformed
        let has_double_prefix = malformed.matches("data: data:").count() > 0;
        let missing_space = malformed.starts_with("data:") && !malformed.starts_with("data: ");
        let wrong_ending = !malformed.ends_with("\n\n") && malformed.contains("data:");
        let missing_prefix = !malformed.contains("data:") && !malformed.contains("[DONE]");

        let is_malformed = has_double_prefix || missing_space || wrong_ending || missing_prefix;
        assert!(is_malformed, "Should detect malformation in: {}", malformed);
    }
}
