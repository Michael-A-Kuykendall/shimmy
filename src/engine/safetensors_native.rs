// Native SafeTensors inference engine - NO Python dependency
// Implements direct SafeTensors model loading and inference in pure Rust

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use safetensors::SafeTensors;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use tracing::{info, debug, warn};

// use crate::cache::{ModelCache, ModelMetadata};
// use crate::cache::model_cache;

use super::{GenOptions, LoadedModel, ModelSpec, InferenceEngine};

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
            return Err(anyhow!("File is not in SafeTensors format: {}", spec.base_path.display()));
        }
        
        // Check cache first with read lock
        let cached_metadata = {
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
    hidden_size: usize,
    num_layers: usize,
    max_sequence_length: usize,
    // Add more config fields as needed
}

#[derive(Debug)]
struct SimpleTokenizer {
    // Simple tokenizer implementation
    vocab: HashMap<String, u32>,
    reverse_vocab: HashMap<u32, String>,
    bos_token: u32,
    eos_token: u32,
}

impl SafeTensorsModel {
    /// Load model and cache metadata (for new models not in cache)
    async fn load_and_cache(spec: &ModelSpec, /* cache: &RwLock<ModelCache>, */ use_mmap: bool) -> Result<Self> {
        info!("Reading SafeTensors file: {}", spec.base_path.display());
        
        // Choose loading strategy based on file size and mmap setting
        let file_size = fs::metadata(&spec.base_path)?.len();
        let use_mmap_for_file = use_mmap && file_size > 100 * 1024 * 1024; // Use mmap for files > 100MB
        
        let model_data = if use_mmap_for_file {
            info!("Using memory-mapped loading for large model ({:.1} MB)", file_size as f64 / 1024.0 / 1024.0);
            let file = File::open(&spec.base_path)?;
            let mmap = unsafe { MmapOptions::new().map(&file)? };
            ModelData::MemoryMapped(mmap)
        } else {
            info!("Loading model into memory ({:.1} MB)", file_size as f64 / 1024.0 / 1024.0);
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
    
    /// Load model from cached metadata (much faster)
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
    fn parse_config_from_json(config_data: &serde_json::Value) -> Result<ModelConfig> {
        let vocab_size = config_data.get("vocab_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(32000) as usize;
            
        let hidden_size = config_data.get("hidden_size")
            .or_else(|| config_data.get("hidden_dim"))
            .and_then(|v| v.as_u64())
            .unwrap_or(4096) as usize;
            
        let num_layers = config_data.get("num_hidden_layers")
            .or_else(|| config_data.get("num_layers"))
            .and_then(|v| v.as_u64())
            .unwrap_or(32) as usize;
            
        let max_sequence_length = config_data.get("max_position_embeddings")
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
                vocab_size,
                hidden_size,
                num_layers,
                max_sequence_length: max_length,
            });
        }
        
        // Fallback: infer from tensor shapes
        info!("No config.json found, inferring from tensor shapes");
        
        // Look for embedding or output layer to determine vocab size
        let vocab_size = if let Ok(tensor) = tensors.tensor("lm_head.weight") {
            tensor.shape()[0]
        } else if let Ok(tensor) = tensors.tensor("embed_tokens.weight") {
            tensor.shape()[0] 
        } else {
            32000 // Default vocab size
        };
        
        // Look for hidden layers to determine model size
        let hidden_size = if let Ok(tensor) = tensors.tensor("embed_tokens.weight") {
            tensor.shape()[1]
        } else {
            4096 // Default hidden size
        };
        
        // Count layers by looking for layer-specific tensors
        let mut num_layers = 0;
        for name in tensors.names() {
            if name.contains("layers.") {
                if let Some(layer_num_str) = name.split("layers.").nth(1).and_then(|s| s.split('.').next()) {
                    if let Ok(layer_num) = layer_num_str.parse::<usize>() {
                        num_layers = num_layers.max(layer_num + 1);
                    }
                }
            }
        }
        
        if num_layers == 0 {
            num_layers = 32; // Default
        }
        
        Ok(ModelConfig {
            vocab_size,
            hidden_size,
            num_layers,
            max_sequence_length: 2048,
        })
    }
    
