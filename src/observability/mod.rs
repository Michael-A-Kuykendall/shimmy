use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Lightweight observability and self-optimization
/// Constitutional compliance: <100MB memory, <2% performance overhead
pub struct ObservabilityManager {
    metrics: Arc<RwLock<SystemMetrics>>,
    config: ObservabilityConfig,
    optimization_state: Arc<RwLock<OptimizationState>>,
}

impl Default for ObservabilityManager {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    pub metrics_retention: Duration,
    pub optimization_enabled: bool,
    pub health_check_interval: Duration,
    pub export_format: ExportFormat,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            metrics_retention: Duration::from_secs(3600), // 1 hour
            optimization_enabled: true,
            health_check_interval: Duration::from_secs(60),
            export_format: ExportFormat::Prometheus,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Prometheus,
    Json,
    Human,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    // Request metrics
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub current_requests: u32,

    // Model metrics
    pub model_stats: HashMap<String, ModelMetrics>,

    // Resource metrics
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub disk_usage_percent: f64,

    // Cache metrics
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_size_mb: f64,

    // Preloading metrics
    pub preloaded_models: u32,
    pub preload_hit_rate: f64,

    // System health
    pub uptime_seconds: u64,
    pub last_updated: u64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub requests: u64,
    pub total_response_time: f64,
    pub errors: u64,
    pub loading_time: f64,
    pub memory_usage_mb: f64,
    pub popularity_score: f64,
}

#[derive(Debug)]
struct OptimizationState {
    last_optimization: SystemTime,
    optimization_count: u32,
}

impl Default for OptimizationState {
    fn default() -> Self {
        Self {
            last_optimization: SystemTime::UNIX_EPOCH,
            optimization_count: 0,
        }
    }
}

impl ObservabilityManager {
    pub fn new() -> Self {
        Self::with_config(ObservabilityConfig::default())
    }

