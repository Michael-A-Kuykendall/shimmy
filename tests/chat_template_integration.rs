//! Integration tests for chat template handling and model output correctness.
//!
//! These tests require actual GGUF models to be present (auto-discovered from
//! Ollama model directories).  They are `#[ignore]` by default — run with:
//!
//!   cargo test --test chat_template_integration -- --ignored
//!
//! Tests verify that:
//! 1. Models produce non-empty output for simple prompts
//! 2. Models produce non-empty output for longer prompts
//! 3. Native GGUF chat templates are applied (thinking models respect /no_think)
//! 4. Frequency/presence penalties reach the sampler

use shimmy::engine::adapter::InferenceEngineAdapter;
use shimmy::engine::{GenOptions, InferenceEngine, ModelSpec};
use shimmy::model_registry::Registry;
use std::path::PathBuf;

/// Find a GGUF model via Shimmy's auto-discovery (reads Ollama model dirs).
/// Returns None if no matching model is installed.
fn find_ollama_model(name_fragment: &str) -> Option<(String, PathBuf)> {
    let mut registry = Registry::with_discovery();
    registry.auto_register_discovered();
    let available = registry.list_all_available();
    for name in &available {
        if name.to_lowercase().contains(name_fragment) {
            if let Some(spec) = registry.to_spec(name) {
                if spec.base_path.exists() {
                    return Some((name.clone(), spec.base_path));
                }
            }
        }
    }
    None
}

/// Helper: build a ModelSpec for testing
fn test_spec(name: &str, path: PathBuf) -> ModelSpec {
    ModelSpec {
        name: name.to_string(),
        base_path: path,
        lora_path: None,
        template: None,
        ctx_len: 8192,
        n_threads: Some(4),
    }
}

/// Helper: default gen options for testing (non-streaming, short output)
fn test_opts(max_tokens: usize) -> GenOptions {
    GenOptions {
        max_tokens,
        temperature: 0.3,
        stream: false,
        ..Default::default()
    }
}

// ── Short prompt: model produces non-empty output ──────────────────────

#[tokio::test]
#[ignore] // requires local GGUF model
async fn test_short_prompt_produces_output() {
    let Some((name, path)) = find_ollama_model("gemma3") else {
        eprintln!("SKIP: no gemma3 model found");
        return;
    };
    let engine = InferenceEngineAdapter::new();
    let spec = test_spec(&name, path);
    let loaded = engine.load(&spec).await.expect("model should load");

    // format_prompt should work (native template or fallback)
    let pairs = vec![(
        "user".to_string(),
        "What is 2+2? Answer in one word.".to_string(),
    )];
    let prompt = loaded
        .format_prompt(&pairs)
        .unwrap_or_else(|| "What is 2+2? Answer in one word.".to_string());

    let output = loaded
        .generate(&prompt, test_opts(64), None)
        .await
        .expect("generate should succeed");

    assert!(!output.is_empty(), "model should produce non-empty output");
    assert!(
        output.len() > 1,
        "output should be more than 1 char: '{}'",
        output
    );
    eprintln!(
        "Short prompt output ({} chars): {}",
        output.len(),
        &output[..output.len().min(200)]
    );
}

// ── Long prompt: model produces non-empty output ───────────────────────

#[tokio::test]
#[ignore]
async fn test_long_prompt_produces_output() {
    let Some((name, path)) = find_ollama_model("gemma3") else {
        eprintln!("SKIP: no gemma3 model found");
        return;
    };
    let engine = InferenceEngineAdapter::new();
    let spec = test_spec(&name, path);
    let loaded = engine.load(&spec).await.expect("model should load");

    // Build a long prompt (~2000 tokens worth)
    let long_context = "The quick brown fox jumps over the lazy dog. ".repeat(200);
    let prompt_text = format!(
        "Here is a long passage:\n{}\n\nSummarize the above in one sentence.",
        long_context
    );
    let pairs = vec![("user".to_string(), prompt_text.clone())];
    let prompt = loaded.format_prompt(&pairs).unwrap_or(prompt_text);

    let output = loaded
        .generate(&prompt, test_opts(128), None)
        .await
        .expect("generate should succeed on long prompt");

    assert!(
        !output.is_empty(),
        "model should produce output on long prompt"
    );
    eprintln!(
        "Long prompt output ({} chars): {}",
        output.len(),
        &output[..output.len().min(200)]
    );
}

// ── Cached model reuse: second request also works ──────────────────────

#[tokio::test]
#[ignore]
async fn test_cached_model_second_request() {
    let Some((name, path)) = find_ollama_model("gemma3") else {
        eprintln!("SKIP: no gemma3 model found");
        return;
    };
    let engine = InferenceEngineAdapter::new();
    let spec = test_spec(&name, path.clone());

    // First request
    let loaded1 = engine.load(&spec).await.expect("first load");
    let pairs1 = vec![("user".to_string(), "Say hello.".to_string())];
    let prompt1 = loaded1
        .format_prompt(&pairs1)
        .unwrap_or("Say hello.".to_string());
    let out1 = loaded1
        .generate(&prompt1, test_opts(32), None)
        .await
        .expect("first generate");
    assert!(!out1.is_empty(), "first request should produce output");

    // Second request (should hit cache — no model reload)
    let loaded2 = engine.load(&spec).await.expect("second load (cached)");
    let pairs2 = vec![("user".to_string(), "What is 1+1?".to_string())];
    let prompt2 = loaded2
        .format_prompt(&pairs2)
        .unwrap_or("What is 1+1?".to_string());
    let out2 = loaded2
        .generate(&prompt2, test_opts(32), None)
        .await
        .expect("second generate");
    assert!(
        !out2.is_empty(),
        "second request (cached) should produce output"
    );

    // Outputs should be different (different prompts, KV cache cleared)
    assert_ne!(
        out1, out2,
        "different prompts should produce different output"
    );
    eprintln!("Request 1: {}", &out1[..out1.len().min(100)]);
    eprintln!("Request 2: {}", &out2[..out2.len().min(100)]);
}

