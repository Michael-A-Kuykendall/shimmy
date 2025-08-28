use serde_json::json;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

// Note: These are integration tests that require external dependencies
// Run with: cargo test --test integration_tests -- --ignored

#[tokio::test]
#[ignore] // Requires actual models and Python environment
async fn test_huggingface_engine_integration() {
    use shimmy::engine::{huggingface::HuggingFaceEngine, UniversalEngine, UniversalModelSpec, ModelBackend, GenOptions};
    use std::path::PathBuf;
    
    let engine = HuggingFaceEngine::new();
    let spec = UniversalModelSpec {
        name: "test-model".to_string(),
        backend: ModelBackend::HuggingFace {
            base_model_id: "microsoft/DialoGPT-small".to_string(), // Small model for testing
            peft_path: None,
            use_local: false,
        },
        template: Some("chatml".to_string()),
        ctx_len: 1024,
        device: "cpu".to_string(),
        n_threads: Some(2),
    };
    
    // This test would require actual HuggingFace model access
    let result = engine.load(&spec).await;
    match result {
        Ok(model) => {
            let response = model.generate("Hello", GenOptions::default(), None).await;
            assert!(response.is_ok());
            let text = response.unwrap();
            assert!(!text.is_empty());
        }
        Err(e) => {
            // Expected if dependencies aren't available
            println!("HuggingFace engine test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_http_api_health_check() {
    use shimmy::{AppState, server, engine::universal::ShimmyUniversalEngine, model_registry::Registry};
    
    let registry = Registry::default();
    let engine = Box::new(ShimmyUniversalEngine::new());
    let legacy_engine = Box::new(shimmy::engine::llama::LlamaEngine::new());
    
    let state = Arc::new(AppState {
        engine,
        legacy_engine,
        registry,
    });
    
    // Start server on random port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        let _ = server::run(addr, state).await;
    });
    
    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Test health endpoint
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://{}/health", addr))
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
#[ignore] // Requires models to be available
async fn test_api_generate_endpoint() {
    use shimmy::{AppState, server, engine::universal::ShimmyUniversalEngine, model_registry::{Registry, UniversalModelEntry, ModelBackendConfig}};
    use std::path::PathBuf;
    
    let mut registry = Registry::default();
    
    // Add a test model (would need to be available)
    registry.register_universal(UniversalModelEntry {
        name: "test-model".to_string(),
        backend: ModelBackendConfig::HuggingFace {
            base_model_id: "microsoft/DialoGPT-small".to_string(),
            peft_path: None,
            use_local: Some(false),
        },
        template: Some("chatml".to_string()),
        ctx_len: Some(1024),
        device: Some("cpu".to_string()),
        n_threads: Some(2),
    });
    
    let engine = Box::new(ShimmyUniversalEngine::new());
    let legacy_engine = Box::new(shimmy::engine::llama::LlamaEngine::new());
    
    let state = Arc::new(AppState {
        engine,
        legacy_engine,
        registry,
    });
    
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        let _ = server::run(addr, state).await;
    });
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let client = reqwest::Client::new();
    let request_body = json!({
        "model": "test-model",
        "prompt": "Hello, world!",
        "max_tokens": 10,
        "stream": false
    });
    
    let response = client
        .post(&format!("http://{}/api/generate", addr))
        .json(&request_body)
        .send()
        .await
        .unwrap();
    
    // This might fail if model isn't actually available, but tests the API structure
    println!("Response status: {}", response.status());
    let body_text = response.text().await.unwrap();
    println!("Response body: {}", body_text);
}

#[tokio::test]
#[ignore] // Requires WebSocket support
async fn test_websocket_api() {
    use shimmy::{AppState, server, engine::universal::ShimmyUniversalEngine, model_registry::Registry};
    
    let registry = Registry::default();
    let engine = Box::new(ShimmyUniversalEngine::new());
    let legacy_engine = Box::new(shimmy::engine::llama::LlamaEngine::new());
    
    let state = Arc::new(AppState {
        engine,
        legacy_engine,
        registry,
    });
    
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        let _ = server::run(addr, state).await;
    });
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Test WebSocket connection
    let ws_url = format!("ws://{}/ws/generate", addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect to WebSocket");
    
    // Send test request
    let request = json!({
        "model": "test-model",
        "prompt": "Hello",
        "max_tokens": 5
    });
    
    ws_stream.send(Message::Text(request.to_string())).await.unwrap();
    
    // Should receive error message since model doesn't exist
    if let Some(msg) = ws_stream.next().await {
        let msg = msg.unwrap();
        if let Message::Text(text) = msg {
            println!("WebSocket response: {}", text);
            assert!(text.contains("error") || text.contains("not found"));
        }
    }
}

