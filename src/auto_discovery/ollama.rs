use super::{DiscoveredModel, ModelAutoDiscovery};
use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub(super) struct OllamaManifest {
    #[serde(rename = "schemaVersion")]
    #[allow(dead_code)] // Ollama manifest schema field — present in JSON, consumed by serde
    schema_version: i32,
    #[serde(rename = "mediaType")]
    #[allow(dead_code)] // Ollama manifest media type — present in JSON, consumed by serde
    media_type: String,
    #[allow(dead_code)] // Ollama manifest config section — present in JSON, consumed by serde
    config: OllamaConfig,
    layers: Vec<OllamaLayer>,
}

#[derive(Debug, Deserialize)]
pub(super) struct OllamaConfig {
    #[serde(rename = "mediaType")]
    #[allow(dead_code)] // Ollama config media type — present in JSON, consumed by serde
    media_type: String,
    #[allow(dead_code)] // Ollama config digest — present in JSON, consumed by serde
    digest: String,
    #[allow(dead_code)] // Ollama config size — present in JSON, consumed by serde
    size: i64,
}

#[derive(Debug, Deserialize)]
pub(super) struct OllamaLayer {
    #[serde(rename = "mediaType")]
    pub(super) media_type: String,
    pub(super) digest: String,
    pub(super) size: i64,
}

impl ModelAutoDiscovery {
    pub(super) fn discover_ollama_models(&self) -> Result<Vec<DiscoveredModel>> {
        let mut models = Vec::new();

        // Collect potential Ollama directories to check
        let mut ollama_dirs = Vec::new();

        // Check OLLAMA_MODELS env var first
        if let Ok(ollama_models) = std::env::var("OLLAMA_MODELS") {
            ollama_dirs.push(PathBuf::from(ollama_models));
        }

        // Check SHIMMY_BASE_GGUF parent directory for Ollama structure
        if let Ok(shimmy_base) = std::env::var("SHIMMY_BASE_GGUF") {
            let path = PathBuf::from(shimmy_base);
            if let Some(parent) = path.parent() {
                // Check if we're directly in an Ollama structure
                ollama_dirs.push(parent.to_path_buf());

                // Also check if we're in a 'blobs' directory - go up one more level
                if parent.file_name().and_then(|n| n.to_str()) == Some("blobs") {
                    if let Some(grandparent) = parent.parent() {
                        ollama_dirs.push(grandparent.to_path_buf());
                    }
                }
            }
        }

        // Add standard Ollama locations
        if let Some(home) = std::env::var_os("HOME") {
            ollama_dirs.push(PathBuf::from(home).join(".ollama/models"));
        }
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            ollama_dirs.push(PathBuf::from(user_profile).join(".ollama").join("models"));
        }

        // Check each potential Ollama directory
        for ollama_dir in ollama_dirs {
            if !ollama_dir.exists() {
                continue;
            }

            let manifests_dir = ollama_dir.join("manifests");
            let blobs_dir = ollama_dir.join("blobs");

            // Try new manifest/blob format first
            if manifests_dir.exists() && blobs_dir.exists() {
                models.extend(self.discover_ollama_manifest_models(&manifests_dir, &blobs_dir)?);
            }

            // Fallback: scan for GGUF files directly in ollama directory structure
            // This handles legacy Ollama installations and custom directory layouts
            models.extend(self.discover_ollama_direct_models(&ollama_dir)?);
        }

