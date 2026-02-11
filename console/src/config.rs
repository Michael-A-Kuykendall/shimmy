//! Configuration for shimmy console

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Console configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to history database
    pub history_db: PathBuf,
    /// Path to session database
    pub session_db: PathBuf,
    /// Backend URL for inference
    pub backend_url: String,
    /// Discovery port for finding backends
    pub discovery_port: u16,
    /// Default model to use
    pub default_model: Option<String>,
    /// Maximum context window tokens
    pub max_context_tokens: usize,
    /// Enable debug mode
    pub debug: bool,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("shimmy");

        Self {
            history_db: data_dir.join("history.redb"),
            session_db: data_dir.join("sessions.redb"),
            backend_url: "http://localhost:8080".to_string(),
            discovery_port: 11430,
            default_model: None,
            max_context_tokens: 8192,
            debug: false,
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(url) = std::env::var("SHIMMY_BACKEND_URL") {
            config.backend_url = url;
        }

        if let Ok(port) = std::env::var("SHIMMY_DISCOVERY_PORT") {
            if let Ok(p) = port.parse() {
                config.discovery_port = p;
            }
        }

        if let Ok(model) = std::env::var("SHIMMY_DEFAULT_MODEL") {
            config.default_model = Some(model);
        }

        if let Ok(tokens) = std::env::var("SHIMMY_MAX_CONTEXT_TOKENS") {
            if let Ok(t) = tokens.parse() {
                config.max_context_tokens = t;
            }
        }

        config.debug = std::env::var("SHIMMY_DEBUG").is_ok();

        config
    }

    /// Load configuration from a file
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}
