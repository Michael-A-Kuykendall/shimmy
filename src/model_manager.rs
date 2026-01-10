#![allow(dead_code)]

use crate::engine::ModelSpec;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Clone)]
pub struct ModelManager {
    // Store loaded model information
    loaded_models: Arc<RwLock<HashMap<String, ModelLoadInfo>>>,
    // Usage tracking for smart preloading
    usage_stats: Arc<RwLock<HashMap<String, ModelUsageStats>>>,
    // Configuration for preloading behavior
    preload_config: PreloadConfig,
    // Background preloading state
    preload_queue: Arc<RwLock<VecDeque<String>>>,
}

#[derive(Debug, Clone)]
pub struct ModelLoadInfo {
    pub name: String,
    pub spec: ModelSpec,
    pub loaded_at: std::time::SystemTime,
    pub last_accessed: std::time::SystemTime,
    pub access_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsageStats {
    pub model_name: String,
    pub total_requests: u64,
    pub last_used: SystemTime,
    pub average_response_time: Duration,
    pub popularity_score: f64,
}

#[derive(Debug, Clone)]
pub struct PreloadConfig {
    pub enabled: bool,
    pub max_preloaded_models: usize,
    pub max_memory_mb: usize,
    pub preload_threshold_score: f64,
    pub min_usage_for_preload: u64,
    pub cleanup_interval: Duration,
}

impl Default for PreloadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_preloaded_models: 3,
            max_memory_mb: 8192, // 8GB default
            preload_threshold_score: 0.5,
            min_usage_for_preload: 2,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreloadStats {
    pub loaded_models: usize,
    pub max_models: usize,
    pub queue_length: usize,
    pub total_tracked_models: usize,
    pub memory_limit_mb: usize,
    pub preloading_enabled: bool,
}

impl ModelManager {
    pub fn new() -> Self {
        Self::with_config(PreloadConfig::default())
    }

