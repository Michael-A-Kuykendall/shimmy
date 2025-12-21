// Native SafeTensors inference engine - NO Python dependency
// Implements direct SafeTensors model loading and inference in pure Rust

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use safetensors::SafeTensors;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
use tracing::{debug, info, warn};

// use crate::cache::{ModelCache, ModelMetadata};
// use crate::cache::model_cache;

use super::{GenOptions, InferenceEngine, LoadedModel, ModelSpec};

// Memory-mapped file support for large models
use memmap2::MmapOptions;

#[derive(Debug)]
pub struct SafeTensorsEngine {
    // Pure Rust implementation - no external dependencies
    // cache: RwLock<ModelCache>,
    use_mmap: bool, // Enable memory-mapped loading for large models
}

impl Default for SafeTensorsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SafeTensorsEngine {
    pub fn new() -> Self {
        // let cache = ModelCache::new().unwrap_or_else(|e| {
        //     warn!("Failed to initialize model cache: {}. Running without cache.", e);
        //     ModelCache::default()
        // });
        Self {
            // cache: RwLock::new(cache),
            use_mmap: true, // Enable memory-mapped loading by default
        }
    }

    #[cfg(test)]
    pub fn with_mmap(mut self, use_mmap: bool) -> Self {
        self.use_mmap = use_mmap;
        self
    }

    /// Check if a model file is SafeTensors format
    pub fn is_safetensors_model(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            return ext == "safetensors";
        }

        // Also check by reading header if no extension
        if let Ok(data) = fs::read(path) {
            if data.len() >= 8 {
                // SafeTensors files start with 8-byte header length
                return SafeTensors::deserialize(&data).is_ok();
            }
        }

        false
    }

    /// Discover SafeTensors models in a directory
    #[cfg(test)]
    pub fn discover_safetensors_models(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut models = Vec::new();

        if !dir.exists() || !dir.is_dir() {
            return Ok(models);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && Self::is_safetensors_model(&path) {
                models.push(path);
            }
        }

        Ok(models)
    }
}

#[async_trait]
impl InferenceEngine for SafeTensorsEngine {
    async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
        info!("Loading SafeTensors model: {}", spec.base_path.display());

        // Check if it's actually a SafeTensors file
        if !Self::is_safetensors_model(&spec.base_path) {
            return Err(anyhow!(
                "File is not in SafeTensors format: {}",
                spec.base_path.display()
            ));
        }

        // Check cache first with read lock
        let _cached_metadata = {
            // let cache = self.cache.read().unwrap();
            // cache.get(&spec.base_path).cloned()
            None::<()> // Temporarily disable cache
        };

        let model = /* if let Some(metadata) = cached_metadata {
            info!("Found cached metadata for {}", spec.base_path.display());
            SafeTensorsModel::load_from_cached_metadata(spec, &metadata, self.use_mmap).await?
        } else */ {
            info!("Loading model file directly (cache disabled)");
            // Load model and cache metadata
            let model = SafeTensorsModel::load_and_cache(spec, self.use_mmap).await?;
            model
        };
        Ok(Box::new(model))
    }
}

#[derive(Debug)]
enum ModelData {
    InMemory(Vec<u8>),
    MemoryMapped(memmap2::Mmap),
}

impl ModelData {
    fn as_bytes(&self) -> &[u8] {
        match self {
            ModelData::InMemory(data) => data,
            ModelData::MemoryMapped(mmap) => mmap,
        }
    }
}

#[derive(Debug)]
struct SafeTensorsModel {
    name: String,
    model_data: ModelData, // Either in-memory or memory-mapped
    config: ModelConfig,
    tokenizer: SimpleTokenizer,
}

#[derive(Debug, Clone)]
struct ModelConfig {
    vocab_size: usize,
    #[allow(dead_code)]
    hidden_size: usize,
    num_layers: usize,
    #[allow(dead_code)]
    max_sequence_length: usize,
    // Add more config fields as needed
}

#[derive(Debug)]
struct SimpleTokenizer {
    // Simple tokenizer implementation
    vocab: HashMap<String, u32>,
    #[allow(dead_code)]
    reverse_vocab: HashMap<u32, String>,
    bos_token: u32,
    #[allow(dead_code)]
    eos_token: u32,
}

impl SafeTensorsModel {
    /// Load model and cache metadata (for new models not in cache)
    async fn load_and_cache(
        spec: &ModelSpec,
        /* cache: &RwLock<ModelCache>, */ use_mmap: bool,
    ) -> Result<Self> {
        info!("Reading SafeTensors file: {}", spec.base_path.display());

        // Choose loading strategy based on file size and mmap setting
        let file_size = fs::metadata(&spec.base_path)?.len();
        let use_mmap_for_file = use_mmap && file_size > 100 * 1024 * 1024; // Use mmap for files > 100MB

        let model_data = if use_mmap_for_file {
            info!(
                "Using memory-mapped loading for large model ({:.1} MB)",
                file_size as f64 / 1024.0 / 1024.0
            );
            let file = File::open(&spec.base_path)?;
            let mmap = unsafe { MmapOptions::new().map(&file)? };
            ModelData::MemoryMapped(mmap)
        } else {
            info!(
                "Loading model into memory ({:.1} MB)",
                file_size as f64 / 1024.0 / 1024.0
            );
            let data = fs::read(&spec.base_path)?;
            ModelData::InMemory(data)
        };

        // Parse SafeTensors format to validate and get info
        let tensors = SafeTensors::deserialize(model_data.as_bytes())?;

        debug!("SafeTensors loaded with {} tensors", tensors.len());

        // Extract metadata for caching
        // let metadata = model_cache::extract_safetensors_metadata(&spec.base_path)?;

        // Cache the metadata for future loads
        // {
        //     let mut cache_guard = cache.write().unwrap();
        //     if let Err(e) = cache_guard.set(metadata.clone()) {
        //         warn!("Failed to cache metadata: {}", e);
        //     }
        // } // Drop the lock before async operations

        // Load configuration from cached metadata if available, otherwise parse
        let config = /* if let Some(config_data) = &metadata.config {
            Self::parse_config_from_json(config_data)?
        } else */ {
            Self::load_config(&spec.base_path, &tensors).await?
        };

        // Load tokenizer from cached metadata if available, otherwise parse
        let tokenizer = /* if let Some(tokenizer_data) = &metadata.tokenizer {
            Self::parse_tokenizer_from_json(tokenizer_data)?
        } else */ {
            Self::load_tokenizer(&spec.base_path).await?
        };

        Ok(SafeTensorsModel {
            name: spec.name.clone(),
            model_data,
            config,
            tokenizer,
        })
    }

