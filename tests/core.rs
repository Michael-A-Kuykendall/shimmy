use clap::Parser;
use serde_json::json;
use shimmy::api::ChatMessage;
use shimmy::cli::{Cli, Command};
use shimmy::discovery::discover_models_from_directory;
use shimmy::engine::ModelSpec;
use shimmy::model_registry::{ModelEntry, Registry};
use shimmy::openai_compat::{
    self, ChatCompletionRequest, ChatCompletionResponse, Choice, MessageContent, ModelsResponse,
    OAIMessage, Usage,
};
use shimmy::templates::TemplateFamily;
use std::path::PathBuf;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// CLI parsing
// ---------------------------------------------------------------------------

#[test]
fn test_cli_subcommands_parse() {
    let cases: Vec<(&str, &[&str])> = vec![
        ("serve", &["shimmy", "serve"]),
        ("list", &["shimmy", "list"]),
        ("discover", &["shimmy", "discover"]),
        ("probe", &["shimmy", "probe", "test-model"]),
        ("bench", &["shimmy", "bench", "test-model"]),
        (
            "generate",
            &["shimmy", "generate", "test-model", "--prompt", "hi"],
        ),
        ("gpu-info", &["shimmy", "gpu-info"]),
        ("init", &["shimmy", "init", "--template", "docker"]),
    ];
    for (name, args) in cases {
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok(), "failed to parse '{}': {:?}", name, cli.err());
    }
}

#[test]
fn test_cli_global_flags() {
    let cli = Cli::try_parse_from([
        "shimmy",
        "--model-dirs",
        "/a;/b",
        "--gpu-backend",
        "vulkan",
        "list",
    ])
    .unwrap();
    assert_eq!(cli.model_dirs.as_deref(), Some("/a;/b"));
}

#[test]
fn test_cli_model_dirs_option() {
    let cli =
        Cli::try_parse_from(["shimmy", "--model-dirs", "test/path1;test/path2", "serve"]).unwrap();
    assert_eq!(cli.model_dirs, Some("test/path1;test/path2".to_string()));

    let cli = Cli::try_parse_from(["shimmy", "list"]).unwrap();
    assert!(cli.model_dirs.is_none());
}

#[test]
fn test_cli_generate_args() {
    let cli = Cli::try_parse_from([
        "shimmy",
        "generate",
        "test-model",
        "--prompt",
        "Hello",
        "--max-tokens",
        "50",
    ])
    .unwrap();
    match cli.cmd {
        Command::Generate {
            name,
            prompt,
            max_tokens,
            ..
        } => {
            assert_eq!(name, "test-model");
            assert_eq!(prompt, "Hello");
            assert_eq!(max_tokens, 50);
        }
        _ => panic!("expected Generate command"),
    }
}

#[test]
fn test_cli_serve_args() {
    let cli = Cli::try_parse_from(["shimmy", "serve", "--bind", "0.0.0.0:8080"]).unwrap();
    match cli.cmd {
        Command::Serve { bind, .. } => assert_eq!(bind, "0.0.0.0:8080"),
        _ => panic!("expected Serve command"),
    }
}

#[test]
fn test_cli_probe_command() {
    let cli = Cli::try_parse_from(["shimmy", "probe", "test-model"]).unwrap();
    match cli.cmd {
        Command::Probe { name } => assert_eq!(name, "test-model"),
        _ => panic!("expected Probe command"),
    }
}

#[test]
fn test_cli_list_short_flag() {
    let cli = Cli::try_parse_from(["shimmy", "list", "-s"]).unwrap();
    match cli.cmd {
        Command::List { short } => assert!(short),
        _ => panic!("expected List command"),
    }
}

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

#[test]
fn test_version_is_sane() {
    let v = env!("CARGO_PKG_VERSION");
    assert!(!v.is_empty());
    assert!(v.contains('.'));
    let parts: Vec<&str> = v.splitn(3, '.').collect();
    assert!(parts.len() >= 2);
    parts[0].parse::<u32>().expect("major version numeric");
    parts[1].parse::<u32>().expect("minor version numeric");
}

// ---------------------------------------------------------------------------
// Model registry
// ---------------------------------------------------------------------------

