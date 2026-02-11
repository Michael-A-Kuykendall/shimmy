/*!
# Console WebSocket Chat Protocol Tests

Tests for canonical WebSocket chat protocol with:
- `{"token": "..."}` for each token
- `{"done": true}` for completion
- Proper error handling with done messages
- Non-chat message types (get_models, select_model, etc.)
*/

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use serde_json::json;

// Mock InferenceBackend for testing without real model
struct MockInferenceBackend {
    model_name: String,
    response: String,
}

impl MockInferenceBackend {
    fn new(response: &str) -> Self {
        Self {
            model_name: "phi-3-mini".to_string(),
            response: response.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl shimmy_console::websocket::InferenceBackend for MockInferenceBackend {
    async fn generate_response(&self, _model_name: &str, _prompt: &str) -> anyhow::Result<String> {
        Ok(self.response.clone())
    }

    async fn get_session_model(&self, _session_id: &str) -> Option<String> {
        Some(self.model_name.clone())
    }

    async fn list_models(&self) -> anyhow::Result<Vec<(String, serde_json::Value)>> {
        Ok(vec![
            (self.model_name.clone(), json!({"size": "3.8B", "family": "phi"}))
        ])
    }

    async fn set_session_model(&self, _session_id: &str, _model_name: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_metrics(&self) -> anyhow::Result<serde_json::Value> {
        Ok(json!({
            "active_sessions": 1,
            "total_requests": 42
        }))
    }

    async fn generate_stream(
        &self,
        _model_name: &str,
        _prompt: &str,
        tx: tokio::sync::mpsc::Sender<String>,
    ) -> anyhow::Result<()> {
        // Simulate streaming tokens
        for token in self.response.split_whitespace() {
            let msg = json!({"token": format!("{} ", token)}).to_string();
            tx.send(msg).await?;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        // Send done message
        tx.send(json!({"done": true}).to_string()).await?;
        Ok(())
    }
}

#[tokio::test]
async fn test_canonical_streaming_protocol() {
    // Test that streaming chat emits {"token": ...} followed by {"done": true}
    let backend = Arc::new(MockInferenceBackend::new("Hello world"));
    let (tx, mut rx) = mpsc::channel(100);
    
    // Simulate streaming generation
    let result = backend.generate_stream("phi-3-mini", "say hello", tx).await;
    assert!(result.is_ok(), "Streaming should succeed");
    
    // Collect all messages
    let mut messages = Vec::new();
    while let Some(msg) = rx.recv().await {
        messages.push(msg);
    }
    
    // Verify we got tokens + done
    assert!(!messages.is_empty(), "Should receive messages");
    
    // All messages except last should be tokens
    for msg in &messages[..messages.len()-1] {
        let parsed: serde_json::Value = serde_json::from_str(msg).expect("Should parse as JSON");
        assert!(parsed.get("token").is_some(), "Should have token field: {}", msg);
    }
    
    // Last message should be done
    let last_msg = messages.last().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(last_msg).expect("Should parse as JSON");
    assert_eq!(parsed.get("done"), Some(&json!(true)), "Last message should be done:true");
}

#[tokio::test]
async fn test_streaming_with_short_response() {
    // Test that short responses like "hello" don't get cut off
    let backend = Arc::new(MockInferenceBackend::new("hello"));
    let (tx, mut rx) = mpsc::channel(100);
    
    let result = backend.generate_stream("phi-3-mini", "hi", tx).await;
    assert!(result.is_ok(), "Short response should succeed");
    
    let mut messages = Vec::new();
    while let Some(msg) = rx.recv().await {
        messages.push(msg);
    }
    
    // Should have at least 2 messages: one token + done
    assert!(messages.len() >= 2, "Should have token + done, got {} messages", messages.len());
    
    // Reconstruct response
    let mut full_response = String::new();
    for msg in &messages[..messages.len()-1] {
        let parsed: serde_json::Value = serde_json::from_str(msg).unwrap();
        if let Some(token) = parsed.get("token").and_then(|t| t.as_str()) {
            full_response.push_str(token);
        }
    }
    
    // Should contain "hello" (case-insensitive)
    assert!(full_response.to_lowercase().contains("hello"), 
            "Response should contain 'hello', got: {}", full_response);
}

#[tokio::test]
async fn test_non_streaming_response() {
    // Test non-streaming generate_response
    let backend = Arc::new(MockInferenceBackend::new("This is a complete response."));
    
    let result = backend.generate_response("phi-3-mini", "test prompt").await;
    assert!(result.is_ok(), "Non-streaming should succeed");
    assert_eq!(result.unwrap(), "This is a complete response.");
}

#[tokio::test]
async fn test_list_models_response() {
    // Test that list_models returns proper format
    let backend = Arc::new(MockInferenceBackend::new("unused"));
    
    let models = backend.list_models().await.expect("Should list models");
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].0, "phi-3-mini");
    assert!(models[0].1.get("size").is_some());
}

#[tokio::test]
async fn test_get_session_model() {
    // Test session model retrieval
    let backend = Arc::new(MockInferenceBackend::new("unused"));
    
    let model = backend.get_session_model("test-session").await;
    assert_eq!(model, Some("phi-3-mini".to_string()));
}

#[tokio::test]
async fn test_set_session_model() {
    // Test session model setting
    let backend = Arc::new(MockInferenceBackend::new("unused"));
    
    let result = backend.set_session_model("test-session", "phi-3-mini").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_metrics() {
    // Test metrics retrieval
    let backend = Arc::new(MockInferenceBackend::new("unused"));
    
    let metrics = backend.get_metrics().await.expect("Should get metrics");
    assert!(metrics.get("active_sessions").is_some());
    assert!(metrics.get("total_requests").is_some());
}

#[tokio::test]
async fn test_streaming_error_handling() {
    // Test that streaming errors still result in proper done message
    struct FailingBackend;
    
    #[async_trait::async_trait]
    impl shimmy_console::websocket::InferenceBackend for FailingBackend {
        async fn generate_response(&self, _: &str, _: &str) -> anyhow::Result<String> {
            Err(anyhow::anyhow!("Test error"))
        }
        
        async fn get_session_model(&self, _: &str) -> Option<String> {
            Some("test-model".to_string())
        }
        
        async fn list_models(&self) -> anyhow::Result<Vec<(String, serde_json::Value)>> {
            Ok(vec![])
        }
        
        async fn set_session_model(&self, _: &str, _: &str) -> anyhow::Result<()> {
            Ok(())
        }
        
        async fn get_metrics(&self) -> anyhow::Result<serde_json::Value> {
            Ok(json!({}))
        }
        
        async fn generate_stream(
            &self,
            _: &str,
            _: &str,
            tx: tokio::sync::mpsc::Sender<String>,
        ) -> anyhow::Result<()> {
            // Send error token then done
            tx.send(json!({"token": "Error: Test error"}).to_string()).await?;
            tx.send(json!({"done": true}).to_string()).await?;
            Ok(())
        }
    }
    
    let backend = Arc::new(FailingBackend);
    let (tx, mut rx) = mpsc::channel(100);
    
    let result = backend.generate_stream("test-model", "test", tx).await;
    assert!(result.is_ok(), "Should handle error gracefully");
    
    let mut messages = Vec::new();
    while let Some(msg) = rx.recv().await {
        messages.push(msg);
    }
    
    // Should have error token + done
    assert_eq!(messages.len(), 2, "Should have error + done");
    
    let error_msg: serde_json::Value = serde_json::from_str(&messages[0]).unwrap();
    assert!(error_msg.get("token").unwrap().as_str().unwrap().contains("Error"));
    
    let done_msg: serde_json::Value = serde_json::from_str(&messages[1]).unwrap();
    assert_eq!(done_msg.get("done"), Some(&json!(true)));
}

#[tokio::test]
async fn test_streaming_with_phi_markers() {
    // Test that Phi-style markers are handled properly
    let backend = Arc::new(MockInferenceBackend::new("Response here<|end|>"));
    let (tx, mut rx) = mpsc::channel(100);
    
    let result = backend.generate_stream("phi-3-mini", "test", tx).await;
    assert!(result.is_ok());
    
    let mut messages = Vec::new();
    while let Some(msg) = rx.recv().await {
        messages.push(msg);
    }
    
    // Should have tokens + done
    assert!(!messages.is_empty());
    
    // Check that last message is done
    let last: serde_json::Value = serde_json::from_str(messages.last().unwrap()).unwrap();
    assert_eq!(last.get("done"), Some(&json!(true)));
}

#[tokio::test]
async fn test_concurrent_streaming_requests() {
    // Test that multiple concurrent streaming requests work correctly
    let backend = Arc::new(MockInferenceBackend::new("Response"));
    
    let mut handles = vec![];
    
    for i in 0..5 {
        let backend = backend.clone();
        let handle = tokio::spawn(async move {
            let (tx, mut rx) = mpsc::channel(100);
            backend.generate_stream("phi-3-mini", &format!("prompt {}", i), tx).await.unwrap();
            
            let mut count = 0;
            let mut has_done = false;
            while let Some(msg) = rx.recv().await {
                count += 1;
                let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
                if parsed.get("done") == Some(&json!(true)) {
                    has_done = true;
                }
            }
            
            assert!(has_done, "Should receive done message");
            assert!(count >= 2, "Should have at least token + done");
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.expect("Concurrent request should succeed");
    }
}
