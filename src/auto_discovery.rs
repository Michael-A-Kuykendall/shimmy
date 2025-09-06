use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use serde::{Deserialize, Serialize};

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
    search_paths: Vec<PathBuf>,
}

impl ModelAutoDiscovery {
    pub fn new() -> Self {
        let mut search_paths = vec![
            PathBuf::from("./models"),
            PathBuf::from("./"),
        ];
        
        // Add paths from environment variables
        if let Ok(shimmy_base) = std::env::var("SHIMMY_BASE_GGUF") {
            let path = PathBuf::from(shimmy_base);
            if let Some(parent) = path.parent() {
                search_paths.push(parent.to_path_buf());
            }
        }
        
        // Add common model directories
        if let Some(home) = std::env::var_os("HOME") {
            search_paths.push(PathBuf::from(home.clone()).join(".cache/huggingface/hub"));
            search_paths.push(PathBuf::from(home.clone()).join(".ollama/models"));
            search_paths.push(PathBuf::from(home.clone()).join("models"));
            search_paths.push(PathBuf::from(home).join(".local/share/shimmy/models"));
        }
        
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            // Focus on likely GGUF model locations
            search_paths.push(PathBuf::from(user_profile.clone()).join(".cache\\huggingface\\hub"));
            search_paths.push(PathBuf::from(user_profile.clone()).join(".ollama\\models"));
            search_paths.push(PathBuf::from(user_profile.clone()).join("models"));
            search_paths.push(PathBuf::from(user_profile.clone()).join("AppData\\Local\\shimmy\\models"));
            search_paths.push(PathBuf::from(user_profile).join("Downloads"));
        }
        