    pub fn with_config(config: PreloadConfig) -> Self {
        Self {
            loaded_models: Arc::new(RwLock::new(HashMap::new())),
            usage_stats: Arc::new(RwLock::new(HashMap::new())),
            preload_config: config,
            preload_queue: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub async fn load_model(&self, name: String, spec: ModelSpec) -> Result<()> {
        let now = SystemTime::now();

        // Create model load info with usage tracking
        let info = ModelLoadInfo {
            name: name.clone(),
            spec,
            loaded_at: now,
            last_accessed: now,
            access_count: 1,
        };

        // Store the loaded model
        {
            let mut models = self.loaded_models.write().await;
            models.insert(name.clone(), info);
        } // Drop write lock before calling other functions

        info!("Model '{}' loaded successfully", name);

        // Update usage statistics
        self.update_usage_stats(&name, Duration::from_millis(100))
            .await;

        // Trigger background preloading evaluation
        if self.preload_config.enabled {
            self.evaluate_preloading().await;
        }

        Ok(())
    }

    /// Record model access for usage tracking
    pub async fn record_access(&self, name: &str, response_time: Duration) {
        // Update loaded model access info
        let mut models = self.loaded_models.write().await;
        if let Some(info) = models.get_mut(name) {
            info.last_accessed = SystemTime::now();
            info.access_count += 1;
        }
        drop(models);

        // Update usage statistics
        self.update_usage_stats(name, response_time).await;
    }

    /// Update usage statistics for smart preloading
    async fn update_usage_stats(&self, name: &str, response_time: Duration) {
        let mut stats = self.usage_stats.write().await;
        let entry = stats
            .entry(name.to_string())
            .or_insert_with(|| ModelUsageStats {
                model_name: name.to_string(),
                total_requests: 0,
                last_used: SystemTime::now(),
                average_response_time: Duration::from_millis(0),
                popularity_score: 0.0,
            });

        entry.total_requests += 1;
        entry.last_used = SystemTime::now();

        // Update rolling average response time
        let current_avg_ms = entry.average_response_time.as_millis() as f64;
        let new_response_ms = response_time.as_millis() as f64;
        let new_avg_ms = (current_avg_ms * (entry.total_requests - 1) as f64 + new_response_ms)
            / entry.total_requests as f64;
        entry.average_response_time = Duration::from_millis(new_avg_ms as u64);

        // Calculate popularity score (frequency + recency)
        let time_since_last_use = SystemTime::now()
            .duration_since(entry.last_used)
            .unwrap_or_default()
            .as_secs() as f64;
        let recency_factor = 1.0 / (1.0 + time_since_last_use / 3600.0); // Decay over hours
        let frequency_factor = (entry.total_requests as f64).ln() + 1.0;
        entry.popularity_score = frequency_factor * recency_factor;
    }

    /// Evaluate which models should be preloaded
    async fn evaluate_preloading(&self) {
        if !self.preload_config.enabled {
            return;
        }

        // Collect candidates while holding read locks, then drop them
        let (candidates_to_queue, current_loaded) = {
            let stats = self.usage_stats.read().await;
            let loaded_models = self.loaded_models.read().await;

            // Find top candidates for preloading
            let mut candidates: Vec<_> = stats
                .values()
                .filter(|stat| {
                    stat.total_requests >= self.preload_config.min_usage_for_preload
                        && stat.popularity_score >= self.preload_config.preload_threshold_score
                        && !loaded_models.contains_key(&stat.model_name)
                })
                .collect();

            // Sort by popularity score
            candidates.sort_by(|a, b| {
                b.popularity_score
                    .partial_cmp(&a.popularity_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let current_loaded = loaded_models.len();
            let candidates_vec: Vec<_> = candidates
                .iter()
                .map(|c| (c.model_name.clone(), c.popularity_score))
                .collect();

            (candidates_vec, current_loaded)
        }; // Drop read locks before acquiring write lock

        // Add to preload queue
        let mut queue = self.preload_queue.write().await;
        let slots_available = self
            .preload_config
            .max_preloaded_models
            .saturating_sub(current_loaded);

        for (model_name, score) in candidates_to_queue.iter().take(slots_available) {
            if !queue.iter().any(|name| name == model_name) {
                queue.push_back(model_name.clone());
                info!(
                    "Queued '{}' for preloading (score: {:.2})",
                    model_name, score
                );
            }
        }
    }

    /// Start background preloading task
    pub async fn start_preloading_task(
        &self,
        model_registry: Arc<crate::model_registry::Registry>,
    ) {
        if !self.preload_config.enabled {
            return;
        }

        let manager = Arc::new(self.clone());
        let registry = model_registry.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;

                // Process preload queue
                let model_to_preload = {
                    let mut queue = manager.preload_queue.write().await;
                    queue.pop_front()
                };

                if let Some(model_name) = model_to_preload {
                    // Check if we're under memory/count limits
                    let current_count = manager.model_count().await;
                    if current_count < manager.preload_config.max_preloaded_models {
                        // Try to preload the model
                        if let Some(spec) = registry.to_spec(&model_name) {
                            match manager.load_model(model_name.clone(), spec).await {
                                Ok(_) => {
                                    info!("Successfully preloaded model '{}'", model_name);
                                }
                                Err(e) => {
                                    warn!("Failed to preload model '{}': {}", model_name, e);
                                }
                            }
                        }
                    } else {
                        // Put it back in queue for later
                        let mut queue = manager.preload_queue.write().await;
                        queue.push_front(model_name);
                    }
                }

                // Cleanup old models if needed
                manager.cleanup_old_models().await;
            }
        });
    }

    /// Cleanup old/unused models to free memory
    async fn cleanup_old_models(&self) {
        let current_count = self.model_count().await;
        if current_count <= self.preload_config.max_preloaded_models {
            return;
        }

        let mut models = self.loaded_models.write().await;
        let cutoff_time = SystemTime::now() - Duration::from_secs(3600); // 1 hour

        // Find models to remove (oldest, least used)
        let mut candidates: Vec<_> = models
            .iter()
            .filter(|(_, info)| info.last_accessed < cutoff_time && info.access_count < 5)
            .map(|(name, info)| (name.clone(), info.last_accessed, info.access_count))
            .collect();

        candidates.sort_by_key(|(_, last_accessed, access_count)| (*last_accessed, *access_count));

        let to_remove = current_count.saturating_sub(self.preload_config.max_preloaded_models);
        for (name, _, _) in candidates.iter().take(to_remove) {
            models.remove(name);
            info!("Cleaned up unused model '{}'", name);
        }
    }

    /// Get preloading statistics
    pub async fn preload_stats(&self) -> PreloadStats {
        let models = self.loaded_models.read().await;
        let stats = self.usage_stats.read().await;
        let queue = self.preload_queue.read().await;

        PreloadStats {
            loaded_models: models.len(),
            max_models: self.preload_config.max_preloaded_models,
            queue_length: queue.len(),
            total_tracked_models: stats.len(),
            memory_limit_mb: self.preload_config.max_memory_mb,
            preloading_enabled: self.preload_config.enabled,
        }
    }

    pub async fn unload_model(&self, name: &str) -> Result<bool> {
        let mut models = self.loaded_models.write().await;
        let removed = models.remove(name).is_some();
        if removed {
            info!("Model '{}' unloaded", name);
        }
        Ok(removed)
    }

    pub async fn model_info(&self, name: &str) -> Option<ModelLoadInfo> {
        let models = self.loaded_models.read().await;
        models.get(name).cloned()
    }

    pub async fn list_loaded_models(&self) -> Vec<String> {
        let models = self.loaded_models.read().await;
        models.keys().cloned().collect()
    }

    pub async fn is_loaded(&self, name: &str) -> bool {
        let models = self.loaded_models.read().await;
        models.contains_key(name)
    }

    pub async fn model_count(&self) -> usize {
        let models = self.loaded_models.read().await;
        models.len()
    }
}

impl Default for ModelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    // Helper function to create test ModelSpec
    fn create_test_spec(name: &str, base_file: &str, lora_file: Option<&str>) -> ModelSpec {
        ModelSpec {
            name: name.to_string(),
            base_path: PathBuf::from(base_file),
            lora_path: lora_file.map(PathBuf::from),
            template: None,
            ctx_len: 2048,
            n_threads: None,
        }
    }

    // Helper function to create test ModelLoadInfo
    fn create_test_load_info(name: &str, spec: ModelSpec) -> ModelLoadInfo {
        let now = SystemTime::now();
        ModelLoadInfo {
            name: name.to_string(),
            spec,
            loaded_at: now,
            last_accessed: now,
            access_count: 1,
        }
    }

    #[tokio::test]
    async fn test_model_manager_creation() {
        let manager = ModelManager::new();
        let count = manager.model_count().await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_model_loading_status() {
        let manager = ModelManager::new();
        let is_loaded = manager.is_loaded("nonexistent").await;
        assert!(!is_loaded);
    }

    #[test]
    fn test_model_path_validation() {
        let path = std::path::Path::new("test.gguf");
        assert_eq!(path.extension().unwrap(), "gguf");
    }

    #[tokio::test]
    async fn test_load_model_success() {
        let manager = ModelManager::new();
        let spec = ModelSpec {
            name: "test-model".to_string(),
            base_path: PathBuf::from("test.gguf"),
            lora_path: None,
            template: None,
            ctx_len: 2048,
            n_threads: None,
        };

        let result = manager.load_model("test-model".to_string(), spec).await;
        assert!(result.is_ok());

        let count = manager.model_count().await;
        assert_eq!(count, 1);

        let is_loaded = manager.is_loaded("test-model").await;
        assert!(is_loaded);
    }

    #[tokio::test]
    async fn test_load_model_with_lora() {
        let manager = ModelManager::new();
        let spec = create_test_spec("model-with-lora", "base.gguf", Some("lora.safetensors"));

        let result = manager
            .load_model("model-with-lora".to_string(), spec)
            .await;
        assert!(result.is_ok());

        let info = manager.model_info("model-with-lora").await;
        assert!(info.is_some());
        assert!(info.unwrap().spec.lora_path.is_some());
    }

    #[tokio::test]
    async fn test_load_multiple_models() {
        let manager = ModelManager::new();

        for i in 0..5 {
            let spec = create_test_spec(&format!("model-{}", i), &format!("model{}.gguf", i), None);
            let result = manager.load_model(format!("model-{}", i), spec).await;
            assert!(result.is_ok());
        }

        let count = manager.model_count().await;
        assert_eq!(count, 5);

        let loaded_models = manager.list_loaded_models().await;
        assert_eq!(loaded_models.len(), 5);
    }

    #[tokio::test]
    async fn test_unload_model_success() {
        let manager = ModelManager::new();
        let spec = ModelSpec {
            name: "test-model".to_string(),
            base_path: PathBuf::from("test.gguf"),
            lora_path: None,
            template: None,
            ctx_len: 2048,
            n_threads: None,
        };

        manager
            .load_model("test-model".to_string(), spec)
            .await
            .unwrap();
        assert!(manager.is_loaded("test-model").await);

        let unload_result = manager.unload_model("test-model").await;
        assert!(unload_result.is_ok());
        assert!(unload_result.unwrap());

        assert!(!manager.is_loaded("test-model").await);
        assert_eq!(manager.model_count().await, 0);
    }

    #[tokio::test]
    async fn test_unload_nonexistent_model() {
        let manager = ModelManager::new();

        let result = manager.unload_model("nonexistent").await;
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false for non-existent model
    }

    #[tokio::test]
    async fn test_get_model_info_existing() {
        let manager = ModelManager::new();
        let spec = create_test_spec("test-model", "test.gguf", Some("adapter.safetensors"));

        manager
            .load_model("test-model".to_string(), spec.clone())
            .await
            .unwrap();

        let info = manager.model_info("test-model").await;
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.name, "test-model");
        assert_eq!(info.spec.base_path, spec.base_path);
        assert_eq!(info.spec.lora_path, spec.lora_path);
        assert!(info.loaded_at <= SystemTime::now());
    }

    #[tokio::test]
    async fn test_get_model_info_nonexistent() {
        let manager = ModelManager::new();

        let info = manager.model_info("nonexistent").await;
        assert!(info.is_none());
    }

    #[tokio::test]
    async fn test_list_loaded_models_empty() {
        let manager = ModelManager::new();

        let models = manager.list_loaded_models().await;
        assert!(models.is_empty());
    }

    #[tokio::test]
    async fn test_list_loaded_models_populated() {
        let manager = ModelManager::new();

        let model_names = vec!["model-a", "model-b", "model-c"];
        for name in &model_names {
            let spec = create_test_spec(name, &format!("{}.gguf", name), None);
            manager.load_model(name.to_string(), spec).await.unwrap();
        }

        let mut loaded = manager.list_loaded_models().await;
        loaded.sort();
        let mut expected = model_names
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        expected.sort();

        assert_eq!(loaded, expected);
    }

    #[tokio::test]
    async fn test_model_count_progression() {
        let manager = ModelManager::new();

        // Start with 0
        assert_eq!(manager.model_count().await, 0);

        // Load 3 models
        for i in 0..3 {
            let spec = create_test_spec(&format!("model-{}", i), &format!("model{}.gguf", i), None);
            manager
                .load_model(format!("model-{}", i), spec)
                .await
                .unwrap();
            assert_eq!(manager.model_count().await, i + 1);
        }

        // Unload 1 model
        manager.unload_model("model-1").await.unwrap();
        assert_eq!(manager.model_count().await, 2);
    }

    #[tokio::test]
    async fn test_concurrent_model_operations() {
        let manager = Arc::new(ModelManager::new());
        let mut handles = vec![];

        // Load models concurrently
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let spec = create_test_spec(
                    &format!("concurrent-ops-{}", i),
                    &format!("concurrent{}.gguf", i),
                    None,
                );
                manager_clone
                    .load_model(format!("concurrent-{}", i), spec)
                    .await
            });
            handles.push(handle);
        }