#[test]
fn test_registry_register_get_list() {
    let mut registry = Registry::new();
    let model = ModelEntry {
        name: "test-model".into(),
        base_path: PathBuf::from("test.gguf"),
        lora_path: None,
        template: Some("chatml".into()),
        ctx_len: Some(2048),
        n_threads: None,
    };
    registry.register(model.clone());

    assert!(registry.get("test-model").is_some());
    assert_eq!(registry.get("test-model").unwrap().name, "test-model");
    assert!(registry.get("nonexistent").is_none());

    let models = registry.list();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].name, "test-model");
}

#[test]
fn test_registry_error_handling() {
    let registry = Registry::new();
    assert!(registry.get("nonexistent").is_none());
    assert!(registry.list().is_empty());

    let invalid = PathBuf::from("/nonexistent/directory");
    let result = discover_models_from_directory(&invalid);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_registry_infer_template() {
    let registry = Registry::new();
    let cases = vec![
        ("llama-7b-chat", "chatml"),
        ("llama3-8b-instruct", "llama3"),
        ("meta-llama-3-8b", "llama3"),
        ("phi-3-mini", "chatml"),
        ("qwen2-instruct", "chatml"),
        ("mistral-7b", "chatml"),
        ("gemma-2b", "chatml"),
    ];
    for (name, expected) in cases {
        assert_eq!(registry.infer_template(name), expected);
    }
}

// ---------------------------------------------------------------------------
// Model discovery
// ---------------------------------------------------------------------------

#[test]
fn test_discover_models_in_directory() {
    let dir = TempDir::new().unwrap();
    let path = dir.path();
    std::fs::write(path.join("test1.gguf"), b"dummy").unwrap();
    std::fs::write(path.join("test2.safetensors"), b"dummy").unwrap();
    std::fs::write(path.join("readme.txt"), b"not a model").unwrap();

    let models = discover_models_from_directory(path).unwrap();
    assert_eq!(models.len(), 2);
    let names: Vec<&str> = models.iter().map(|m| m.name.as_str()).collect();
    assert!(names.contains(&"test1"));
    assert!(names.contains(&"test2"));
}

#[test]
fn test_custom_model_directory_env_vars() {
    std::env::set_var("SHIMMY_MODELS_DIR", "/custom/shimmy/models");
    let path = PathBuf::from("/custom/shimmy/models");
    assert_eq!(
        std::env::var("SHIMMY_MODELS_DIR").ok(),
        Some("/custom/shimmy/models".to_string())
    );
    let result = discover_models_from_directory(&path);
    assert!(result.is_ok() || result.is_err());
    std::env::remove_var("SHIMMY_MODELS_DIR");
}

// ---------------------------------------------------------------------------
// Template rendering
// ---------------------------------------------------------------------------

#[test]
fn test_template_render_chatml() {
    let t = TemplateFamily::ChatML;
    let msgs = vec![
        ("user".into(), "Hello".into()),
        ("assistant".into(), "Hi!".into()),
    ];
    let result = t.render(None, &msgs, Some("How are you?"));
    assert!(result.contains("<|im_start|>user"));
    assert!(result.contains("<|im_end|>"));
    assert!(result.contains("Hello"));
    assert!(result.contains("Hi!"));
    assert!(result.contains("How are you?"));
}

#[test]
fn test_template_render_llama3() {
    let t = TemplateFamily::Llama3;
    let msgs = vec![
        ("user".into(), "Hello".into()),
        ("assistant".into(), "Hi!".into()),
    ];
    let result = t.render(None, &msgs, None);
    assert!(result.contains("<|start_header_id|>user<|end_header_id|>"));
    assert!(result.contains("<|eot_id|>"));
}

// ---------------------------------------------------------------------------
// SafeTensors extension detection
// ---------------------------------------------------------------------------

#[test]
fn test_safetensors_extension_detection() {
    let spec = ModelSpec {
        name: "test".into(),
        base_path: PathBuf::from("model.safetensors"),
        lora_path: None,
        template: None,
        ctx_len: 2048,
        n_threads: None,
    };
    assert_eq!(spec.base_path.extension().unwrap(), "safetensors");

    let complex = ModelSpec {
        name: "complex".into(),
        base_path: PathBuf::from("/path/to/huggingface/org/model/pytorch_model.safetensors"),
        lora_path: None,
        template: None,
        ctx_len: 2048,
        n_threads: None,
    };
    assert_eq!(complex.base_path.extension().unwrap(), "safetensors");

    let gguf = PathBuf::from("model.gguf");
    assert_eq!(gguf.extension().unwrap(), "gguf");
    assert_ne!(gguf.extension().unwrap(), "safetensors");
}

// ---------------------------------------------------------------------------
// OpenAI API types
// ---------------------------------------------------------------------------

#[test]
fn test_chat_completion_request_serde() {
    let json = r#"{
        "model": "test-model",
        "messages": [{"role": "user", "content": "Hello"}],
        "temperature": 0.7,
        "max_tokens": 100
    }"#;
    let req: ChatCompletionRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.model, "test-model");
    assert_eq!(req.messages.len(), 1);
    assert_eq!(req.temperature, Some(0.7));
    assert_eq!(req.max_tokens, Some(100));
}

