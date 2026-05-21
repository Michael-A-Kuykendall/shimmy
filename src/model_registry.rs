use super::engine::ModelSpec;
use crate::auto_discovery::{DiscoveredModel, ModelAutoDiscovery};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

/// Read `SHIMMY_MAX_CTX` env var and return a validated context window size.
/// Accepted range: 512–131072 tokens. Defaults to 2048 when unset or invalid.
pub fn shimmy_ctx_len() -> usize {
    std::env::var("SHIMMY_MAX_CTX")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&c| c >= 512 && c <= 131_072)
        .unwrap_or(2048)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub name: String,
    pub base_path: PathBuf,
    pub lora_path: Option<PathBuf>,
    pub template: Option<String>,
    pub ctx_len: Option<usize>,
    pub n_threads: Option<i32>,
}

#[derive(Default, Clone)]
pub struct Registry {
    inner: HashMap<String, ModelEntry>,
    pub discovered_models: HashMap<String, DiscoveredModel>,
}

// Alias for backward compatibility and mission expectations; use `Registry` directly.
impl Registry {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            discovered_models: HashMap::new(),
        }
    }

    pub fn with_discovery() -> Self {
        let mut registry = Self::new();
        registry.refresh_discovered_models();
        registry
    }

    pub fn refresh_discovered_models(&mut self) {
        let discovery = ModelAutoDiscovery::new();
        if let Ok(models) = discovery.discover_models() {
            self.discovered_models.clear();
            for model in models {
                self.discovered_models.insert(model.name.clone(), model);
            }
        }
    }

    pub fn auto_register_discovered(&mut self) {
        // Convert discovered models to registry entries
        for (name, discovered) in &self.discovered_models {
            if !self.inner.contains_key(name) {
                let entry = ModelEntry {
                    name: name.clone(),
                    base_path: discovered.path.clone(),
                    lora_path: discovered.lora_path.clone(),
                    template: Some(self.infer_template(name)),
                    ctx_len: Some(shimmy_ctx_len()),
                    n_threads: None,
                };
                self.inner.insert(name.clone(), entry);
            }
        }
    }

    pub fn infer_template(&self, model_name: &str) -> String {
        let name_lower = model_name.to_lowercase();

        // Only route to llama3 template when explicitly Llama 3 (uses different special tokens)
        if name_lower.contains("llama-3") || name_lower.contains("llama3") || name_lower.contains("meta-llama-3") {
            "llama3".to_string()
        } else {
            // Everything else (TinyLlama, Llama 1/2, Mistral, Phi, Qwen, etc.) uses ChatML
            "chatml".to_string()
        }
    }

    pub fn register(&mut self, e: ModelEntry) {
        self.inner.insert(e.name.clone(), e);
    }
    pub fn get(&self, name: &str) -> Option<&ModelEntry> {
        // First check manually registered models, then auto-discovered
        self.inner.get(name)
    }
    pub fn list(&self) -> Vec<&ModelEntry> {
        self.inner.values().collect()
    }
    pub fn list_all_available(&self) -> Vec<String> {
        let mut available = Vec::new();
        available.extend(self.inner.keys().cloned());
        available.extend(self.discovered_models.keys().cloned());
        available.sort();
        available.dedup();
        available
    }

    pub fn to_spec(&self, name: &str) -> Option<ModelSpec> {
        // Try manually registered first
        if let Some(e) = self.inner.get(name) {
            return Some(ModelSpec {
                name: e.name.clone(),
                base_path: e.base_path.clone(),
                lora_path: e.lora_path.clone(),
                template: e.template.clone(),
                ctx_len: e.ctx_len.unwrap_or_else(shimmy_ctx_len),
                n_threads: e.n_threads,
            });
        }

        // Fall back to discovered models
        if let Some(discovered) = self.discovered_models.get(name) {
            return Some(ModelSpec {
                name: discovered.name.clone(),
                base_path: discovered.path.clone(),
                lora_path: discovered.lora_path.clone(),
                template: Some(self.infer_template(&discovered.name)),
                ctx_len: shimmy_ctx_len(),
                n_threads: None,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = Registry::new();
        assert!(registry.inner.is_empty());
        assert!(registry.discovered_models.is_empty());
    }

    #[test]
    fn test_registry_default() {
        let registry = Registry::default();
        assert!(registry.inner.is_empty());
    }

    #[test]
    fn test_register_model() {
        let mut registry = Registry::new();
        let entry = ModelEntry {
            name: "test-model".to_string(),
            base_path: PathBuf::from("/path/to/model"),
            lora_path: None,
            template: Some("chatml".to_string()),
            ctx_len: Some(2048),
            n_threads: Some(4),
        };

        registry.register(entry.clone());
        assert_eq!(registry.inner.len(), 1);
        assert!(registry.get("test-model").is_some());
    }

    #[test]
    fn test_list_models() {
        let mut registry = Registry::new();
        let entry = ModelEntry {
            name: "test".to_string(),
            base_path: PathBuf::from("/test"),
            lora_path: None,
            template: None,
            ctx_len: None,
            n_threads: None,
        };

        registry.register(entry);
        let models = registry.list();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "test");
    }

    #[test]
    fn test_infer_template_llama3_variants() {
        let registry = Registry::new();
        assert_eq!(registry.infer_template("llama-3-8b"), "llama3");
        assert_eq!(registry.infer_template("llama3-70b"), "llama3");
        assert_eq!(registry.infer_template("meta-llama-3-instruct"), "llama3");
        assert_eq!(registry.infer_template("Meta-Llama-3.1-8B"), "llama3");
    }

    #[test]
    fn test_infer_template_chatml_variants() {
        let registry = Registry::new();
        // These should all fall back to chatml
        assert_eq!(registry.infer_template("tinyllama-1.1b"), "chatml");
        assert_eq!(registry.infer_template("mistral-7b"), "chatml");
        assert_eq!(registry.infer_template("phi-3-mini"), "chatml");
        assert_eq!(registry.infer_template("qwen2-7b"), "chatml");
        assert_eq!(registry.infer_template("llama-2-7b"), "chatml"); // Llama 2, not 3
    }

    #[test]
    fn test_to_spec_registered_model() {
        let mut registry = Registry::new();
        let entry = ModelEntry {
            name: "my-model".to_string(),
            base_path: PathBuf::from("/models/my-model.gguf"),
            lora_path: None,
            template: Some("chatml".to_string()),
            ctx_len: Some(4096),
            n_threads: Some(8),
        };
        registry.register(entry);

        let spec = registry.to_spec("my-model").unwrap();
        assert_eq!(spec.name, "my-model");
        assert_eq!(spec.ctx_len, 4096);
        assert_eq!(spec.template.as_deref(), Some("chatml"));
        assert_eq!(spec.n_threads, Some(8));
    }

    #[test]
    fn test_to_spec_missing_model_returns_none() {
        let registry = Registry::new();
        assert!(registry.to_spec("does-not-exist").is_none());
    }

    #[test]
    fn test_to_spec_ctx_len_defaults_to_2048() {
        let mut registry = Registry::new();
        let entry = ModelEntry {
            name: "ctx-model".to_string(),
            base_path: PathBuf::from("/models/ctx-model.gguf"),
            lora_path: None,
            template: None,
            ctx_len: None, // No ctx_len set
            n_threads: None,
        };
        registry.register(entry);

        let spec = registry.to_spec("ctx-model").unwrap();
        assert_eq!(spec.ctx_len, 2048);
    }

    #[test]
    fn test_list_all_available_deduplicates() {
        let mut registry = Registry::new();
        registry.register(ModelEntry {
            name: "model-a".to_string(),
            base_path: PathBuf::from("/a"),
            lora_path: None,
            template: None,
            ctx_len: None,
            n_threads: None,
        });
        registry.register(ModelEntry {
            name: "model-b".to_string(),
            base_path: PathBuf::from("/b"),
            lora_path: None,
            template: None,
            ctx_len: None,
            n_threads: None,
        });

        let all = registry.list_all_available();
        assert!(all.contains(&"model-a".to_string()));
        assert!(all.contains(&"model-b".to_string()));
        // All entries should be unique
        let mut deduped = all.clone();
        deduped.dedup();
        assert_eq!(all, deduped);
    }
}
