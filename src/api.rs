use axum::{extract::State, response::{IntoResponse, Sse, sse::Event}, Json};
use axum::extract::ws::{WebSocketUpgrade, WebSocket, Message as WsMessage};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{engine::{GenOptions}, templates::TemplateFamily, AppState};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: Option<String>,            // raw mode
    pub messages: Option<Vec<ChatMessage>>,    // chat mode
    pub system: Option<String>,
    #[serde(default)] pub temperature: Option<f32>,
    #[serde(default)] pub top_p: Option<f32>,
    #[serde(default)] pub top_k: Option<i32>,
    #[serde(default)] pub max_tokens: Option<usize>,
    #[serde(default)] pub stream: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatMessage { pub role: String, pub content: String }

#[derive(Debug, Serialize)]
pub struct GenerateResponse { pub response: String }

pub async fn generate(State(state): State<Arc<AppState>>, Json(req): Json<GenerateRequest>) -> impl IntoResponse {
    let Some(spec) = state.registry.to_spec(&req.model) else { return axum::http::StatusCode::NOT_FOUND.into_response(); };
    let engine = &state.engine;
    let Ok(loaded) = engine.load(&spec).await else { return axum::http::StatusCode::BAD_GATEWAY.into_response(); };

    // Construct prompt
    let prompt = if let Some(ms) = &req.messages {
        let fam = match spec.template.as_deref() {
            Some("chatml") => TemplateFamily::ChatML,
            Some("llama3") | Some("llama-3") => TemplateFamily::Llama3,
            _ => TemplateFamily::OpenChat,
        };
        let pairs = ms.iter().map(|m| (m.role.clone(), m.content.clone())).collect::<Vec<_>>();
        fam.render(req.system.as_deref(), &pairs, None)
    } else {
        req.prompt.unwrap_or_default()
    };

    let mut opts = GenOptions::default();
    if let Some(t) = req.temperature { opts.temperature = t; }
    if let Some(p) = req.top_p { opts.top_p = p; }
    if let Some(k) = req.top_k { opts.top_k = k; }
    if let Some(m) = req.max_tokens { opts.max_tokens = m; }
    if let Some(s) = req.stream { opts.stream = s; }

    if opts.stream { // SSE streaming
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let mut opts_clone = opts.clone(); opts_clone.stream = false; // internal generation collects tokens while we push per token
        let prompt_clone = prompt.clone();
        tokio::spawn(async move {
            let tx_tokens = tx.clone();
            let _ = loaded.generate(&prompt_clone, opts_clone, Some(Box::new(move |tok| { let _ = tx_tokens.send(tok); }))).await;
            let _ = tx.send("[DONE]".into());
        });
    let stream = UnboundedReceiverStream::new(rx).map(|s| Ok::<Event, std::convert::Infallible>(Event::default().data(s)));
        Sse::new(stream).into_response()
    } else {
        match loaded.generate(&prompt, opts, None).await {
            Ok(full) => Json(GenerateResponse { response: full }).into_response(),
            Err(_) => axum::http::StatusCode::BAD_GATEWAY.into_response(),
        }
    }
}

// WebSocket endpoint: client connects to /ws/generate, sends a single JSON GenerateRequest text frame.
// Server streams each token as a Text frame and finally sends a JSON {"done":true} frame.
pub async fn ws_generate(State(state): State<Arc<AppState>>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_generate(state, socket))
}

async fn handle_ws_generate(state: Arc<AppState>, mut socket: WebSocket) {
    // Expect first message with request JSON
    let Some(Ok(first)) = socket.recv().await else { return; };
    let req_json = match first {
    WsMessage::Text(t) => t,
    WsMessage::Binary(b) => String::from_utf8_lossy(&b).to_string(),
        _ => return,
    };
    let req: GenerateRequest = match serde_json::from_str(&req_json) {
        Ok(r) => r,
        Err(e) => {
            let _ = socket.send(WsMessage::Text(format!("{{\"error\":\"bad request: {e}\"}}"))).await;
            return;
        }
    };
    let Some(spec) = state.registry.to_spec(&req.model) else {
    let _ = socket.send(WsMessage::Text("{\"error\":\"model not found\"}".into())).await;
        return;
    };
    let Ok(loaded) = state.engine.load(&spec).await else {
    let _ = socket.send(WsMessage::Text("{\"error\":\"load failed\"}".into())).await;
        return;
    };

    // Build prompt (reuse logic)
    let prompt = if let Some(ms) = &req.messages {
        let fam = match spec.template.as_deref() {
            Some("chatml") => TemplateFamily::ChatML,
            Some("llama3") | Some("llama-3") => TemplateFamily::Llama3,
            _ => TemplateFamily::OpenChat,
        };
        let pairs = ms.iter().map(|m| (m.role.clone(), m.content.clone())).collect::<Vec<_>>();
        fam.render(req.system.as_deref(), &pairs, None)
    } else { req.prompt.clone().unwrap_or_default() };

    let mut opts = GenOptions::default();
    if let Some(t) = req.temperature { opts.temperature = t; }
    if let Some(p) = req.top_p { opts.top_p = p; }
    if let Some(k) = req.top_k { opts.top_k = k; }
    if let Some(m) = req.max_tokens { opts.max_tokens = m; }
    // Force internal non-stream; we push per-token ourselves
    let mut internal = opts.clone(); internal.stream = false;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    tokio::spawn({
        let prompt = prompt.clone();
        let tx_done = tx.clone();
        async move {
            let tx_tokens = tx.clone();
            let _ = loaded.generate(&prompt, internal, Some(Box::new(move |tok| { let _ = tx_tokens.send(tok); }))).await;
            let _ = tx_done.send("[DONE]".into());
        }
    });
    while let Some(piece) = rx.recv().await {
        if piece == "[DONE]" { break; }
        if socket.send(WsMessage::Text(piece)).await.is_err() { break; }
    }
    let _ = socket.send(WsMessage::Text("{\"done\":true}".into())).await;
}

