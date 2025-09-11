#![allow(dead_code)]

use crate::engine::ModelSpec;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ModelManager {
    // Store loaded model information
    loaded_models: Arc<RwLock<HashMap<String, ModelLoadInfo>>>,
}

#[derive(Debug, Clone)]
pub struct ModelLoadInfo {
    pub name: String,
    pub spec: ModelSpec,
    pub loaded_at: std::time::SystemTime,
}

impl ModelManager {
    pub fn new() -> Self {
        Self {
            loaded_models: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn load_model(&self, name: String, spec: ModelSpec) -> Result<()> {
        // For now, just track that we've "loaded" the model
        // In a full implementation, this would create and store the actual engine instance
        let info = ModelLoadInfo {
            name: name.clone(),
            spec,
            loaded_at: std::time::SystemTime::now(),
        };

        let mut models = self.loaded_models.write().await;
        models.insert(name, info);

        Ok(())
    }

    pub async fn unload_model(&self, name: &str) -> Result<bool> {
        let mut models = self.loaded_models.write().await;
        Ok(models.remove(name).is_some())
    }

    pub async fn get_model_info(&self, name: &str) -> Option<ModelLoadInfo> {
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

        let info = manager.get_model_info("model-with-lora").await;
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

        let info = manager.get_model_info("test-model").await;
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

        let info = manager.get_model_info("nonexistent").await;
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
                tokio::spawn(async move {
                    manager_clone
                        .get_model_info(&format!("concurrent-{}", i))
                        .await
                })
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

        let info = manager.get_model_info("test-props").await.unwrap();

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
        let info1 = manager.get_model_info("overwrite-test").await.unwrap();
        assert_eq!(info1.spec.base_path, PathBuf::from("original.gguf"));
        assert!(info1.spec.lora_path.is_none());

        // Overwrite with second version
        manager
            .load_model("overwrite-test".to_string(), spec2)
            .await
            .unwrap();
        let info2 = manager.get_model_info("overwrite-test").await.unwrap();
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
            let info = manager.get_model_info(&format!("large-{}", i)).await;
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

        let info = manager.get_model_info("timing-test").await.unwrap();
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
        let info = manager.get_model_info("").await;
        assert!(info.is_none());

        // Test very long model name
        let long_name = "a".repeat(1000);
        let info = manager.get_model_info(&long_name).await;
        assert!(info.is_none());

        // Test special characters in model name
        let special_name = "model/with:special#chars@test";
        let info = manager.get_model_info(special_name).await;
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
