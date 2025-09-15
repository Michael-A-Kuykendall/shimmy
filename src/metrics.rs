// Basic metrics collection and opt-in telemetry
#![allow(dead_code)]
#![allow(unused_imports)]

use axum::response::Json;
use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Metrics {
    pub requests_total: u64,
    pub generation_errors: u64,
    pub uptime_seconds: u64,
}

pub struct MetricsCollector {
    pub requests: AtomicU64,
    pub errors: AtomicU64,
    pub start_time: std::time::Instant,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new_inner()
    }
}

impl MetricsCollector {
    fn new_inner() -> Self {
        Self {
            requests: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn new() -> Arc<Self> {
        Arc::new(Self::new_inner())
    }

    pub fn record_request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_metrics(&self) -> Metrics {
        Metrics {
            requests_total: self.requests.load(Ordering::Relaxed),
            generation_errors: self.errors.load(Ordering::Relaxed),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }
}

pub async fn metrics_handler(metrics: Arc<MetricsCollector>) -> Json<Metrics> {
    Json(metrics.get_metrics())
}

// Opt-in telemetry system
#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub session_id: Uuid,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Opt-in only
            session_id: Uuid::new_v4(),
            last_updated: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TelemetryData {
    pub session_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub platform: String,
    pub metrics: TelemetryMetrics,
}

#[derive(Debug, Serialize)]
pub struct TelemetryMetrics {
    // Business metrics for VC presentations
    pub daily_active_usage: bool,
    pub session_duration_minutes: u64,
    pub requests_per_session: u64,
    pub total_tokens_generated: u64,
    pub models_used_count: u64,
    pub api_endpoints_used: Vec<String>,
    pub integration_type: Option<String>, // "vscode", "cursor", "api", "cli"

    // Market size indicators
    pub deployment_type: String, // "development", "production", "enterprise"
    pub concurrent_model_usage: u64,
    pub peak_requests_per_hour: u64,

    // Technical performance (still valuable)
    pub startup_time_ms: u64,
    pub avg_response_time_ms: u64,
    pub uptime_seconds: u64,
    pub errors_count: u64,

    // Market segmentation data
    pub gpu_detected: bool,
    pub gpu_vendor: Option<String>,
    pub hardware_tier: String,    // "consumer", "workstation", "server"
    pub region_indicator: String, // timezone-based region estimate
}

pub struct TelemetryCollector {
    config: Option<TelemetryConfig>,
    config_path: PathBuf,
    startup_time: std::time::Instant,
    request_times: std::sync::Mutex<Vec<u64>>,
    endpoints_used: std::sync::Mutex<std::collections::HashSet<String>>,
    models_used: std::sync::Mutex<std::collections::HashSet<String>>,
    hourly_request_counts: std::sync::Mutex<Vec<u64>>,
}

impl TelemetryCollector {
    pub fn new() -> Self {
        let config_path = Self::get_config_path();
        Self {
            config: None,
            config_path,
            startup_time: std::time::Instant::now(),
            request_times: std::sync::Mutex::new(Vec::new()),
            endpoints_used: std::sync::Mutex::new(std::collections::HashSet::new()),
            models_used: std::sync::Mutex::new(std::collections::HashSet::new()),
            hourly_request_counts: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config = Some(self.load_or_create_config().await?);
        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.config.as_ref().map_or(false, |c| c.enabled)
    }

    pub async fn prompt_first_run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.is_some() {
            return Ok(()); // Already configured
        }

        println!("\n🚀 Welcome to Shimmy!");
        println!("\n📊 Optional Usage Analytics");
        println!("Shimmy can collect anonymous usage data to understand how it's used");
        println!("in the wild. This helps prioritize development and may support future");
        println!("enterprise tooling (while keeping core Shimmy free forever).");
        println!();
        println!("✅ What's collected: Session length, request patterns, integration types");
        println!("❌ What's NOT collected: Your code, prompts, responses, or personal info");
        println!("🔒 Full details: https://github.com/Michael-A-Kuykendall/shimmy/blob/main/docs/METRICS.md");
        println!();
        print!("Share anonymous usage data? [Y/n]: ");

        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let enabled = !matches!(input.trim().to_lowercase().as_str(), "n" | "no");

        let mut config = TelemetryConfig::default();
        config.enabled = enabled;

        self.save_config(&config).await?;
        self.config = Some(config);

        if enabled {
            println!("✅ Thank you! Data helps keep Shimmy improving while staying free.");
            println!("💡 Disable anytime: shimmy config metrics disable");
        } else {
            println!("👍 No problem! Shimmy works perfectly without analytics.");
            println!("💡 Enable later: shimmy config metrics enable");
        }
        println!();

        Ok(())
    }

    pub async fn enable_metrics(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = self.config.take().unwrap_or_default();
        config.enabled = true;
        config.last_updated = chrono::Utc::now();

        self.save_config(&config).await?;
        self.config = Some(config);
        Ok(())
    }

    pub async fn disable_metrics(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = self.config.take().unwrap_or_default();
        config.enabled = false;
        config.last_updated = chrono::Utc::now();

        self.save_config(&config).await?;
        self.config = Some(config);
        Ok(())
    }

    pub fn record_request_time(&self, duration_ms: u64) {
        if let Ok(mut times) = self.request_times.lock() {
            times.push(duration_ms);
            // Keep only last 100 requests to avoid memory growth
            if times.len() > 100 {
                let drain_count = times.len() - 100;
                times.drain(0..drain_count);
            }
        }
    }

    pub fn record_endpoint_usage(&self, endpoint: &str) {
        if let Ok(mut endpoints) = self.endpoints_used.lock() {
            endpoints.insert(endpoint.to_string());
        }
    }

    pub fn record_model_usage(&self, model_name: &str) {
        if let Ok(mut models) = self.models_used.lock() {
            models.insert(model_name.to_string());
        }
    }

    pub fn record_hourly_request(&self) {
        if let Ok(mut counts) = self.hourly_request_counts.lock() {
            let current_hour = chrono::Utc::now().hour() as usize;
            // Ensure we have enough slots for 24 hours
            while counts.len() <= current_hour {
                counts.push(0);
            }
            counts[current_hour] += 1;
        }
    }

    pub fn preview_data(&self, metrics: &MetricsCollector) -> Option<TelemetryData> {
        self.config
            .as_ref()
            .map(|config| self.build_telemetry_data(config, metrics))
    }

    pub async fn send_telemetry(
        &self,
        metrics: &MetricsCollector,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_enabled() {
            return Ok(());
        }

        let config = self.config.as_ref().unwrap();
        let data = self.build_telemetry_data(config, metrics);

        // Send data with timeout
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        let response = client
            .post("https://metrics.shimmy-ai.dev/v1/usage")
            .json(&data)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                tracing::debug!("Telemetry sent successfully");
            }
            Ok(resp) => {
                tracing::debug!("Telemetry send failed with status: {}", resp.status());
            }
            Err(e) => {
                tracing::debug!("Telemetry send failed: {}", e);
                // Silent failure - don't impact user experience
            }
        }

        Ok(())
    }

    fn build_telemetry_data(
        &self,
        config: &TelemetryConfig,
        metrics: &MetricsCollector,
    ) -> TelemetryData {
        let current_metrics = metrics.get_metrics();

        let avg_response_time = {
            let times = self.request_times.lock().unwrap();
            if times.is_empty() {
                0
            } else {
                times.iter().sum::<u64>() / times.len() as u64
            }
        };

        let endpoints_used: Vec<String> = {
            self.endpoints_used
                .lock()
                .unwrap()
                .iter()
                .cloned()
                .collect()
        };

        let models_count = self.models_used.lock().unwrap().len() as u64;

        let peak_requests_per_hour = {
            self.hourly_request_counts
                .lock()
                .unwrap()
                .iter()
                .max()
                .copied()
                .unwrap_or(0)
        };

        let session_duration_minutes = self.startup_time.elapsed().as_secs() / 60;
        let daily_active_usage = session_duration_minutes >= 5; // 5+ minutes = active usage

        TelemetryData {
            session_id: config.session_id,
            timestamp: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            metrics: TelemetryMetrics {
                // Business metrics for VC presentations
                daily_active_usage,
                session_duration_minutes,
                requests_per_session: current_metrics.requests_total,
                total_tokens_generated: current_metrics.requests_total * 50, // Rough estimate
                models_used_count: models_count,
                api_endpoints_used: endpoints_used,
                integration_type: Self::detect_integration_type(),

                // Market size indicators
                deployment_type: Self::detect_deployment_type(
                    current_metrics.requests_total,
                    session_duration_minutes,
                ),
                concurrent_model_usage: models_count,
                peak_requests_per_hour,

                // Technical performance
                startup_time_ms: self.startup_time.elapsed().as_millis() as u64,
                avg_response_time_ms: avg_response_time,
                uptime_seconds: current_metrics.uptime_seconds,
                errors_count: current_metrics.generation_errors,

                // Market segmentation
                gpu_detected: Self::detect_gpu(),
                gpu_vendor: Self::get_gpu_vendor(),
                hardware_tier: Self::detect_hardware_tier(),
                region_indicator: Self::get_region_indicator(),
            },
        }
    }

    fn get_memory_usage_mb() -> u64 {
        // Simple memory estimation - could be improved with sys-info crate
        // For now, return 0 to avoid adding dependencies
        0
    }

    fn detect_gpu() -> bool {
        // Simple GPU detection
        std::process::Command::new("nvidia-smi")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn get_gpu_vendor() -> Option<String> {
        if std::process::Command::new("nvidia-smi")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            Some("nvidia".to_string())
        } else {
            None
        }
    }

    fn detect_integration_type() -> Option<String> {
        // Detect common IDE/tool integrations based on process environment
        if std::env::var("VSCODE_PID").is_ok()
            || std::env::var("TERM_PROGRAM").map_or(false, |v| v.contains("vscode"))
        {
            Some("vscode".to_string())
        } else if std::env::var("CURSOR_USER_DATA").is_ok() {
            Some("cursor".to_string())
        } else if std::env::var("CONTINUE_GLOBAL_DIR").is_ok() {
            Some("continue".to_string())
        } else if std::env::args().any(|arg| arg.contains("api") || arg.contains("serve")) {
            Some("api".to_string())
        } else {
            Some("cli".to_string())
        }
    }

    fn detect_deployment_type(requests_total: u64, session_duration_minutes: u64) -> String {
        if requests_total > 1000 || session_duration_minutes > 480 {
            // 8+ hours
            "production".to_string()
        } else if requests_total > 100 || session_duration_minutes > 60 {
            "development".to_string()
        } else {
            "evaluation".to_string()
        }
    }

    fn detect_hardware_tier() -> String {
        let has_nvidia = Self::detect_gpu();
        let total_memory = sys_info::mem_info().map_or(0, |m| m.total);

        if has_nvidia && total_memory > 32_000_000 {
            // 32GB+ with GPU
            "workstation".to_string()
        } else if total_memory > 64_000_000 {
            // 64GB+ (likely server)
            "server".to_string()
        } else {
            "consumer".to_string()
        }
    }

    fn get_region_indicator() -> String {
        // Use timezone to estimate region (anonymous)
        match chrono::Local::now().offset().local_minus_utc() / 3600 {
            -12..=-5 => "americas".to_string(),
            -4..=3 => "emea".to_string(), // Europe/Middle East/Africa
            4..=12 => "apac".to_string(), // Asia Pacific
            _ => "unknown".to_string(),
        }
    }

    fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("shimmy");
        path.push("config.json");
        path
    }

    async fn load_or_create_config(&self) -> Result<TelemetryConfig, Box<dyn std::error::Error>> {
        if !self.config_path.exists() {
            // Create default config (metrics disabled)
            let config = TelemetryConfig::default();
            self.save_config(&config).await?;
            return Ok(config);
        }

        let content = fs::read_to_string(&self.config_path).await?;
        let config: TelemetryConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    async fn save_config(
        &self,
        config: &TelemetryConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure config directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, content).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_new() {
        let metrics = MetricsCollector::new();
        assert_eq!(metrics.requests.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.errors.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_record_request() {
        let metrics = MetricsCollector::new();
        metrics.record_request();
        assert_eq!(metrics.requests.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_record_error() {
        let metrics = MetricsCollector::new();
        metrics.record_error();
        assert_eq!(metrics.errors.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_get_metrics() {
        let metrics = MetricsCollector::new();
        metrics.record_request();
        metrics.record_error();

        let result = metrics.get_metrics();
        assert_eq!(result.requests_total, 1);
        assert_eq!(result.generation_errors, 1);
        assert!(result.uptime_seconds < 60);
    }
}
