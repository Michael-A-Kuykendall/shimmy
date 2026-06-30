use axum::{extract::State, routing::get, Json, Router};
use shimmy::openai_compat::{self, ChatCompletionRequest, MessageContent, OAIMessage};
use shimmy::{engine::adapter::InferenceEngineAdapter, model_registry::Registry, AppState};
use std::sync::Arc;
use tokio::net::TcpListener;

fn test_state() -> Arc<AppState> {
    let mut registry = Registry::default();
    registry.register(shimmy::model_registry::ModelEntry {
        name: "phi3-mini-4k-instruct".into(),
        base_path: "./test.gguf".into(),
        lora_path: None,
        template: Some("chatml".into()),
        ctx_len: Some(4096),
        n_threads: None,
    });
    let engine = Box::new(InferenceEngineAdapter::new());
    Arc::new(AppState::new(engine, registry))
}

fn build_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async { axum::Json(serde_json::json!({"status":"ok"})) }),
        )
        .route(
            "/v1/models",
            get(|state: State<Arc<AppState>>| async move {
                openai_compat::models(State(state.0.clone())).await
            }),
        )
        .route(
            "/v1/chat/completions",
            axum::routing::post(
                |state: State<Arc<AppState>>, body: Json<ChatCompletionRequest>| async move {
                    openai_compat::chat_completions(State(state.0.clone()), body).await
                },
            ),
        )
        .route(
            "/api/tags",
            get(|state: State<Arc<AppState>>| async move {
                openai_compat::api_tags(State(state.0.clone())).await
            }),
        )
        .with_state(state)
}

async fn spawn_server() -> (String, tokio::task::JoinHandle<()>) {
    let state = test_state();
    let app = build_app(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    (url, handle)
}

#[tokio::test]
async fn test_health_endpoint() {
    let (url, handle) = spawn_server().await;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let resp = client.get(format!("{}/health", url)).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    handle.abort();
}

#[tokio::test]
async fn test_models_endpoint() {
    let (url, handle) = spawn_server().await;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let resp = client
        .get(format!("{}/v1/models", url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["object"], "list");
    let models = body["data"].as_array().unwrap();
    assert!(!models.is_empty());
    let ids: Vec<&str> = models.iter().map(|m| m["id"].as_str().unwrap()).collect();
    assert!(ids.contains(&"phi3-mini-4k-instruct"));
    handle.abort();
}

#[tokio::test]
async fn test_chat_completions_model_not_found() {
    let (url, handle) = spawn_server().await;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let body = serde_json::json!({
        "model": "nonexistent-model",
        "messages": [{"role": "user", "content": "Hello"}],
        "stream": false
    });
    let resp = client
        .post(format!("{}/v1/chat/completions", url))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    let err: serde_json::Value = resp.json().await.unwrap();
    assert!(err["error"]["message"]
        .as_str()
        .unwrap()
        .contains("not found"));
    assert_eq!(err["error"]["type"], "invalid_request_error");
    assert_eq!(err["error"]["code"], "model_not_found");
    handle.abort();
}

#[tokio::test]
async fn test_api_tags_endpoint() {
    let (url, handle) = spawn_server().await;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let resp = client
        .get(format!("{}/api/tags", url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.get("models").is_some());
    let models = body["models"].as_array().unwrap();
    assert!(!models.is_empty());
    assert_eq!(models[0]["name"], "phi3-mini-4k-instruct");
    handle.abort();
}

#[tokio::test]
async fn test_concurrent_health_requests() {
    let (url, handle) = spawn_server().await;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    let mut handles = vec![];
    for i in 0..5 {
        let c = client.clone();
        let u = url.clone();
        handles.push(tokio::spawn(async move {
            let resp = c.get(format!("{}/health", u)).send().await.unwrap();
            (i, resp.status())
        }));
    }
    for h in handles {
        let (i, status) = h.await.unwrap();
        assert_eq!(status, 200, "request {} failed", i);
    }
    handle.abort();
}