#[derive(Debug, Serialize)]
pub struct ModelListResponse {
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub name: String,
    pub size_bytes: Option<u64>,
    pub model_type: Option<String>,
    pub parameter_count: Option<String>,
    pub source: String, // "registered" or "discovered"
}

pub async fn list_models(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut models = Vec::new();
    
    // Add manually registered models
    for entry in state.registry.list() {
        models.push(ModelInfo {
            name: entry.name.clone(),
            size_bytes: None, // Could read file size if needed
            model_type: None,
            parameter_count: None,
            source: "registered".to_string(),
        });
    }
    
    // Add discovered models
    for (name, discovered) in &state.registry.discovered_models {
        models.push(ModelInfo {
            name: name.clone(),
            size_bytes: Some(discovered.size_bytes),
            model_type: Some(discovered.model_type.clone()),
            parameter_count: discovered.parameter_count.clone(),
            source: "discovered".to_string(),
        });
    }
    
    Json(ModelListResponse { models })
}

pub async fn discover_models(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    // Note: This is a simplified version - in practice we'd need to make the registry mutable
    // through Arc<Mutex<Registry>> or similar, but for now this shows the API shape
    
    use crate::auto_discovery::ModelAutoDiscovery;
    let discovery = ModelAutoDiscovery::new();
    
    match discovery.discover_models() {
        Ok(models) => {
            let model_infos: Vec<ModelInfo> = models.iter().map(|m| ModelInfo {
                name: m.name.clone(),
                size_bytes: Some(m.size_bytes),
                model_type: Some(m.model_type.clone()),
                parameter_count: m.parameter_count.clone(),
                source: "discovered".to_string(),
            }).collect();
            
            Json(serde_json::json!({
                "discovered": model_infos.len(),
                "models": model_infos
            })).into_response()
        },
        Err(_e) => {
            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

use axum::extract::Path;
use crate::tools::{ToolRegistry, ToolCall};

pub async fn load_model(State(_state): State<Arc<AppState>>, Path(name): Path<String>) -> impl IntoResponse {
    // TODO: Integrate with ModelManager and Registry
    // For now, return a placeholder response
    Json(serde_json::json!({
        "message": format!("Model {} load requested", name),
        "status": "pending"
    }))
}

pub async fn unload_model(State(_state): State<Arc<AppState>>, Path(name): Path<String>) -> impl IntoResponse {
    // TODO: Integrate with ModelManager
    Json(serde_json::json!({
        "message": format!("Model {} unload requested", name),
        "status": "pending"
    }))
}

pub async fn model_status(State(_state): State<Arc<AppState>>, Path(name): Path<String>) -> impl IntoResponse {
    // TODO: Check actual model status
    Json(serde_json::json!({
        "model": name,
        "status": "unknown",
        "loaded": false
    }))
}

pub async fn list_tools(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let registry = ToolRegistry::default();
    let tools = registry.list_tools();
    Json(serde_json::json!({
        "tools": tools
    }))
}

pub async fn execute_tool(State(_state): State<Arc<AppState>>, Path(name): Path<String>, Json(arguments): Json<serde_json::Value>) -> impl IntoResponse {
    let registry = ToolRegistry::default();
    let tool_call = ToolCall {
        name,
        arguments,
    };
    
    match registry.execute_tool(&tool_call) {
        Ok(result) => Json(serde_json::json!(result)).into_response(),
        Err(_e) => {
            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn execute_workflow(State(_state): State<Arc<AppState>>, Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    // Workflow execution - placeholder for now
    Json(serde_json::json!({
        "message": "Workflow execution not yet implemented",
        "status": "pending"
    }))
}
