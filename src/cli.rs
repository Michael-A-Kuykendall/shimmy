use clap::{Parser, Subcommand};
use crate::port_manager::GLOBAL_PORT_ALLOCATOR;

#[derive(Parser, Debug)]
#[command(name = "shimmy", version, about = "Shimmy: single-binary GGUF + LoRA server")] 
pub struct Cli { #[command(subcommand)] pub cmd: Command }

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run the HTTP server
    Serve { 
        #[arg(long, default_value = "auto")] 
        bind: String 
    },
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

impl Command {
    pub fn get_bind_address(&self) -> String {
        match self {
            Command::Serve { bind } => {
                if bind == "auto" {
                    match GLOBAL_PORT_ALLOCATOR.find_available_port("shimmy-server") {
                        Ok(port) => format!("127.0.0.1:{}", port),
                        Err(_) => {
                            eprintln!("Warning: Could not allocate dynamic port, falling back to 127.0.0.1:11435");
                            "127.0.0.1:11435".to_string()
                        }
                    }
                } else {
                    bind.clone()
                }
            },
            _ => "127.0.0.1:11435".to_string(), // Default fallback for other commands
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    
    #[test]
    fn test_cli_serve_command_default() {
        let cli = Cli::try_parse_from(&["shimmy", "serve"]).unwrap();
        match cli.cmd {
            Command::Serve { bind } => assert_eq!(bind, "auto"),
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_cli_serve_command_manual_bind() {
        let cli = Cli::try_parse_from(&["shimmy", "serve", "--bind", "127.0.0.1:8080"]).unwrap();
        match cli.cmd {
            Command::Serve { bind } => assert_eq!(bind, "127.0.0.1:8080"),
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_get_bind_address_auto() {
        let command = Command::Serve { bind: "auto".to_string() };
        let address = command.get_bind_address();
        
        // Should either be dynamic port or fallback
        assert!(address.starts_with("127.0.0.1:"));
        let port_part = address.split(':').nth(1).unwrap();
        let port: u16 = port_part.parse().unwrap();
        assert!(port > 0);
    }

    #[test]
    fn test_get_bind_address_manual() {
        let command = Command::Serve { bind: "192.168.1.100:9000".to_string() };
        let address = command.get_bind_address();
        
        assert_eq!(address, "192.168.1.100:9000");
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
    
    #[test]
    fn test_cli_discover_command() {
        let cli = Cli::try_parse_from(&["shimmy", "discover"]).unwrap();
        matches!(cli.cmd, Command::Discover);
    }
    
    #[test]
    fn test_cli_probe_command() {
        let cli = Cli::try_parse_from(&["shimmy", "probe", "test-model"]).unwrap();
        match cli.cmd {
            Command::Probe { name } => assert_eq!(name, "test-model"),
            _ => panic!("Expected Probe command"),
        }
    }
    
    #[test]
    fn test_cli_bench_command() {
        let cli = Cli::try_parse_from(&["shimmy", "bench", "test-model", "--max-tokens", "128"]).unwrap();
        match cli.cmd {
            Command::Bench { name, max_tokens } => {
                assert_eq!(name, "test-model");
                assert_eq!(max_tokens, 128);
            },
            _ => panic!("Expected Bench command"),
        }
    }
    
    #[test]
    fn test_cli_bench_command_default_tokens() {
        let cli = Cli::try_parse_from(&["shimmy", "bench", "test-model"]).unwrap();
        match cli.cmd {
            Command::Bench { name, max_tokens } => {
                assert_eq!(name, "test-model");
                assert_eq!(max_tokens, 64); // Default value
            },
            _ => panic!("Expected Bench command"),
        }
    }
}
