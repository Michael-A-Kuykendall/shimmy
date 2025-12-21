use crate::websocket::InferenceBackend;
use async_trait::async_trait;
use serde_json;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock adapter for testing - returns deterministic responses without HTTP calls
#[derive(Clone)]
pub struct MockInferenceAdapter {
    /// Optional override for list_models response
    models_override: Arc<Mutex<Option<Vec<(String, serde_json::Value)>>>>,
    /// Optional override for get_metrics response
    metrics_override: Arc<Mutex<Option<serde_json::Value>>>,
    /// Track calls for verification in tests
    call_history: Arc<Mutex<Vec<String>>>,
    /// Session model mappings for testing
    session_models: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

impl MockInferenceAdapter {
    pub fn new() -> Self {
        Self {
            models_override: Arc::new(Mutex::new(None)),
            metrics_override: Arc::new(Mutex::new(None)),
            call_history: Arc::new(Mutex::new(Vec::new())),
            session_models: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Set up mock models response for testing
    pub async fn set_models(&self, models: Vec<(String, serde_json::Value)>) {
        *self.models_override.lock().await = Some(models);
    }

    /// Set up mock metrics response for testing
    pub async fn set_metrics(&self, metrics: serde_json::Value) {
        *self.metrics_override.lock().await = Some(metrics);
    }

    /// Get call history for assertion in tests
    pub async fn get_call_history(&self) -> Vec<String> {
        self.call_history.lock().await.clone()
    }

    /// Clear call history
    pub async fn clear_history(&self) {
        self.call_history.lock().await.clear();
    }

    /// Helper to record a call
    async fn record_call(&self, call: &str) {
        self.call_history.lock().await.push(call.to_string());
    }
}

impl Default for MockInferenceAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InferenceBackend for MockInferenceAdapter {
    async fn generate_response(&self, model_name: &str, prompt: &str) -> anyhow::Result<String> {
        self.record_call(&format!("generate_response({}, {})", model_name, prompt.len())).await;
        Ok("Mock response from model".to_string())
    }

    async fn get_session_model(&self, session_id: &str) -> Option<String> {
        self.record_call(&format!("get_session_model({})", session_id)).await;
        self.session_models.lock().await.get(session_id).cloned()
    }

    async fn list_models(&self) -> anyhow::Result<Vec<(String, serde_json::Value)>> {
        self.record_call("list_models").await;
        
        // Return override if set, otherwise return default test models
        if let Some(models) = self.models_override.lock().await.clone() {
            Ok(models)
        } else {
            Ok(vec![
                (
                    "phi3-mini".to_string(),
                    serde_json::json!({
                        "name": "phi3-mini",
                        "size": "3.8B",
                        "family": "phi3",
                        "quantization": "q4"
                    }),
                ),
                (
                    "phi3-medium".to_string(),
                    serde_json::json!({
                        "name": "phi3-medium",
                        "size": "14B",
                        "family": "phi3",
                        "quantization": "q4"
                    }),
                ),
            ])
        }
    }

    async fn set_session_model(&self, session_id: &str, model_name: &str) -> anyhow::Result<()> {
        self.record_call(&format!("set_session_model({}, {})", session_id, model_name)).await;
        self.session_models.lock().await.insert(session_id.to_string(), model_name.to_string());
        Ok(())
    }

    async fn get_metrics(&self) -> anyhow::Result<serde_json::Value> {
        self.record_call("get_metrics").await;
        
        // Return override if set, otherwise return default metrics
        if let Some(metrics) = self.metrics_override.lock().await.clone() {
            Ok(metrics)
        } else {
            Ok(serde_json::json!({
                "system": {
                    "cpu_usage": 25.5,
                    "memory_usage": 512.0,
                    "disk_usage": 1024.0,
                    "uptime_seconds": 3600
                },
                "inference": {
                    "active_sessions": 1,
                    "total_tokens": 5000,
                    "tokens_per_second": 42.0
                }
            }))
        }
    }

    async fn generate_stream(&self, model_name: &str, prompt: &str, tx: tokio::sync::mpsc::Sender<String>) -> anyhow::Result<()> {
        self.record_call(&format!("generate_stream({}, {})", model_name, prompt.len())).await;
        
        // Send mock tokens for testing
        let mock_response = "This is a mock streaming response from the test adapter. ";
        for word in mock_response.split_whitespace() {
            let msg = serde_json::json!({"type": "chat_token", "token": format!("{} ", word)}).to_string();
            let _ = tx.send(msg).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_list_models() {
        let adapter = MockInferenceAdapter::new();
        let models = adapter.list_models().await.unwrap();
        
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].0, "phi3-mini");
        assert_eq!(models[1].0, "phi3-medium");
        
        let history = adapter.get_call_history().await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0], "list_models");
    }

    #[tokio::test]
    async fn test_mock_list_models_override() {
        let adapter = MockInferenceAdapter::new();
        let custom_models = vec![
            (
                "gpt2".to_string(),
                serde_json::json!({ "name": "gpt2", "size": "1.5B" }),
            ),
        ];
        
        adapter.set_models(custom_models.clone()).await;
        let models = adapter.list_models().await.unwrap();
        
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].0, "gpt2");
    }

    #[tokio::test]
    async fn test_mock_get_metrics() {
        let adapter = MockInferenceAdapter::new();
        let metrics = adapter.get_metrics().await.unwrap();
        
        assert!(metrics["system"]["cpu_usage"].is_number());
        assert!(metrics["inference"]["tokens_per_second"].is_number());
    }

    #[tokio::test]
    async fn test_mock_metrics_override() {
        let adapter = MockInferenceAdapter::new();
        let custom_metrics = serde_json::json!({
            "custom": "value",
            "nested": { "field": 42 }
        });
        
        adapter.set_metrics(custom_metrics.clone()).await;
        let metrics = adapter.get_metrics().await.unwrap();
        
        assert_eq!(metrics["custom"], "value");
        assert_eq!(metrics["nested"]["field"], 42);
    }

    #[tokio::test]
    async fn test_mock_session_model_tracking() {
        let adapter = MockInferenceAdapter::new();
        
        // Set a session model
        adapter.set_session_model("session1", "phi3-mini").await.unwrap();
        
        // Retrieve it
        let model = adapter.get_session_model("session1").await;
        assert_eq!(model, Some("phi3-mini".to_string()));
        
        // Non-existent session returns None
        let missing = adapter.get_session_model("nonexistent").await;
        assert_eq!(missing, None);
    }

    #[tokio::test]
    async fn test_mock_call_history() {
        let adapter = MockInferenceAdapter::new();
        
        adapter.generate_response("test-model", "test prompt").await.ok();
        adapter.list_models().await.ok();
        adapter.get_metrics().await.ok();
        
        let history = adapter.get_call_history().await;
        assert_eq!(history.len(), 3);
        assert!(history[0].contains("generate_response"));
        assert_eq!(history[1], "list_models");
        assert_eq!(history[2], "get_metrics");
    }

    #[tokio::test]
    async fn test_mock_clear_history() {
        let adapter = MockInferenceAdapter::new();
        
        adapter.list_models().await.ok();
        assert_eq!(adapter.get_call_history().await.len(), 1);
        
        adapter.clear_history().await;
        assert_eq!(adapter.get_call_history().await.len(), 0);
    }
}
