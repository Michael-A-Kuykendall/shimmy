//! Config command

use clap::{Args, Subcommand};

/// Arguments for the config command
#[derive(Debug, Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub action: ConfigAction,
}

/// Config subcommands
#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Get a configuration value
    Get {
        /// Key to retrieve
        key: String,
    },
    /// Set a configuration value
    Set {
        /// Key to set
        key: String,
        /// Value to set
        value: String,
    },
    /// List all configuration values
    List,
    /// Reset configuration to defaults
    Reset,
}

impl ConfigCommand {
    /// Execute the config command
    pub async fn run(&self) -> anyhow::Result<()> {
        use crate::config::Config;

        match &self.action {
            ConfigAction::Get { key } => {
                let config = Config::from_env();
                let value = match key.as_str() {
                    "backend_url" => config.backend_url,
                    "discovery_port" => config.discovery_port.to_string(),
                    "default_model" => config.default_model.unwrap_or_default(),
                    "max_context_tokens" => config.max_context_tokens.to_string(),
                    "debug" => config.debug.to_string(),
                    _ => {
                        eprintln!("Unknown config key: {}", key);
                        return Ok(());
                    }
                };
                println!("{}", value);
            }
            ConfigAction::Set { key, value } => {
                println!("Setting {} = {}", key, value);
                println!("Note: Config persistence not yet implemented");
            }
            ConfigAction::List => {
                let config = Config::from_env();
                println!("backend_url = {}", config.backend_url);
                println!("discovery_port = {}", config.discovery_port);
                println!(
                    "default_model = {}",
                    config.default_model.unwrap_or_default()
                );
                println!("max_context_tokens = {}", config.max_context_tokens);
                println!("debug = {}", config.debug);
            }
            ConfigAction::Reset => {
                println!("Resetting configuration to defaults...");
                let config = Config::default();
                println!("Done. New config:");
                println!("backend_url = {}", config.backend_url);
                println!("discovery_port = {}", config.discovery_port);
            }
        }

        Ok(())
    }
}