        // Wait for all loads to complete
        for handle in handles {
            assert!(handle.await.unwrap().is_ok());
        }

        assert_eq!(manager.model_count().await, 10);

        // Test concurrent access
        let info_handles: Vec<_> = (0..10)
            .map(|i| {
                let manager_clone = Arc::clone(&manager);
                tokio::spawn(
                    async move { manager_clone.model_info(&format!("concurrent-{}", i)).await },
                )
            })
            .collect();

        for handle in info_handles {
            let info = handle.await.unwrap();
            assert!(info.is_some());
        }
    }

    #[tokio::test]
    async fn test_model_load_info_properties() {
        let manager = ModelManager::new();
        let before_load = SystemTime::now();

        let spec = create_test_spec(
            "test-props",
            "test-props.gguf",
            Some("test-lora.safetensors"),
        );

        manager
            .load_model("test-props".to_string(), spec.clone())
            .await
            .unwrap();

        let info = manager.model_info("test-props").await.unwrap();

        assert_eq!(info.name, "test-props");
        assert_eq!(info.spec.base_path, PathBuf::from("test-props.gguf"));
        assert_eq!(
            info.spec.lora_path,
            Some(PathBuf::from("test-lora.safetensors"))
        );
        assert!(info.loaded_at >= before_load);
        assert!(info.loaded_at <= SystemTime::now());
    }

    #[tokio::test]
    async fn test_model_load_info_clone() {
        let spec = create_test_spec("clone-test", "clone-test.gguf", None);

        let info1 = ModelLoadInfo {
            name: "clone-test".to_string(),
            spec: spec.clone(),
            loaded_at: SystemTime::now(),
            access_count: 0,
            last_accessed: SystemTime::now(),
        };

        let info2 = info1.clone();
        assert_eq!(info1.name, info2.name);
        assert_eq!(info1.spec.base_path, info2.spec.base_path);
        assert_eq!(info1.spec.lora_path, info2.spec.lora_path);
    }

    #[tokio::test]
    async fn test_model_load_info_debug() {
        let spec = create_test_spec("debug-test", "debug-test.gguf", None);

        let info = ModelLoadInfo {
            name: "debug-test".to_string(),
            spec,
            loaded_at: SystemTime::now(),
            access_count: 0,
            last_accessed: SystemTime::now(),
        };

        let debug_string = format!("{:?}", info);
        assert!(debug_string.contains("debug-test"));
        assert!(debug_string.contains("debug-test.gguf"));
        assert!(debug_string.contains("ModelLoadInfo"));
    }

    #[test]
    fn test_model_manager_default() {
        let manager = ModelManager::default();
        // Can't easily test async behavior in sync test, just verify creation
        assert!(manager.loaded_models.try_read().is_ok());
    }

    #[tokio::test]
    async fn test_model_overwrite() {
        let manager = ModelManager::new();

        let spec1 = create_test_spec("overwrite-test", "original.gguf", None);
        let spec2 = create_test_spec(
            "overwrite-test",
            "updated.gguf",
            Some("new-lora.safetensors"),
        );

        // Load first version
        manager
            .load_model("overwrite-test".to_string(), spec1)
            .await
            .unwrap();
        let info1 = manager.model_info("overwrite-test").await.unwrap();
        assert_eq!(info1.spec.base_path, PathBuf::from("original.gguf"));
        assert!(info1.spec.lora_path.is_none());

        // Overwrite with second version
        manager
            .load_model("overwrite-test".to_string(), spec2)
            .await
            .unwrap();
        let info2 = manager.model_info("overwrite-test").await.unwrap();
        assert_eq!(info2.spec.base_path, PathBuf::from("updated.gguf"));
        assert_eq!(
            info2.spec.lora_path,
            Some(PathBuf::from("new-lora.safetensors"))
        );

        // Should still have only 1 model
        assert_eq!(manager.model_count().await, 1);
    }

    #[tokio::test]
    async fn test_large_model_collection() {
        let manager = ModelManager::new();

        // Load 100 models to test scalability
        for i in 0..100 {
            let lora_file = if i % 3 == 0 {
                Some(format!("lora-{}.safetensors", i))
            } else {
                None
            };
            let spec = create_test_spec(
                &format!("large-{}", i),
                &format!("large-collection-{}.gguf", i),
                lora_file.as_deref(),
            );

            let result = manager.load_model(format!("large-{}", i), spec).await;
            assert!(result.is_ok());
        }

        assert_eq!(manager.model_count().await, 100);

        // Verify all models are properly loaded
        for i in 0..100 {
            assert!(manager.is_loaded(&format!("large-{}", i)).await);
            let info = manager.model_info(&format!("large-{}", i)).await;
            assert!(info.is_some());

            let info = info.unwrap();
            assert_eq!(info.name, format!("large-{}", i));
            if i % 3 == 0 {
                assert!(info.spec.lora_path.is_some());
            } else {
                assert!(info.spec.lora_path.is_none());
            }
        }

        // Test bulk unload
        for i in 0..50 {
            let unload_result = manager.unload_model(&format!("large-{}", i)).await;
            assert!(unload_result.is_ok());
            assert!(unload_result.unwrap());
        }

        assert_eq!(manager.model_count().await, 50);
    }

    #[tokio::test]
    async fn test_model_load_info_timing() {
        let manager = ModelManager::new();
        let before_load = SystemTime::now();

        std::thread::sleep(Duration::from_millis(10)); // Small delay to ensure timing difference

        let spec = create_test_spec("timing-test", "timing-test.gguf", None);

        manager
            .load_model("timing-test".to_string(), spec)
            .await
            .unwrap();

        std::thread::sleep(Duration::from_millis(10)); // Small delay to ensure timing difference
        let after_load = SystemTime::now();

        let info = manager.model_info("timing-test").await.unwrap();
        assert!(info.loaded_at > before_load);
        assert!(info.loaded_at < after_load);
    }

    #[tokio::test]
    async fn test_list_loaded_models_ordering() {
        let manager = ModelManager::new();

        // Load models in specific order
        let model_names = vec!["zebra", "alpha", "middle", "beta"];
        for name in &model_names {
            let spec = create_test_spec(name, &format!("{}.gguf", name), None);
            manager.load_model(name.to_string(), spec).await.unwrap();
        }

        let loaded = manager.list_loaded_models().await;
        assert_eq!(loaded.len(), 4);

        // All models should be present (order may vary due to HashMap)
        for name in &model_names {
            assert!(loaded.contains(&name.to_string()));
        }
    }

    #[tokio::test]
    async fn test_model_info_edge_cases() {
        let manager = ModelManager::new();

        // Test empty string model name
        let info = manager.model_info("").await;
        assert!(info.is_none());

        // Test very long model name
        let long_name = "a".repeat(1000);
        let info = manager.model_info(&long_name).await;
        assert!(info.is_none());

        // Test special characters in model name
        let special_name = "model/with:special#chars@test";
        let info = manager.model_info(special_name).await;
        assert!(info.is_none());
    }

    #[tokio::test]
    async fn test_concurrent_load_unload() {
        let manager = Arc::new(ModelManager::new());
        let mut handles = vec![];

        // Concurrent load and unload operations
        for i in 0..20 {
            let manager_clone = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let spec = create_test_spec(
                    &format!("concurrent-ops-{}", i),
                    &format!("concurrent-ops-{}.gguf", i),
                    None,
                );

                // Load
                let load_result = manager_clone
                    .load_model(format!("concurrent-ops-{}", i), spec)
                    .await;
                assert!(load_result.is_ok());

                // Check loaded
                assert!(
                    manager_clone
                        .is_loaded(&format!("concurrent-ops-{}", i))
                        .await
                );

                // Unload every other model
                if i % 2 == 0 {
                    let unload_result = manager_clone
                        .unload_model(&format!("concurrent-ops-{}", i))
                        .await;
                    assert!(unload_result.is_ok());
                    assert!(unload_result.unwrap());
                }
            });
            handles.push(handle);
        }

        // Wait for all operations
        for handle in handles {
            handle.await.unwrap();
        }

        // Should have 10 models remaining (even numbers unloaded)
        assert_eq!(manager.model_count().await, 10);
    }

    #[test]
    fn test_model_spec_paths() {
        let spec = create_test_spec(
            "test-spec",
            "/absolute/path/model.gguf",
            Some("./relative/lora.safetensors"),
        );

        assert!(spec.base_path.to_string_lossy().contains("model.gguf"));
        assert!(spec
            .lora_path
            .as_ref()
            .unwrap()
            .to_string_lossy()
            .contains("lora.safetensors"));
    }
}
