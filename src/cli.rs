use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "shimmy",
    version,
    about = "Shimmy: single-binary GGUF + LoRA server"
)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,

    /// Additional directories to search for models (semicolon-separated)
    #[arg(
        long,
        global = true,
        help = "Additional model directories to search (e.g., --model-dirs 'D:\\models;E:\\ollama\\models')"
    )]
    pub model_dirs: Option<String>,

    /// GPU backend to use for llama.cpp inference
    #[arg(
        long,
        global = true,
        help = "GPU backend: auto, cpu, cuda, vulkan, opencl"
    )]
    pub gpu_backend: Option<String>,

    /// Offload ALL MoE expert tensors to CPU (saves VRAM for large MoE models)
    #[arg(long, global = true)]
    pub cpu_moe: bool,

    /// Offload first N MoE layers' expert tensors to CPU
    #[arg(long, global = true, value_name = "N", conflicts_with = "cpu_moe")]
    pub n_cpu_moe: Option<usize>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run the HTTP server
    Serve {
        #[arg(long, default_value = "auto")]
        bind: String,
        /// Direct path to a specific model file (bypasses auto-discovery)
        #[arg(long)]
        model_path: Option<String>,
    },
    /// List registered and auto-discovered models
    List {
        /// Output short list format (model names only)
        #[arg(short, long)]
        short: bool,
    },
    /// Refresh auto-discovery and list all available models
    Discover {
        /// Show only LLM models (filter out text-to-image, video, clip models, etc.)
        #[arg(long)]
        llm_only: bool,
    },
    /// Load a model once (verifies base + optional LoRA)
    Probe { name: String },
    /// Simple throughput benchmark
    Bench {
        name: String,
        #[arg(long, default_value_t = 64)]
        max_tokens: usize,
    },
    /// One-off generation (non-streaming) for quick manual testing
    Generate {
        name: String,
        #[arg(long)]
        prompt: String,
        #[arg(long, default_value_t = 64)]
        max_tokens: usize,
    },
    /// Show GPU backend information and capabilities
    GpuInfo,
    /// Initialize integration templates for deployment platforms
    Init {
        /// Template type: docker, kubernetes, railway, fly, fastapi, express
        #[arg(short, long)]
        template: String,
        /// Output directory for generated files
        #[arg(short, long, default_value = ".")]
        output: String,
        /// Project name for template customization
        #[arg(short, long)]
        name: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_serve_command_default() {
        let cli = Cli::try_parse_from(["shimmy", "serve"]).unwrap();
        match cli.cmd {
            Command::Serve { bind, .. } => assert_eq!(bind, "auto"),
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_cli_serve_command_manual_bind() {
        let cli = Cli::try_parse_from(["shimmy", "serve", "--bind", "127.0.0.1:8080"]).unwrap();
        match cli.cmd {
            Command::Serve { bind, .. } => assert_eq!(bind, "127.0.0.1:8080"),
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_get_bind_address_auto() {
        let command = Command::Serve {
            bind: "auto".to_string(),
            model_path: None,
        };

        // Test that we can access the bind field
        match command {
            Command::Serve { bind, .. } => {
                assert!(bind.starts_with("auto") || bind.starts_with("127.0.0.1:"));
            }
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_get_bind_address_manual() {
        let command = Command::Serve {
            bind: "192.168.1.100:9000".to_string(),
            model_path: None,
        };

        match command {
            Command::Serve { bind, .. } => {
                assert_eq!(bind, "192.168.1.100:9000");
            }
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_cli_list_command() {
        let cli = Cli::try_parse_from(["shimmy", "list"]).unwrap();
        matches!(cli.cmd, Command::List { short: false });
    }

    #[test]
    fn test_cli_list_short_command() {
        let cli = Cli::try_parse_from(["shimmy", "list", "--short"]).unwrap();
        matches!(cli.cmd, Command::List { short: true });

        let cli = Cli::try_parse_from(["shimmy", "list", "-s"]).unwrap();
        matches!(cli.cmd, Command::List { short: true });
    }

    #[test]
    fn test_cli_generate_command() {
        let cli = Cli::try_parse_from([
            "shimmy",
            "generate",
            "model",
            "--prompt",
            "test",
            "--max-tokens",
            "100",
        ])
        .unwrap();
        match cli.cmd {
            Command::Generate {
                name,
                prompt,
                max_tokens,
            } => {
                assert_eq!(name, "model");
                assert_eq!(prompt, "test");
                assert_eq!(max_tokens, 100);
            }
            _ => panic!("Expected Generate command"),
        }
    }

    #[test]
    fn test_cli_discover_command() {
        let cli = Cli::try_parse_from(["shimmy", "discover"]).unwrap();
        matches!(cli.cmd, Command::Discover { llm_only: _ });
    }

    #[test]
    fn test_cli_probe_command() {
        let cli = Cli::try_parse_from(["shimmy", "probe", "test-model"]).unwrap();
        match cli.cmd {
            Command::Probe { name } => assert_eq!(name, "test-model"),
            _ => panic!("Expected Probe command"),
        }
    }

    #[test]
    fn test_cli_bench_command() {
        let cli =
            Cli::try_parse_from(["shimmy", "bench", "test-model", "--max-tokens", "128"]).unwrap();
        match cli.cmd {
            Command::Bench { name, max_tokens } => {
                assert_eq!(name, "test-model");
                assert_eq!(max_tokens, 128);
            }
            _ => panic!("Expected Bench command"),
        }
    }

    #[test]
    fn test_cli_bench_command_default_tokens() {
        let cli = Cli::try_parse_from(["shimmy", "bench", "test-model"]).unwrap();
        match cli.cmd {
            Command::Bench { name, max_tokens } => {
                assert_eq!(name, "test-model");
                assert_eq!(max_tokens, 64); // Default value
            }
            _ => panic!("Expected Bench command"),
        }
    }
}
