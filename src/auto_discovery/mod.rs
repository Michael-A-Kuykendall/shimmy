mod ollama;
mod scan;

use crate::invariant_ppt::shimmy_invariants;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredModel {
    pub name: String,
    pub path: PathBuf,
    pub lora_path: Option<PathBuf>,
    pub size_bytes: u64,
    pub model_type: String,
    pub parameter_count: Option<String>,
    pub quantization: Option<String>,
}

pub struct ModelAutoDiscovery {
    pub search_paths: Vec<PathBuf>,
}

impl ModelAutoDiscovery {
    pub fn new() -> Self {
        let mut search_paths = vec![PathBuf::from("./models")];

        // Add paths from environment variables
        if let Ok(shimmy_base) = std::env::var("SHIMMY_BASE_GGUF") {
            let path = PathBuf::from(shimmy_base);
            if let Some(parent) = path.parent() {
                search_paths.push(parent.to_path_buf());
            }
        }

        // Add custom model directories from environment variable
        if let Ok(custom_dirs) = std::env::var("SHIMMY_MODEL_PATHS") {
            for dir in custom_dirs.split(';').filter(|s| !s.is_empty()) {
                search_paths.push(PathBuf::from(dir));
            }
        }

        // Add OLLAMA_MODELS environment variable if set
        if let Ok(ollama_models) = std::env::var("OLLAMA_MODELS") {
            search_paths.push(PathBuf::from(ollama_models));
        }

        // Add common model directories
        if let Some(home) = std::env::var_os("HOME") {
            search_paths.push(PathBuf::from(home.clone()).join(".cache/huggingface/hub"));
            search_paths.push(PathBuf::from(home.clone()).join(".ollama/models"));
            search_paths.push(PathBuf::from(home.clone()).join(".lmstudio/models"));
            search_paths.push(PathBuf::from(home.clone()).join(".cache/lm-studio/models"));
            search_paths.push(PathBuf::from(home.clone()).join("models"));
            search_paths.push(PathBuf::from(home).join(".local/share/shimmy/models"));
        }

        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            // Focus on likely GGUF model locations
            search_paths.push(PathBuf::from(user_profile.clone()).join(".cache\\huggingface\\hub"));
            search_paths.push(PathBuf::from(user_profile.clone()).join(".ollama\\models"));
            search_paths.push(PathBuf::from(user_profile.clone()).join(".lmstudio\\models"));
            search_paths.push(PathBuf::from(user_profile.clone()).join(".cache\\lm-studio\\models"));
            search_paths.push(PathBuf::from(user_profile.clone()).join("AppData\\Roaming\\LM Studio\\models"));
            search_paths.push(PathBuf::from(user_profile.clone()).join("models"));
            search_paths
                .push(PathBuf::from(user_profile.clone()).join("AppData\\Local\\shimmy\\models"));
            search_paths.push(PathBuf::from(user_profile).join("Downloads"));
        }

        // Search common Ollama installation paths on different drives (Windows)
        #[cfg(windows)]
        {
            if let Ok(username) = std::env::var("USERNAME") {
                for drive in &["C:", "D:", "E:", "F:"] {
                    let ollama_path = PathBuf::from(format!(
                        "{}\\Users\\{}\\AppData\\Local\\Ollama\\models",
                        drive, username
                    ));
                    search_paths.push(ollama_path);

                    // Also check alternate Ollama paths
                    let alt_ollama = PathBuf::from(format!("{}\\Ollama\\models", drive));
                    search_paths.push(alt_ollama);

                    // Check common model storage locations
                    let models_path = PathBuf::from(format!("{}\\models", drive));
                    search_paths.push(models_path);
                }
            }
        }

        Self { search_paths }
    }

    #[allow(dead_code)] // utility method — available for dynamic path registration at runtime
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    pub fn discover_models(&self) -> Result<Vec<DiscoveredModel>> {
        let mut discovered = Vec::new();

        for search_path in &self.search_paths {
            if search_path.exists() && search_path.is_dir() {
                // Add error handling to prevent one bad directory from killing discovery
                match self.scan_directory(search_path) {
                    Ok(models) => discovered.extend(models),
                    Err(e) => {
                        eprintln!("Warning: Failed to scan {}: {}", search_path.display(), e);
                        continue; // Skip problematic directories instead of failing
                    }
                }
            }
        }

        // Discover Ollama models specifically
        match self.discover_ollama_models() {
            Ok(ollama_models) => discovered.extend(ollama_models),
            Err(e) => eprintln!("Warning: Failed to discover Ollama models: {}", e),
        }

        // Remove duplicates based on file hash or path
        discovered.sort_by(|a, b| a.path.cmp(&b.path));
        discovered.dedup_by(|a, b| a.path == b.path);

        // PPT Invariant: Validate discovery results before returning
        shimmy_invariants::assert_discovery_valid(discovered.len());

        // PPT Invariant: Validate each discovered model
        for model in &discovered {
            // Windows path normalization for Issue #106
            let path_str = if cfg!(target_os = "windows") {
                model.path.to_string_lossy().replace('\\', "/")
            } else {
                model.path.to_string_lossy().to_string()
            };
            shimmy_invariants::assert_backend_selection_valid(&path_str, &model.model_type);
        }

        Ok(discovered)
    }
}

impl Default for ModelAutoDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovered_model_creation() {
        let model = DiscoveredModel {
            name: "test".to_string(),
            path: PathBuf::from("/test"),
            lora_path: None,
            size_bytes: 1024,
            model_type: "Llama".to_string(),
            parameter_count: Some("7B".to_string()),
            quantization: Some("Q4_K_M".to_string()),
        };
        assert_eq!(model.name, "test");
        assert_eq!(model.size_bytes, 1024);
    }

    #[test]
    fn test_model_auto_discovery_new() {
        let discovery = ModelAutoDiscovery::new();
        assert!(!discovery.search_paths.is_empty());
    }

    #[test]
    fn test_filename_parsing() {
        let discovery = ModelAutoDiscovery::new();
        let (model_type, params, quant) = discovery.parse_filename("llama-7b-q4_k_m.gguf");
        assert_eq!(model_type, "Llama");
        assert_eq!(params, Some("7B".to_string()));
        assert_eq!(quant, Some("Q4_K_M".to_string()));
    }
}