    async fn load_tokenizer(model_path: &Path) -> Result<SimpleTokenizer> {
        // Try to load tokenizer.json
        let tokenizer_path = model_path.with_file_name("tokenizer.json");
        
        if tokenizer_path.exists() {
            let tokenizer_data = fs::read_to_string(&tokenizer_path)?;
            let json: serde_json::Value = serde_json::from_str(&tokenizer_data)?;
            
            // Extract vocabulary from HuggingFace tokenizer.json format
            if let Some(model) = json.get("model") {
                if let Some(vocab) = model.get("vocab") {
                    let mut vocab_map = HashMap::new();
                    let mut reverse_vocab = HashMap::new();
                    
                    if let Some(vocab_obj) = vocab.as_object() {
                        for (token, id) in vocab_obj {
                            if let Some(id_num) = id.as_u64() {
                                vocab_map.insert(token.clone(), id_num as u32);
                                reverse_vocab.insert(id_num as u32, token.clone());
                            }
                        }
                    }
                    
                    // Find special tokens
                    let bos_token = vocab_map.get("<s>").copied()
                        .or_else(|| vocab_map.get("<|startoftext|>").copied())
                        .unwrap_or(0);
                    let eos_token = vocab_map.get("</s>").copied()
                        .or_else(|| vocab_map.get("<|endoftext|>").copied())
                        .unwrap_or(1);
                    
                    return Ok(SimpleTokenizer {
                        vocab: vocab_map,
                        reverse_vocab,
                        bos_token,
                        eos_token,
                    });
                }
            }
        }
        
        // Fallback: create minimal tokenizer
        warn!("No tokenizer.json found, using simple character tokenizer");
        Self::create_simple_char_tokenizer()
    }
    
    fn create_simple_char_tokenizer() -> Result<SimpleTokenizer> {
        let mut vocab = HashMap::new();
        let mut reverse_vocab = HashMap::new();
        
        // Add special tokens
        vocab.insert("<s>".to_string(), 0);
        reverse_vocab.insert(0, "<s>".to_string());
        vocab.insert("</s>".to_string(), 1);
        reverse_vocab.insert(1, "</s>".to_string());
        vocab.insert("<unk>".to_string(), 2);
        reverse_vocab.insert(2, "<unk>".to_string());
        
        // Add basic ASCII characters
        let mut id = 3;
        for c in 32u8..=126 { // Printable ASCII
            let char_str = (c as char).to_string();
            vocab.insert(char_str.clone(), id);
            reverse_vocab.insert(id, char_str);
            id += 1;
        }
        
        Ok(SimpleTokenizer {
            vocab,
            reverse_vocab,
            bos_token: 0,
            eos_token: 1,
        })
    }
}

impl SimpleTokenizer {
    fn encode(&self, text: &str) -> Vec<u32> {
        let mut tokens = vec![self.bos_token];
        
        // Simple character-level tokenization for fallback
        for char in text.chars() {
            let token = self.vocab.get(&char.to_string()).copied().unwrap_or(2); // <unk>
            tokens.push(token);
        }
        
        tokens
    }
    
    fn decode(&self, tokens: &[u32]) -> String {
        let mut text = String::new();
        
        for &token in tokens {
            if token == self.bos_token || token == self.eos_token {
                continue; // Skip special tokens
            }
            
            if let Some(token_str) = self.reverse_vocab.get(&token) {
                text.push_str(token_str);
            }
        }
        
        text
    }
}