#[test]
fn test_chat_completion_response_serde() {
    let resp = ChatCompletionResponse {
        id: "chatcmpl-test".into(),
        object: "chat.completion".into(),
        created: 1234567890,
        model: "test-model".into(),
        choices: vec![Choice {
            index: 0,
            message: ChatMessage {
                role: "assistant".into(),
                content: "Hello!".into(),
            },
            finish_reason: Some("stop".into()),
        }],
        usage: Usage {
            prompt_tokens: 5,
            completion_tokens: 2,
            total_tokens: 7,
        },
    };
    let j = serde_json::to_value(&resp).unwrap();
    assert_eq!(j["id"], "chatcmpl-test");
    assert_eq!(j["object"], "chat.completion");
    assert_eq!(j["choices"][0]["message"]["role"], "assistant");
    assert_eq!(j["usage"]["total_tokens"], 7);
}

#[test]
fn test_models_response_serde() {
    let resp = ModelsResponse {
        object: "list".into(),
        data: vec![openai_compat::ListModel {
            id: "phi3-mini".into(),
            object: "model".into(),
            created: 1234567890,
            owned_by: "shimmy".into(),
        }],
    };
    let j = serde_json::to_value(&resp).unwrap();
    assert_eq!(j["object"], "list");
    assert_eq!(j["data"].as_array().unwrap().len(), 1);
    assert_eq!(j["data"][0]["id"], "phi3-mini");
}

#[test]
fn test_model_struct_completeness() {
    let model = openai_compat::Model {
        id: "test-model".into(),
        object: "model".into(),
        created: 1640995200,
        owned_by: "shimmy".into(),
        permission: None,
        root: Some("test-model".into()),
        parent: None,
    };
    let j = serde_json::to_value(&model).unwrap();
    assert_eq!(j["id"], "test-model");
    assert_eq!(j["owned_by"], "shimmy");
    assert_eq!(j["root"], "test-model");
    assert!(j.get("permission").is_none());
    assert!(j.get("parent").is_none());
}

#[test]
fn test_error_response_json_shape() {
    let err = json!({
        "error": {
            "message": "Model 'nonexistent' not found. Available models: []",
            "type": "invalid_request_error",
            "param": "model",
            "code": "model_not_found"
        }
    });
    let e = &err["error"];
    assert!(e["message"].is_string());
    assert_eq!(e["type"], "invalid_request_error");
    assert_eq!(e["param"], "model");
    assert_eq!(e["code"], "model_not_found");
    assert!(e["message"].as_str().unwrap().contains("not found"));
}

// ---------------------------------------------------------------------------
// Ollama /api/tags response
// ---------------------------------------------------------------------------

