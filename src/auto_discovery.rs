use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
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

#[derive(Debug, Deserialize)]
struct OllamaManifest {
    #[serde(rename = "schemaVersion")]
    #[allow(dead_code)]
    schema_version: i32,
    #[serde(rename = "mediaType")]
    #[allow(dead_code)]
    media_type: String,
    #[allow(dead_code)]
    config: OllamaConfig,
    layers: Vec<OllamaLayer>,
}

#[derive(Debug, Deserialize)]
struct OllamaConfig {
    #[serde(rename = "mediaType")]
    #[allow(dead_code)]
    media_type: String,
    #[allow(dead_code)]
    digest: String,
    #[allow(dead_code)]
    size: i64,
}

#[derive(Debug, Deserialize)]
struct OllamaLayer {
    #[serde(rename = "mediaType")]
    media_type: String,
    digest: String,
    size: i64,
}

pub struct ModelAutoDiscovery {
    pub search_paths: Vec<PathBuf>,
}

impl ModelAutoDiscovery {
    pub fn new() -> Self {
        let mut search_paths = vec![PathBuf::from("./models"), PathBuf::from("./")];

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
            search_paths.push(PathBuf::from(home.clone()).join("models"));
            search_paths.push(PathBuf::from(home).join(".local/share/shimmy/models"));
        }

        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            // Focus on likely GGUF model locations
            search_paths.push(PathBuf::from(user_profile.clone()).join(".cache\\huggingface\\hub"));
            search_paths.push(PathBuf::from(user_profile.clone()).join(".ollama\\models"));
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
                    let ollama_path = PathBuf::from(format!("{}\\Users\\{}\\AppData\\Local\\Ollama\\models", 
                        drive, username));
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

    #[allow(dead_code)]
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

