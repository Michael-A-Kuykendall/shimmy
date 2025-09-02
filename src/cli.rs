use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "shimmy", version, about = "Shimmy: single-binary GGUF + LoRA server")] 
pub struct Cli { #[command(subcommand)] pub cmd: Command }

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run the HTTP server
    Serve { #[arg(long, default_value_t=String::from("127.0.0.1:11435"))] bind: String },
    /// List registered and auto-discovered models
    List,
    /// Refresh auto-discovery and list all available models
    Discover,
    /// Load a model once (verifies base + optional LoRA)
    Probe { name: String },
    /// Simple throughput benchmark
    Bench { name: String, #[arg(long, default_value_t=64)] max_tokens: usize },
    /// One-off generation (non-streaming) for quick manual testing
    Generate { name: String, #[arg(long)] prompt: String, #[arg(long, default_value_t=64)] max_tokens: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    
    #[test]
    fn test_cli_serve_command() {
        let cli = Cli::try_parse_from(&["shimmy", "serve"]).unwrap();
        match cli.cmd {
            Command::Serve { bind } => assert_eq!(bind, "127.0.0.1:11435"),
            _ => panic!("Expected Serve command"),
        }
    }
    
    #[test]
    fn test_cli_list_command() {
        let cli = Cli::try_parse_from(&["shimmy", "list"]).unwrap();
        matches!(cli.cmd, Command::List);
    }
    
    #[test]
    fn test_cli_generate_command() {
        let cli = Cli::try_parse_from(&["shimmy", "generate", "model", "--prompt", "test", "--max-tokens", "100"]).unwrap();
        match cli.cmd {
            Command::Generate { name, prompt, max_tokens } => {
                assert_eq!(name, "model");
                assert_eq!(prompt, "test");
                assert_eq!(max_tokens, 100);
            },
            _ => panic!("Expected Generate command"),
        }
    }
}
