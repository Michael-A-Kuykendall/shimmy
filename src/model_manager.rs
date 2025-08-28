use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use crate::engine::{InferenceEngine, ModelSpec};

pub struct ModelManager {
    // For now, keep it simple - store the loaded state info rather than the engines themselves
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
