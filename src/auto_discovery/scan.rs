use super::{DiscoveredModel, ModelAutoDiscovery};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

impl ModelAutoDiscovery {
    pub(super) fn scan_directory(&self, dir: &Path) -> Result<Vec<DiscoveredModel>> {
        self.scan_directory_with_depth(dir, 0)
    }

    pub(super) fn scan_directory_with_depth(
        &self,
        dir: &Path,
        depth: usize,
    ) -> Result<Vec<DiscoveredModel>> {
        // Prevent infinite recursion - limit depth to 4 levels for performance
        if depth >= 4 {
            return Ok(Vec::new());
        }

        // Skip system directories that cause problems on macOS and other systems
        if let Some(dir_name) = dir.file_name().and_then(|n| n.to_str()) {
            // Skip hidden directories except known model directories
            if dir_name.starts_with('.')
                && dir_name != ".cache"
                && dir_name != ".ollama"
                && dir_name != ".local"
            {
                return Ok(Vec::new());
            }

            // Skip problematic macOS directories
            match dir_name {
                "Library" | "Applications" | "System" | "Developer" | "usr" | "var" | "tmp"
                | "private" | "Volumes" | "cores" | "dev" | "etc" | "home" | "net" | "proc"
                | "opt" | "sbin" | "bin" => {
                    return Ok(Vec::new());
                }
                _ => {}
            }

            // Skip Windows system directories
            #[cfg(windows)]
            match dir_name.to_lowercase().as_str() {
                "windows"
                | "program files"
                | "program files (x86)"
                | "programdata"
                | "users"
                | "system volume information"
                | "$recycle.bin"
                | "recovery" => {
                    return Ok(Vec::new());
                }
                _ => {}
            }
        }

        let mut models = Vec::new();
        let mut model_files = Vec::new();

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
                model_files.push(path);
            }
        }

        // Group sharded models and analyze them
        let grouped_models = self.group_sharded_models(dir, &model_files)?;
        models.extend(grouped_models);

        Ok(models)
    }

    /// Group sharded model files together (Issue #147)
    /// Detects patterns like model-00001-of-00004.safetensors and groups them as single models
    pub(super) fn group_sharded_models(
        &self,
        dir: &Path,
        model_files: &[PathBuf],
    ) -> Result<Vec<DiscoveredModel>> {
        use regex::Regex;
        use std::collections::HashMap;

        let mut grouped_models = Vec::new();
        let mut processed_files = std::collections::HashSet::new();

        // Regex to match sharded model patterns: model-XXXXX-of-XXXXX.ext
        let shard_pattern = Regex::new(r"^(.+)-\d{5}-of-\d{5}(\..+)$").unwrap();

        // Group files by their base name (without shard numbers)
        let mut shard_groups: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for file_path in model_files {
            let filename = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if let Some(caps) = shard_pattern.captures(filename) {
                let base_name = caps[1].to_string();
                let ext = caps[2].to_string();
                let key = format!("{}{}", base_name, ext);
                shard_groups.entry(key).or_default().push(file_path.clone());
                processed_files.insert(file_path.clone());
            }
        }

        // Process shard groups
        for (group_key, shard_files) in &shard_groups {
            if shard_files.len() > 1 {
                // Multiple shards: create a single entry using the first shard as representative
                let mut sorted_shards = shard_files.clone();
                sorted_shards.sort();
                let representative = &sorted_shards[0];

                if let Ok(mut model) = self.analyze_model_file(representative) {
                    // Total size is sum of all shards
                    let total_size: u64 = sorted_shards
                        .iter()
                        .filter_map(|p| fs::metadata(p).ok())
                        .map(|m| m.len())
                        .sum();
                    model.size_bytes = total_size;

                    // Use base name without shard suffix for display
                    let base_without_ext = group_key
                        .rsplit_once('.')
                        .map(|(name, _)| name)
                        .unwrap_or(group_key);
                    model.name = base_without_ext
                        .replace("_", "-")
                        .replace(" ", "-")
                        .to_lowercase();

                    grouped_models.push(model);
                }
            }
        }

        // Process non-sharded files
        for file_path in model_files {
            if !processed_files.contains(file_path) {
                if let Ok(model) = self.analyze_model_file(file_path) {
                    grouped_models.push(model);
                }
            }
        }

        Ok(grouped_models)
    }

    pub(super) fn is_model_file(&self, path: &Path) -> bool {
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

    pub(super) fn is_lora_file(&self, path: &Path) -> bool {
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

    pub(super) fn analyze_model_file(&self, path: &Path) -> Result<DiscoveredModel> {
        let metadata = fs::metadata(path)?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let (model_type, parameter_count, quantization) = self.parse_filename(&filename);

        // CRITICAL: All GGUF files must use Llama backend (PPT Invariant requirement)
        // GGUF is the llama.cpp format, regardless of model family name
        let backend_type = if path.extension().and_then(|s| s.to_str()) == Some("gguf") {
            "Llama".to_string()
        } else {
            model_type
        };

        // Generate a clean model name
        let name = self.generate_model_name(&filename);

        // Look for paired LoRA adapter
        let lora_path = self.find_lora_for_model(path);

        Ok(DiscoveredModel {
            name,
            path: path.to_path_buf(),
            lora_path,
            size_bytes: metadata.len(),
            model_type: backend_type,
            parameter_count,
            quantization,
        })
    }

    pub(super) fn parse_filename(&self, filename: &str) -> (String, Option<String>, Option<String>) {
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

    pub(super) fn generate_model_name(&self, filename: &str) -> String {
        // Remove file extension
        let name = if let Some(pos) = filename.rfind('.') {
            &filename[..pos]
        } else {
            filename
        };

        // Replace common separators with dashes
        name.replace("_", "-").replace(" ", "-").to_lowercase()
    }
}
