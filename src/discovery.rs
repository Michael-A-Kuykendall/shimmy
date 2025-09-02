// Auto-discovery system for GGUF and SafeTensors models
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use std::path::Path;
use std::env;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredModel {
    pub name: String,
    pub path: PathBuf,
    pub format: ModelFormat,
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelFormat {
    Gguf,
    SafeTensors,
}

#[derive(Debug)]
pub struct ModelDiscovery {
    search_paths: Vec<PathBuf>,
}

impl ModelDiscovery {
    pub fn new() -> Self {
        Self {
            search_paths: Vec::new(),
        }
    }

    pub fn from_env() -> Self {
        let mut discovery = Self::new();
        
        // Add SHIMMY_BASE_GGUF parent directory
        if let Ok(base_path) = env::var("SHIMMY_BASE_GGUF") {
            if let Some(parent) = Path::new(&base_path).parent() {
                discovery.add_search_path(parent.to_path_buf());
            }
        }
        
        // Add common model directories
        if let Ok(home) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
            let home_path = PathBuf::from(home);
            discovery.add_search_path(home_path.join(".cache/huggingface"));
            discovery.add_search_path(home_path.join("models"));
        }
        
        discovery
    }

    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    pub fn discover_models(&self) -> Result<Vec<DiscoveredModel>> {
        let mut models = Vec::new();
        
        for path in &self.search_paths {
            if path.exists() {
                self.scan_directory(path, &mut models)?;
            }
        }
        
        Ok(models)
    }

    fn scan_directory(&self, dir: &Path, models: &mut Vec<DiscoveredModel>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                self.scan_directory(&path, models)?;
            } else if self.is_model_file(&path) {
                if let Ok(model) = self.analyze_model_file(&path) {
                    models.push(model);
                }
            }
        }
        Ok(())
    }

    fn is_model_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(ext.to_str(), Some("gguf") | Some("safetensors"))
        } else {
            false
        }
    }

    fn analyze_model_file(&self, path: &Path) -> Result<DiscoveredModel> {
        let format = match path.extension().and_then(|s| s.to_str()) {
            Some("gguf") => ModelFormat::Gguf,
            Some("safetensors") => ModelFormat::SafeTensors,
            _ => return Err(anyhow::anyhow!("Unknown model format")),
        };

        let size_bytes = fs::metadata(path).ok().map(|m| m.len());
        
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(DiscoveredModel {
            name,
            path: path.to_path_buf(),
            format,
            size_bytes,
        })
    }
}