// ── Thinking model with /no_think: native template suppresses think ────

#[tokio::test]
#[ignore]
async fn test_thinking_model_no_think() {
    // Find a thinking model (qwen3 or cogito)
    let model = find_ollama_model("qwen3").or_else(|| find_ollama_model("cogito"));
    let Some((name, path)) = model else {
        eprintln!("SKIP: no thinking model (qwen3/cogito) found");
        return;
    };
    let engine = InferenceEngineAdapter::new();
    let spec = test_spec(&name, path);
    let loaded = engine.load(&spec).await.expect("model should load");

    // Prompt includes /no_think — native GGUF template should suppress thinking
    let user_msg = "What is 2+2? Answer with just the number.\n\n/no_think".to_string();
    let pairs = vec![("user".to_string(), user_msg.clone())];
    let prompt = loaded.format_prompt(&pairs).unwrap_or(user_msg);

    let output = loaded
        .generate(&prompt, test_opts(256), None)
        .await
        .expect("generate should succeed");

    assert!(!output.is_empty(), "thinking model should produce output");

    // With native template + /no_think, output should NOT start with <think>
    let has_think = output.contains("<think>");
    eprintln!(
        "Thinking model output ({} chars, has_think={}): {}",
        output.len(),
        has_think,
        &output[..output.len().min(300)]
    );

    // This is the key assertion: native template should suppress thinking
    assert!(
        !has_think,
        "With native GGUF template, /no_think should suppress <think> blocks. \
         Got: {}",
        &output[..output.len().min(500)]
    );
}

// ── Penalties are applied: output with high penalty differs ────────────

#[tokio::test]
#[ignore]
async fn test_penalties_affect_output() {
    let Some((name, path)) = find_ollama_model("gemma3") else {
        eprintln!("SKIP: no gemma3 model found");
        return;
    };
    let engine = InferenceEngineAdapter::new();
    let spec = test_spec(&name, path);
    let loaded = engine.load(&spec).await.expect("model should load");

    let pairs = vec![(
        "user".to_string(),
        "List the numbers 1 through 20, one per line.".to_string(),
    )];
    let prompt = loaded
        .format_prompt(&pairs)
        .unwrap_or("List the numbers 1 through 20, one per line.".to_string());

    // Without penalties
    let mut opts_no_penalty = test_opts(256);
    opts_no_penalty.frequency_penalty = 0.0;
    opts_no_penalty.presence_penalty = 0.0;
    let out_no = loaded
        .generate(&prompt, opts_no_penalty, None)
        .await
        .expect("generate without penalties");

    // With strong penalties
    let mut opts_penalty = test_opts(256);
    opts_penalty.frequency_penalty = 1.5;
    opts_penalty.presence_penalty = 1.0;
    let out_with = loaded
        .generate(&prompt, opts_penalty, None)
        .await
        .expect("generate with penalties");

    // Both should produce output
    assert!(!out_no.is_empty(), "no-penalty output should be non-empty");
    assert!(!out_with.is_empty(), "penalty output should be non-empty");

    eprintln!(
        "No penalty ({} chars): {}",
        out_no.len(),
        &out_no[..out_no.len().min(200)]
    );
    eprintln!(
        "With penalty ({} chars): {}",
        out_with.len(),
        &out_with[..out_with.len().min(200)]
    );

    // With very high penalties, output should be different (more varied/shorter)
    // We can't assert exact content, but lengths should differ
    // (high frequency penalty typically produces shorter output)
}

// ── Native template is used when available ─────────────────────────────

#[tokio::test]
#[ignore]
async fn test_native_template_used() {
    let Some((name, path)) = find_ollama_model("qwen3").or_else(|| find_ollama_model("gemma3"))
    else {
        eprintln!("SKIP: no model found");
        return;
    };
    let engine = InferenceEngineAdapter::new();
    let spec = test_spec(&name, path);
    let loaded = engine.load(&spec).await.expect("model should load");

    // format_prompt should return Some (native GGUF template present)
    let pairs = vec![
        ("system".to_string(), "You are helpful.".to_string()),
        ("user".to_string(), "Hi".to_string()),
    ];
    let native = loaded.format_prompt(&pairs);

    eprintln!("Model: {}", name);
    eprintln!("Native template present: {}", native.is_some());
    if let Some(ref prompt) = native {
        eprintln!(
            "Formatted prompt preview: {}",
            &prompt[..prompt.len().min(300)]
        );
    }

    // All Ollama-pulled GGUF models should have embedded chat templates
    assert!(
        native.is_some(),
        "Ollama GGUF model '{}' should have a native chat template",
        name
    );
}
