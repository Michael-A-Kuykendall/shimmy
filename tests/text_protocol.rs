//! Protocol regressions run without GPU hardware or downloaded weights.
use async_trait::async_trait;
use axum::{body::to_bytes, extract::State, response::IntoResponse, Json};
use serde_json::{json, Value};
use shimmy::{
    engine::{GenOptions, InferenceEngine, LoadedModel, ModelSpec},
    model_registry::{ModelEntry, Registry},
    AppState,
};
use std::sync::{Arc, Mutex};

type Calls = Arc<Mutex<Vec<(String, GenOptions)>>>;
struct CaptureEngine(Calls);
struct CaptureModel(Calls);

#[async_trait]
impl InferenceEngine for CaptureEngine {
    async fn load(&self, _: &ModelSpec) -> anyhow::Result<Box<dyn LoadedModel>> {
        Ok(Box::new(CaptureModel(self.0.clone())))
    }
}

#[async_trait]
impl LoadedModel for CaptureModel {
    async fn generate(
        &self,
        prompt: &str,
        opts: GenOptions,
        mut on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> anyhow::Result<String> {
        self.0.lock().unwrap().push((prompt.to_owned(), opts));
        if let Some(callback) = on_token.as_mut() {
            callback("test response".to_owned());
        }
        Ok("test response".to_owned())
    }

    async fn generate_with_metadata(
        &self,
        prompt: &str,
        opts: GenOptions,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> anyhow::Result<shimmy::engine::GenerationOutput> {
        anyhow::ensure!(!prompt.contains("FAIL_GENERATION"), "fixture failure");
        let completion_tokens = if opts.max_tokens == 1 { 1 } else { 3 };
        let reason = if opts.max_tokens == 1 {
            "length"
        } else {
            "stop"
        };
        Ok(shimmy::engine::GenerationOutput {
            text: self.generate(prompt, opts, on_token).await?,
            usage: Some(shimmy::engine::TokenUsage {
                prompt_tokens: 17,
                completion_tokens,
                total_tokens: 17 + completion_tokens,
            }),
            finish_reason: Some(reason.into()),
        })
    }
}

fn state() -> (Arc<AppState>, Calls) {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut registry = Registry::default();
    registry.register(ModelEntry {
        name: "test-chat".to_owned(),
        base_path: "unused-test.gguf".into(),
        lora_path: None,
        template: Some("chatml".to_owned()),
        ctx_len: Some(512),
        n_threads: None,
    });
    (
        Arc::new(AppState::new(
            Box::new(CaptureEngine(calls.clone())),
            registry,
        )),
        calls,
    )
}

fn secured_router() -> axum::Router {
    let (state, _) = state();
    shimmy::server::router(
        state,
        shimmy::security::ApiSecurity::new(
            Some("test-credential-123456789".into()),
            vec!["https://client.example".into()],
        )
        .unwrap(),
    )
}

async fn request(
    app: axum::Router,
    path: &str,
    key: Option<&str>,
    body: Value,
) -> axum::response::Response {
    use tower::ServiceExt;
    let mut builder = axum::http::Request::builder()
        .uri(path)
        .method("POST")
        .header("Content-Type", "application/json");
    if let Some(key) = key {
        builder = builder.header("Authorization", format!("Bearer {key}"));
    }
    app.oneshot(
        builder
            .body(axum::body::Body::from(body.to_string()))
            .unwrap(),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn every_inference_and_management_route_requires_credentials() {
    for path in [
        "/api/generate",
        "/v1/chat/completions",
        "/v1/completions",
        "/v1/messages",
        "/api/models/discover",
        "/api/models/test/load",
        "/api/models/test/unload",
        "/api/tools/test/execute",
        "/api/workflows/execute",
    ] {
        for key in [None, Some("incorrect-key")] {
            let response = request(secured_router(), path, key, json!({})).await;
            assert_eq!(response.status(), 401, "{path}");
            assert_eq!(response.headers()["www-authenticate"], "Bearer");
        }
    }
    use tower::ServiceExt;
    for path in [
        "/api/models",
        "/v1/models",
        "/api/tags",
        "/metrics",
        "/diag",
        "/api/tools",
        "/ws/generate",
    ] {
        let response = secured_router()
            .oneshot(
                axum::http::Request::builder()
                    .uri(path)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 401, "{path}");
    }
    let response = secured_router()
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn exact_usage_and_finish_reason_reach_json_protocols() {
    for max_tokens in [1, 8] {
        for path in ["/v1/chat/completions", "/v1/completions", "/api/generate"] {
            let response = request(
                secured_router(),
                path,
                Some("test-credential-123456789"),
                json!({
                    "model": "test-chat", "messages": [{"role":"user","content":"hello"}],
                    "prompt": "hello", "stream": false, "max_tokens": max_tokens
                }),
            )
            .await;
            assert_eq!(response.status(), 200, "{path}");
            let body: Value =
                serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap())
                    .unwrap();
            let count = if max_tokens == 1 { 1 } else { 3 };
            assert_eq!(
                body["usage"],
                json!({"prompt_tokens":17,"completion_tokens":count,"total_tokens":17+count})
            );
            let reason = if path == "/api/generate" {
                &body["finish_reason"]
            } else {
                &body["choices"][0]["finish_reason"]
            };
            assert_eq!(reason, if max_tokens == 1 { "length" } else { "stop" });
        }
    }
}

#[tokio::test]
async fn authenticated_sse_reports_length_and_optional_usage() {
    for include_usage in [false, true] {
        let response = request(
            secured_router(),
            "/v1/chat/completions",
            Some("test-credential-123456789"),
            json!({
                "model":"test-chat", "messages":[{"role":"user","content":"hello"}],
                "stream":true, "max_tokens":1, "stream_options":{"include_usage":include_usage}
            }),
        )
        .await;
        assert_eq!(response.status(), 200);
        let text = String::from_utf8(
            to_bytes(response.into_body(), 65536)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        let chunks: Vec<Value> = text
            .lines()
            .filter_map(|line| line.strip_prefix("data: "))
            .filter(|data| *data != "[DONE]")
            .map(|data| serde_json::from_str(data).unwrap())
            .collect();
        assert!(chunks
            .iter()
            .any(|c| c["choices"][0]["finish_reason"] == "length"));
        let usage: Vec<_> = chunks.iter().filter(|c| c.get("usage").is_some()).collect();
        assert_eq!(usage.len(), usize::from(include_usage));
        if include_usage {
            assert_eq!(usage[0]["usage"]["completion_tokens"], 1);
            assert_eq!(usage[0]["usage"]["prompt_tokens"], 17);
        }
        assert!(text.ends_with("data: [DONE]\n\n"));
    }
}

#[tokio::test]
async fn streaming_failure_is_not_reported_as_a_successful_stop() {
    let response = request(secured_router(), "/v1/chat/completions", Some("test-credential-123456789"), json!({
        "model":"test-chat","messages":[{"role":"user","content":"FAIL_GENERATION"}],"stream":true
    })).await;
    let text = String::from_utf8(
        to_bytes(response.into_body(), 65536)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(text.contains("generation_failed"));
    assert!(!text.contains("\"finish_reason\":\"stop\""));
}

#[tokio::test]
async fn browser_origins_are_explicit_and_preflight_does_not_run_generation() {
    use tower::ServiceExt;
    for (origin, expected) in [
        ("https://untrusted.example", 403),
        ("https://client.example", 204),
    ] {
        let response = secured_router()
            .oneshot(
                axum::http::Request::builder()
                    .method("OPTIONS")
                    .uri("/v1/chat/completions")
                    .header("Origin", origin)
                    .header("Access-Control-Request-Method", "POST")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
        if expected == 204 {
            assert_eq!(response.headers()["access-control-allow-origin"], origin);
        }
    }
    let response = secured_router()
        .oneshot(
            axum::http::Request::builder()
                .uri("/v1/models")
                .header("Origin", "https://untrusted.example")
                .header("Authorization", "Bearer test-credential-123456789")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 403);
}

#[tokio::test]
async fn anthropic_api_key_header_is_accepted() {
    use tower::ServiceExt;
    let response = secured_router().oneshot(axum::http::Request::builder().method("POST").uri("/v1/messages")
        .header("Content-Type", "application/json").header("x-api-key","test-credential-123456789")
        .body(axum::body::Body::from(json!({
            "model":"test-chat","messages":[{"role":"user","content":"hello"}],"max_tokens":8
        }).to_string())).unwrap()).await.unwrap();
    assert_eq!(response.status(), 200);
}

#[test]
fn external_binds_require_keys_and_origin_wildcards_are_rejected() {
    let local = shimmy::security::ApiSecurity::new(None, vec![]).unwrap();
    assert!(local
        .validate_bind("127.0.0.1:11435".parse().unwrap())
        .is_ok());
    assert!(local.validate_bind("[::1]:11435".parse().unwrap()).is_ok());
    assert!(local
        .validate_bind("0.0.0.0:11435".parse().unwrap())
        .is_err());
    assert!(local.validate_bind("[::]:11435".parse().unwrap()).is_err());
    let keyed =
        shimmy::security::ApiSecurity::new(Some("test-credential-123456789".into()), vec![])
            .unwrap();
    assert!(keyed
        .validate_bind("0.0.0.0:11435".parse().unwrap())
        .is_ok());
    assert!(shimmy::security::ApiSecurity::new(Some("".into()), vec![]).is_err());
    assert!(shimmy::security::ApiSecurity::new(None, vec!["*".into()]).is_err());
}

#[tokio::test]
async fn chat_defaults_to_json_and_preserves_ordered_conversation() {
    let (state, calls) = state();
    let request = serde_json::from_value(json!({
        "model": "test-chat",
        "messages": [
            {"role": "system", "content": "Keep it short."},
            {"role": "user", "content": "First question"},
            {"role": "assistant", "content": "First answer"},
            {"role": "user", "content": "Second question"},
            {"role": "assistant", "content": "Answer prefix"}
        ]
    }))
    .unwrap();
    let response = shimmy::openai_compat::chat_completions(State(state), Json(request))
        .await
        .into_response();
    assert_eq!(response.status(), 200);
    assert!(response.headers()["content-type"]
        .to_str()
        .unwrap()
        .starts_with("application/json"));
    let body: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
    assert_eq!(body["choices"][0]["message"]["content"], "test response");
    let calls = calls.lock().unwrap();
    let (prompt, opts) = &calls[0];
    assert!(
        opts.raw_prompt,
        "rendered chat must bypass engine templating"
    );
    assert!(!opts.stream);
    for text in [
        "First question",
        "First answer",
        "Second question",
        "Answer prefix",
    ] {
        assert_eq!(prompt.matches(text).count(), 1, "{prompt}");
    }
    assert!(prompt.find("Second question").unwrap() < prompt.find("Answer prefix").unwrap());
}

#[tokio::test]
async fn chat_streams_only_when_requested_and_finishes() {
    let (state, calls) = state();
    let request = serde_json::from_value(json!({
        "model": "test-chat", "stream": true,
        "messages": [{"role": "user", "content": "Hello"}]
    }))
    .unwrap();
    let response = shimmy::openai_compat::chat_completions(State(state), Json(request))
        .await
        .into_response();
    assert_eq!(response.status(), 200);
    assert!(response.headers()["content-type"]
        .to_str()
        .unwrap()
        .starts_with("text/event-stream"));
    let body = to_bytes(response.into_body(), 65536).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("test response"));
    assert!(body.contains("[DONE]"));
    assert!(calls.lock().unwrap()[0].1.raw_prompt);
}

#[tokio::test]
async fn native_completion_and_anthropic_prompts_are_rendered_once() {
    for endpoint in ["native", "completion", "anthropic"] {
        let (state, calls) = state();
        let response = match endpoint {
            "native" => shimmy::api::generate(
                State(state),
                Json(
                    serde_json::from_value(json!({
                        "model": "test-chat", "prompt": "Hello once", "stream": false
                    }))
                    .unwrap(),
                ),
            )
            .await
            .into_response(),
            "completion" => shimmy::openai_compat::completions(
                State(state),
                Json(
                    serde_json::from_value(json!({
                        "model": "test-chat", "prompt": "Hello once"
                    }))
                    .unwrap(),
                ),
            )
            .await
            .into_response(),
            _ => shimmy::anthropic_compat::messages(
                State(state),
                Json(
                    serde_json::from_value(json!({
                        "model": "test-chat", "max_tokens": 16,
                        "messages": [{"role": "user", "content": "Hello once"}]
                    }))
                    .unwrap(),
                ),
            )
            .await
            .into_response(),
        };
        assert_eq!(response.status(), 200, "{endpoint}");
        let calls = calls.lock().unwrap();
        let (prompt, opts) = &calls[0];
        assert!(opts.raw_prompt, "{endpoint} must bypass a second render");
        assert_eq!(prompt.matches("Hello once").count(), 1);
        assert_eq!(
            prompt.matches("<|im_start|>user").count(),
            1,
            "{endpoint}: {prompt}"
        );
    }
}

#[tokio::test]
async fn native_raw_prompt_reaches_engine_unchanged() {
    let (state, calls) = state();
    let request = serde_json::from_value(json!({
        "model": "test-chat", "prompt": "literal completion prefix", "raw_prompt": true, "stream": false
    })).unwrap();
    let response = shimmy::api::generate(State(state), Json(request))
        .await
        .into_response();
    assert_eq!(response.status(), 200);
    let calls = calls.lock().unwrap();
    assert_eq!(calls[0].0, "literal completion prefix");
    assert!(calls[0].1.raw_prompt);
}
