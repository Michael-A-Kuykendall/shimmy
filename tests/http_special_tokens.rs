#![cfg(feature = "airframe")]
use async_trait::async_trait;
use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use serde_json::{json, Value};
use shimmy::{
    engine::{GenOptions, InferenceEngine, LoadedModel, ModelSpec},
    model_registry::{ModelEntry, Registry},
    AppState,
};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

type Calls = Arc<Mutex<Vec<(String, GenOptions)>>>;
#[derive(Clone, Copy)]
enum Tokens {
    Model,
    Unknown,
    Failure,
}
struct Engine(Calls, Tokens);
struct Model(Calls, Tokens);
#[async_trait]
impl InferenceEngine for Engine {
    async fn load(&self, _: &ModelSpec) -> anyhow::Result<Box<dyn LoadedModel>> {
        Ok(Box::new(Model(self.0.clone(), self.1)))
    }
}
#[async_trait]
impl LoadedModel for Model {
    async fn special_tokens(&self) -> anyhow::Result<Option<(String, String)>> {
        match self.1 {
            Tokens::Model => Ok(Some(("<|begin_of_text|>".into(), "<|eot_id|>".into()))),
            Tokens::Unknown => Ok(None),
            Tokens::Failure => anyhow::bail!("tokenizer unavailable"),
        }
    }
    async fn generate(
        &self,
        prompt: &str,
        opts: GenOptions,
        mut on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> anyhow::Result<String> {
        self.0.lock().unwrap().push((prompt.into(), opts));
        if let Some(callback) = on_token.as_mut() {
            callback("ok".into());
        }
        Ok("ok".into())
    }
}
fn fixture(tokens: Tokens) -> (axum::Router, Calls, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("model.gguf");
    let mut bytes = b"GGUF".to_vec();
    bytes.extend(3u32.to_le_bytes());
    bytes.extend(0u64.to_le_bytes());
    bytes.extend(5u64.to_le_bytes());
    for (key, value) in [
        ("llama.embedding_length", 8u32),
        ("llama.block_count", 1),
        ("llama.feed_forward_length", 16),
        ("llama.attention.head_count", 2),
    ] {
        bytes.extend((key.len() as u64).to_le_bytes());
        bytes.extend(key.as_bytes());
        bytes.extend(4u32.to_le_bytes());
        bytes.extend(value.to_le_bytes());
    }
    let key = "tokenizer.chat_template";
    let template = "{{ bos_token }}{% for message in messages %}{{ message['role'] }}:{{ message['content'] }}{{ eos_token }}{% endfor %}assistant:";
    bytes.extend((key.len() as u64).to_le_bytes());
    bytes.extend(key.as_bytes());
    bytes.extend(8u32.to_le_bytes());
    bytes.extend((template.len() as u64).to_le_bytes());
    bytes.extend(template.as_bytes());
    std::fs::write(&path, bytes).unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut registry = Registry::default();
    registry.register(ModelEntry {
        name: "test-chat".into(),
        base_path: path,
        lora_path: None,
        template: Some("llama3".into()),
        ctx_len: Some(512),
        n_threads: None,
    });
    let state = Arc::new(AppState::new(
        Box::new(Engine(calls.clone(), tokens)),
        registry,
    ));
    let security =
        shimmy::security::ApiSecurity::new(Some("test-credential-123456789".into()), vec![])
            .unwrap();
    (shimmy::server::router(state, security), calls, dir)
}
async fn post(app: axum::Router, path: &str, body: Value) -> (u16, String) {
    let response = app
        .oneshot(
            Request::builder()
                .uri(path)
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Authorization", "Bearer test-credential-123456789")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status().as_u16();
    let body = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}
fn requests() -> Vec<(&'static str, Value)> {
    vec![
        (
            "/v1/chat/completions",
            json!({"model":"test-chat","messages":[{"role":"user","content":"hello"}],"max_tokens":1}),
        ),
        (
            "/v1/completions",
            json!({"model":"test-chat","prompt":"hello","max_tokens":1}),
        ),
        (
            "/api/generate",
            json!({"model":"test-chat","messages":[{"role":"user","content":"hello"}],"max_tokens":1}),
        ),
        (
            "/api/generate",
            json!({"model":"test-chat","prompt":"hello","max_tokens":1}),
        ),
        (
            "/v1/messages",
            json!({"model":"test-chat","messages":[{"role":"user","content":"hello"}],"max_tokens":1}),
        ),
    ]
}
#[tokio::test]
async fn every_http_renderer_uses_loaded_tokenizer_and_renders_once() {
    for (path, body) in requests() {
        let (app, calls, _dir) = fixture(Tokens::Model);
        let (status, response) = post(app, path, body).await;
        assert_eq!(status, 200, "{path}: {response}");
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        let (prompt, opts) = &calls[0];
        assert_eq!(
            prompt.matches("<|begin_of_text|>").count(),
            1,
            "{path}: {prompt}"
        );
        assert_eq!(prompt.matches("<|eot_id|>").count(), 1, "{path}: {prompt}");
        assert!(!prompt.contains("<s>"), "{path}: {prompt}");
        assert!(prompt.contains("user:hello"), "{path}: {prompt}");
        assert!(
            opts.raw_prompt,
            "{path}: rendered input must bypass backend templating"
        );
    }
}
#[tokio::test]
async fn unknown_backend_preserves_fallback_and_token_failure_prevents_generation() {
    let (app, calls, _dir) = fixture(Tokens::Unknown);
    let (path, body) = requests().remove(0);
    assert_eq!(post(app, path, body).await.0, 200);
    assert!(calls.lock().unwrap()[0].0.starts_with("<s>"));
    for (path, body) in requests() {
        let (app, calls, _dir) = fixture(Tokens::Failure);
        assert!(post(app, path, body).await.0 >= 500, "{path}");
        assert!(calls.lock().unwrap().is_empty());
    }
}
#[tokio::test]
async fn raw_http_prompt_is_not_wrapped() {
    let (app, calls, _dir) = fixture(Tokens::Model);
    let (status, response) = post(
        app,
        "/api/generate",
        json!({"model":"test-chat","prompt":"literal","raw_prompt":true,"max_tokens":1}),
    )
    .await;
    assert_eq!(status, 200, "{response}");
    let calls = calls.lock().unwrap();
    assert_eq!(calls[0].0, "literal");
    assert!(calls[0].1.raw_prompt);
}
#[tokio::test]
async fn websocket_chat_uses_model_tokens_once_and_raw_mode_stays_raw() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};
    for chat in [true, false] {
        let (app, calls, _dir) = fixture(Tokens::Model);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let outcome = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let mut request = format!("ws://{addr}/ws/generate").into_client_request().unwrap();
            request.headers_mut().insert("Authorization", "Bearer test-credential-123456789".parse().unwrap());
            let (mut socket, _) = tokio_tungstenite::connect_async(request).await.unwrap();
            let body = if chat { json!({"model":"test-chat","messages":[{"role":"user","content":"hello"}],"max_tokens":1}) }
                else { json!({"model":"test-chat","prompt":"literal","raw_prompt":true,"max_tokens":1}) };
            socket.send(Message::Text(body.to_string())).await.unwrap();
            while let Some(message) = socket.next().await {
                if message.unwrap().into_text().unwrap().contains("\"done\":true") { return; }
            }
            panic!("WebSocket ended before completion");
        }).await;
        server.abort();
        let _ = server.await;
        outcome.unwrap();
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert!(calls[0].1.raw_prompt);
        if chat {
            assert_eq!(calls[0].0.matches("<|begin_of_text|>").count(), 1);
            assert_eq!(calls[0].0.matches("<|eot_id|>").count(), 1);
            assert!(!calls[0].0.contains("<s>"));
        } else {
            assert_eq!(calls[0].0, "literal");
        }
    }
}
