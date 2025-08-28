use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredModel {
    pub name: String,
    pub path: PathBuf,
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
        
        // Add common model directories
        if let Some(home) = std::env::var_os("HOME") {
            search_paths.push(PathBuf::from(home.clone()).join(".cache/huggingface/hub"));
            search_paths.push(PathBuf::from(home).join("models"));
        }
        
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            search_paths.push(PathBuf::from(user_profile.clone()).join(".cache\\huggingface\\hub"));
            search_paths.push(PathBuf::from(user_profile).join("models"));
        }
        
        Self { search_paths }
    }
    
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
            
            if path.is_file() && self.is_model_file(&path) {
                if let Ok(model) = self.analyze_model_file(&path) {
                    models.push(model);
                }
            } else if path.is_dir() {
                // Recursively scan subdirectories
                models.extend(self.scan_directory(&path)?);
            }
        }
        
        Ok(models)
    }
    
    fn is_model_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            ext == "gguf" || ext == "ggml" || ext == "bin"
        } else {
            false
        }
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
        
        Ok(DiscoveredModel {
            name,
            path: path.to_path_buf(),
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