    pub fn with_config(config: ObservabilityConfig) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            config,
            optimization_state: Arc::new(RwLock::new(OptimizationState::default())),
        }
    }

    /// Record a request with timing and model information
    pub async fn record_request(&self, model_name: &str, response_time: Duration, success: bool) {
        let mut metrics = self.metrics.write().await;

        metrics.total_requests += 1;
        if success {
            metrics.successful_requests += 1;
        } else {
            metrics.failed_requests += 1;
        }

        // Update average response time
        let response_ms = response_time.as_secs_f64() * 1000.0;
        metrics.average_response_time =
            (metrics.average_response_time * (metrics.total_requests - 1) as f64 + response_ms)
                / metrics.total_requests as f64;

        // Store uptime for later use
        let uptime_hours = metrics.uptime_seconds.max(1) as f64 / 3600.0;

        // Update model-specific metrics
        let model_metrics = metrics
            .model_stats
            .entry(model_name.to_string())
            .or_insert_with(ModelMetrics::default);

        model_metrics.requests += 1;
        model_metrics.total_response_time += response_ms;
        if !success {
            model_metrics.errors += 1;
        }

        // Calculate popularity score (requests per hour)
        model_metrics.popularity_score = model_metrics.requests as f64 / uptime_hours;

        metrics.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        debug!(
            "Recorded request for model '{}': {}ms, success: {}",
            model_name, response_ms, success
        );
    }

    /// Update system resource metrics
    pub async fn update_system_metrics(&self) {
        let mut metrics = self.metrics.write().await;

        // Simplified system monitoring - in production this would use proper system APIs
        metrics.memory_usage_mb = self.memory_usage().await;
        metrics.cpu_usage_percent = self.cpu_usage().await;
        metrics.disk_usage_percent = self.disk_usage().await;

        // Update uptime
        metrics.uptime_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Update cache metrics
    pub async fn update_cache_metrics(&self, hits: u64, misses: u64, size_mb: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_hits = hits;
        metrics.cache_misses = misses;
        metrics.cache_size_mb = size_mb;
    }

    /// Update preloading metrics
    pub async fn update_preload_metrics(&self, preloaded_count: u32, hit_rate: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.preloaded_models = preloaded_count;
        metrics.preload_hit_rate = hit_rate;
    }

    /// Perform self-optimization based on observed metrics
    pub async fn optimize_system(&self) -> Result<Vec<OptimizationAction>> {
        if !self.config.optimization_enabled {
            return Ok(vec![]);
        }

        let metrics = self.metrics.read().await;
        let mut optimization_state = self.optimization_state.write().await;
        let mut actions = Vec::new();

        // Check if enough time has passed since last optimization
        if optimization_state
            .last_optimization
            .elapsed()
            .unwrap_or_default()
            < Duration::from_secs(300)
        {
            return Ok(actions);
        }

        // Memory optimization
        if metrics.memory_usage_mb > 4096.0 && metrics.cache_size_mb > 1024.0 {
            actions.push(OptimizationAction::ReduceCacheSize {
                current_mb: metrics.cache_size_mb,
                recommended_mb: 512.0,
            });
        }

        // Preloading optimization based on model popularity
        let popular_models: Vec<_> = metrics
            .model_stats
            .iter()
            .filter(|(_, stats)| stats.popularity_score > 1.0)
            .take(3)
            .map(|(name, _)| name.clone())
            .collect();

        if !popular_models.is_empty() {
            actions.push(OptimizationAction::UpdatePreloadList {
                models: popular_models,
            });
        }

        // Performance tuning based on response times
        if metrics.average_response_time > 1000.0 {
            actions.push(OptimizationAction::TunePerformance {
                issue: "High response times detected".to_string(),
                recommendation: "Consider increasing preloaded models or optimizing model selection".to_string(),
            });
        }

        optimization_state.last_optimization = SystemTime::now();
        optimization_state.optimization_count += 1;

        info!("Generated {} optimization actions", actions.len());
        Ok(actions)
    }

    /// Export metrics in specified format
    pub async fn export_metrics(&self) -> String {
        let metrics = self.metrics.read().await;

        match self.config.export_format {
            ExportFormat::Prometheus => self.export_prometheus(&metrics).await,
            ExportFormat::Json => serde_json::to_string_pretty(&*metrics).unwrap_or_default(),
            ExportFormat::Human => self.export_human_readable(&metrics).await,
        }
    }

    async fn export_prometheus(&self, metrics: &SystemMetrics) -> String {
        let mut output = String::new();

        // Request metrics
        output.push_str(&format!(
            "shimmy_requests_total {}\n",
            metrics.total_requests
        ));
        output.push_str(&format!(
            "shimmy_requests_successful {}\n",
            metrics.successful_requests
        ));
        output.push_str(&format!(
            "shimmy_requests_failed {}\n",
            metrics.failed_requests
        ));
        output.push_str(&format!(
            "shimmy_response_time_avg {}\n",
            metrics.average_response_time
        ));

        // Resource metrics
        output.push_str(&format!(
            "shimmy_memory_usage_mb {}\n",
            metrics.memory_usage_mb
        ));
        output.push_str(&format!(
            "shimmy_cpu_usage_percent {}\n",
            metrics.cpu_usage_percent
        ));

        // Cache metrics
        output.push_str(&format!("shimmy_cache_hits {}\n", metrics.cache_hits));
        output.push_str(&format!("shimmy_cache_misses {}\n", metrics.cache_misses));
        output.push_str(&format!("shimmy_cache_size_mb {}\n", metrics.cache_size_mb));

        // Model-specific metrics
        for (model, stats) in &metrics.model_stats {
            output.push_str(&format!(
                "shimmy_model_requests{{model=\"{}\"}} {}\n",
                model, stats.requests
            ));
            output.push_str(&format!(
                "shimmy_model_errors{{model=\"{}\"}} {}\n",
                model, stats.errors
            ));
            output.push_str(&format!(
                "shimmy_model_popularity{{model=\"{}\"}} {}\n",
                model, stats.popularity_score
            ));
        }

        output
    }

    async fn export_human_readable(&self, metrics: &SystemMetrics) -> String {
        let mut output = String::new();

        output.push_str("📊 Shimmy Observability Dashboard\n\n");

        // Request summary
        let success_rate = if metrics.total_requests > 0 {
            (metrics.successful_requests as f64 / metrics.total_requests as f64) * 100.0
        } else {
            0.0
        };

        output.push_str(&format!(
            "🚀 Requests: {} total ({:.1}% success)\n",
            metrics.total_requests, success_rate
        ));
        output.push_str(&format!(
            "⚡ Avg Response Time: {:.1}ms\n",
            metrics.average_response_time
        ));

        // Resource usage
        output.push_str(&format!("💾 Memory: {:.1}MB\n", metrics.memory_usage_mb));
        output.push_str(&format!("⚙️  CPU: {:.1}%\n", metrics.cpu_usage_percent));

        // Cache performance
        let cache_hit_rate = if metrics.cache_hits + metrics.cache_misses > 0 {
            (metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64) * 100.0
        } else {
            0.0
        };
        output.push_str(&format!("🎯 Cache Hit Rate: {:.1}%\n", cache_hit_rate));

        // Top models
        output.push_str("\n🎯 Popular Models:\n");
        let mut sorted_models: Vec<_> = metrics.model_stats.iter().collect();
        sorted_models.sort_by(|a, b| {
            b.1.popularity_score
                .partial_cmp(&a.1.popularity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (model, stats) in sorted_models.iter().take(5) {
            output.push_str(&format!(
                "  • {}: {} requests ({:.1} score)\n",
                model, stats.requests, stats.popularity_score
            ));
        }

        output
    }

    /// Start background metrics collection task
    pub fn start_metrics_collector(&self) {
        let manager = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(manager.config.health_check_interval);

            loop {
                interval.tick().await;
                manager.update_system_metrics().await;

                if let Ok(actions) = manager.optimize_system().await {
                    if !actions.is_empty() {
                        info!("Self-optimization suggestions: {:?}", actions);
                    }
                }
            }
        });
    }

    /// Get current metrics snapshot
    pub async fn metrics(&self) -> SystemMetrics {
        self.metrics.read().await.clone()
    }

    // Simplified system monitoring methods (placeholders)
    async fn memory_usage(&self) -> f64 {
        // In production, this would use proper system APIs
        // For now, return a reasonable estimate
        128.0 // 128MB baseline
    }

    async fn cpu_usage(&self) -> f64 {
        // In production, this would calculate actual CPU usage
        15.0 // 15% estimate
    }

    async fn disk_usage(&self) -> f64 {
        // In production, this would check actual disk usage
        25.0 // 25% estimate
    }
}

impl Clone for ObservabilityManager {
    fn clone(&self) -> Self {
        Self {
            metrics: self.metrics.clone(),
            config: self.config.clone(),
            optimization_state: self.optimization_state.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub enum OptimizationAction {
    ReduceCacheSize {
        current_mb: f64,
        recommended_mb: f64,
    },
    UpdatePreloadList {
        models: Vec<String>,
    },
    TunePerformance {
        issue: String,
        recommendation: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_request_recording() {
        let obs = ObservabilityManager::new();

        obs.record_request("phi3-mini", Duration::from_millis(150), true)
            .await;
        obs.record_request("phi3-mini", Duration::from_millis(200), true)
            .await;

        let metrics = obs.metrics().await;
        assert_eq!(metrics.total_requests, 2);
        assert_eq!(metrics.successful_requests, 2);
        assert!(metrics.model_stats.contains_key("phi3-mini"));
    }

    #[tokio::test]
    async fn test_metrics_export() {
        let obs = ObservabilityManager::new();

        obs.record_request("test-model", Duration::from_millis(100), true)
            .await;

        let prometheus_export = obs.export_metrics().await;
        assert!(prometheus_export.contains("shimmy_requests_total"));
        assert!(prometheus_export.contains("test-model"));
    }

    #[tokio::test]
    async fn test_optimization_suggestions() {
        let obs = ObservabilityManager::new();

        // Record high response times to trigger optimization
        for _ in 0..10 {
            obs.record_request("slow-model", Duration::from_millis(1500), true)
                .await;
        }

        let actions = obs.optimize_system().await.unwrap();
        assert!(!actions.is_empty());
    }
}
