//! Orchestrator: High-level process and stack management.
//!
//! The orchestrator is responsible for:
//! - **Lifecycle management**: Build → Serve → Discover → Theme → Verify (shimmy dev / shimmy dev --verify)
//! - **Process supervision**: Spawn and manage child processes (build, shimmy server, theme dev server)
//! - **Discovery polling**: Wait for backend discovery service to become available
//! - **Theme management**: Start theme dev servers and poll for readiness
//! - **Verification**: Run stack verification (schema export, theme validator, browser tests)
//! - **Cross-platform support**: Windows + Unix process termination, shell-specific spawning
//!
//! # Example: Using the orchestrator
//!
//! ```no_run
//! use shimmy::orchestrator::{OrchestratorConfig, lifecycle::run_lifecycle};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let cfg = OrchestratorConfig::default();
//! let result = run_lifecycle(&cfg, "shimmy-default", false, false).await?;
//! println!("Orchestrator started theme at {}", result.theme_url);
//! # Ok(())
//! # }
//! ```
//!
//! # Extending the orchestrator
//!
//! - For custom process launching: implement [`supervisor::SupervisorTrait`] and inject into lifecycle
//! - For testing: use [`test_shims::FakeSupervisor`] to mock process execution
//! - For cross-platform spawning: see [`platform::stop_all`] for platform-specific utilities

pub mod supervisor;
pub mod platform;
pub mod discovery_watcher;
pub mod theme_manager;
pub mod verification;
pub mod lifecycle;
pub mod license;
pub mod test_shims;

use std::time::Duration;

/// Orchestrator configuration (defaults mirror existing shell script semantics).
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub discovery_url: String,
    pub theme_url: String,
    pub max_discovery_wait: Option<Duration>,
    pub max_theme_wait: Option<Duration>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            discovery_url: "http://127.0.0.1:11430/api/discovery".to_string(),
            theme_url: "http://localhost:8080".to_string(),
            // Default: wait-until-ready semantics — None means no timeout (wait indefinitely)
            max_discovery_wait: None,
            max_theme_wait: None,
        }
    }
}

pub use platform::stop_all;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_sensible() {
        let c = OrchestratorConfig::default();
        assert!(c.discovery_url.contains("127.0.0.1"));
        assert!(c.theme_url.contains("localhost"));
        // Defaults now represent indefinite waits (None)
        assert!(c.max_discovery_wait.is_none());
        assert!(c.max_theme_wait.is_none());
    }
}