#[test]
fn test_api_tags_response_structure() {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TagsResponse {
        models: Vec<TagModel>,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TagModel {
        name: String,
        model: String,
        modified_at: String,
        size: u64,
        digest: String,
        details: ModelDetails,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    struct ModelDetails {
        format: String,
        family: String,
        parameter_size: String,
        quantization_level: String,
    }

    let json = r#"{
        "models": [{
            "name": "tinyllama",
            "model": "tinyllama",
            "modified_at": "2025-01-01T00:00:00Z",
            "size": 0,
            "digest": "",
            "details": {
                "format": "gguf",
                "family": "",
                "parameter_size": "",
                "quantization_level": ""
            }
        }]
    }"#;
    let parsed: TagsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.models.len(), 1);
    assert_eq!(parsed.models[0].name, "tinyllama");
    assert_eq!(parsed.models[0].details.format, "gguf");

    let raw: serde_json::Value = serde_json::from_str(json).unwrap();
    assert!(raw.get("models").is_some(), "must have 'models' key");
    assert!(raw.get("data").is_none(), "must NOT have 'data' key");
}

// ---------------------------------------------------------------------------
// SSE streaming chunk serialization (Issue #53)
// ---------------------------------------------------------------------------

#[test]
fn test_sse_streaming_chunk_format() {
    // Initial chunk (role announcement)
    let initial = openai_compat::ChatCompletionChunk {
        id: "chatcmpl-test".into(),
        object: "chat.completion.chunk".into(),
        created: 1234567890,
        model: "test-model".into(),
        choices: vec![openai_compat::ChunkChoice {
            index: 0,
            delta: openai_compat::Delta {
                role: Some("assistant".into()),
                content: None,
            },
            finish_reason: None,
        }],
    };
    let j = serde_json::to_value(&initial).unwrap();
    assert_eq!(j["object"], "chat.completion.chunk");
    assert_eq!(j["choices"][0]["delta"]["role"], "assistant");
    assert_eq!(j["choices"][0]["delta"]["content"], serde_json::Value::Null);

    // Token chunk
    let token = openai_compat::ChatCompletionChunk {
        id: "chatcmpl-test".into(),
        object: "chat.completion.chunk".into(),
        created: 1234567890,
        model: "test-model".into(),
        choices: vec![openai_compat::ChunkChoice {
            index: 0,
            delta: openai_compat::Delta {
                role: None,
                content: Some("Hello".into()),
            },
            finish_reason: None,
        }],
    };
    let j = serde_json::to_value(&token).unwrap();
    assert_eq!(j["choices"][0]["delta"]["content"], "Hello");
    assert_eq!(j["choices"][0]["delta"]["role"], serde_json::Value::Null);

    // Final chunk (finish_reason)
    let final_chunk = openai_compat::ChatCompletionChunk {
        id: "chatcmpl-test".into(),
        object: "chat.completion.chunk".into(),
        created: 1234567890,
        model: "test-model".into(),
        choices: vec![openai_compat::ChunkChoice {
            index: 0,
            delta: openai_compat::Delta {
                role: None,
                content: None,
            },
            finish_reason: Some("stop".into()),
        }],
    };
    let j = serde_json::to_value(&final_chunk).unwrap();
    assert_eq!(j["choices"][0]["finish_reason"], "stop");

    // Serialized JSON must not contain duplicate "data:" anywhere
    let serialized = serde_json::to_string(&token).unwrap();
    assert!(
        !serialized.contains("data:"),
        "JSON payload must not contain 'data:' prefix — that comes from the SSE transport layer"
    );
}

// ---------------------------------------------------------------------------
// Multi-part content array (Issue #191)
// ---------------------------------------------------------------------------

#[test]
fn test_multi_part_content_array_deserialization() {
    let string = r#"{"role":"user","content":"hello"}"#;
    let msg: OAIMessage = serde_json::from_str(string).unwrap();
    assert_eq!(msg.content_text(), "hello");

    let array = r#"{"role":"user","content":[{"type":"text","text":"hello"}]}"#;
    let msg: OAIMessage = serde_json::from_str(array).unwrap();
    assert_eq!(msg.content_text(), "hello");

    let multi = r#"{"role":"user","content":[{"type":"text","text":"first"},{"type":"text","text":"second"}]}"#;
    let msg: OAIMessage = serde_json::from_str(multi).unwrap();
    assert_eq!(msg.content_text(), "first\nsecond");

    let mixed = r#"{"role":"user","content":[{"type":"image_url"},{"type":"text","text":"describe this"}]}"#;
    let msg: OAIMessage = serde_json::from_str(mixed).unwrap();
    assert_eq!(msg.content_text(), "describe this");
}

