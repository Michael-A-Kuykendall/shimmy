//! Smart Model Preloading & Warmup System
//!
//! This module implements intelligent model preloading to minimize latency by:
//! - Tracking model usage patterns
//! - Preloading frequently used models
//! - Warming up models with test generations
//! - Managing memory efficiently with LRU eviction

use crate::engine::{ModelSpec, InferenceEngine, LoadedModel, GenOptions};
use crate::error::{Result, ShimmyError};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{SystemTime, Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use tracing::{info, warn, debug};

#[derive(Debug, Clone)]
pub struct ModelUsageStats {
    pub model_name: String,
    pub load_count: u64,
    pub last_used: SystemTime,
    pub avg_load_time: Duration,
    pub total_requests: u64,
    pub warmup_completed: bool,
}

impl Default for ModelUsageStats {
    fn default() -> Self {
        Self {
            model_name: String::new(),
            load_count: 0,
            last_used: SystemTime::UNIX_EPOCH,
            avg_load_time: Duration::from_secs(0),
            total_requests: 0,
            warmup_completed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PreloadingConfig {
    /// Maximum number of models to keep loaded
    pub max_loaded_models: usize,
    /// Minimum usage count before considering for preloading
    pub min_usage_threshold: u64,
    /// How long to keep unused models loaded
    pub model_ttl: Duration,
    /// Enable automatic warmup generation
    pub enable_warmup: bool,
    /// Warmup prompt for testing generation
    pub warmup_prompt: String,
    /// Maximum tokens for warmup generation
    pub warmup_max_tokens: usize,
}

impl Default for PreloadingConfig {
    fn default() -> Self {
        Self {
            max_loaded_models: 3,
            min_usage_threshold: 2,
            model_ttl: Duration::from_secs(300), // 5 minutes
            enable_warmup: true,
            warmup_prompt: "Hello".to_string(),
            warmup_max_tokens: 10,
        }
    }
}

pub struct SmartPreloader {
    /// Configuration for preloading behavior
    config: PreloadingConfig,
    /// Track usage statistics for each model
    usage_stats: Arc<RwLock<HashMap<String, ModelUsageStats>>>,
    /// Currently loaded models with their engines
    loaded_models: Arc<RwLock<HashMap<String, (Arc<dyn LoadedModel>, SystemTime)>>>,
    /// LRU queue for eviction (model_name, last_access_time)
    lru_queue: Arc<Mutex<VecDeque<(String, SystemTime)>>>,
    /// Available model specifications
    available_specs: Arc<RwLock<HashMap<String, ModelSpec>>>,
    /// Inference engine for loading models
    engine: Arc<dyn InferenceEngine>,
}

impl SmartPreloader {
    pub fn new(
        config: PreloadingConfig,
        engine: Arc<dyn InferenceEngine>,
    ) -> Self {
        Self {
            config,
            usage_stats: Arc::new(RwLock::new(HashMap::new())),
            loaded_models: Arc::new(RwLock::new(HashMap::new())),
            lru_queue: Arc::new(Mutex::new(VecDeque::new())),
            available_specs: Arc::new(RwLock::new(HashMap::new())),
            engine,
        }
    }

    /// Register a model specification for potential preloading
    pub async fn register_model(&self, name: String, spec: ModelSpec) {
        let mut specs = self.available_specs.write().await;
        specs.insert(name.clone(), spec);

        // Initialize usage stats if not present
        let mut stats = self.usage_stats.write().await;
        stats.entry(name).or_default();
    }

    /// Record model usage and trigger preloading decisions
    pub async fn record_usage(&self, model_name: &str) -> Result<()> {
        let now = SystemTime::now();

        // Update usage statistics
        {
            let mut stats = self.usage_stats.write().await;
            let entry = stats.entry(model_name.to_string()).or_default();
            entry.total_requests += 1;
            entry.last_used = now;
        }

        // Update LRU queue
        {
            let mut lru = self.lru_queue.lock().await;
            // Remove existing entry if present
            lru.retain(|(name, _)| name != model_name);
            // Add to front (most recently used)
            lru.push_front((model_name.to_string(), now));
        }

        // Trigger intelligent preloading
        self.evaluate_preloading_opportunities().await?;

        Ok(())
    }

    /// Get a loaded model, loading if necessary
    pub async fn model(&self, model_name: &str) -> Result<Arc<dyn LoadedModel>> {
        // Record usage
        self.record_usage(model_name).await?;

        // Check if already loaded
        {
            let loaded = self.loaded_models.read().await;
            if let Some((model, _)) = loaded.get(model_name) {
                debug!("Model {} served from preloaded cache", model_name);
                return Ok(Arc::clone(model));
            }
        }

        // Load the model
        info!("Loading model {} on demand", model_name);
        let start_time = Instant::now();

        let spec = {
            let specs = self.available_specs.read().await;
            specs.get(model_name)
                .cloned()
                .ok_or_else(|| ShimmyError::ModelNotFound { name: model_name.to_string() })?
        };

        let loaded_model = self.engine.load(&spec).await?;
        let load_duration = start_time.elapsed();

        // Update statistics
        {
            let mut stats = self.usage_stats.write().await;
            let entry = stats.entry(model_name.to_string()).or_default();
            entry.load_count += 1;

            // Update average load time
            if entry.load_count == 1 {
                entry.avg_load_time = load_duration;
            } else {
                let total_time = entry.avg_load_time * (entry.load_count - 1) as u32 + load_duration;
                entry.avg_load_time = total_time / entry.load_count as u32;
            }
        }

        // Store in loaded models
        let model_arc = Arc::from(loaded_model);
        {
            let mut loaded = self.loaded_models.write().await;
            loaded.insert(model_name.to_string(), (Arc::clone(&model_arc), SystemTime::now()));
        }

        // Perform warmup if enabled and not done yet
        if self.config.enable_warmup {
            self.warmup_model(model_name, &model_arc).await?;
        }

        // Enforce memory limits
        self.enforce_memory_limits().await?;

        info!("Model {} loaded in {:?}", model_name, load_duration);
        Ok(model_arc)
    }

    /// Perform warmup generation on a model
    async fn warmup_model(&self, model_name: &str, model: &Arc<dyn LoadedModel>) -> Result<()> {
        // Check if warmup already completed
        {
            let stats = self.usage_stats.read().await;
            if let Some(stat) = stats.get(model_name) {
                if stat.warmup_completed {
                    return Ok(());
                }
            }
        }

        debug!("Warming up model {}", model_name);
        let warmup_start = Instant::now();

        let warmup_opts = GenOptions {
            max_tokens: self.config.warmup_max_tokens,
            temperature: 0.1, // Low temperature for consistent warmup
            stream: false,
            ..Default::default()
        };

        // Perform warmup generation
        match model.generate(&self.config.warmup_prompt, warmup_opts, None).await {
            Ok(_) => {
                let warmup_duration = warmup_start.elapsed();
                debug!("Model {} warmed up in {:?}", model_name, warmup_duration);

                // Mark warmup as completed
                let mut stats = self.usage_stats.write().await;
                if let Some(stat) = stats.get_mut(model_name) {
                    stat.warmup_completed = true;
                }
            }
            Err(e) => {
                warn!("Warmup failed for model {}: {}", model_name, e);
                // Don't fail the entire operation on warmup failure
            }
        }

        Ok(())
    }

    /// Evaluate which models should be preloaded
    async fn evaluate_preloading_opportunities(&self) -> Result<()> {
        let candidates = self.identify_preloading_candidates().await;

        for candidate in candidates {
            if !self.is_model_loaded(&candidate).await {
                info!("Preloading model {} based on usage patterns", candidate);

                // Load in background to avoid blocking current request
                let preloader = Arc::new(Self {
                    config: self.config.clone(),
                    usage_stats: Arc::clone(&self.usage_stats),
                    loaded_models: Arc::clone(&self.loaded_models),
                    lru_queue: Arc::clone(&self.lru_queue),
                    available_specs: Arc::clone(&self.available_specs),
                    engine: Arc::clone(&self.engine),
                });

                let model_name = candidate.clone();
                tokio::spawn(async move {
                    if let Err(e) = preloader.preload_model(&model_name).await {
                        warn!("Background preloading failed for {}: {}", model_name, e);
                    }
                });
            }
        }

        Ok(())
    }

    /// Identify models that should be preloaded based on usage patterns
    async fn identify_preloading_candidates(&self) -> Vec<String> {
        let stats = self.usage_stats.read().await;
        let mut candidates = Vec::new();

        for (model_name, stat) in stats.iter() {
            // Check if model meets preloading criteria
            if stat.total_requests >= self.config.min_usage_threshold {
                // Recently used models are good candidates
                if let Ok(since_last_use) = SystemTime::now().duration_since(stat.last_used) {
                    if since_last_use < self.config.model_ttl {
                        candidates.push(model_name.clone());
                    }
                }
            }
        }

        // Sort by usage frequency (descending)
        candidates.sort_by(|a, b| {
            let stat_a = stats.get(a).unwrap_or(&ModelUsageStats::default());
            let stat_b = stats.get(b).unwrap_or(&ModelUsageStats::default());
            stat_b.total_requests.cmp(&stat_a.total_requests)
        });

        // Limit to available memory slots
        let loaded_count = self.loaded_models.read().await.len();
        let available_slots = self.config.max_loaded_models.saturating_sub(loaded_count);

        candidates.truncate(available_slots);
        candidates
    }

    /// Check if a model is currently loaded
    async fn is_model_loaded(&self, model_name: &str) -> bool {
        let loaded = self.loaded_models.read().await;
        loaded.contains_key(model_name)
    }

    /// Preload a specific model in the background
    async fn preload_model(&self, model_name: &str) -> Result<()> {
        let start_time = Instant::now();

        let spec = {
            let specs = self.available_specs.read().await;
            specs.get(model_name)
                .cloned()
                .ok_or_else(|| ShimmyError::ModelNotFound { name: model_name.to_string() })?
        };

        let loaded_model = self.engine.load(&spec).await?;
        let load_duration = start_time.elapsed();

        // Store in loaded models
        {
            let mut loaded = self.loaded_models.write().await;
            loaded.insert(model_name.to_string(), (Arc::from(loaded_model), SystemTime::now()));
        }

        // Perform warmup if enabled
        if self.config.enable_warmup {
            let loaded = self.loaded_models.read().await;
            if let Some((model, _)) = loaded.get(model_name) {
                self.warmup_model(model_name, model).await?;
            }
        }

        info!("Preloaded model {} in {:?}", model_name, load_duration);
        Ok(())
    }

    /// Enforce memory limits by evicting least recently used models
    async fn enforce_memory_limits(&self) -> Result<()> {
        let mut loaded = self.loaded_models.write().await;

        while loaded.len() > self.config.max_loaded_models {
            // Find least recently used model
            let mut oldest_time = SystemTime::now();
            let mut oldest_model = String::new();

            for (model_name, (_, load_time)) in loaded.iter() {
                if *load_time < oldest_time {
                    oldest_time = *load_time;
                    oldest_model = model_name.clone();
                }
            }

            if !oldest_model.is_empty() {
                loaded.remove(&oldest_model);
                info!("Evicted model {} to free memory", oldest_model);
            } else {
                break; // Safety break
            }
        }

        Ok(())
    }

    /// Get usage statistics for all models
    pub async fn usage_stats(&self) -> HashMap<String, ModelUsageStats> {
        self.usage_stats.read().await.clone()
    }

    /// Get currently loaded models count
    pub async fn loaded_model_count(&self) -> usize {
        self.loaded_models.read().await.len()
    }

    /// Clear all loaded models and statistics
    pub async fn clear(&self) -> Result<()> {
        {
            let mut loaded = self.loaded_models.write().await;
            loaded.clear();
        }
        {
            let mut stats = self.usage_stats.write().await;
            stats.clear();
        }
        {
            let mut lru = self.lru_queue.lock().await;
            lru.clear();
        }

        info!("Cleared all preloaded models and statistics");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{LoadedModel, GenOptions};
    use async_trait::async_trait;
    use std::path::PathBuf;

    // Mock implementations for testing
    struct MockLoadedModel {
        name: String,
    }

    #[async_trait]
    impl LoadedModel for MockLoadedModel {
        async fn generate(
            &self,
            _prompt: &str,
            _opts: GenOptions,
            _on_token: Option<Box<dyn FnMut(String) + Send>>,
        ) -> Result<String> {
            // Simulate generation delay
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(format!("Generated from {}", self.name))
        }
    }

    struct MockEngine;

    #[async_trait]
    impl InferenceEngine for MockEngine {
        async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
            // Simulate loading delay
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok(Box::new(MockLoadedModel {
                name: spec.name.clone(),
            }))
        }
    }

    #[tokio::test]
    async fn test_preloader_creation() {
        let config = PreloadingConfig::default();
        let engine = Arc::new(MockEngine);
        let preloader = SmartPreloader::new(config, engine);

        assert_eq!(preloader.loaded_model_count().await, 0);
    }

    #[tokio::test]
    async fn test_model_registration() {
        let config = PreloadingConfig::default();
        let engine = Arc::new(MockEngine);
        let preloader = SmartPreloader::new(config, engine);

        let spec = ModelSpec {
            name: "test-model".to_string(),
            base_path: PathBuf::from("test.gguf"),
            lora_path: None,
            template: None,
            ctx_len: 2048,
            n_threads: Some(4),
        };

        preloader.register_model("test-model".to_string(), spec).await;

        let stats = preloader.usage_stats().await;
        assert!(stats.contains_key("test-model"));
    }

    #[tokio::test]
    async fn test_model_loading_and_caching() {
        let config = PreloadingConfig::default();
        let engine = Arc::new(MockEngine);
        let preloader = SmartPreloader::new(config, engine);

        let spec = ModelSpec {
            name: "cache-test".to_string(),
            base_path: PathBuf::from("cache-test.gguf"),
            lora_path: None,
            template: None,
            ctx_len: 2048,
            n_threads: Some(4),
        };

        preloader.register_model("cache-test".to_string(), spec).await;

        // First load should actually load the model
        let start = Instant::now();
        let model1 = preloader.model("cache-test").await.unwrap();
        let first_load_time = start.elapsed();

        // Second load should be faster (cached)
        let start = Instant::now();
        let model2 = preloader.model("cache-test").await.unwrap();
        let second_load_time = start.elapsed();

        // Verify caching worked
        assert!(second_load_time < first_load_time);
        assert_eq!(preloader.loaded_model_count().await, 1);

        // Test generation works
        let result = model1.generate("test", GenOptions::default(), None).await.unwrap();
        assert!(result.contains("Generated from cache-test"));
    }

    #[tokio::test]
    async fn test_usage_tracking() {
        let config = PreloadingConfig::default();
        let engine = Arc::new(MockEngine);
        let preloader = SmartPreloader::new(config, engine);

        let spec = ModelSpec {
            name: "usage-test".to_string(),
            base_path: PathBuf::from("usage-test.gguf"),
            lora_path: None,
            template: None,
            ctx_len: 2048,
            n_threads: Some(4),
        };

        preloader.register_model("usage-test".to_string(), spec).await;

        // Use the model multiple times
        for _ in 0..3 {
            preloader.model("usage-test").await.unwrap();
        }

        let stats = preloader.usage_stats().await;
        let usage_stat = stats.get("usage-test").unwrap();

        assert_eq!(usage_stat.total_requests, 3);
        assert_eq!(usage_stat.load_count, 1); // Should only load once due to caching
        assert!(usage_stat.warmup_completed);
    }

    #[tokio::test]
    async fn test_memory_limits() {
        let config = PreloadingConfig {
            max_loaded_models: 2,
            ..Default::default()
        };
        let engine = Arc::new(MockEngine);
        let preloader = SmartPreloader::new(config, engine);

        // Register 3 models
        for i in 0..3 {
            let spec = ModelSpec {
                name: format!("model-{}", i),
                base_path: PathBuf::from(format!("model-{}.gguf", i)),
                lora_path: None,
                template: None,
                ctx_len: 2048,
                n_threads: Some(4),
            };
            preloader.register_model(format!("model-{}", i), spec).await;
        }

        // Load all 3 models
        for i in 0..3 {
            preloader.model(&format!("model-{}", i)).await.unwrap();
            // Small delay to ensure different load times
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        // Should only have 2 models loaded due to memory limit
        assert_eq!(preloader.loaded_model_count().await, 2);
    }

    #[tokio::test]
    async fn test_preloading_candidates() {
        let config = PreloadingConfig {
            min_usage_threshold: 2,
            max_loaded_models: 5,
            ..Default::default()
        };
        let engine = Arc::new(MockEngine);
        let preloader = SmartPreloader::new(config, engine);

        // Register models
        for i in 0..3 {
            let spec = ModelSpec {
                name: format!("candidate-{}", i),
                base_path: PathBuf::from(format!("candidate-{}.gguf", i)),
                lora_path: None,
                template: None,
                ctx_len: 2048,
                n_threads: Some(4),
            };
            preloader.register_model(format!("candidate-{}", i), spec).await;
        }

        // Use models with different frequencies
        for _ in 0..3 {
            preloader.record_usage("candidate-0").await.unwrap();
        }
        for _ in 0..2 {
            preloader.record_usage("candidate-1").await.unwrap();
        }
        preloader.record_usage("candidate-2").await.unwrap();

        let candidates = preloader.identify_preloading_candidates().await;

        // candidate-0 and candidate-1 should be candidates (>= min_usage_threshold)
        // candidate-2 should not be (only 1 usage)
        assert_eq!(candidates.len(), 2);
        assert!(candidates.contains(&"candidate-0".to_string()));
        assert!(candidates.contains(&"candidate-1".to_string()));
        assert!(!candidates.contains(&"candidate-2".to_string()));
    }

    #[tokio::test]
    async fn test_clear_functionality() {
        let config = PreloadingConfig::default();
        let engine = Arc::new(MockEngine);
        let preloader = SmartPreloader::new(config, engine);

        let spec = ModelSpec {
            name: "clear-test".to_string(),
            base_path: PathBuf::from("clear-test.gguf"),
            lora_path: None,
            template: None,
            ctx_len: 2048,
            n_threads: Some(4),
        };

        preloader.register_model("clear-test".to_string(), spec).await;
        preloader.model("clear-test").await.unwrap();

        assert_eq!(preloader.loaded_model_count().await, 1);
        assert!(!preloader.usage_stats().await.is_empty());

        preloader.clear().await.unwrap();

        assert_eq!(preloader.loaded_model_count().await, 0);
        assert!(preloader.usage_stats().await.is_empty());
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let config = PreloadingConfig::default();
        let engine = Arc::new(MockEngine);
        let preloader = Arc::new(SmartPreloader::new(config, engine));

        let spec = ModelSpec {
            name: "concurrent-test".to_string(),
            base_path: PathBuf::from("concurrent-test.gguf"),
            lora_path: None,
            template: None,
            ctx_len: 2048,
            n_threads: Some(4),
        };

        preloader.register_model("concurrent-test".to_string(), spec).await;

        // Spawn multiple concurrent requests for the same model
        let mut handles = vec![];
        for _ in 0..10 {
            let preloader_clone = Arc::clone(&preloader);
            let handle = tokio::spawn(async move {
                preloader_clone.model("concurrent-test").await
            });
            handles.push(handle);
        }

        // All requests should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        // Should only load the model once
        let stats = preloader.usage_stats().await;
        let usage_stat = stats.get("concurrent-test").unwrap();
        assert_eq!(usage_stat.load_count, 1);
        assert_eq!(usage_stat.total_requests, 10);
    }
}
