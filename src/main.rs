mod api;
mod api_errors;
mod auto_discovery;
mod cli;
mod discovery;
mod engine;
mod main_integration;
mod metrics;
mod model_manager;
mod model_registry;
mod openai_compat;
mod rustchain_compat;
mod safetensors_adapter;
mod server;
mod templates;
mod tools;
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

    // Temporary inline registry: one model (phi3-lora) pulling paths from env vars.
    let mut reg = Registry::default();
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
            info!(%addr, "shimmy serving");
            server::run(addr, state).await?;
        }
        cli::Command::List => {
            for e in state.registry.list() { println!("{} => {:?}", e.name, e.base_path); }
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
