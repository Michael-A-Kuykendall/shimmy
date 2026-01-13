#!/usr/bin/env rust
//! Shimmy Discovery Daemon
//!
//! Lightweight HTTP service that auto-starts shimmy when themes need it.
//! Theme makers just start their theme - this daemon handles shimmy startup.
//!
//! Flow:
//! 1. Theme (browser) POSTs to http://127.0.0.1:11440/startup
//! 2. Daemon spawns shimmy serve --bind auto
//! 3. Daemon waits for shimmy IPC ready
//! 4. Returns success to theme
//! 5. Theme discovers models via IPC and shows model chooser

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Daemon state tracking shimmy process
#[derive(Default)]
struct DaemonState {
    shimmy_process: Option<Child>,
    last_startup: Option<Instant>,
}

/// Startup request from theme
#[derive(Debug, Deserialize)]
struct StartupRequest {
    action: String,
    #[serde(default)]
    config: StartupConfig,
}

#[derive(Debug, Deserialize, Default)]
struct StartupConfig {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    bind: Option<String>,
}

/// Startup response to theme
#[derive(Debug, Serialize)]
struct StartupResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
}

impl IntoResponse for StartupResponse {
    fn into_response(self) -> Response {
        let status = if self.success {
            StatusCode::OK
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, Json(self)).into_response()
    }
}

/// Check if shimmy is already running by trying common ports
async fn is_shimmy_running() -> Option<u16> {
    let common_ports = [11435, 11434, 11436, 11430, 11431, 11432, 11433];
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
        .ok()?;

    for port in common_ports {
        let url = format!("http://127.0.0.1:{}/v1/models", port);
        if client.get(&url).send().await.is_ok() {
            return Some(port);
        }
    }
    None
}

/// Wait for shimmy to be ready and responding
async fn wait_for_shimmy_ready(max_wait: Duration) -> Result<u16, String> {
    let start = Instant::now();
    
    while start.elapsed() < max_wait {
        if let Some(port) = is_shimmy_running().await {
            return Ok(port);
        }
        sleep(Duration::from_secs(1)).await;
    }
    
    Err("Shimmy failed to start within timeout".to_string())
}

/// Start shimmy process
fn start_shimmy_process(config: &StartupConfig) -> Result<Child, String> {
    let shimmy_exe = if cfg!(windows) {
        "shimmy.exe"
    } else {
        "shimmy"
    };

    let bind = config.bind.as_deref().unwrap_or("auto");
    
    Command::new(shimmy_exe)
        .args(&["serve", "--bind", bind])
        .spawn()
        .map_err(|e| format!("Failed to start shimmy: {}", e))
}

/// Handle startup request from theme
async fn handle_startup(
    axum::extract::State(state): axum::extract::State<Arc<Mutex<DaemonState>>>,
    Json(request): Json<StartupRequest>,
) -> StartupResponse {
    // Check if shimmy already running
    if let Some(port) = is_shimmy_running().await {
        return StartupResponse {
            success: true,
            message: format!("Shimmy already running on port {}", port),
            port: Some(port),
        };
    }

    // Check if we recently started shimmy (avoid multiple rapid starts)
    {
        let state_lock = state.lock().unwrap();
        if let Some(last_startup) = state_lock.last_startup {
            if last_startup.elapsed() < Duration::from_secs(5) {
                return StartupResponse {
                    success: false,
                    message: "Shimmy startup already in progress, please wait".to_string(),
                    port: None,
                };
            }
        }
    }

    // Start shimmy
    eprintln!("🚀 Starting shimmy...");
    let child = match start_shimmy_process(&request.config) {
        Ok(child) => child,
        Err(e) => {
            return StartupResponse {
                success: false,
                message: e,
                port: None,
            };
        }
    };

    // Update state
    {
        let mut state_lock = state.lock().unwrap();
        state_lock.shimmy_process = Some(child);
        state_lock.last_startup = Some(Instant::now());
    }

    // Wait for shimmy to be ready
    match wait_for_shimmy_ready(Duration::from_secs(30)).await {
        Ok(port) => {
            eprintln!("✅ Shimmy ready on port {}", port);
            StartupResponse {
                success: true,
                message: format!("Shimmy started successfully on port {}", port),
                port: Some(port),
            }
        }
        Err(e) => {
            eprintln!("❌ {}", e);
            StartupResponse {
                success: false,
                message: e,
                port: None,
            }
        }
    }
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Discovery daemon running")
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(DaemonState::default()));

    let app = Router::new()
        .route("/startup", post(handle_startup))
        .route("/health", axum::routing::get(health_check))
        .with_state(state);

    let bind_addr = "127.0.0.1:11440";
    eprintln!("🔧 Shimmy Discovery Daemon starting on {}", bind_addr);
    eprintln!("   Themes can now auto-start shimmy by requesting:");
    eprintln!("   POST http://{}/startup", bind_addr);

    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("Failed to bind daemon port");

    eprintln!("✅ Discovery daemon ready");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