        Ok(models)
    }

    fn discover_ollama_manifest_models(
        &self,
        manifests_dir: &Path,
        blobs_dir: &Path,
    ) -> Result<Vec<DiscoveredModel>> {
        let mut models = Vec::new();

        // Recursively scan manifests directory to find all manifest files
        self.scan_manifest_directory(manifests_dir, blobs_dir, &mut models, Vec::new())?;

        Ok(models)
    }

    fn scan_manifest_directory(
        &self,
        dir: &Path,
        blobs_dir: &Path,
        models: &mut Vec<DiscoveredModel>,
        path_components: Vec<String>,
    ) -> Result<()> {
        for entry in
            fs::read_dir(dir).map_err(|_| anyhow::anyhow!("Cannot read directory: {:?}", dir))?
        {
            let entry = entry?;
            let entry_name = entry.file_name().to_string_lossy().to_string();
            let mut new_path_components = path_components.clone();
            new_path_components.push(entry_name.clone());

            if entry.path().is_dir() {
                // Recursively scan subdirectories
                self.scan_manifest_directory(
                    &entry.path(),
                    blobs_dir,
                    models,
                    new_path_components,
                )?;
            } else if entry.path().is_file() {
                // This is a manifest file, try to parse it
                if let Ok(manifest_content) = fs::read_to_string(entry.path()) {
                    if let Ok(manifest) = serde_json::from_str::<OllamaManifest>(&manifest_content)
                    {
                        // Find the model blob (largest layer that's likely a GGUF)
                        for layer in &manifest.layers {
                            if layer.media_type == "application/vnd.ollama.image.model" {
                                if let Some(hash) = layer.digest.strip_prefix("sha256:") {
                                    let blob_path = blobs_dir.join(format!("sha256-{}", hash));
                                    if blob_path.exists()
                                        && self.is_gguf_blob(&blob_path).unwrap_or(false)
                                    {
                                        // Build display name from path components
                                        let display_name = if path_components.len() >= 2 {
                                            // Format: registry/namespace/model:tag or namespace/model:tag
                                            let mut name_parts = path_components.clone();
                                            name_parts.push(entry_name.clone());
                                            name_parts.join("/")
                                        } else {
                                            // Fallback to simple name
                                            format!("{}:{}", path_components.join("/"), entry_name)
                                        };

                                        let discovered = DiscoveredModel {
                                            name: display_name,
                                            path: blob_path,
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
        Ok(())
    }

    fn discover_ollama_direct_models(&self, ollama_dir: &Path) -> Result<Vec<DiscoveredModel>> {
        let mut models = Vec::new();

        // Skip manifest and blob directories to avoid duplicate detection
        let skip_dirs = ["manifests", "blobs"];

        // Recursively scan ollama directory for GGUF files
        if let Ok(entries) = fs::read_dir(ollama_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    // Skip manifest/blob dirs and hidden dirs
                    if skip_dirs.contains(&dir_name) || dir_name.starts_with('.') {
                        continue;
                    }

                    // Recursively scan subdirectories
                    models.extend(self.discover_ollama_direct_models_recursive(&path, 0)?);
                } else if self.is_model_file(&path) {
                    // Found a model file directly in ollama directory
                    if let Ok(mut model) = self.analyze_model_file(&path) {
                        // Prefix with ollama: to distinguish from other sources
                        model.name = format!("ollama:{}", model.name);
                        model.model_type = "Ollama".to_string();
                        models.push(model);
                    }
                }
            }
        }

        Ok(models)
    }

    fn discover_ollama_direct_models_recursive(
        &self,
        dir: &Path,
        depth: usize,
    ) -> Result<Vec<DiscoveredModel>> {
        let mut models = Vec::new();

        // Limit recursion depth to prevent infinite loops
        if depth >= 5 {
            return Ok(models);
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    // Skip hidden directories and common non-model dirs
                    if dir_name.starts_with('.') || dir_name == "tmp" || dir_name == "cache" {
                        continue;
                    }

                    models.extend(self.discover_ollama_direct_models_recursive(&path, depth + 1)?);
                } else if self.is_model_file(&path) {
                    if let Ok(mut model) = self.analyze_model_file(&path) {
                        // Extract model name from directory structure for better naming
                        let relative_path = path
                            .strip_prefix(dir.ancestors().nth(depth).unwrap_or(dir))
                            .unwrap_or(&path);
                        let parent_name = relative_path
                            .parent()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");

                        model.name = format!("ollama:{}", parent_name);
                        model.model_type = "Ollama".to_string();
                        models.push(model);
                    }
                }
            }
        }

        Ok(models)
    }

    pub(super) fn is_gguf_blob(&self, path: &Path) -> Result<bool> {
        let mut file = std::fs::File::open(path)?;
        let mut buffer = [0u8; 4];
        use std::io::Read;
        file.read_exact(&mut buffer)?;
        Ok(&buffer == b"GGUF")
    }
}
