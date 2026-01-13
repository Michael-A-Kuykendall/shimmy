use crate::api::{GenOptions, GenResponse};
use crate::engine::Engine;
use crate::error::ShimmyError;
use crate::model_manager::ModelManager;
use crate::templates::TemplateFamily;
use crate::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct ConsoleMessage {
    content: String,
    model_name: Option<String>,
}

#[derive(Serialize)]
struct TokenResponse {
    token: String,
}

#[derive(Serialize)]
struct DoneResponse {
    done: bool,
}

pub async fn console_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_console_socket(socket, state))
}

async fn handle_console_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(console_msg) = serde_json::from_str::<ConsoleMessage>(&text) {
                let model_name = console_msg.model_name.unwrap_or_else(|| "default".to_string());

                // Get model and template
                let model = match state.model_manager.get_model(&model_name).await {
                    Ok(m) => m,
                    Err(e) => {
                        let _ = sender.send(Message::Text(format!("Error: {}", e))).await;
                        continue;
                    }
                };

                let template = TemplateFamily::detect(&model.name).unwrap_or_default();

                // Prepare generation options
                let mut opts = GenOptions {
                    prompt: template.apply(&console_msg.content).unwrap_or(console_msg.content),
                    max_tokens: None, // No cap for console
                    temperature: 0.7,
                    top_p: 0.9,
                    top_k: 40,
                    repeat_penalty: 1.2, // Stronger for streaming
                    stop_tokens: template.stop_tokens(),
                    stream: true,
                };

                // Generate with streaming
                let engine = state.engine.clone();
                let mut stream = match engine.generate_stream(model, opts).await {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = sender.send(Message::Text(format!("Error: {}", e))).await;
                        continue;
                    }
                };

                while let Some(token_result) = stream.next().await {
                    match token_result {
                        Ok(token) => {
                            let response = TokenResponse { token };
                            if let Ok(json) = serde_json::to_string(&response) {
                                let _ = sender.send(Message::Text(json)).await;
                            }
                        }
                        Err(e) => {
                            let _ = sender.send(Message::Text(format!("Error: {}", e))).await;
                            break;
                        }
                    }
                }

                // Send done
                let done = DoneResponse { done: true };
                if let Ok(json) = serde_json::to_string(&done) {
                    let _ = sender.send(Message::Text(json)).await;
                }
            }
        }
    }
}
