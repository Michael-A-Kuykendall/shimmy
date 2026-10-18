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
