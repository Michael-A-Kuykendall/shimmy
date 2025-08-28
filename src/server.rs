use axum::{routing::{post, get}, Router};
use std::{net::SocketAddr, sync::Arc};
use crate::{api, util::diag::diag_handler, openai_compat, AppState};

pub async fn run(addr: SocketAddr, state: Arc<AppState>) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let app = Router::new()
        .route("/health", get(|| async { axum::Json(serde_json::json!({"status":"ok"})) }))
        .route("/diag", get(diag_handler))
        .route("/api/generate", post(api::generate))
        .route("/api/models", get(api::list_models))
        .route("/api/models/discover", post(api::discover_models))
        .route("/api/models/:name/load", post(api::load_model))
        .route("/api/models/:name/unload", post(api::unload_model))
        .route("/api/models/:name/status", get(api::model_status))
        .route("/api/tools", get(api::list_tools))
        .route("/api/tools/:name/execute", post(api::execute_tool))
        .route("/api/workflows/execute", post(api::execute_workflow))
        .route("/ws/generate", get(api::ws_generate))
        .route("/v1/chat/completions", post(openai_compat::chat_completions))
        .with_state(state);
    axum::serve(listener, app).await?;
    Ok(())
}