    // Load model from cached metadata (much faster)
    /* async fn load_from_cached_metadata(spec: &ModelSpec, metadata: &ModelMetadata, use_mmap: bool) -> Result<Self> {
        info!("Loading model from cached metadata");

        // Choose loading strategy based on file size and mmap setting
        let file_size = fs::metadata(&spec.base_path)?.len();
        let use_mmap_for_file = use_mmap && file_size > 100 * 1024 * 1024; // Use mmap for files > 100MB

        let model_data = if use_mmap_for_file {
            info!("Using memory-mapped loading for cached model ({:.1} MB)", file_size as f64 / 1024.0 / 1024.0);
            let file = File::open(&spec.base_path)?;
            let mmap = unsafe { MmapOptions::new().map(&file)? };
            ModelData::MemoryMapped(mmap)
        } else {
            info!("Loading cached model into memory ({:.1} MB)", file_size as f64 / 1024.0 / 1024.0);
            let data = fs::read(&spec.base_path)?;
            ModelData::InMemory(data)
        };

        // Parse config from cached metadata
        let config = if let Some(config_data) = &metadata.config {
            Self::parse_config_from_json(config_data)?
        } else {
            // Fallback to file-based loading if not in cache
            let tensors = SafeTensors::deserialize(model_data.as_bytes())?;
            Self::load_config(&spec.base_path, &tensors).await?
        };

        // Parse tokenizer from cached metadata
        let tokenizer = if let Some(tokenizer_data) = &metadata.tokenizer {
            Self::parse_tokenizer_from_json(tokenizer_data)?
        } else {
            // Fallback to file-based loading if not in cache
            Self::load_tokenizer(&spec.base_path).await?
        };

        debug!("Model loaded from cache with {} cached tensors", metadata.tensors.len());

        Ok(SafeTensorsModel {
            name: spec.name.clone(),
            model_data,
            config,
            tokenizer,
        })
    } */

    /// Parse configuration from cached JSON data
    #[allow(dead_code)]
    fn parse_config_from_json(config_data: &serde_json::Value) -> Result<ModelConfig> {
        let vocab_size = config_data
            .get("vocab_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(32000) as usize;

        let hidden_size = config_data
            .get("hidden_size")
            .or_else(|| config_data.get("hidden_dim"))
            .and_then(|v| v.as_u64())
            .unwrap_or(4096) as usize;

        let num_layers = config_data
            .get("num_hidden_layers")
            .or_else(|| config_data.get("num_layers"))
            .and_then(|v| v.as_u64())
            .unwrap_or(32) as usize;

        let max_sequence_length = config_data
            .get("max_position_embeddings")
            .or_else(|| config_data.get("max_seq_len"))
            .and_then(|v| v.as_u64())
            .unwrap_or(2048) as usize;

        Ok(ModelConfig {
            vocab_size,
            hidden_size,
            num_layers,
            max_sequence_length,
        })
    }

    /// Parse tokenizer from cached JSON data
    #[allow(dead_code)]
    fn parse_tokenizer_from_json(tokenizer_data: &serde_json::Value) -> Result<SimpleTokenizer> {
        let mut vocab = HashMap::new();
        let mut reverse_vocab = HashMap::new();

        // Parse vocab from tokenizer JSON
        if let Some(vocab_obj) = tokenizer_data.get("model").and_then(|m| m.get("vocab")) {
            if let Some(vocab_map) = vocab_obj.as_object() {
                for (token, id) in vocab_map {
                    if let Some(token_id) = id.as_u64() {
                        let token_id = token_id as u32;
                        vocab.insert(token.clone(), token_id);
                        reverse_vocab.insert(token_id, token.clone());
                    }
                }
            }
        }

        // Default special tokens
        let bos_token = vocab.get("<s>").copied().unwrap_or(1);
        let eos_token = vocab.get("</s>").copied().unwrap_or(2);

        Ok(SimpleTokenizer {
            vocab,
            reverse_vocab,
            bos_token,
            eos_token,
        })
    }

    async fn load_config(model_path: &Path, tensors: &SafeTensors<'_>) -> Result<ModelConfig> {
        // Try to load config.json from same directory
        let config_path = model_path.with_file_name("config.json");

        if config_path.exists() {
            let config_data = fs::read_to_string(&config_path)?;
            let json: serde_json::Value = serde_json::from_str(&config_data)?;

            let vocab_size = json["vocab_size"].as_u64().unwrap_or(32000) as usize;
            let hidden_size = json["hidden_size"].as_u64().unwrap_or(4096) as usize;
            let num_layers = json["num_hidden_layers"].as_u64().unwrap_or(32) as usize;
            let max_length = json["max_position_embeddings"].as_u64().unwrap_or(2048) as usize;

            return Ok(ModelConfig {
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             