        Self { search_paths }
    }
    
    #[allow(dead_code)]
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }
    
    pub fn discover_models(&self) -> Result<Vec<DiscoveredModel>> {
        let mut discovered = Vec::new();
        
        for search_path in &self.search_paths {
            if search_path.exists() && search_path.is_dir() {
                discovered.extend(self.scan_directory(search_path)?);
            }
        }
        
        // Remove duplicates based on file hash or path
        discovered.sort_by(|a, b| a.path.cmp(&b.path));
        discovered.dedup_by(|a, b| a.path == b.path);
        
        Ok(discovered)
    }
    
    fn scan_directory(&self, dir: &Path) -> Result<Vec<DiscoveredModel>> {
        let mut models = Vec::new();
        
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Skip build and cache directories
            if path.is_dir() {
                let dir_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if dir_name == "target" || dir_name == "cmake" || 
                   dir_name == "incremental" || dir_name.starts_with(".git") ||
                   dir_name.contains("whisper") || dir_name.contains("wav2vec") ||
                   dir_name.contains("bert") || dir_name.contains("clip") {
                    continue;
                }
                // Only scan directories that might contain LLM models
                if path.to_string_lossy().contains("huggingface") {
                    let path_str = path.to_string_lossy().to_lowercase();
                    if !(path_str.contains("llama") || path_str.contains("phi") ||
                         path_str.contains("mistral") || path_str.contains("qwen") ||
                         path_str.contains("gemma") || path_str.contains("gguf")) {
                        continue;
                    }
                }
                // Recursively scan subdirectories
                models.extend(self.scan_directory(&path)?);
            } else if self.is_model_file(&path) {
                if let Ok(model) = self.analyze_model_file(&path) {
                    models.push(model);
                }
            }
        }
        
        Ok(models)
    }
    
    fn is_model_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            // Only accept GGUF files for now, as they are the primary format
            if ext == "gguf" {
                return true;
            }
            // Be very selective with .bin files - only include obvious model files
            if ext == "bin" {
                let path_str = path.to_string_lossy().to_lowercase();
                // Skip build artifacts, cache files, and non-LLM models
                if path_str.contains("target\\") || path_str.contains("target/") ||
                   path_str.contains("cmake") || path_str.contains("incremental") ||
                   path_str.contains("work-products") || path_str.contains("dep-graph") ||
                   path_str.contains("query-cache") || path_str.contains("ompver") ||
                   path_str.contains("whisper") || path_str.contains("wav2vec") ||
                   path_str.contains("pytorch_model") {
                    return false;
                }
                // Only include .bin files that are clearly LLM models
                return (path_str.contains("model") || path_str.contains("llama") || 
                        path_str.contains("phi") || path_str.contains("mistral") ||
                        path_str.contains("qwen") || path_str.contains("gemma")) &&
                       !path_str.contains("config") && !path_str.contains("tokenizer");
            }
        }
        false
    }
    
    fn is_lora_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            if ext == "gguf" || ext == "ggml" {
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                return filename.contains("lora") || filename.contains("adapter");
            }
        }
        false
    }
    
    pub fn find_lora_for_model(&self, model_path: &Path) -> Option<PathBuf> {
        let model_dir = model_path.parent()?;
        let model_stem = model_path.file_stem()?.to_str()?;
        
        // Look for LoRA files in the same directory
        if let Ok(entries) = fs::read_dir(model_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if self.is_lora_file(&path) {
                    let lora_stem = path.file_stem()?.to_str()?;
                    // Check if LoRA filename contains model name or vice versa
                    if lora_stem.contains(model_stem) || model_stem.contains(lora_stem) {
                        return Some(path);
                    }
                }
            }
        }
        
        None
    }
    
    fn analyze_model_file(&self, path: &Path) -> Result<DiscoveredModel> {
        let metadata = fs::metadata(path)?;
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
            
        let (model_type, parameter_count, quantization) = self.parse_filename(&filename);
        
        // Generate a clean model name
        let name = self.generate_model_name(&filename);
        
        // Look for paired LoRA adapter
        let lora_path = self.find_lora_for_model(path);
        
        Ok(DiscoveredModel {
            name,
            path: path.to_path_buf(),
            lora_path,
            size_bytes: metadata.len(),
            model_type,
            parameter_count,
            quantization,
        })
    }
    
    fn parse_filename(&self, filename: &str) -> (String, Option<String>, Option<String>) {
        let lower = filename.to_lowercase();
        
        // Extract model type
        let model_type = if lower.contains("llama") {
            "Llama"
        } else if lower.contains("phi") {
            "Phi"
        } else if lower.contains("gemma") {
            "Gemma"
        } else if lower.contains("mistral") {
            "Mistral"
        } else if lower.contains("qwen") {
            "Qwen"
        } else {
            "Unknown"
        }.to_string();
        
        // Extract parameter count
        let parameter_count = if lower.contains("3b") || lower.contains("3.0b") {
            Some("3B".to_string())
        } else if lower.contains("7b") || lower.contains("7.0b") {
            Some("7B".to_string())
        } else if lower.contains("13b") || lower.contains("13.0b") {
            Some("13B".to_string())
        } else if lower.contains("70b") || lower.contains("70.0b") {
            Some("70B".to_string())
        } else {
            None
        };
        
        // Extract quantization
        let quantization = if lower.contains("q4_k_m") {
            Some("Q4_K_M".to_string())
        } else if lower.contains("q4_0") {
            Some("Q4_0".to_string())
        } else if lower.contains("q8_0") {
            Some("Q8_0".to_string())
        } else if lower.contains("f16") {
            Some("F16".to_string())
        } else if lower.contains("f32") {
            Some("F32".to_string())
        } else {
            None
        };
        
        (model_type, parameter_count, quantization)
    }
    
    fn generate_model_name(&self, filename: &str) -> String {
        // Remove file extension
        let name = if let Some(pos) = filename.rfind('.') {
            &filename[..pos]
        } else {
            filename
        };
        
        // Replace common separators with dashes
        name.replace("_", "-")
            .replace(" ", "-")
            .to_lowercase()
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
        assert!(discovery.search_paths.len() >= 1);
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
