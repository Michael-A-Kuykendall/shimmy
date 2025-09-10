// Model metadata caching system
// Avoids repeated file parsing for faster model loading

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub mod model_cache;

/// Cached metadata for a model file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelMetadata {
    /// Original model file path
    pub model_path: PathBuf,
    /// File size in bytes
    pub file_size: u64,
    /// Last modification time (seconds since epoch)
    pub modified_time: u64,
    /// Model format type
    pub format: ModelFormat,
    /// Cached tensor information
    pub tensors: Vec<TensorInfo>,
    /// Configuration data (parsed from config.json)
    pub config: Option<serde_json::Value>,
    /// Tokenizer data (parsed from tokenizer.json)
    pub tokenizer: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModelFormat {
    SafeTensors,
    GGUF,
    HuggingFace,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TensorInfo {
    pub name: String,
    pub shape: Vec<usize>,
    pub dtype: String,
    pub offset: Option<u64>,
    pub size_bytes: Option<u64>,
}

/// Model metadata cache manager
#[derive(Debug)]
pub struct ModelCache {
    cache_dir: PathBuf,
    cache: HashMap<PathBuf, ModelMetadata>,
}

impl ModelCache {
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir)?;
        
        let mut cache = Self {
            cache_dir,
            cache: HashMap::new(),
        };
        
        // Load existing cache entries
        cache.load_cache()?;
        
        Ok(cache)
    }
    
    /// Get the cache directory path
    fn get_cache_dir() -> Result<PathBuf> {
        // Use platform-appropriate cache directory
        #[cfg(target_os = "windows")]
        let cache_dir = {
            let appdata = std::env::var("APPDATA")
                .map_err(|_| anyhow!("APPDATA environment variable not found"))?;
            PathBuf::from(appdata).join("shimmy").join("cache")
        };
        
        #[cfg(not(target_os = "windows"))]
        let cache_dir = {
            let home = std::env::var("HOME")
                .map_err(|_| anyhow!("HOME environment variable not found"))?;
            PathBuf::from(home).join(".cache").join("shimmy")
        };
        
        Ok(cache_dir)
    }
    
    /// Get cached metadata for a model, if valid
    pub fn get(&self, model_path: &Path) -> Option<&ModelMetadata> {
        if let Some(metadata) = self.cache.get(model_path) {
            // Validate cache entry is still fresh
            if self.is_cache_valid(model_path, metadata) {
                return Some(metadata);
            }
        }
        None
    }
    
    /// Store metadata in cache
    pub fn set(&mut self, metadata: ModelMetadata) -> Result<()> {
        let cache_file = self.get_cache_file_path(&metadata.model_path);
        
        // Serialize and save to disk
        let cache_data = serde_json::to_string_pretty(&metadata)?;
        fs::write(&cache_file, cache_data)?;
        
        // Update in-memory cache
        self.cache.insert(metadata.model_path.clone(), metadata);
        
        Ok(())
    }
    
    /// Check if cached metadata is still valid
    fn is_cache_valid(&self, model_path: &Path, metadata: &ModelMetadata) -> bool {
        if let Ok(file_metadata) = fs::metadata(model_path) {
            if let Ok(modified) = file_metadata.modified() {
                if let Ok(duration) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                    let current_modified = duration.as_secs();
                    let cached_modified = metadata.modified_time;
                    let cached_size = metadata.file_size;
                    let current_size = file_metadata.len();
                    
                    // Cache is valid if file size and modification time match
                    return current_modified == cached_modified && current_size == cached_size;
                }
            }
        }
        false
    }
    
    /// Load cache entries from disk
    fn load_cache(&mut self) -> Result<()> {
        if !self.cache_dir.exists() {
            return Ok(());
        }
        
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(cache_data) = fs::read_to_string(&path) {
                    if let Ok(metadata) = serde_json::from_str::<ModelMetadata>(&cache_data) {
                        // Only load if cache is still valid
                        if self.is_cache_valid(&metadata.model_path, &metadata) {
                            self.cache.insert(metadata.model_path.clone(), metadata);
                        } else {
                            // Remove invalid cache file
                            let _ = fs::remove_file(&path);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get cache file path for a model
    fn get_cache_file_path(&self, model_path: &Path) -> PathBuf {
        // Create a safe filename from the model path
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        model_path.hash(&mut hasher);
        let hash = hasher.finish();
        
        self.cache_dir.join(format!("model_{:x}.json", hash))
    }
    
    /// Clear all cached metadata
    pub fn clear(&mut self) -> Result<()> {
        self.cache.clear();
        
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                    fs::remove_file(entry.path())?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            cache_dir: self.cache_dir.clone(),
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub entries: usize,
    pub cache_dir: PathBuf,
}

impl Default for ModelCache {
    fn default() -> Self {
        Self::new().expect("Failed to initialize model cache")
    }
}