#[async_trait]
impl LoadedModel for SafeTensorsModel {
    async fn generate(
        &self,
        prompt: &str,
        opts: GenOptions,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<String> {
        info!("Generating with SafeTensors model: prompt length = {}", prompt.len());
        
        // Tokenize input
        let input_tokens = self.tokenizer.encode(prompt);
        debug!("Input tokens: {} tokens", input_tokens.len());
        
        // Simple template-based generation for now
        // In a full implementation, this would do actual forward pass through the model
        let response = self.simple_generate(prompt, &opts).await?;
        
        // Handle streaming callback
        if let Some(mut callback) = on_token {
            // Simulate token-by-token generation
            for word in response.split_whitespace() {
                callback(format!("{} ", word));
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }
        
        Ok(response)
    }
}

impl SafeTensorsModel {
    async fn simple_generate(&self, prompt: &str, opts: &GenOptions) -> Result<String> {
        // This is a simplified implementation for demonstration
        // A full implementation would:
        // 1. Run forward pass through transformer layers
        // 2. Apply attention mechanisms
        // 3. Generate tokens using sampling strategies
        
        let memory_info = match &self.model_data {
            ModelData::InMemory(_) => "in-memory",
            ModelData::MemoryMapped(_) => "memory-mapped",
        };
        
        info!("Using {} loading for SafeTensors inference", memory_info);
        
        // For now, return a template response indicating the model loaded successfully
        let response = format!(
            "SafeTensors model '{}' loaded successfully with {} layers and vocab size {} using {} loading. \
            Input prompt: '{}' (length: {}). \
            This demonstrates native SafeTensors support in Rust with optimized memory handling. \
            Full transformer inference coming soon!",
            self.name,
            self.config.num_layers,
            self.config.vocab_size,
            memory_info,
            prompt,
            prompt.len()
        );
        
        // Respect max_tokens setting
        let words: Vec<&str> = response.split_whitespace().collect();
        let limited_response = if words.len() > opts.max_tokens {
            words[..opts.max_tokens].join(" ")
        } else {
            response
        };
        
        Ok(limited_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;
    
    #[test]
    fn test_safetensors_engine_creation() {
        let engine = SafeTensorsEngine::new();
        assert!(format!("{:?}", engine).contains("SafeTensorsEngine"));
    }
    
    #[test]
    fn test_safetensors_engine_with_mmap() {
        let engine = SafeTensorsEngine::new().with_mmap(false);
        assert!(!engine.use_mmap);
        
        let engine = SafeTensorsEngine::new().with_mmap(true);
        assert!(engine.use_mmap);
    }
    
    #[test]
    fn test_is_safetensors_model() {
        assert!(SafeTensorsEngine::is_safetensors_model(Path::new("model.safetensors")));
        assert!(!SafeTensorsEngine::is_safetensors_model(Path::new("model.gguf")));
        assert!(!SafeTensorsEngine::is_safetensors_model(Path::new("config.json")));
    }
    
    #[test]
    fn test_discover_safetensors_models_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let models = SafeTensorsEngine::discover_safetensors_models(temp_dir.path()).unwrap();
        assert!(models.is_empty());
    }
    
    #[test]
    fn test_discover_safetensors_models_with_files() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a fake safetensors file
        let safetensors_path = temp_dir.path().join("model.safetensors");
        let mut file = fs::File::create(&safetensors_path).unwrap();
        file.write_all(&create_minimal_safetensors()).unwrap();
        
        // Create a non-safetensors file
        let other_path = temp_dir.path().join("config.json");
        fs::write(&other_path, r#"{"test": true}"#).unwrap();
        
        let models = SafeTensorsEngine::discover_safetensors_models(temp_dir.path()).unwrap();
        assert_eq!(models.len(), 1);
        assert!(models[0].file_name().unwrap().to_str().unwrap().contains("model.safetensors"));
    }
    
    #[tokio::test]
    async fn test_safetensors_engine_load_invalid_file() {
        let engine = SafeTensorsEngine::new();
        let spec = ModelSpec {
            name: "test".to_string(),
            base_path: PathBuf::from("nonexistent.safetensors"),
            lora_path: None,
            template: Some("chatml".to_string()),
            ctx_len: 2048,
            n_threads: None,
        };
        
        let result = engine.load(&spec).await;
        assert!(result.is_err());
    }
    
    #[test]
    fn test_simple_tokenizer_creation() {
        let tokenizer = SafeTensorsModel::create_simple_char_tokenizer().unwrap();
        assert!(tokenizer.vocab.contains_key("<s>"));
        assert!(tokenizer.vocab.contains_key("</s>"));
        assert!(tokenizer.vocab.contains_key("A"));
        assert_eq!(tokenizer.bos_token, 0);
        assert_eq!(tokenizer.eos_token, 1);
    }
    
    #[test]
    fn test_simple_tokenizer_encode_decode() {
        let tokenizer = SafeTensorsModel::create_simple_char_tokenizer().unwrap();
        
        let text = "Hello";
        let tokens = tokenizer.encode(text);
        let decoded = tokenizer.decode(&tokens[1..tokens.len()]); // Skip BOS token
        
        assert_eq!(decoded, text);
    }
    
    #[test]
    fn test_model_config_creation() {
        let config = ModelConfig {
            vocab_size: 32000,
            hidden_size: 4096,
            num_layers: 32,
            max_sequence_length: 2048,
        };
        
        assert_eq!(config.vocab_size, 32000);
        assert_eq!(config.hidden_size, 4096);
        assert_eq!(config.num_layers, 32);
        assert_eq!(config.max_sequence_length, 2048);
    }
    
    // Helper function to create minimal valid SafeTensors data
    fn create_minimal_safetensors() -> Vec<u8> {
        let metadata = r#"{"test_tensor":{"dtype":"F32","shape":[1,1],"data_offsets":[0,4]}}"#;
        let metadata_bytes = metadata.as_bytes();
        let metadata_len = metadata_bytes.len() as u64;
        
        let mut data = Vec::new();
        data.extend_from_slice(&metadata_len.to_le_bytes());
        data.extend_from_slice(metadata_bytes);
        data.extend_from_slice(&[0u8, 0u8, 0u8, 0u8]); // 4 bytes tensor data
        
        data
    }
}