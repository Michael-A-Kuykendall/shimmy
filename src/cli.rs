use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "shimmy", version, about = "Shimmy: single-binary GGUF + LoRA server")] 
pub struct Cli { #[command(subcommand)] pub cmd: Command }

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run the HTTP server
    Serve { #[arg(long, default_value_t=String::from("127.0.0.1:11435"))] bind: String },
    /// List registered models
    List,
    /// Load a model once (verifies base + optional LoRA)
    Probe { name: String },
    /// Simple throughput benchmark
    Bench { name: String, #[arg(long, default_value_t=64)] max_tokens: usize },
    /// One-off generation (non-streaming) for quick manual testing
    Generate { name: String, #[arg(long)] prompt: String, #[arg(long, default_value_t=64)] max_tokens: usize },
}

pub fn handle_list_command_with_discovery() {
    use crate::model_registry::ModelRegistry;
    let registry = ModelRegistry::with_discovery();
    
    println!("Discovered Models:");
    for (name, model) in &registry.discovered_models {
        println!("  {} - {} ({:?})", name, model.path.display(), model.model_type);
    }
    
    if registry.discovered_models.is_empty() {
        println!("  No models found in search paths");
        println!("  Set SHIMMY_BASE_GGUF or place models in ~/.cache/huggingface or ~/models");
    }
}
