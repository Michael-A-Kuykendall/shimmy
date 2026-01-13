/**
 * Discovery Service - The Real One
 * 
 * When themes ping this service, it:
 * 1. Starts shimmy automatically with Champion model
 * 2. Handles all port/model bullshit internally
 * 3. Returns clean model list to theme
 * 4. Theme makers just do npm run dev and everything works
 */

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    pub status: String,
    pub shimmy_url: String,
    pub models: Vec<ModelInfo>,
    pub selected_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub display_name: String,
    pub backend_type: String,
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectModelRequest {
    pub model_name: String,
}

#[derive(Debug)]
pub struct DiscoveryService {
    shimmy_port: Option<u16>,
    shimmy_url: Option<String>,
    models: Vec<ModelInfo>,
    selected_model: Option<String>,
    last_health_check: Option<Instant>,
}

impl Default for DiscoveryService {
    fn default() -> Self {
        Self {
            shimmy_port: None,
            shimmy_url: None,
            models: Vec::new(),
            selected_model: None,
            last_health_check: None,
        }
    }
}

type SharedState = Arc<Mutex<DiscoveryService>>;

impl DiscoveryService {
    pub async fn ensure_shimmy_running(&mut self) -> Result<()> {
        // Check if shimmy is already running and healthy
        if let Some(url) = &self.shimmy_url {
            if self.is_shimmy_healthy(url).await {
                return Ok(());
            }
        }

        // Start shimmy with your Champion model
        println!("🚀 Starting shimmy with Champion model...");
        
        let shimmy_port = self.find_free_port().await?;
        
        // Build the command to start shimmy with Champion model
        let mut cmd = Command::new("./target/release/shimmy.exe");
        cmd.args(&["serve", "--bind", &format!("127.0.0.1:{}", shimmy_port)])
            .env("SHIMMY_BASE_GGUF", "meta-llama/Llama-3.2-1B")
            .env("SHIMMY_LORA_GGUF", "/c/Users/micha/repos/command-center/llama-3.2-1b-personal")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        let shimmy_url = format!("http://127.0.0.1:{}", shimmy_port);
        
        // Wait for shimmy to be ready
        for _ in 0..30 {
            sleep(Duration::from_secs(1)).await;
            if self.is_shimmy_healthy(&shimmy_url).await {
                self.shimmy_port = Some(shimmy_port);
                self.shimmy_url = Some(shimmy_url.clone());
                self.last_health_check = Some(Instant::now());
                
                // Get models from shimmy
                self.refresh_models().await?;
                
                println!("✅ Shimmy ready with Champion model on {}", shimmy_url);
                return Ok(());
            }
        }

        Err(anyhow::anyhow!("Shimmy failed to start within 30 seconds"))
    }

    async fn find_free_port(&self) -> Result<u16> {
        // Try common ports
        let ports = [11435, 11434, 11436, 11437, 11438];
        
        for port in ports {
            if self.is_port_free(port).await {
                return Ok(port);
            }
        }
        
        Err(anyhow::anyhow!("No free ports available"))
    }

    async fn is_port_free(&self, port: u16) -> bool {
        tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .is_ok()
    }

    async fn is_shimmy_healthy(&self, url: &str) -> bool {
        let client = reqwest::Client::new();
        match client
            .get(&format!("{}/v1/models", url))
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    async fn refresh_models(&mut self) -> Result<()> {
        let Some(url) = &self.shimmy_url else {
            return Err(anyhow::anyhow!("Shimmy not running"));
        };

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/v1/models", url))
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to get models from shimmy"));
        }

        let models_response: serde_json::Value = response.json().await?;
        
        self.models.clear();
        
        if let Some(models) = models_response.get("data").and_then(|d| d.as_array()) {
            for model in models {
                if let Some(name) = model.get("id").and_then(|n| n.as_str()) {
                    self.models.push(ModelInfo {
                        name: name.to_string(),
                        display_name: self.clean_model_name(name),
                        backend_type: "shimmy".to_string(),
                        ready: true,
                    });
                }
            }
        }

        // Auto-select Champion model if available
        if self.selected_model.is_none() {
            if let Some(champion) = self.models.iter().find(|m| m.name.contains("phi3-lora")) {
                self.selected_model = Some(champion.name.clone());
                println!("🏆 Auto-selected Champion model: {}", champion.display_name);
            } else if !self.models.is_empty() {
                self.selected_model = Some(self.models[0].name.clone());
            }
        }

        Ok(())
    }

    fn clean_model_name(&self, name: &str) -> String {
        if name.contains("phi3-lora") {
            "🏆 Champion Model".to_string()
        } else {
            name.replace(" [Ollama]", "")
                .replace("phi-3.5-moe-f16", "Phi-3.5 MoE")
                .replace("gpt-oss-20b-f16", "GPT-OSS 20B")
                .replace("llama3.2:1b", "Llama 3.2 1B")
                .replace("tinyllama:1.1b", "TinyLlama 1.1B")
                .replace("qwen2.5:1.5b", "Qwen 2.5 1.5B")
        }
    }
}

// HTTP handlers
async fn discover(State(state): State<SharedState>) -> Result<Json<DiscoveryResponse>, StatusCode> {
    let mut service = state.lock().unwrap();
    
    // Ensure shimmy is running with Champion model
    if let Err(e) = service.ensure_shimmy_running().await {
        eprintln!("Failed to start shimmy: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let response = DiscoveryResponse {
        status: "ready".to_string(),
        shimmy_url: service.shimmy_url.clone().unwrap_or_default(),
        models: service.models.clone(),
        selected_model: service.selected_model.clone(),
    };

    Ok(Json(response))
}

async fn select_model(
    State(state): State<SharedState>,
    Json(request): Json<SelectModelRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut service = state.lock().unwrap();
    
    // Validate model exists
    if !service.models.iter().any(|m| m.name == request.model_name) {
        return Err(StatusCode::BAD_REQUEST);
    }

    service.selected_model = Some(request.model_name.clone());
    
    Ok(Json(serde_json::json!({
        "status": "selected",
        "model": request.model_name
    })))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "shimmy-discovery"
    }))
}

pub async fn run_discovery_service() -> Result<()> {
    let state = Arc::new(Mutex::new(DiscoveryService::default()));

    let app = Router::new()
        .route("/discover", get(discover))
        .route("/select-model", post(select_model))
        .route("/health", get(health))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:11440").await?;
    
    println!("🔍 Discovery Service running on http://127.0.0.1:11440");
    println!("   Themes can now just do 'npm run dev' and everything works!");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
