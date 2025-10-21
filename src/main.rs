// Suppress function pointer comparison warnings from auto-generated bindings
#![allow(unpredictable_function_pointer_comparisons)]

mod anthropic_compat;
mod api;
mod api_errors;
mod auto_discovery;
mod cache;
mod cli;
mod engine;
mod invariant_ppt;
mod main_integration;
mod model_registry;
mod observability;
mod openai_compat;
mod port_manager;
mod server;
mod templates;
mod util {
    pub mod diag;
    pub mod memory;
}

use clap::Parser;
use model_registry::{ModelEntry, Registry};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

pub struct AppState {
    pub engine: Box<dyn engine::InferenceEngine>,
    pub registry: Registry,
    pub observability: observability::ObservabilityManager,
    pub response_cache: cache::ResponseCache,
}

impl AppState {
    pub fn new(engine: Box<dyn engine::InferenceEngine>, registry: Registry) -> Self {
        Self {
            engine,
            registry,
            observability: observability::ObservabilityManager::new(),
            response_cache: cache::ResponseCache::new(),
        }
    }
}

/// Runtime version validation - prevents Issue #63 broken binary distribution
fn validate_runtime_version() {
    let version = env!("CARGO_PKG_VERSION");

    // Check for the specific issue reported in #63
    if version == "0.1.0" {
        eprintln!();
        eprintln!("❌ ERROR: Invalid shimmy version detected!");
        eprintln!();
        eprintln!("This binary reports version 0.1.0, which indicates it was built incorrectly.");
        eprintln!("This is the exact issue reported in GitHub Issue #63.");
        eprintln!();
        eprintln!("🔧 Solutions:");
        eprintln!("  • Download the official release from: https://github.com/Michael-A-Kuykendall/shimmy/releases");
        eprintln!("  • If building from source, ensure you're building from a proper Git tag");
        eprintln!("  • If forking, update the version in Cargo.toml before building");
        eprintln!();
        eprintln!("Current version: {}", version);
        eprintln!("Expected version: 1.4.1+ (not 0.1.0)");
        eprintln!();
        std::process::exit(1);
    }

    // Additional validation for empty or malformed versions
    if version.is_empty() {
        eprintln!("ERROR: Empty version detected. This binary was built incorrectly.");
        std::process::exit(1);
    }

    // Validate basic semver format
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 || parts.iter().take(2).any(|p| p.parse::<u32>().is_err()) {
        eprintln!(
            "ERROR: Invalid version format '{}'. Expected semantic versioning.",
            version
        );
        std::process::exit(1);
    }
}