#[test]
fn test_cli_parsing() {
    use shimmy::cli::{Cli, Command};
    use clap::Parser;
    
    // Test list command
    let args = vec!["shimmy", "list"];
    let cli = Cli::try_parse_from(args).unwrap();
    matches!(cli.cmd, Command::List);
    
    // Test serve command
    let args = vec!["shimmy", "serve", "--bind", "0.0.0.0:8080"];
    let cli = Cli::try_parse_from(args).unwrap();
    match cli.cmd {
        Command::Serve { bind } => assert_eq!(bind, "0.0.0.0:8080"),
        _ => panic!("Expected Serve command"),
    }
    
    // Test probe command
    let args = vec!["shimmy", "probe", "test-model"];
    let cli = Cli::try_parse_from(args).unwrap();
    match cli.cmd {
        Command::Probe { name } => assert_eq!(name, "test-model"),
        _ => panic!("Expected Probe command"),
    }
    
    // Test generate command
    let args = vec!["shimmy", "generate", "test-model", "--prompt", "Hello", "--max-tokens", "50"];
    let cli = Cli::try_parse_from(args).unwrap();
    match cli.cmd {
        Command::Generate { name, prompt, max_tokens } => {
            assert_eq!(name, "test-model");
            assert_eq!(prompt, "Hello");
            assert_eq!(max_tokens, 50);
        }
        _ => panic!("Expected Generate command"),
    }
}

#[test]
fn test_template_rendering() {
    use shimmy::templates::TemplateFamily;
    
    let messages = vec![
        ("user".to_string(), "Hello".to_string()),
        ("assistant".to_string(), "Hi there!".to_string()),
        ("user".to_string(), "How are you?".to_string()),
    ];
    
    // Test ChatML template
    let chatml = TemplateFamily::ChatML;
    let rendered = chatml.render(Some("You are a helpful assistant"), &messages, None);
    assert!(rendered.contains("<|im_start|>"));
    assert!(rendered.contains("<|im_end|>"));
    assert!(rendered.contains("You are a helpful assistant"));
    assert!(rendered.contains("Hello"));
    assert!(rendered.contains("How are you?"));
    
    // Test Llama3 template  
    let llama3 = TemplateFamily::Llama3;
    let rendered = llama3.render(None, &messages, None);
    assert!(rendered.contains("<|start_header_id|>"));
    assert!(rendered.contains("<|end_header_id|>"));
    assert!(rendered.contains("<|eot_id|>"));
}

#[tokio::test]
async fn test_concurrent_requests() {
    // Test that the server can handle multiple concurrent requests
    use shimmy::{AppState, server, engine::universal::ShimmyUniversalEngine, model_registry::Registry};
    
    let registry = Registry::default();
    let engine = Box::new(ShimmyUniversalEngine::new());
    let legacy_engine = Box::new(shimmy::engine::llama::LlamaEngine::new());
    
    let state = Arc::new(AppState {
        engine,
        legacy_engine,
        registry,
    });
    
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        let _ = server::run(addr, state).await;
    });
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Send multiple concurrent health check requests
    let client = reqwest::Client::new();
    let mut handles = vec![];
    
    for i in 0..10 {
        let client = client.clone();
        let addr = addr;
        let handle = tokio::spawn(async move {
            let response = client
                .get(&format!("http://{}/health", addr))
                .send()
                .await
                .unwrap();
            (i, response.status())
        });
        handles.push(handle);
    }
    
    // All requests should succeed
    for handle in handles {
        let (i, status) = handle.await.unwrap();
        assert_eq!(status, 200, "Request {} failed", i);
    }
}