        Ok(discovered)
    }

    fn scan_directory(&self, dir: &Path) -> Result<Vec<DiscoveredModel>> {
        self.scan_directory_with_depth(dir, 0)
    }

    fn scan_directory_with_depth(&self, dir: &Path, depth: usize) -> Result<Vec<DiscoveredModel>> {
        // Prevent infinite recursion - limit depth to 8 levels
        if depth >= 8 {
            return Ok(Vec::new());
        }

        // Skip system directories that cause problems on macOS
        if let Some(dir_name) = dir.file_name().and_then(|n| n.to_str()) {
            if dir_name.starts_with('.')
                && dir_name != ".cache"
                && dir_name != ".ollama"
                && dir_name != ".local"
            {
                return Ok(Vec::new());
            }
        }

        let mut models = Vec::new();

        // Use error handling for read_dir to handle permission issues
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(Vec::new()), // Skip directories we can't read
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue, // Skip problematic entries
            };
            let path = entry.path();

            // Skip build and cache directories
            if path.is_dir() {
                let dir_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if dir_name == "target"
                    || dir_name == "cmake"
                    || dir_name == "incremental"
                    || dir_name.starts_with(".git")
                    || dir_name.contains("whisper")
                    || dir_name.contains("wav2vec")
                    || dir_name.contains("bert")
                    || dir_name.contains("clip")
                {
                    continue;
                }
                // Only scan directories that might contain LLM models
                if path.to_string_lossy().contains("huggingface") {
                    let path_str = path.to_string_lossy().to_lowercase();
                    if !(path_str.contains("llama")
                        || path_str.contains("phi")
                        || path_str.contains("mistral")
                        || path_str.contains("qwen")
                        || path_str.contains("gemma")
                        || path_str.contains("gguf"))
                    {
                        continue;
                    }
                }
                // Recursively scan subdirectories with depth tracking
                models.extend(self.scan_directory_with_depth(&path, depth + 1)?);
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
            // Accept GGUF files (primary format)
            if ext == "gguf" {
                return true;
            }
            // Accept SafeTensors files (native Rust support - no Python needed!)
            if ext == "safetensors" {
                let path_str = path.to_string_lossy().to_lowercase();
                // Only include obvious model files, skip tokenizer/config files
                return !path_str.contains("tokenizer") && !path_str.contains("config");
            }
            // Be very selective with .bin files - only include obvious model files
            if ext == "bin" {
                let path_str = path.to_string_lossy().to_lowercase();
                // Skip build artifacts, cache files, and non-LLM models
                if path_str.contains("target\\")
                    || path_str.contains("target/")
                    || path_str.contains("cmake")
                    || path_str.contains("incremental")
                    || path_str.contains("work-products")
                    || path_str.contains("dep-graph")
                    || path_str.contains("query-cache")
                    || path_str.contains("ompver")
                    || path_str.contains("whisper")
                    || path_str.contains("wav2vec")
                    || path_str.contains("pytorch_model")
                {
                    return false;
                }
                // Only include .bin files that are clearly LLM models
                return (path_str.contains("model")
                    || path_str.contains("llama")
                    || path_str.contains("phi")
                    || path_str.contains("mistral")
                    || path_str.contains("qwen")
                    || path_str.contains("gemma"))
                    && !path_str.contains("config")
                    && !path_str.contains("tokenizer");
            }
        }
        false
    }

    fn is_lora_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            if ext == "gguf" || ext == "ggml" {
                let filename = path
                    .file_name()
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
        let filename = path
            .file_name()
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
        }
        .to_string();

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
        name.replace("_", "-").replace(" ", "-").to_lowercase()
    }

    fn discover_ollama_models(&self) -> Result<Vec<DiscoveredModel>> {
        let mut models = Vec::new();

        // Find Ollama models directory
        let ollama_dir = if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join(".ollama/models")
        } else if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            PathBuf::from(user_profile).join(".ollama").join("models")
        } else {
            return Ok(models);
        };

        if !ollama_dir.exists() {
            return Ok(models);
        }

        let manifests_dir = ollama_dir.join("manifests").join("registry.ollama.ai");
        let blobs_dir = ollama_dir.join("blobs");

        if !manifests_dir.exists() || !blobs_dir.exists() {
            return Ok(models);
        }

        // Scan manifest directories for model names
        for namespace_entry in fs::read_dir(&manifests_dir)
            .map_err(|_| anyhow::anyhow!("Cannot read manifests directory"))?
        {
            let namespace_entry = namespace_entry?;
            if !namespace_entry.path().is_dir() {
                continue;
            }

            for model_entry in fs::read_dir(namespace_entry.path())
                .map_err(|_| anyhow::anyhow!("Cannot read model directory"))?
            {
                let model_entry = model_entry?;
                if !model_entry.path().is_dir() {
                    continue;
                }

                // Get model name from directory structure
                let namespace = namespace_entry.file_name().to_string_lossy().to_string();
                let model_name = model_entry.file_name().to_string_lossy().to_string();

                for tag_entry in fs::read_dir(model_entry.path())
                    .map_err(|_| anyhow::anyhow!("Cannot read tag directory"))?
                {
                    let tag_entry = tag_entry?;
                    if tag_entry.path().is_file() {
                        // Parse the manifest file
                        if let Ok(manifest_content) = fs::read_to_string(tag_entry.path()) {
                            if let Ok(manifest) =
                                serde_json::from_str::<OllamaManifest>(&manifest_content)
                            {
                                // Find the model blob (largest layer that's likely a GGUF)
                                for layer in &manifest.layers {
                                    if layer.media_type == "application/vnd.ollama.image.model" {
                                        if let Some(hash) = layer.digest.strip_prefix("sha256:") {
                                            let blob_path =
                                                blobs_dir.join(format!("sha256-{}", hash));
                                            if blob_path.exists()
                                                && self.is_gguf_blob(&blob_path).unwrap_or(false)
                                            {
                                                let tag = tag_entry
                                                    .file_name()
                                                    .to_string_lossy()
                                                    .to_string();
                                                let display_name = if namespace == "library" {
                                                    format!("{}:{}", model_name, tag)
                                                } else {
                                                    format!("{}{}:{}", namespace, model_name, tag)
                                                };

                                                let discovered = DiscoveredModel {
                                                    name: display_name.clone(),
                                                    path: blob_path.clone(),
                                                    lora_path: None,
                                                    size_bytes: layer.size as u64,
                                                    model_type: "Ollama".to_string(),
                                                    parameter_count: None,
                                                    quantization: None,
                                                };
                                                models.push(discovered);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(models)
    }

    fn is_gguf_blob(&self, path: &Path) -> Result<bool> {
        let mut file = std::fs::File::open(path)?;
        let mut buffer = [0u8; 4];
        use std::io::Read;
        file.read_exact(&mut buffer)?;
        Ok(&buffer == b"GGUF")
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
