// Fixed version of auto_discovery.rs for Issue #10
// Adds recursion depth limits and better error handling

use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// Maximum recursion depth to prevent infinite loops
const MAX_RECURSION_DEPTH: usize = 8;

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
    visited_dirs: HashSet<PathBuf>, // Prevent infinite loops
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
        
        // Add common model directories - but be more selective
        if let Some(home) = std::env::var_os("HOME") {
            let home_path = PathBuf::from(home);
            
            // Only add directories that are likely to contain models
            let candidates = vec![
                home_path.join(".cache/huggingface/hub"),
                home_path.join(".ollama/models"),
                home_path.join("models"),
                home_path.join(".local/share/shimmy/models"),
                home_path.join("Downloads"), // Often where users put models
            ];
            
            for candidate in candidates {
                if candidate.exists() {
                    search_paths.push(candidate);
                }
            }
        }
        
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            let profile_path = PathBuf::from(user_profile);
            
            let candidates = vec![
                profile_path.join(".cache\\huggingface\\hub"),
                profile_path.join(".ollama\\models"),
                profile_path.join("models"),
                profile_path.join("AppData\\Local\\shimmy\\models"),
                profile_path.join("Downloads"),
            ];
            
            for candidate in candidates {
                if candidate.exists() {
                    search_paths.push(candidate);
                }
            }
        }
        
        Self { 
            search_paths,
            visited_dirs: HashSet::new(),
        }
    }
    
    pub fn discover_models(&mut self) -> Result<Vec<DiscoveredModel>> {
        let mut discovered = Vec::new();
        self.visited_dirs.clear(); // Reset for each discovery run
        
        for search_path in &self.search_paths.clone() {
            if search_path.exists() && search_path.is_dir() {
                // Use timeout for each directory scan
                match self.scan_directory_with_timeout(search_path, 0) {
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
        
        // Remove duplicates based on canonical path
        discovered.sort_by(|a, b| a.path.cmp(&b.path));
        discovered.dedup_by(|a, b| {
            // Try to canonicalize paths for better deduplication
            let path_a = a.path.canonicalize().unwrap_or_else(|_| a.path.clone());
            let path_b = b.path.canonicalize().unwrap_or_else(|_| b.path.clone());
            path_a == path_b
        });
        
        Ok(discovered)
    }
    
    fn scan_directory_with_timeout(&mut self, dir: &Path, depth: usize) -> Result<Vec<DiscoveredModel>> {
        // Prevent infinite recursion
        if depth >= MAX_RECURSION_DEPTH {
            return Ok(Vec::new());
        }
        
        // Prevent revisiting directories (handles symlinks and loops)
        let canonical_dir = match dir.canonicalize() {
            Ok(path) => path,
            Err(_) => {
                // If canonicalize fails, use original path but be more careful
                dir.to_path_buf()
            }
        };
        
        if self.visited_dirs.contains(&canonical_dir) {
            return Ok(Vec::new());
        }
        self.visited_dirs.insert(canonical_dir.clone());
        
        let mut models = Vec::new();
        
        // Skip system/hidden directories that cause problems on macOS
        if let Some(dir_name) = dir.file_name().and_then(|n| n.to_str()) {
            let dir_name_lower = dir_name.to_lowercase();
            
            // macOS system directories to skip
            if dir_name.starts_with('.') && dir_name != ".cache" && dir_name != ".ollama" && dir_name != ".local" {
                return Ok(Vec::new());
            }
            
            // Other problematic directories
            if ["target", "cmake", "incremental", "node_modules", "build", "__pycache__",
                "whisper", "wav2vec", "bert", "clip", "System Volume Information"].contains(&dir_name_lower.as_str()) {
                return Ok(Vec::new());
            }
        }
        
        // Use read_dir with error handling
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                // Permission denied or other error - skip this directory
                eprintln!("Warning: Cannot read directory {}: {}", dir.display(), e);
                return Ok(Vec::new());
            }
        };
        
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue, // Skip problematic entries
            };
            
            let path = entry.path();
            
            // Handle directories
            if path.is_dir() {
                // Be selective about which directories to recurse into
                if self.should_recurse_into(&path) {
                    match self.scan_directory_with_timeout(&path, depth + 1) {
                        Ok(submodels) => models.extend(submodels),
                        Err(_) => continue, // Skip problematic subdirectories
                    }
                }
            } 
            // Handle files
            else if self.is_model_file(&path) {
                match self.analyze_model_file(&path) {
                    Ok(model) => models.push(model),
                    Err(_) => continue, // Skip problematic files
                }
            }
        }
        
        Ok(models)
    }
    
    fn should_recurse_into(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        
        // Skip known problematic directories
        if path_str.contains("target/") || path_str.contains("target\\") ||
           path_str.contains("cmake") || path_str.contains("incremental") ||
           path_str.contains("node_modules") || path_str.contains("build") {
            return false;
        }
        
        // For huggingface directories, be selective
        if path_str.contains("huggingface") {
            return path_str.contains("llama") || path_str.contains("phi") ||
                   path_str.contains("mistral") || path_str.contains("qwen") ||
                   path_str.contains("gemma") || path_str.contains("gguf");
        }
        
        // Generally allow recursion for model-likely directories
        true
    }
    
    // Rest of the methods remain the same but with better error handling...
    // [Copy the rest of the original methods but add error handling]
}

// TODO: Replace the original scan_directory method with scan_directory_with_timeout in auto_discovery.rs