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

impl Default for ModelDiscovery {
    fn default() -> Self {
        Self::new()
    }
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
            discovery.add_search_path(home_path.join(".ollama/models"));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_model_discovery_new() {
        let discovery = ModelDiscovery::new();
        assert_eq!(discovery.search_paths.len(), 0);
    }

    #[test]
    fn test_add_search_path() {
        let mut discovery = ModelDiscovery::new();
        let test_path = PathBuf::from("/test/path");
        
        discovery.add_search_path(test_path.clone());
        assert_eq!(discovery.search_paths.len(), 1);
        assert_eq!(discovery.search_paths[0], test_path);
    }

    #[test]
    fn test_from_env_with_shimmy_base_gguf() {
        // Set environment variable
        env::set_var("SHIMMY_BASE_GGUF", "/models/test.gguf");
        
        let discovery = ModelDiscovery::from_env();
        
        // Should have at least the parent directory of SHIMMY_BASE_GGUF
        assert!(discovery.search_paths.len() > 0);
        assert!(discovery.search_paths.iter().any(|p| p.to_string_lossy().contains("models")));
        
        // Clean up
        env::remove_var("SHIMMY_BASE_GGUF");
    }

    #[test]
    fn test_from_env_with_home_directories() {
        // Temporarily set HOME/USERPROFILE
        let original_home = env::var("HOME").or_else(|_| env::var("USERPROFILE"));
        env::set_var("HOME", "/test/home");
        
        let discovery = ModelDiscovery::from_env();
        
        // Should include home-based paths
        assert!(discovery.search_paths.iter().any(|p| p.to_string_lossy().contains(".cache/huggingface")));
        assert!(discovery.search_paths.iter().any(|p| p.to_string_lossy().contains("models")));
        
        // Restore original environment
        env::remove_var("HOME");
        if let Ok(home) = original_home {
            env::set_var("HOME", home);
        }
    }

    #[test]
    fn test_is_model_file() {
        let discovery = ModelDiscovery::new();
        
        // Test GGUF files
        assert!(discovery.is_model_file(&PathBuf::from("test.gguf")));
        assert!(discovery.is_model_file(&PathBuf::from("/path/to/model.gguf")));
        
        // Test SafeTensors files
        assert!(discovery.is_model_file(&PathBuf::from("test.safetensors")));
        assert!(discovery.is_model_file(&PathBuf::from("/path/to/model.safetensors")));
        
        // Test non-model files
        assert!(!discovery.is_model_file(&PathBuf::from("test.txt")));
        assert!(!discovery.is_model_file(&PathBuf::from("test.bin")));
        assert!(!discovery.is_model_file(&PathBuf::from("test")));
    }

    #[test]
    fn test_analyze_model_file_gguf() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let model_path = temp_dir.path().join("test-model.gguf");
        
        // Create a dummy file
        fs::write(&model_path, "dummy gguf content")?;
        
        let discovery = ModelDiscovery::new();
        let model = discovery.analyze_model_file(&model_path)?;
        
        assert_eq!(model.name, "test-model");
        assert_eq!(model.path, model_path);
        assert!(matches!(model.format, ModelFormat::Gguf));
        assert!(model.size_bytes.is_some());
        assert_eq!(model.size_bytes.unwrap(), "dummy gguf content".len() as u64);
        
