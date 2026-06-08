use std::path::PathBuf;

/// Configuration for the shimmy console.
/// Reads from environment variables with sensible defaults.
#[derive(Debug, Clone)]
pub struct ConsoleConfig {
    /// shimmy serve base URL (default: http://127.0.0.1:11435)
    pub base_url: String,
    /// Model name to use (default: "default")
    pub model: String,
    /// Directory for session storage (default: ~/.shimmy/sessions/)
    pub session_dir: PathBuf,
    /// Directory for tool manifests (default: ~/.shimmy/tools/)
    pub tools_dir: PathBuf,
}

impl ConsoleConfig {
    pub fn from_env() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let shimmy_dir = home.join(".shimmy");
        Self {
            base_url: std::env::var("SHIMMY_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:11435".to_string()),
            model: std::env::var("SHIMMY_MODEL")
                .unwrap_or_else(|_| "default".to_string()),
            session_dir: std::env::var("SHIMMY_SESSION_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| shimmy_dir.join("sessions")),
            tools_dir: std::env::var("SHIMMY_TOOLS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| shimmy_dir.join("tools")),
        }
    }
}
