use axum::{routing::get, Router};

pub fn create_websocket_router() -> Router {
    Router::new()
        .route(
            "/ws/console",
            get(|ws: axum::extract::ws::WebSocketUpgrade| async move {
                ws.on_upgrade(|socket| async move {
                    // Handle WebSocket connection
                    let _ = socket;
                })
            }),
        )
        .route("/ws/health", get(|| async { "WebSocket OK" }))
}
