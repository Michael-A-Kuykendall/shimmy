use crate::websocket::InferenceBackend;
use async_trait::async_trait;
use serde_json;

/// HTTP adapter for InferenceBackend - communicates with shimmy via localhost HTTP
pub struct HttpInferenceAdapter {
    base_url: String,
    client: reqwest::Client,
}

impl HttpInferenceAdapter {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl InferenceBackend for HttpInferenceAdapter {
    async fn generate_response(&self, model_name: &str, prompt: &str) -> anyhow::Result<String> {
        let url = format!("{}/v1/generate", self.base_url);
        let body = serde_json::json!({
            "model": model_name,
            "prompt": prompt,
            "stream": false
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            Ok(data["response"]
                .as_str()
                .unwrap_or("No response")
                .to_string())
        } else {
            Err(anyhow::anyhow!("HTTP error: {}", response.status()))
        }
    }

    async fn get_session_model(&self, session_id: &str) -> Option<String> {
        let url = format!("{}/v1/sessions/{}/model", self.base_url, session_id);
        match self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                resp.json::<serde_json::Value>()
                    .await
                    .ok()
                    .and_then(|v| v["model"].as_str().map(|s| s.to_string()))
            }
            _ => None,
        }
    }

    async fn list_models(&self) -> anyhow::Result<Vec<(String, serde_json::Value)>> {
        let url = format!("{}/v1/models", self.base_url);
        let response = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            let models = data["models"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|m| {
                    let name = m["name"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string();
                    (name, m.clone())
                })
                .collect();
            Ok(models)
        } else {
            Err(anyhow::anyhow!("Failed to list models: {}", response.status()))
        }
    }

    async fn set_session_model(&self, session_id: &str, model_name: &str) -> anyhow::Result<()> {
        let url = format!("{}/v1/sessions/{}/model", self.base_url, session_id);
        let body = serde_json::json!({
            "model": model_name
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to set session model: {}", response.status()))
        }
    }

    async fn get_metrics(&self) -> anyhow::Result<serde_json::Value> {
        let url = format!("{}/v1/metrics", self.base_url);
        let response = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;

        if response.status().is_success() {
            let metrics: serde_json::Value = response.json().await?;
            Ok(metrics)
        } else {
            Err(anyhow::anyhow!("Failed to get metrics: {}", response.status()))
        }
    }

    async fn generate_stream(&self, model_name: &str, prompt: &str, tx: tokio::sync::mpsc::Sender<String>) -> anyhow::Result<()> {
        use futures_util::StreamExt;
        
        // The prompt may contain "User request: " prefix from the WebSocket handler
        // Extract just the user message for cleaner prompts
        let user_message = if prompt.contains("User request: ") {
            prompt.split("User request: ").last().unwrap_or(prompt).trim()
        } else {
            prompt.trim()
        };
        
        let url = format!("{}/v1/chat/completions", self.base_url);
        let body = serde_json::json!({
            "model": model_name,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful AI assistant. Be concise and helpful."
                },
                {
                    "role": "user",
                    "content": user_message
                }
            ],
            "stream": true,
            "temperature": 0.7,
            "max_tokens": 2000
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    let text = String::from_utf8_lossy(&chunk);
                    buffer.push_str(&text);

                    // Process complete SSE lines
                    while let Some(line_end) = buffer.find('\n') {
                        let line = buffer[..line_end].trim().to_string();
                        buffer = buffer[line_end + 1..].to_string();

                        if line.starts_with("data: ") {
                            let json_str = &line[6..];

                            if json_str == "[DONE]" {
                                return Ok(());
                            }

                            // Parse SSE JSON
                            if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_str) {
                                // Extract token from OpenAI-compatible format
                                if let Some(token) = data
                                    .get("choices")
                                    .and_then(|c| c.get(0))
                                    .and_then(|choice| choice.get("delta"))
                                    .and_then(|delta| delta.get("content"))
                                    .and_then(|content| content.as_str())
                                {
                                    // Send canonical token format
                                    let msg = serde_json::json!({"token": token}).to_string();
                                    let _ = tx.send(msg).await;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Stream error: {}", e));
                }
            }
        }

        // Send canonical completion when stream ends
        let _ = tx.send(serde_json::json!({"done": true}).to_string()).await;
        Ok(())
    }
}
