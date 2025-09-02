mod api;
mod api_errors;
mod auto_discovery;
mod cli;
mod engine;
mod main_integration;
mod model_registry;
mod openai_compat;
mod server;
mod templates;
mod util { pub mod diag; }

use clap::Parser;
use model_registry::{ModelEntry, Registry};
use std::net::SocketAddr;
use tracing::info;
use std::sync::Arc;

pub struct AppState {
    pub engine: Box<dyn engine::InferenceEngine>,
    pub registry: Registry,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    let cli = cli::Cli::parse();

    // Initialize registry with auto-discovery
    let mut reg = Registry::with_discovery();
    
    // Add default model from environment variables if available
    reg.register(ModelEntry {
        name: "phi3-lora".into(),
        base_path: std::env::var("SHIMMY_BASE_GGUF").unwrap_or_else(|_| "./models/phi3-mini.gguf".into()).into(),
        lora_path: std::env::var("SHIMMY_LORA_GGUF").ok().map(Into::into),
        template: Some("chatml".into()),
        ctx_len: Some(4096),
        n_threads: None,
    });

    let engine: Box<dyn engine::InferenceEngine> = Box::new(engine::llama::LlamaEngine::new());
    let state = AppState { engine, registry: reg };
    let state = Arc::new(state);

    match cli.cmd {
        cli::Command::Serve { bind } => {
            let addr: SocketAddr = bind.parse().expect("bad --bind address");
            
            // Auto-register discovered models if we only have the default
            let manual_count = state.registry.list().len();
            if manual_count <= 1 { // Only the default phi3-lora entry
                let mut enhanced_state = AppState {
                    engine: Box::new(engine::llama::LlamaEngine::new()),
                    registry: state.registry.clone(),
                };
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
            
            info!(%addr, models=%available_models.len(), "shimmy serving with {} available models", available_models.len());
            server::run(addr, state).await?;
        }
        cli::Command::List => {
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
                if !manual_models.is_empty() { println!(); }
                println!("🔍 Auto-Discovered Models:");
                for (name, model) in auto_discovered {
                    let size_mb = model.size_bytes / (1024 * 1024);
                    let type_info = match (&model.parameter_count, &model.quantization) {
                        (Some(params), Some(quant)) => format!(" ({}·{})", params, quant),
                        (Some(params), None) => format!(" ({})", params),
                        (None, Some(quant)) => format!(" ({})", quant),
                        _ => String::new(),
                    };
                    let lora_info = if model.lora_path.is_some() { " + LoRA" } else { "" };
                    println!("  {} => {:?} [{}MB{}{}]", name, model.path, size_mb, type_info, lora_info);
                }
            }
            
            // Show total available models 
            let all_available = state.registry.list_all_available();
            if all_available.is_empty() {
                println!("❌ No models found. Set SHIMMY_BASE_GGUF or place .gguf files in ./models/");
            } else {
                println!("\n✅ Total available models: {}", all_available.len());
            }
        }
        cli::Command::Discover => {
            println!("🔍 Refreshing model discovery...");
            let registry = Registry::with_discovery();
            
            let discovered = registry.discovered_models.clone();
            if discovered.is_empty() {
                println!("❌ No models found in search paths:");
                println!("   • ./models/");
                println!("   • ~/.cache/huggingface/hub/");
                println!("   • ~/models/");
                println!("   • ~/Downloads/");
                println!("   • Current directory");
                println!("\n💡 Try downloading a GGUF model or setting SHIMMY_BASE_GGUF");
            } else {
                println!("✅ Found {} models:", discovered.len());
                for (name, model) in discovered {
                    let size_mb = model.size_bytes / (1024 * 1024);
                    let lora_info = if model.lora_path.is_some() { " + LoRA" } else { "" };
                    println!("  {} [{}MB{}]", name, size_mb, lora_info);
                    println!("    Base: {:?}", model.path);
                    if let Some(lora) = &model.lora_path {
                        println!("    LoRA: {:?}", lora);
                    }
                }
            }
        }
        cli::Command::Probe { name } => {
            let Some(spec) = state.registry.to_spec(&name) else { anyhow::bail!("no model {name}"); };
            match state.engine.load(&spec).await {
                Ok(_) => println!("ok: loaded {name}"),
                Err(e) => {
                    eprintln!("probe failed: {e}");
                    std::process::exit(2);
                }
            }
        }
        cli::Command::Bench { name, max_tokens } => {
            let Some(spec) = state.registry.to_spec(&name) else { anyhow::bail!("no model {name}"); };
            let loaded = state.engine.load(&spec).await?;
            let t0 = std::time::Instant::now();
            let out = loaded.generate(
                "Say hi.",
                engine::GenOptions { max_tokens, stream: false, ..Default::default() },
                None,
            ).await?;
            let elapsed = t0.elapsed();
            println!("bench output (truncated): {}", &out[..out.len().min(120)]);
            println!("elapsed: {:?}", elapsed);
        }
        cli::Command::Generate { name, prompt, max_tokens } => {
            let Some(spec) = state.registry.to_spec(&name) else { anyhow::bail!("no model {name}"); };
            let loaded = state.engine.load(&spec).await?;
            let out = loaded.generate(&prompt, engine::GenOptions { max_tokens, stream: false, ..Default::default() }, None).await?;
            println!("{}", out);
        }
    }
    Ok(())
}
