//! WebSocket inference adapter

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// WebSocket-based inference adapter for streaming responses
pub struct WsInferenceAdapter {
    url: String,
    connection: Arc<Mutex<Option<WsConnection>>>,
}

struct WsConnection {
    // WebSocket connection state
    _connected: bool,
}

/// Request to send over WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsRequest {
    pub model: String,
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: bool,
}

/// Response received over WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsResponse {
    pub text: String,
    pub done: bool,
    pub tokens_generated: Option<u32>,
}

impl WsInferenceAdapter {
    /// Create a new WebSocket adapter
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            connection: Arc::new(Mutex::new(None)),
        }
    }

    /// Connect to the WebSocket server
    pub async fn connect(&self) -> anyhow::Result<()> {
        let mut conn = self.connection.lock().await;
        *conn = Some(WsConnection { _connected: true });
        tracing::info!("Connected to WebSocket at {}", self.url);
        Ok(())
    }

    /// Disconnect from the WebSocket server
    pub async fn disconnect(&self) -> anyhow::Result<()> {
        let mut conn = self.connection.lock().await;
        *conn = None;
        tracing::info!("Disconnected from WebSocket");
        Ok(())
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        self.connection.lock().await.is_some()
    }

    /// Send a request and get streaming response
    pub async fn infer_stream(
        &self,
        request: WsRequest,
    ) -> anyhow::Result<impl futures_util::Stream<Item = WsResponse>> {
        if !self.is_connected().await {
            self.connect().await?;
        }

        // Placeholder: return a simple stream with the prompt echoed
        let response = WsResponse {
            text: format!("Echo: {}", request.prompt),
            done: true,
            tokens_generated: Some(1),
        };

        Ok(futures_util::stream::iter(vec![response]))
    }

    /// Send a request and get complete response
    pub async fn infer(&self, request: WsRequest) -> anyhow::Result<WsResponse> {
        use futures_util::StreamExt;

        let mut stream = self.infer_stream(request).await?;
        let mut full_text = String::new();
        let mut total_tokens = 0u32;

        while let Some(response) = stream.next().await {
            full_text.push_str(&response.text);
            if let Some(tokens) = response.tokens_generated {
                total_tokens += tokens;
            }
            if response.done {
                break;
            }
        }

        Ok(WsResponse {
            text: full_text,
            done: true,
            tokens_generated: Some(total_tokens),
        })
    }

    /// Get the URL this adapter is connected to
    pub fn url(&self) -> &str {
        &self.url
    }
}