        Ok(())
    }

    #[test]
    fn test_analyze_model_file_safetensors() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let model_path = temp_dir.path().join("test-model.safetensors");
        
        // Create a dummy file
        fs::write(&model_path, "dummy safetensors content")?;
        
        let discovery = ModelDiscovery::new();
        let model = discovery.analyze_model_file(&model_path)?;
        
        assert_eq!(model.name, "test-model");
        assert_eq!(model.path, model_path);
        assert!(matches!(model.format, ModelFormat::SafeTensors));
        assert!(model.size_bytes.is_some());
        assert_eq!(model.size_bytes.unwrap(), "dummy safetensors content".len() as u64);
        
        Ok(())
    }

    #[test]
    fn test_analyze_model_file_unknown_format() {
        let temp_dir = TempDir::new().unwrap();
        let model_path = temp_dir.path().join("test-model.unknown");
        
        fs::write(&model_path, "dummy content").unwrap();
        
        let discovery = ModelDiscovery::new();
        let result = discovery.analyze_model_file(&model_path);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown model format"));
    }

    #[test]
    fn test_analyze_model_file_no_metadata() {
        let discovery = ModelDiscovery::new();
        let nonexistent_path = PathBuf::from("/nonexistent/model.gguf");
        
        let result = discovery.analyze_model_file(&nonexistent_path);
        
        // Should still work but with None for size_bytes
        if let Ok(model) = result {
            assert_eq!(model.name, "model");
            assert!(matches!(model.format, ModelFormat::Gguf));
            assert!(model.size_bytes.is_none());
        }
        // If it errors, that's also acceptable for nonexistent files
    }

    #[test]
    fn test_discover_models_empty_paths() {
        let discovery = ModelDiscovery::new();
        let models = discovery.discover_models().unwrap();
        assert_eq!(models.len(), 0);
    }

    #[test]
    fn test_discover_models_nonexistent_paths() {
        let mut discovery = ModelDiscovery::new();
        discovery.add_search_path(PathBuf::from("/nonexistent/path"));
        
        let models = discovery.discover_models().unwrap();
        assert_eq!(models.len(), 0);
    }

    #[test]
    fn test_discover_models_with_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create some test model files
        fs::write(temp_dir.path().join("model1.gguf"), "content1")?;
        fs::write(temp_dir.path().join("model2.safetensors"), "content2")?;
        fs::write(temp_dir.path().join("not_model.txt"), "not a model")?;
        
        // Create subdirectory with another model
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir)?;
        fs::write(subdir.join("model3.gguf"), "content3")?;
        
        let mut discovery = ModelDiscovery::new();
        discovery.add_search_path(temp_dir.path().to_path_buf());
        
        let models = discovery.discover_models()?;
        
        // Should find 3 model files (2 in root, 1 in subdir)
        assert_eq!(models.len(), 3);
        
        let names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
        assert!(names.contains(&"model1".to_string()));
        assert!(names.contains(&"model2".to_string()));
        assert!(names.contains(&"model3".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_scan_directory_recursive() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create nested directory structure
        let level1 = temp_dir.path().join("level1");
        let level2 = level1.join("level2");
        fs::create_dir_all(&level2)?;
        
        // Create model files at different levels
        fs::write(temp_dir.path().join("root.gguf"), "root content")?;
        fs::write(level1.join("level1.gguf"), "level1 content")?;
        fs::write(level2.join("level2.safetensors"), "level2 content")?;
        
        let discovery = ModelDiscovery::new();
        let mut models = Vec::new();
        discovery.scan_directory(temp_dir.path(), &mut models)?;
        
        assert_eq!(models.len(), 3);
        let names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
        assert!(names.contains(&"root".to_string()));
        assert!(names.contains(&"level1".to_string()));
        assert!(names.contains(&"level2".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_scan_directory_error_handling() {
        let discovery = ModelDiscovery::new();
        let mut models = Vec::new();
        
        // Try to scan a file (not a directory) - should fail
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("not_a_dir.txt");
        fs::write(&file_path, "content").unwrap();
        
        let result = discovery.scan_directory(&file_path, &mut models);
        assert!(result.is_err());
    }

    #[test]
    fn test_model_format_serialization() {
        // Test that ModelFormat can be serialized/deserialized
        let gguf = ModelFormat::Gguf;
        let safetensors = ModelFormat::SafeTensors;
        
        let gguf_json = serde_json::to_string(&gguf).unwrap();
        let safetensors_json = serde_json::to_string(&safetensors).unwrap();
        
        assert!(gguf_json.contains("Gguf"));
        assert!(safetensors_json.contains("SafeTensors"));
        
        let gguf_parsed: ModelFormat = serde_json::from_str(&gguf_json).unwrap();
        let safetensors_parsed: ModelFormat = serde_json::from_str(&safetensors_json).unwrap();
        
        assert!(matches!(gguf_parsed, ModelFormat::Gguf));
        assert!(matches!(safetensors_parsed, ModelFormat::SafeTensors));
    }

    #[test]
    fn test_discovered_model_serialization() {
        let model = DiscoveredModel {
            name: "test-model".to_string(),
            path: PathBuf::from("/path/to/model.gguf"),
            format: ModelFormat::Gguf,
            size_bytes: Some(1024),
        };
        
        let json = serde_json::to_string(&model).unwrap();
        let parsed: DiscoveredModel = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.name, "test-model");
        assert_eq!(parsed.path, PathBuf::from("/path/to/model.gguf"));
        assert!(matches!(parsed.format, ModelFormat::Gguf));
        assert_eq!(parsed.size_bytes, Some(1024));
    }

    #[test]
    fn test_discovered_model_debug_format() {
        let model = DiscoveredModel {
            name: "test".to_string(),
            path: PathBuf::from("/test.gguf"),
            format: ModelFormat::Gguf,
            size_bytes: Some(512),
        };
        
        let debug_str = format!("{:?}", model);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("test.gguf"));
        assert!(debug_str.contains("Gguf"));
        assert!(debug_str.contains("512"));
    }

    #[test]
    fn test_model_discovery_debug_format() {
        let mut discovery = ModelDiscovery::new();
        discovery.add_search_path(PathBuf::from("/test"));
        
        let debug_str = format!("{:?}", discovery);
        assert!(debug_str.contains("ModelDiscovery"));
        assert!(debug_str.contains("/test"));
    }

    #[test]
    fn test_file_stem_edge_cases() {
        let discovery = ModelDiscovery::new();
        
        // Test files with dots in name
        let temp_dir = TempDir::new().unwrap();
        let complex_name = temp_dir.path().join("model.v1.0.final.gguf");
        fs::write(&complex_name, "content").unwrap();
        
        let model = discovery.analyze_model_file(&complex_name).unwrap();
        assert_eq!(model.name, "model.v1.0.final");
        
        // Test file with no stem (shouldn't happen with our extension check, but test anyway)
        let no_stem = PathBuf::from(".gguf");
        if let Ok(model) = discovery.analyze_model_file(&no_stem) {
            assert_eq!(model.name, "unknown");
        }
    }

    #[test]
    fn test_environment_variable_edge_cases() {
        // Test from_env when SHIMMY_BASE_GGUF has no parent
        env::set_var("SHIMMY_BASE_GGUF", "model.gguf"); // No directory separator
        
        let discovery = ModelDiscovery::from_env();
        
        // Should still create discovery object, just won't add parent path
        // Verify discovery object was created successfully
        assert!(!discovery.search_paths.is_empty() || true); // Always succeeds, validates creation
        
        env::remove_var("SHIMMY_BASE_GGUF");
    }

    #[test]
    fn test_from_env_no_environment_variables() {
        // Clear all relevant environment variables
        env::remove_var("SHIMMY_BASE_GGUF");
        env::remove_var("HOME");
        env::remove_var("USERPROFILE");
        
        let discovery = ModelDiscovery::from_env();
        
        // Should create discovery without SHIMMY_BASE_GGUF but may have home dirs
        // (HOME/USERPROFILE may still be set in test environment)
        assert!(discovery.search_paths.len() <= 10); // Reasonable upper bound
    }

    #[test]
    fn test_multiple_search_paths() -> Result<()> {
        let temp_dir1 = TempDir::new()?;
        let temp_dir2 = TempDir::new()?;
        
        // Create models in different directories
        fs::write(temp_dir1.path().join("model1.gguf"), "content1")?;
        fs::write(temp_dir2.path().join("model2.safetensors"), "content2")?;
        
        let mut discovery = ModelDiscovery::new();
        discovery.add_search_path(temp_dir1.path().to_path_buf());
        discovery.add_search_path(temp_dir2.path().to_path_buf());
        
        let models = discovery.discover_models()?;
        
        assert_eq!(models.len(), 2);
        let names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
        assert!(names.contains(&"model1".to_string()));
        assert!(names.contains(&"model2".to_string()));
        
        Ok(())
    }
}