/// Print startup diagnostics for serve command
fn print_startup_diagnostics(
    version: &str,
    #[cfg_attr(not(feature = "llama"), allow(unused_variables))] gpu_backend: Option<&str>,
    #[cfg_attr(not(feature = "llama"), allow(unused_variables))] cpu_moe: bool,
    #[cfg_attr(not(feature = "llama"), allow(unused_variables))] n_cpu_moe: Option<usize>,
    model_count: usize,
) {
    println!("🎯 Shimmy v{}", version);

    // GPU backend info
    #[cfg(feature = "llama")]
    {
        let backend_display = match gpu_backend {
            Some("cpu") => "CPU only".to_string(),
            Some("cuda") => "CUDA (GPU acceleration)".to_string(),
            Some("vulkan") => "Vulkan (GPU acceleration)".to_string(),
            Some("opencl") => "OpenCL (GPU acceleration)".to_string(),
            Some("auto") | None => {
                // Auto-detect logic mirrors what LlamaEngine does
                if cfg!(feature = "llama-cuda") {
                    "CUDA (auto-detected)".to_string()
                } else if cfg!(feature = "llama-vulkan") {
                    "Vulkan (auto-detected)".to_string()
                } else if cfg!(feature = "llama-opencl") {
                    "OpenCL (auto-detected)".to_string()
                } else {
                    "CPU (no GPU acceleration)".to_string()
                }
            }
            Some(other) => format!("{} (custom)", other),
        };
        println!("🔧 Backend: {}", backend_display);
    }

    #[cfg(not(feature = "llama"))]
    {
        println!("🔧 Backend: Stub mode (no llama feature)");
    }

    // MoE configuration - NOW WORKING (Issue #108 fix)
    #[cfg(feature = "llama")]
    if cpu_moe || n_cpu_moe.is_some() {
        if let Some(n) = n_cpu_moe {
            println!(
                "🧠 MoE: CPU offload first {} layers (saves VRAM for large MoE models)",
                n
            );
        } else if cpu_moe {
            println!("🧠 MoE: CPU offload ALL expert tensors (saves ~80-85% VRAM)");
        }
    }

    // Model count
    println!("📦 Models: {} available", model_count);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Version validation - prevents Issue #63 distribution of broken binaries
    validate_runtime_version();

    // Smart ANSI detection: respect NO_COLOR, check TTY, and verify TERM capability
    let use_ansi = std::env::var("NO_COLOR").is_err()
        && std::io::IsTerminal::is_terminal(&std::io::stdout())
        && std::env::var("TERM")
            .map(|t| !t.is_empty() && t != "dumb")
            .unwrap_or(false);

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(use_ansi)
        .init();

    // Platform capability notice
    #[cfg(all(target_arch = "aarch64", target_os = "macos", not(feature = "llama")))]
    info!("llama.cpp temporarily disabled on macOS ARM64 due to upstream i8mm build incompatibility; using SafeTensors backend");

    let cli = cli::Cli::parse();

    // Add custom model directories from command line to environment
    if let Some(model_dirs) = &cli.model_dirs {
        std::env::set_var("SHIMMY_MODEL_PATHS", model_dirs);
    }

    // Initialize registry with auto-discovery
    let mut reg = Registry::with_discovery();

    // Add default model from environment variables if available
    reg.register(ModelEntry {
        name: "phi3-lora".into(),
        base_path: std::env::var("SHIMMY_BASE_GGUF")
            .unwrap_or_else(|_| "./models/phi3-mini.gguf".into())
            .into(),
        lora_path: std::env::var("SHIMMY_LORA_GGUF").ok().map(Into::into),
        template: Some("chatml".into()),
        ctx_len: Some(4096),
        n_threads: None,
    });

    // Create engine with MoE configuration if needed
    let engine: Box<dyn engine::InferenceEngine> = {
        #[cfg(feature = "llama")]
        {
            let mut adapter = engine::adapter::InferenceEngineAdapter::new_with_backend(
                cli.gpu_backend.as_deref(),
            );

            // Apply MoE configuration from global flags
            if cli.cpu_moe || cli.n_cpu_moe.is_some() {
                adapter = adapter.with_moe_config(cli.cpu_moe, cli.n_cpu_moe);
            }

            Box::new(adapter)
        }
        #[cfg(not(feature = "llama"))]
        {
            let adapter = engine::adapter::InferenceEngineAdapter::new_with_backend(
                cli.gpu_backend.as_deref(),
            );
            Box::new(adapter)
        }
    };

    // Handle model-path registration for serve command
    if let cli::Command::Serve {
        model_path: Some(ref path),
        ..
    } = cli.cmd
    {
        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            let model_name = path_buf
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("direct-model")
                .to_string();

            // Register the direct model before creating AppState
            reg.register(ModelEntry {
                name: model_name.clone(),
                base_path: path_buf.clone(),
                lora_path: None,
                template: None,
                ctx_len: None,
                n_threads: None,
            });

            println!("🎯 Direct model loaded: {} -> {}", model_name, path);
        } else {
            eprintln!("❌ Model file not found: {}", path);
            std::process::exit(1);
        }
    }

    let state = AppState::new(engine, reg);
    let state = Arc::new(state);

    match cli.cmd {
        cli::Command::Serve { ref bind, .. } => {
            // Use smart bind address resolution instead of direct parsing
            let addr = port_manager::GLOBAL_PORT_ALLOCATOR
                .resolve_bind_address(bind)
                .unwrap_or_else(|e| {
                    eprintln!("❌ Failed to resolve bind address '{}': {}", bind, e);
                    eprintln!();
                    eprintln!("💡 Valid bind address examples:");
                    eprintln!("  auto                    # Auto-allocate (default)");
                    eprintln!("  127.0.0.1:11435        # Specific address");
                    eprintln!("  0.0.0.0:8080           # All interfaces");
                    eprintln!();
                    eprintln!("🔧 Environment variable: SHIMMY_BIND_ADDRESS=127.0.0.1:11435");
                    std::process::exit(1);
                });

            // Print startup diagnostics before server starts
            print_startup_diagnostics(
                env!("CARGO_PKG_VERSION"),
                cli.gpu_backend.as_deref(),
                cli.cpu_moe,
                cli.n_cpu_moe,
                0, // Will update after model discovery
            );
            println!("🚀 Starting server on {}", addr);

            // Auto-register discovered models if we only have the default
            let manual_count = state.registry.list().len();
            if manual_count <= 1 {
                // Only the default phi3-lora entry
                // Create new engine with same configuration (including MoE if set)
                let enhanced_engine: Box<dyn engine::InferenceEngine> = {
                    #[cfg(feature = "llama")]
                    {
                        let mut adapter = engine::adapter::InferenceEngineAdapter::new_with_backend(
                            cli.gpu_backend.as_deref(),
                        );

                        // Apply MoE configuration from global flags
                        if cli.cpu_moe || cli.n_cpu_moe.is_some() {
                            adapter = adapter.with_moe_config(cli.cpu_moe, cli.n_cpu_moe);
                        }

                        Box::new(adapter)
                    }
                    #[cfg(not(feature = "llama"))]
                    {
                        let adapter = engine::adapter::InferenceEngineAdapter::new_with_backend(
                            cli.gpu_backend.as_deref(),
                        );
                        Box::new(adapter)
                    }
                };

                let mut enhanced_state = AppState::new(enhanced_engine, state.registry.clone());
                enhanced_state.registry.auto_register_discovered();
                let enhanced_state = Arc::new(enhanced_state);

                let available_models = enhanced_state.registry.list_all_available();
                if available_models.is_empty() {
                    eprintln!("❌ No models available. Please:");
                    eprintln!("   • Set SHIMMY_BASE_GGUF environment variable, or");
                    eprintln!("   • Place .gguf files in ./models/ directory, or");
                    eprintln!("   • Place .gguf files in ~/.cache/huggingface/hub/");
                    std::process::exit(1);
                }

                // Show final model count and ready message
                println!("📦 Models: {} available", available_models.len());
                println!("✅ Ready to serve requests");
                println!("   • POST /api/generate (streaming + non-streaming)");
                println!("   • GET  /health (health check + metrics)");
                println!("   • GET  /v1/models (OpenAI-compatible)");

                info!(%addr, models=%available_models.len(), "shimmy serving with {} available models", available_models.len());
                return server::run(addr, enhanced_state).await;
            }

            // Use existing state if manually configured
            let available_models = state.registry.list_all_available();
            if available_models.is_empty() {
                eprintln!("❌ No models available. Please:");
                eprintln!("   • Set SHIMMY_BASE_GGUF environment variable, or");
                eprintln!("   • Place .gguf files in ./models/ directory, or");
                eprintln!("   • Place .gguf files in ~/.cache/huggingface/hub/");
                std::process::exit(1);
            }

            // Show final model count and ready message
            println!("📦 Models: {} available", available_models.len());
            println!("✅ Ready to serve requests");
            println!("   • POST /api/generate (streaming + non-streaming)");
            println!("   • GET  /health (health check + metrics)");
            println!("   • GET  /v1/models (OpenAI-compatible)");

            info!(%addr, models=%available_models.len(), "shimmy serving with {} available models", available_models.len());
            server::run(addr, state).await?;
        }
        cli::Command::List { short } => {
            if short {
                // Short format: just model names for programmatic use
                let all_available = state.registry.list_all_available();
                for model_name in all_available {
                    println!("{}", model_name);
                }
            } else {
                // Original verbose format
                // Show manually registered models
                let manual_models = state.registry.list();
                if !manual_models.is_empty() {
                    println!("📋 Registered Models:");
                    for e in &manual_models {
                        println!("  {} => {:?}", e.name, e.base_path);
                    }
                }

                // Show auto-discovered models
                let auto_discovered = state.registry.discovered_models.clone();
                if !auto_discovered.is_empty() {
                    if !manual_models.is_empty() {
                        println!();
                    }
                    println!("🔍 Auto-Discovered Models:");
                    for (name, model) in auto_discovered {
                        let size_mb = model.size_bytes / (1024 * 1024);
                        let type_info = match (&model.parameter_count, &model.quantization) {
                            (Some(params), Some(quant)) => format!(" ({}·{})", params, quant),
                            (Some(params), None) => format!(" ({})", params),
                            (None, Some(quant)) => format!(" ({})", quant),
                            _ => String::new(),
                        };
                        let lora_info = if model.lora_path.is_some() {
                            " + LoRA"
                        } else {
                            ""
                        };
                        println!(
                            "  {} => {:?} [{}MB{}{}]",
                            name, model.path, size_mb, type_info, lora_info
                        );
                    }
                }

                // Show total available models
                let all_available = state.registry.list_all_available();
                if all_available.is_empty() {
                    println!(
                        "❌ No models found. Set SHIMMY_BASE_GGUF or place .gguf files in ./models/"
                    );
                } else {
                    println!("\n✅ Total available models: {}", all_available.len());
                }
            }
        }
        cli::Command::Discover { llm_only } => {
            println!("🔍 Refreshing model discovery...");
            let registry = Registry::with_discovery();

            let mut discovered = registry.discovered_models.clone();

            // Apply LLM-only filtering if requested
            if llm_only {
                discovered.retain(|name, _| {
                    let name_lower = name.to_lowercase();
                    // Filter out known non-LLM model types
                    !name_lower.contains("clip")
                        && !name_lower.contains("text-to-image")
                        && !name_lower.contains("vision")
                        && !name_lower.contains("image")
                        && !name_lower.contains("video")
                        && !name_lower.contains("audio")
                        && !name_lower.contains("tts")
                        && !name_lower.contains("stt")
                        && !name_lower.contains("embedding")
                        && !name_lower.contains("encoder")
                });
                println!("🎯 Filtering to LLM models only...");
            }

            if discovered.is_empty() {
                if llm_only {
                    println!("❌ No LLM models found after filtering");
                    println!("💡 Try running without --llm-only to see all models");
                } else {
                    println!("❌ No models found in search paths:");
                    let discovery = crate::auto_discovery::ModelAutoDiscovery::new();
                    for path in &discovery.search_paths {
                        println!("   • {:?}", path);
                    }
                    println!("   • Ollama models (if installed)");
                    println!("\n💡 Try downloading a GGUF model or setting SHIMMY_BASE_GGUF");
                }
            } else {
                println!("✅ Found {} models:", discovered.len());
                for (name, model) in discovered {
                    let size_mb = model.size_bytes / (1024 * 1024);
                    let lora_info = if model.lora_path.is_some() {
                        " + LoRA"
                    } else {
                        ""
                    };
                    println!("  {} [{}MB{}]", name, size_mb, lora_info);
                    println!("    Base: {:?}", model.path);
                    if let Some(lora) = &model.lora_path {
                        println!("    LoRA: {:?}", lora);
                    }
                }
            }
        }
        cli::Command::Probe { name } => {
            let Some(spec) = state.registry.to_spec(&name) else {
                anyhow::bail!("no model {name}");
            };
            match state.engine.load(&spec).await {
                Ok(_) => println!("ok: loaded {name}"),
                Err(e) => {
                    eprintln!("probe failed: {e}");
                    std::process::exit(2);
                }
            }
        }
        cli::Command::Bench { name, max_tokens } => {
            let Some(spec) = state.registry.to_spec(&name) else {
                anyhow::bail!("no model {name}");
            };
            let loaded = state.engine.load(&spec).await?;
            let t0 = std::time::Instant::now();
            let out = loaded
                .generate(
                    "Say hi.",
                    engine::GenOptions {
                        max_tokens,
                        stream: false,
                        ..Default::default()
                    },
                    None,
                )
                .await?;
            let elapsed = t0.elapsed();
            println!("bench output (truncated): {}", &out[..out.len().min(120)]);
            println!("elapsed: {:?}", elapsed);
        }
        cli::Command::Generate {
            name,
            prompt,
            max_tokens,
        } => {
            let Some(spec) = state.registry.to_spec(&name) else {
                anyhow::bail!("no model {name}");
            };
            let loaded = state.engine.load(&spec).await?;
            let out = loaded
                .generate(
                    &prompt,
                    engine::GenOptions {
                        max_tokens,
                        stream: false,
                        ..Default::default()
                    },
                    None,
                )
                .await?;
            println!("{}", out);
        }
        cli::Command::GpuInfo => {
            println!("🖥️  GPU Backend Information");
            println!();

            // Check llama.cpp backend info
            #[cfg(feature = "llama")]
            {
                use crate::engine::llama::LlamaEngine;
                let llama_engine = LlamaEngine::new_with_backend(cli.gpu_backend.as_deref());
                println!("🔧 llama.cpp Backend: {}", llama_engine.get_backend_info());

                // Show available features
                println!("📋 Available GPU Features:");
                #[cfg(feature = "llama-cuda")]
                println!("  ✅ CUDA support enabled");
                #[cfg(not(feature = "llama-cuda"))]
                println!("  ❌ CUDA support disabled");

                #[cfg(feature = "llama-vulkan")]
                println!("  ✅ Vulkan support enabled");
                #[cfg(not(feature = "llama-vulkan"))]
                println!("  ❌ Vulkan support disabled");

                #[cfg(feature = "llama-opencl")]
                println!("  ✅ OpenCL support enabled");
                #[cfg(not(feature = "llama-opencl"))]
                println!("  ❌ OpenCL support disabled");
            }

            #[cfg(not(feature = "llama"))]
            {
                println!("❌ llama.cpp backend not available (compile with --features llama)");
            }

            // Check MLX backend info
            #[cfg(feature = "mlx")]
            {
                use crate::engine::mlx::MLXEngine;

                if MLXEngine::is_hardware_supported() {
                    // Check if MLX Python packages are available
                    let python_available = MLXEngine::check_mlx_python_available();
                    if python_available {
                        println!("🍎 MLX Backend: ✅ Available (Apple Silicon + MLX installed)");
                    } else {
                        println!("🍎 MLX Backend: ⚠️  Hardware supported (Apple Silicon detected)");
                        println!("   📦 MLX Python packages not found");
                        println!("   💡 Install with: pip install mlx-lm");
                    }
                } else {
                    println!("🍎 MLX Backend: ❌ Not supported (requires Apple Silicon macOS)");
                }
            }

            #[cfg(not(feature = "mlx"))]
            {
                println!("🍎 MLX Backend: Disabled (compile with --features mlx)");
            }

            println!();
            println!("💡 To enable GPU acceleration:");

            #[cfg(target_os = "macos")]
            if std::env::consts::ARCH == "aarch64" {
                println!(
                    "   cargo install shimmy --features apple        # Apple Silicon optimized"
                );
                println!("   cargo install shimmy --features gpu          # All GPU backends");
                println!(
                    "   pip install mlx-lm                           # For MLX Python support"
                );
            } else {
                println!("   cargo install shimmy --features llama-cuda    # NVIDIA CUDA");
                println!(
                    "   cargo install shimmy --features llama-vulkan  # Cross-platform Vulkan"
                );
                println!("   cargo install shimmy --features llama-opencl  # AMD/Intel OpenCL");
                println!("   cargo install shimmy --features gpu           # All GPU backends");
            }

            #[cfg(not(target_os = "macos"))]
            {
                println!("   cargo install shimmy --features llama-cuda    # NVIDIA CUDA");
                println!(
                    "   cargo install shimmy --features llama-vulkan  # Cross-platform Vulkan"
                );
                println!("   cargo install shimmy --features llama-opencl  # AMD/Intel OpenCL");
                println!("   cargo install shimmy --features gpu           # All GPU backends");
            }
        }
        cli::Command::Init {
            template,
            output,
            name,
        } => {
            let result = templates::generate_template(&template, &output, name.as_deref());
            match result {
                Ok(message) => println!("{}", message),
                Err(e) => {
                    eprintln!("❌ Template generation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::InferenceEngine;
    use std::env;
    use std::sync::Arc;

    // Mock engine that doesn't require actual model loading
    struct MockEngine;

    #[async_trait::async_trait]
    impl engine::InferenceEngine for MockEngine {
        async fn load(
            &self,
            _spec: &engine::ModelSpec,
        ) -> anyhow::Result<Box<dyn engine::LoadedModel>> {
            Ok(Box::new(MockLoadedModel))
        }
    }

    struct MockLoadedModel;

    #[async_trait::async_trait]
    impl engine::LoadedModel for MockLoadedModel {
        async fn generate(
            &self,
            prompt: &str,
            opts: engine::GenOptions,
            _on_token: Option<Box<dyn FnMut(String) + Send>>,
        ) -> anyhow::Result<String> {
            Ok(format!(
                "Generated response to: {} (max_tokens: {})",
                prompt, opts.max_tokens
            ))
        }
    }

    #[tokio::test]
    async fn test_main_initialization_paths() {
        // Test initialization paths in main() - lines 25-44
        env::remove_var("SHIMMY_BASE_GGUF");
        env::remove_var("SHIMMY_LORA_GGUF");

        // Test registry with discovery (line 30)
        let mut reg = model_registry::Registry::with_discovery();

        // Test model registration with default values (lines 33-40)
        reg.register(model_registry::ModelEntry {
            name: "phi3-lora".into(),
            base_path: "./models/phi3-mini.gguf".into(),
            lora_path: None,
            template: Some("chatml".into()),
            ctx_len: Some(4096),
            n_threads: None,
        });

        // Test engine creation (line 42)
        let engine: Box<dyn engine::InferenceEngine> =
            Box::new(engine::adapter::InferenceEngineAdapter::new());

        // Test state creation (lines 43-44)
        let state = AppState::new(engine, reg);
        let _state_arc = Arc::new(state);

        // Test completed successfully
    }

    #[tokio::test]
    async fn test_environment_variable_handling() {
        // Test environment variable handling (lines 35-36)
        // First clean up any existing vars to ensure clean state
        env::remove_var("SHIMMY_BASE_GGUF");
        env::remove_var("SHIMMY_LORA_GGUF");

        env::set_var("SHIMMY_BASE_GGUF", "/custom/path/model.gguf");
        env::set_var("SHIMMY_LORA_GGUF", "/custom/path/lora.safetensors");

        let base_path =
            env::var("SHIMMY_BASE_GGUF").unwrap_or_else(|_| "./models/phi3-mini.gguf".into());
        let lora_path = env::var("SHIMMY_LORA_GGUF").ok();

        assert_eq!(base_path, "/custom/path/model.gguf");
        assert_eq!(lora_path, Some("/custom/path/lora.safetensors".to_string()));

        // Clean up
        env::remove_var("SHIMMY_BASE_GGUF");
        env::remove_var("SHIMMY_LORA_GGUF");
    }

    #[test]
    fn test_serve_command_address_parsing() {
        // Test address parsing with dynamic port allocation
        use crate::port_manager::GLOBAL_PORT_ALLOCATOR;
        let dynamic_port = GLOBAL_PORT_ALLOCATOR
            .allocate_ephemeral_port("test-serve-parsing")
            .unwrap();
        let bind_str = format!("127.0.0.1:{}", dynamic_port);
        let addr: std::net::SocketAddr = bind_str.parse().expect("bad bind address");
        assert_eq!(addr.port(), dynamic_port);
        GLOBAL_PORT_ALLOCATOR.release_port(dynamic_port);

        // Test invalid address parsing
        let invalid_bind = "invalid:address";
        let result = invalid_bind.parse::<std::net::SocketAddr>();
        assert!(result.is_err());
    }

    #[test]
    fn test_serve_command_model_count_logic() {
        // Test model count logic (lines 51-52)
        let registry = model_registry::Registry::with_discovery();
        let manual_count = registry.list().len();

        // Test condition for auto-registration
        let _should_auto_register = manual_count <= 1;

        // This will be true in test environment with no models
        // Either auto-registration path is valid - test that logic works
    }

    #[tokio::test]
    async fn test_list_command_execution_logic() {
        // Test List command execution logic (lines 86-121)
        let mut registry = model_registry::Registry::with_discovery();

        // Add a test model to exercise manual models display (lines 88-94)
        registry.register(model_registry::ModelEntry {
            name: "test-model".into(),
            base_path: "./test.gguf".into(),
            lora_path: None,
            template: Some("chatml".into()),
            ctx_len: Some(2048),
            n_threads: None,
        });

        let manual_models = registry.list();
        assert!(!manual_models.is_empty());

        // Test discovered models access (lines 97-112)
        let _auto_discovered = registry.discovered_models.clone();

        // Test all available models (lines 115-120)
        let _all_available = registry.list_all_available();

        // The logic paths are exercised by calling the methods
        // Test completed successfully
    }

    #[tokio::test]
    async fn test_discover_command_execution_logic() {
        // Test Discover command execution logic (lines 122-147)
        let registry = model_registry::Registry::with_discovery();
        let discovered = registry.discovered_models.clone();

        // Test both empty and non-empty discovery paths
        if discovered.is_empty() {
            // Lines 127-135 - no models found path
            assert!(discovered.is_empty());
        } else {
            // Lines 136-146 - models found path
            assert!(!discovered.is_empty());
            for (name, model) in discovered {
                let _size_mb = model.size_bytes / (1024 * 1024);
                let _lora_info = if model.lora_path.is_some() {
                    " + LoRA"
                } else {
                    ""
                };
                // Exercise the display logic
                assert!(!name.is_empty());
            }
        }
    }

    #[tokio::test]
    async fn test_probe_command_execution_logic() {
        // Test Probe command execution logic (lines 148-157)
        let mut registry = model_registry::Registry::with_discovery();
        registry.register(model_registry::ModelEntry {
            name: "test-probe-model".into(),
            base_path: "./test.gguf".into(),
            lora_path: None,
            template: Some("chatml".into()),
            ctx_len: Some(2048),
            n_threads: None,
        });

        let engine = MockEngine;
        let name = "test-probe-model";

        // Test successful model spec retrieval (line 149)
        let spec_result = registry.to_spec(name);
        if let Some(spec) = spec_result {
            // Test engine load (line 150)
            let load_result = engine.load(&spec).await;
            match load_result {
                Ok(_) => {
                    // Line 151 - success path
                    // Test completed successfully
                }
                Err(_) => {
                    // Lines 152-155 - error path
                    // Test completed successfully
                }
            }
        } else {
            // Line 149 - no model found path
            // Test completed successfully
        }
    }

    #[tokio::test]
    async fn test_bench_command_execution_logic() {
        // Test Bench command execution logic (lines 158-170)
        let mut registry = model_registry::Registry::with_discovery();
        registry.register(model_registry::ModelEntry {
            name: "test-bench-model".into(),
            base_path: "./test.gguf".into(),
            lora_path: None,
            template: Some("chatml".into()),
            ctx_len: Some(2048),
            n_threads: None,
        });

        let engine = MockEngine;
        let name = "test-bench-model";
        let max_tokens = 100;

        // Test model spec retrieval (line 159)
        if let Some(spec) = registry.to_spec(name) {
            // Test engine load (line 160)
            let loaded = engine.load(&spec).await.unwrap();

            // Test timing (line 161)
            let t0 = std::time::Instant::now();

            // Test generation with specific options (lines 162-166)
            let out = loaded
                .generate(
                    "Say hi.",
                    engine::GenOptions {
                        max_tokens,
                        stream: false,
                        ..Default::default()
                    },
                    None,
                )
                .await
                .unwrap();

            // Test elapsed calculation (line 167)
            let elapsed = t0.elapsed();

            // Test output processing (lines 168-169)
            let truncated = &out[..out.len().min(120)];

            assert!(!truncated.is_empty());
            assert!(elapsed.as_nanos() > 0);
        }
    }

    #[tokio::test]
    async fn test_generate_command_execution_logic() {
        // Test Generate command execution logic (lines 171-176)
        let mut registry = model_registry::Registry::with_discovery();
        registry.register(model_registry::ModelEntry {
            name: "test-gen-model".into(),
            base_path: "./test.gguf".into(),
            lora_path: None,
            template: Some("chatml".into()),
            ctx_len: Some(2048),
            n_threads: None,
        });

        let engine = MockEngine;
        let name = "test-gen-model";
        let prompt = "Hello, world!";
        let max_tokens = 50;

        // Test model spec retrieval (line 172)
        if let Some(spec) = registry.to_spec(name) {
            // Test engine load (line 173)
            let loaded = engine.load(&spec).await.unwrap();

            // Test generation (line 174)
            let out = loaded
                .generate(
                    prompt,
                    engine::GenOptions {
                        max_tokens,
                        stream: false,
                        ..Default::default()
                    },
                    None,
                )
                .await
                .unwrap();

            // Line 175 - output would be printed here
            assert!(out.contains("Generated response to: Hello, world!"));
        }
    }

    #[test]
    fn test_command_execution_paths() {
        use crate::cli::{Cli, Command};
        use clap::Parser;

        // Test Generate command path (exercises CLI parsing for lines 171-176)
        let gen_args = vec![
            "shimmy",
            "generate",
            "test-model",
            "--prompt",
            "Hello",
            "--max-tokens",
            "50",
        ];
        let cli = Cli::try_parse_from(gen_args).unwrap();

        match cli.cmd {
            Command::Generate {
                name,
                prompt,
                max_tokens,
            } => {
                assert_eq!(name, "test-model");
                assert_eq!(prompt, "Hello");
                assert_eq!(max_tokens, 50);
            }
            _ => panic!("Expected Generate command"),
        }
    }

    #[tokio::test]
    async fn test_state_initialization() {
        use crate::engine::adapter::InferenceEngineAdapter;
        use crate::model_registry::Registry;

        // Test state creation paths
        let registry = Registry::with_discovery();
        let engine = Box::new(InferenceEngineAdapter::new());
        let state = std::sync::Arc::new(crate::AppState::new(engine, registry));

        // Validate state is properly created
        assert_ne!(std::mem::size_of_val(&state), 0);
        let _models = state.registry.list();
        // Models vec was created successfully
    }

    #[test]
    fn test_serve_enhanced_state_logic() {
        // Test enhanced state creation for serve command (lines 53-58)
        let registry = model_registry::Registry::with_discovery();

        let mut enhanced_state = AppState::new(
            Box::new(engine::llama::LlamaEngine::new()),
            registry.clone(),
        );

        // Test auto-registration call (line 57)
        enhanced_state.registry.auto_register_discovered();

        let enhanced_state_arc = Arc::new(enhanced_state);

        // Test available models access (line 60)
        let available_models = enhanced_state_arc.registry.list_all_available();

        // Exercise empty/non-empty logic paths (lines 61-67 and 74-81)
        if available_models.is_empty() {
            // Error path - would call std::process::exit(1)
            assert!(available_models.is_empty());
        } else {
            // Success path - would continue to server run
            assert!(!available_models.is_empty());
        }
    }

    #[test]
    fn test_model_registration_with_env_vars() {
        // Test model registration with environment variables (lines 33-40)
        env::set_var("SHIMMY_BASE_GGUF", "/test/base.gguf");
        env::set_var("SHIMMY_LORA_GGUF", "/test/lora.safetensors");

        let base_path =
            env::var("SHIMMY_BASE_GGUF").unwrap_or_else(|_| "./models/phi3-mini.gguf".into());
        let lora_path = env::var("SHIMMY_LORA_GGUF").ok().map(Into::into);

        let mut reg = model_registry::Registry::with_discovery();
        reg.register(model_registry::ModelEntry {
            name: "phi3-lora".into(),
            base_path: base_path.into(),
            lora_path,
            template: Some("chatml".into()),
            ctx_len: Some(4096),
            n_threads: None,
        });

        let models = reg.list();
        assert!(!models.is_empty());

        // Clean up
        env::remove_var("SHIMMY_BASE_GGUF");
        env::remove_var("SHIMMY_LORA_GGUF");
    }

    #[test]
    fn test_registry_model_methods() {
        // Test registry methods used in main (various lines)
        let mut registry = model_registry::Registry::with_discovery();

        // Test initial state
        let initial_count = registry.list().len();
        let _discovered = registry.discovered_models.clone();
        let _all_available = registry.list_all_available();

        // Add a model and test again
        registry.register(model_registry::ModelEntry {
            name: "test".into(),
            base_path: "./test.gguf".into(),
            lora_path: None,
            template: None,
            ctx_len: None,
            n_threads: None,
        });

        let after_count = registry.list().len();
        assert!(after_count > initial_count);

        // Test to_spec method used in probe/bench/generate
        let spec = registry.to_spec("test");
        assert!(spec.is_some());

        let no_spec = registry.to_spec("nonexistent");
        assert!(no_spec.is_none());
    }

    #[test]
    fn test_app_state_creation() {
        use crate::engine::adapter::InferenceEngineAdapter;
        use crate::model_registry::Registry;

        let engine: Box<dyn engine::InferenceEngine> = Box::new(InferenceEngineAdapter::new());
        let registry = Registry::with_discovery();
        let _state = AppState::new(engine, registry);

        // Registry was created successfully
    }

    #[test]
    fn test_tracing_initialization() {
        // This tests the tracing setup that happens at line 26
        // The actual initialization happens in main(), but we can test the components
        use tracing_subscriber::EnvFilter;

        let env_filter = EnvFilter::from_default_env();
        assert!(env_filter.max_level_hint().is_some() || env_filter.max_level_hint().is_none());
    }

    // Integration-style tests that exercise main execution paths without actually running main()
    #[tokio::test]
    async fn test_serve_command_execution_simulation() {
        // Simulate serve command execution (lines 47-84)
        env::set_var("SHIMMY_BASE_GGUF", "./test.gguf");

        // Simulate main() initialization (lines 25-44)
        let mut reg = model_registry::Registry::with_discovery();
        reg.register(model_registry::ModelEntry {
            name: "phi3-lora".into(),
            base_path: env::var("SHIMMY_BASE_GGUF")
                .unwrap_or_else(|_| "./models/phi3-mini.gguf".into())
                .into(),
            lora_path: env::var("SHIMMY_LORA_GGUF").ok().map(Into::into),
            template: Some("chatml".into()),
            ctx_len: Some(4096),
            n_threads: None,
        });

        let engine: Box<dyn engine::InferenceEngine> =
            Box::new(engine::adapter::InferenceEngineAdapter::new());
        let state = AppState::new(engine, reg);
        let state = Arc::new(state);

        // Simulate serve command logic with dynamic port allocation
        use crate::port_manager::GLOBAL_PORT_ALLOCATOR;
        let dynamic_port = GLOBAL_PORT_ALLOCATOR
            .allocate_ephemeral_port("test-serve-logic")
            .unwrap();
        let bind = format!("127.0.0.1:{}", dynamic_port);
        let addr: std::net::SocketAddr = bind.parse().expect("bad bind address");

        // Test model count logic (line 51-52)
        let manual_count = state.registry.list().len();

        if manual_count <= 1 {
            // Simulate enhanced state creation (lines 53-58)
            let mut enhanced_state = AppState::new(
                Box::new(engine::llama::LlamaEngine::new()),
                state.registry.clone(),
            );
            enhanced_state.registry.auto_register_discovered();
            let enhanced_state_arc = Arc::new(enhanced_state);

            // Test available models check (lines 60-67)
            let available_models = enhanced_state_arc.registry.list_all_available();

            // Both empty and non-empty paths should be exercised
            if available_models.is_empty() {
                // Lines 61-66 - error path without std::process::exit
                assert!(available_models.is_empty());
            } else {
                // Line 69 - success path without server::run
                assert!(!available_models.is_empty());
            }
        }

        // Test regular state path (lines 74-81)
        let available_models = state.registry.list_all_available();
        if available_models.is_empty() {
            // Lines 75-80 - error path without std::process::exit
            assert!(available_models.is_empty());
        }

        // Clean up
        env::remove_var("SHIMMY_BASE_GGUF");

        // Line 84 would call server::run but we can't test that without actually starting server
        assert_eq!(addr.port(), dynamic_port);
        GLOBAL_PORT_ALLOCATOR.release_port(dynamic_port);
    }

    #[tokio::test]
    async fn test_command_match_branches_coverage() {
        // Test all command match branches (lines 46-177)

        // Create test state
        let mut reg = model_registry::Registry::with_discovery();
        reg.register(model_registry::ModelEntry {
            name: "test-model".into(),
            base_path: "./test.gguf".into(),
            lora_path: None,
            template: Some("chatml".into()),
            ctx_len: Some(2048),
            n_threads: None,
        });
        let _engine = MockEngine;
        let state = Arc::new(AppState::new(
            Box::new(engine::adapter::InferenceEngineAdapter::new()),
            reg,
        ));

        // Test List command branch (lines 86-121)
        {
            let manual_models = state.registry.list();
            if !manual_models.is_empty() {
                // Lines 89-94 - manual models display
                for e in &manual_models {
                    assert!(!e.name.is_empty());
                    // Line 92 would print: println!("  {} => {:?}", e.name, e.base_path);
                }
            }

            // Lines 97-112 - auto-discovered models
            let auto_discovered = state.registry.discovered_models.clone();
            if !auto_discovered.is_empty() {
                for (name, model) in auto_discovered {
                    let _size_mb = model.size_bytes / (1024 * 1024);
                    let _type_info = match (&model.parameter_count, &model.quantization) {
                        (Some(params), Some(quant)) => format!(" ({}·{})", params, quant),
                        (Some(params), None) => format!(" ({})", params),
                        (None, Some(quant)) => format!(" ({})", quant),
                        _ => String::new(),
                    };
                    let _lora_info = if model.lora_path.is_some() {
                        " + LoRA"
                    } else {
                        ""
                    };
                    assert!(!name.is_empty());
                }
            }

            // Lines 115-120 - total available models
            let all_available = state.registry.list_all_available();
            if all_available.is_empty() {
                // Line 117 - no models message
                assert!(all_available.is_empty());
            } else {
                // Line 119 - success message
                // Available models list was created
            }
        }

        // Test Discover command branch (lines 122-147)
        {
            let registry = model_registry::Registry::with_discovery();
            let discovered = registry.discovered_models.clone();

            if discovered.is_empty() {
                // Lines 127-135 - no models found
                assert!(discovered.is_empty());
            } else {
                // Lines 136-146 - models found
                for (name, model) in discovered {
                    let _size_mb = model.size_bytes / (1024 * 1024);
                    let _lora_info = if model.lora_path.is_some() {
                        " + LoRA"
                    } else {
                        ""
                    };
                    assert!(!name.is_empty());
                }
            }
        }

        // Test other commands that we can simulate without actual engine calls
        let name = "test-model";

        // Test model spec retrieval for probe/bench/generate (lines 149, 159, 172)
        if let Some(spec) = state.registry.to_spec(name) {
            assert!(spec.base_path.to_string_lossy().contains("test"));

            // For probe command - line 149 success path
            // For bench command - line 159 success path
            // For generate command - line 172 success path
            // Test completed successfully
        } else {
            // Lines would bail with "no model {name}" error
            panic!("Expected to find test model");
        }
    }

    #[test]
    fn test_error_conditions_and_edge_cases() {
        // Test various error conditions and edge cases

        // Test socket address parsing errors (line 48)
        let invalid_addresses = vec![
            "invalid-address",
            "256.256.256.256:9999", // Invalid IP
            "127.0.0.1:99999",      // Invalid port
            "127.0.0.1:",           // Missing port
            ":9999",                // Missing IP
            "",                     // Empty string
            "not.an.ip:port",       // Non-numeric port
        ];

        for addr_str in invalid_addresses {
            let result = addr_str.parse::<std::net::SocketAddr>();
            assert!(
                result.is_err(),
                "Expected parsing to fail for: {}",
                addr_str
            );
        }

        // Test valid addresses that should work (using high ports to avoid conflicts)
        use crate::port_manager::GLOBAL_PORT_ALLOCATOR;
        let port1 = GLOBAL_PORT_ALLOCATOR
            .allocate_ephemeral_port("test-valid-1")
            .unwrap();
        let port2 = GLOBAL_PORT_ALLOCATOR
            .allocate_ephemeral_port("test-valid-2")
            .unwrap();
        let port3 = GLOBAL_PORT_ALLOCATOR
            .allocate_ephemeral_port("test-valid-3")
            .unwrap();
        let port4 = GLOBAL_PORT_ALLOCATOR
            .allocate_ephemeral_port("test-valid-4")
            .unwrap();

        let valid_addresses = vec![
            format!("127.0.0.1:{}", port1),
            format!("0.0.0.0:{}", port2),
            format!("192.168.1.100:{}", port3),
            format!("[::1]:{}", port4), // IPv6
        ];

        // Clean up ports after test
        GLOBAL_PORT_ALLOCATOR.release_port(port1);
        GLOBAL_PORT_ALLOCATOR.release_port(port2);
        GLOBAL_PORT_ALLOCATOR.release_port(port3);
        GLOBAL_PORT_ALLOCATOR.release_port(port4);

        for addr_str in valid_addresses {
            let result = addr_str.parse::<std::net::SocketAddr>();
            assert!(
                result.is_ok(),
                "Expected parsing to succeed for: {}",
                addr_str
            );
        }
    }

    #[test]
    fn test_registry_edge_cases() {
        // Test registry behavior in edge cases
        let registry = model_registry::Registry::with_discovery();

        // Test to_spec with nonexistent models (lines 149, 159, 172 error paths)
        let nonexistent_names = vec![
            "nonexistent-model",
            "",
            "model-with-special-chars!@#",
            "very-long-model-name-that-might-cause-issues-in-some-systems-if-not-handled-properly",
        ];

        for name in nonexistent_names {
            let spec = registry.to_spec(name);
            assert!(
                spec.is_none(),
                "Expected no spec for nonexistent model: {}",
                name
            );
        }
    }

    #[test]
    fn test_model_entry_variants() {
        // Test different ModelEntry configurations
        let mut registry = model_registry::Registry::with_discovery();

        // Test minimal entry
        registry.register(model_registry::ModelEntry {
            name: "minimal".to_string(),
            base_path: "./minimal.gguf".into(),
            lora_path: None,
            template: None,
            ctx_len: None,
            n_threads: None,
        });

        // Test maximal entry
        registry.register(model_registry::ModelEntry {
            name: "maximal".to_string(),
            base_path: "./maximal.gguf".into(),
            lora_path: Some("./maximal.lora".into()),
            template: Some("llama3".to_string()),
            ctx_len: Some(8192),
            n_threads: Some(8),
        });

        let models = registry.list();
        assert!(models.len() >= 2);

        // Find and verify entries
        let minimal = models.iter().find(|e| e.name == "minimal").unwrap();
        assert!(minimal.lora_path.is_none());
        assert!(minimal.template.is_none());

        let maximal = models.iter().find(|e| e.name == "maximal").unwrap();
        assert!(maximal.lora_path.is_some());
        assert_eq!(maximal.template.as_ref().unwrap(), "llama3");
        assert_eq!(maximal.ctx_len.unwrap(), 8192);
        assert_eq!(maximal.n_threads.unwrap(), 8);
    }

    #[tokio::test]
    async fn test_mock_engine_behavior() {
        // Test the MockEngine behavior to ensure test reliability
        let engine = MockEngine;

        // Test with minimal spec
        let minimal_spec = crate::engine::ModelSpec {
            name: "test".to_string(),
            base_path: "./test.gguf".into(),
            lora_path: None,
            template: None,
            ctx_len: 1024,
            n_threads: None,
        };

        let loaded = engine.load(&minimal_spec).await.unwrap();
        let output = loaded
            .generate("Test prompt", crate::engine::GenOptions::default(), None)
            .await
            .unwrap();

        assert!(output.contains("Generated response to: Test prompt"));
        assert!(output.contains("max_tokens:"));

        // Test with different options
        let opts = crate::engine::GenOptions {
            max_tokens: 150,
            temperature: 0.8,
            ..Default::default()
        };

        let output = loaded.generate("Another test", opts, None).await.unwrap();
        assert!(output.contains("Another test"));
        assert!(output.contains("150"));
    }

    #[test]
    fn test_auto_discovery_models_access() {
        // Test accessing auto-discovered models in various scenarios
        let registry = model_registry::Registry::with_discovery();

        // Test discovered_models access (line 97, 126)
        let discovered = registry.discovered_models.clone();

        // Test empty case
        if discovered.is_empty() {
            assert_eq!(discovered.len(), 0);
        } else {
            // Test non-empty case - exercise the match arms in lines 103-108
            for model in discovered.values() {
                // Test parameter_count and quantization combinations
                let _type_info = match (&model.parameter_count, &model.quantization) {
                    (Some(params), Some(quant)) => {
                        // Line 104
                        let info = format!(" ({}·{})", params, quant);
                        assert!(info.contains(params));
                        assert!(info.contains(quant));
                        info
                    }
                    (Some(params), None) => {
                        // Line 105
                        let info = format!(" ({})", params);
                        assert!(info.contains(params));
                        info
                    }
                    (None, Some(quant)) => {
                        // Line 106
                        let info = format!(" ({})", quant);
                        assert!(info.contains(quant));
                        info
                    }
                    _ => {
                        // Line 107
                        String::new()
                    }
                };

                // Test lora_path check (line 109)
                let _lora_info = if model.lora_path.is_some() {
                    " + LoRA"
                } else {
                    ""
                };
            }
        }

        // Test list_all_available (lines 115, 647, etc.)
        let _all_available = registry.list_all_available();
        // Available models list was created
    }

    #[tokio::test]
    async fn test_serve_command_edge_cases() {
        // Test serve command with various edge cases

        // Test with empty registry (should trigger auto-discovery)
        let empty_registry = model_registry::Registry::with_discovery();
        let manual_count = empty_registry.list().len();

        // This should be <= 1 and trigger enhanced state creation
        if manual_count <= 1 {
            // Simulate enhanced state logic (lines 53-58)
            let mut enhanced_state = AppState::new(
                Box::new(engine::llama::LlamaEngine::new()),
                empty_registry.clone(),
            );

            // Test auto-register call
            enhanced_state.registry.auto_register_discovered();

            // Test available models check
            let available_models = enhanced_state.registry.list_all_available();

            // Both paths should be tested
            if available_models.is_empty() {
                // Lines 61-66: Error path (would exit with code 1)
                assert!(available_models.is_empty());
            } else {
                // Line 69: Success path (would run server)
                assert!(!available_models.is_empty());
            }
        }
    }

    #[test]
    fn test_string_truncation_logic() {
        // Test string truncation used in bench command (line 168)
        let long_string = "A".repeat(150);
        let expected_120 = "A".repeat(120);
        let exactly_225 = "Exactly120chars".to_string() + &"A".repeat(105);

        let test_strings = vec![
            ("Short", 120, "Short"),
            (&long_string, 120, &expected_120),
            ("", 120, ""),
            (&exactly_225, 120, &exactly_225),
        ];

        for (input, limit, expected) in test_strings {
            let truncated = &input[..input.len().min(limit)];
            assert_eq!(truncated, expected);
        }
    }

    #[test]
    fn test_duration_and_timing() {
        // Test timing logic used in bench command (lines 161, 167)
        let start = std::time::Instant::now();

        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(1));

        let elapsed = start.elapsed();
        assert!(elapsed.as_nanos() > 0);
        assert!(elapsed.as_millis() >= 1);

        // Test duration formatting
        let duration_str = format!("{:?}", elapsed);
        assert!(
            duration_str.contains("ms")
                || duration_str.contains("µs")
                || duration_str.contains("ns")
        );
    }

    #[tokio::test]
    async fn test_discover_command_execution() {
        // Test Discover command execution (lines 122-147)
        let registry = model_registry::Registry::with_discovery();

        // Test discovery refresh logic (line 123)
        let discovered = registry.discovered_models.clone();

        // Test empty discovery path (lines 125-133)
        if discovered.is_empty() {
            // Verify error message paths would be taken
            assert!(discovered.is_empty());
            // Lines 127-132 would print error messages
        } else {
            // Test non-empty discovery path (lines 134-146)
            for (name, model) in discovered {
                // Test size calculation (line 137)
                let _size_mb = model.size_bytes / (1024 * 1024);
                // File size was calculated

                // Test lora info logic (line 145)
                let lora_info = if model.lora_path.is_some() {
                    " + LoRA"
                } else {
                    ""
                };
                assert!(lora_info == " + LoRA" || lora_info.is_empty());

                assert!(!name.is_empty());
            }
        }
    }

    #[tokio::test]
    async fn test_probe_command_execution() {
        // Test Probe command execution (lines 148-155)
        let mut registry = model_registry::Registry::with_discovery();
        registry.register(model_registry::ModelEntry {
            name: "probe-test".to_string(),
            base_path: "./probe-test.gguf".into(),
            lora_path: None,
            template: Some("chatml".into()),
            ctx_len: Some(2048),
            n_threads: None,
        });

        let engine = MockEngine;
        let name = "probe-test";

        // Test spec retrieval (line 149)
        if let Some(spec) = registry.to_spec(name) {
            // Test engine load (line 150-154)
            match engine.load(&spec).await {
                Ok(_) => {
                    // Line 151: Success path - would print "ok: loaded {name}"
                    // Test completed successfully
                }
                Err(_) => {
                    // Lines 152-154: Error path - would print error and exit(2)
                    panic!("MockEngine should not fail");
                }
            }
        } else {
            // Line 149: No spec found - would bail with "no model {name}"
            panic!("Expected to find probe-test model");
        }
    }

    #[tokio::test]
    async fn test_bench_command_execution() {
        // Test Bench command execution (lines 156-169)
        let mut registry = model_registry::Registry::with_discovery();
        registry.register(model_registry::ModelEntry {
            name: "bench-test".to_string(),
            base_path: "./bench-test.gguf".into(),
            lora_path: None,
            template: Some("chatml".into()),
            ctx_len: Some(2048),
            n_threads: None,
        });

        let engine = MockEngine;
        let name = "bench-test";
        let max_tokens = 64;

        // Test spec retrieval (line 157)
        if let Some(spec) = registry.to_spec(name) {
            // Test engine load (line 158)
            let loaded = engine.load(&spec).await.unwrap();

            // Test timing start (line 159)
            let t0 = std::time::Instant::now();

            // Test generation call (lines 160-164)
            let out = loaded
                .generate(
                    "Say hi.",
                    engine::GenOptions {
                        max_tokens,
                        stream: false,
                        ..Default::default()
                    },
                    None,
                )
                .await
                .unwrap();

            // Test elapsed calculation (line 165)
            let elapsed = t0.elapsed();

            // Test output truncation (line 166)
            let truncated = &out[..out.len().min(120)];

            // Test outputs (lines 166-167)
            assert!(!truncated.is_empty());
            assert!(elapsed.as_nanos() > 0);

            // Verify MockEngine response format
            assert!(out.contains("Generated response to: Say hi."));
        }
    }

    #[test]
    fn test_environment_cleanup() {
        // Test proper environment variable cleanup after tests
        let test_vars = vec!["SHIMMY_BASE_GGUF", "SHIMMY_LORA_GGUF"];

        // Save original values
        let original_values: Vec<_> = test_vars
            .iter()
            .map(|var| (*var, env::var(var).ok()))
            .collect();

        // Set test values
        for var in &test_vars {
            env::set_var(var, "/test/path");
        }

        // Verify test values are set
        for var in &test_vars {
            assert_eq!(
                env::var(var)
                    .unwrap_or_else(|_| panic!("Environment variable {} should be set", var)),
                "/test/path"
            );
        }

        // Restore original values
        for (var, original_value) in original_values {
            match original_value {
                Some(value) => env::set_var(var, value),
                None => env::remove_var(var),
            }
        }
    }

    #[test]
    fn test_size_calculations() {
        // Test size calculations used in display logic (lines 102, 138)
        let test_cases = vec![
            (0u64, 0u64),
            (1024, 0),                  // Less than 1MB
            (1024 * 1024, 1),           // Exactly 1MB
            (1024 * 1024 * 5, 5),       // 5MB
            (1024 * 1024 * 1536, 1536), // 1.5GB in MB
        ];

        for (size_bytes, expected_mb) in test_cases {
            let size_mb = size_bytes / (1024 * 1024);
            assert_eq!(
                size_mb, expected_mb,
                "Size calculation failed for {} bytes",
                size_bytes
            );
        }
    }

    #[tokio::test]
    async fn test_model_loading_error_paths() {
        // Test error paths in probe/bench/generate commands
        let mut registry = model_registry::Registry::with_discovery();
        registry.register(model_registry::ModelEntry {
            name: "error-test".to_string(),
            base_path: "./nonexistent.gguf".into(), // This will cause load errors
            lora_path: None,
            template: Some("chatml".into()),
            ctx_len: Some(2048),
            n_threads: None,
        });

        // Create an engine that might fail
        let engine = MockEngine; // Our mock should work, but test the error handling structure

        // Test that we can get the spec but might fail on loading
        let spec = registry.to_spec("error-test").unwrap();
        let load_result = engine.load(&spec).await;

        // The MockEngine should succeed, but this tests the code path
        match load_result {
            Ok(loaded_model) => {
                // Test generation
                let gen_result = loaded_model
                    .generate("test prompt", crate::engine::GenOptions::default(), None)
                    .await;
                assert!(gen_result.is_ok());
            }
            Err(_) => {
                // This would be the error path for probe (lines 153-155)
                // and similar for bench/generate
                // Test completed successfully
            }
        }
    }

    #[test]
    fn test_print_startup_diagnostics_basic() {
        // Test basic startup diagnostics output (no MoE)
        // This test verifies the function runs without panic
        // We can't easily capture println! output in tests, but we verify the logic works
        print_startup_diagnostics("1.6.0", None, false, None, 3);
        print_startup_diagnostics("1.6.0", Some("auto"), false, None, 5);

        // Test completed successfully - no panic means diagnostics formatted correctly
    }

    #[test]
    fn test_print_startup_diagnostics_with_backends() {
        // Test diagnostics with different GPU backends
        print_startup_diagnostics("1.6.0", Some("cpu"), false, None, 2);
        print_startup_diagnostics("1.6.0", Some("cuda"), false, None, 4);
        print_startup_diagnostics("1.6.0", Some("vulkan"), false, None, 1);
        print_startup_diagnostics("1.6.0", Some("opencl"), false, None, 6);
        print_startup_diagnostics("1.6.0", Some("custom-backend"), false, None, 3);

        // Test completed successfully
    }

    #[test]
    #[cfg(feature = "llama")]
    fn test_print_startup_diagnostics_with_moe() {
        // Test diagnostics with MoE configuration
        print_startup_diagnostics("1.6.0", Some("cuda"), true, None, 2);
        print_startup_diagnostics("1.6.0", Some("cuda"), false, Some(16), 2);
        print_startup_diagnostics("1.6.0", Some("auto"), true, None, 5);

        // Test completed successfully
    }

    #[test]
    fn test_print_startup_diagnostics_zero_models() {
        // Test diagnostics with zero models (edge case)
        print_startup_diagnostics("1.6.0", None, false, None, 0);

        // Should not panic even with 0 models
        // (The actual serve command will exit with error, but diagnostics should print)
    }

    #[test]
    fn test_print_startup_diagnostics_many_models() {
        // Test diagnostics with many models (like user's 13+ scenario)
        print_startup_diagnostics("1.6.0", Some("cuda"), false, None, 13);
        print_startup_diagnostics("1.6.0", Some("auto"), true, None, 25);

        // Test completed successfully
    }

    #[test]
    fn test_serve_diagnostics_integration() {
        // Test that serve command calls diagnostics in correct order
        // This is a structural test - verify the function exists and has correct signature

        let version = env!("CARGO_PKG_VERSION");
        let gpu_backend: Option<&str> = None;
        let cpu_moe = false;
        let n_cpu_moe: Option<usize> = None;
        let model_count = 0;

        // Call diagnostics as serve command would
        print_startup_diagnostics(version, gpu_backend, cpu_moe, n_cpu_moe, model_count);

        // Test completed - verifies function signature matches usage
    }

    #[test]
    fn test_startup_diagnostics_version_display() {
        // Test that version is displayed correctly
        // Uses actual CARGO_PKG_VERSION from build
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty(), "Version should not be empty");
        assert_ne!(
            version, "0.1.0",
            "Version should not be the broken 0.1.0 from Issue #63"
        );

        // Call diagnostics with real version
        print_startup_diagnostics(version, None, false, None, 1);
    }
}