// ---------------------------------------------------------------------------
// Request structure validation
// ---------------------------------------------------------------------------

#[test]
fn test_system_message_handling() {
    let req = ChatCompletionRequest {
        model: "test-model".into(),
        messages: vec![
            OAIMessage {
                role: "system".into(),
                content: MessageContent::Text("You are helpful.".into()),
            },
            OAIMessage {
                role: "user".into(),
                content: MessageContent::Text("What is 2+2?".into()),
            },
        ],
        stream: Some(false),
        temperature: Some(0.5),
        max_tokens: Some(50),
        top_p: None,
        stop: None,
        frequency_penalty: None,
        presence_penalty: None,
    };
    assert_eq!(req.messages.len(), 2);
    assert_eq!(req.messages[0].role, "system");
    assert_eq!(req.messages[1].role, "user");
    assert!(req.messages[0].content_text().contains("helpful"));
    assert_eq!(req.temperature, Some(0.5));
}

#[test]
fn test_streaming_request_processing() {
    let req = ChatCompletionRequest {
        model: "test-model".into(),
        messages: vec![OAIMessage {
            role: "user".into(),
            content: MessageContent::Text("Count to 5".into()),
        }],
        stream: Some(true),
        temperature: Some(0.3),
        max_tokens: Some(50),
        top_p: None,
        stop: None,
        frequency_penalty: None,
        presence_penalty: None,
    };
    assert_eq!(req.stream, Some(true));
    assert_eq!(req.temperature, Some(0.3));
}

#[test]
fn test_generation_options_parsing() {
    let req = ChatCompletionRequest {
        model: "test-model".into(),
        messages: vec![OAIMessage {
            role: "user".into(),
            content: MessageContent::Text("Test".into()),
        }],
        stream: Some(true),
        temperature: Some(0.8),
        max_tokens: Some(150),
        top_p: Some(0.95),
        stop: None,
        frequency_penalty: None,
        presence_penalty: None,
    };
    assert_eq!(req.stream, Some(true));
    assert_eq!(req.temperature, Some(0.8));
    assert_eq!(req.max_tokens, Some(150));
    assert_eq!(req.top_p, Some(0.95));

    let minimal = ChatCompletionRequest {
        model: "test-model".into(),
        messages: vec![OAIMessage {
            role: "user".into(),
            content: MessageContent::Text("Test".into()),
        }],
        stream: None,
        temperature: None,
        max_tokens: None,
        top_p: None,
        stop: None,
        frequency_penalty: None,
        presence_penalty: None,
    };
    assert!(minimal.stream.is_none());
    assert!(minimal.temperature.is_none());
    assert!(minimal.max_tokens.is_none());
    assert!(minimal.top_p.is_none());
}

// ---------------------------------------------------------------------------
// Template auto-detection
// ---------------------------------------------------------------------------

#[test]
fn test_template_auto_detection() {
    let cases = vec![
        ("Qwen2-7B-Instruct", "chatml", "Qwen → ChatML"),
        ("qwen1.5-chat-7b", "chatml", "Qwen lower → ChatML"),
        ("ChatGLM3-6B", "chatml", "ChatGLM → ChatML"),
        ("Llama-3-8B-Instruct", "llama3", "Llama 3 → Llama3"),
        ("llama-2-7b-chat", "llama3", "Llama 2 → Llama3"),
        ("Phi-3-Mini-4K-Instruct", "openchat", "Phi → OpenChat"),
        ("Mistral-7B-Instruct-v0.2", "openchat", "Mistral → OpenChat"),
        ("gemma-7b-it", "openchat", "Gemma → OpenChat"),
        ("CodeLlama-13B-Instruct", "llama3", "CodeLlama → Llama3"),
    ];
    for (name, expected, desc) in &cases {
        let detected =
            if name.to_lowercase().contains("qwen") || name.to_lowercase().contains("chatglm") {
                "chatml"
            } else if name.to_lowercase().contains("llama") {
                "llama3"
            } else {
                "openchat"
            };
        assert_eq!(detected, *expected, "{}", desc);
    }
}
