use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use shimmy::model_registry::ModelAutoDiscovery;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sharded_model_grouping() {
        // Create a temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let models_dir = temp_dir.path().join("models");
        fs::create_dir(&models_dir).unwrap();

        // Create sharded model files (4 files)
        let shard_files = vec![
            "model-00001-of-00004.safetensors",
            "model-00002-of-00004.safetensors",
            "model-00003-of-00004.safetensors",
            "model-00004-of-00004.safetensors",
        ];

        for filename in &shard_files {
            let file_path = models_dir.join(filename);
            fs::write(&file_path, "dummy content").unwrap();
        }

        // Create a non-sharded model file
        let single_file = models_dir.join("single-model.gguf");
        fs::write(&single_file, "dummy gguf content").unwrap();

        // Initialize auto-discovery
        let discovery = ModelAutoDiscovery::new();

        // Test the grouping logic
        let model_files = vec![
            models_dir.join("model-00001-of-00004.safetensors"),
            models_dir.join("model-00002-of-00004.safetensors"),
            models_dir.join("model-00003-of-00004.safetensors"),
            models_dir.join("model-00004-of-00004.safetensors"),
            single_file.clone(),
        ];

        let grouped_models = discovery.group_sharded_models(&models_dir, &model_files).unwrap();

        // Verify results
        assert_eq!(grouped_models.len(), 2, "Should have 2 model entries: 1 grouped sharded + 1 single");

        // Find the sharded model entry
        let sharded_model = grouped_models.iter().find(|m| m.name == "models").unwrap();

        // Verify sharded model properties
        assert!(sharded_model.path.to_string_lossy().contains("(+3 more files)"),
                "Sharded model path should indicate additional files: {}", sharded_model.path.display());
        assert_eq!(sharded_model.size_bytes, 52, "Total size should be sum of all 4 files (13 bytes each * 4)");

        // Find the single model entry
        let single_model = grouped_models.iter().find(|m| m.name == "single-model").unwrap();

        // Verify single model properties
        assert_eq!(single_model.path, single_file, "Single model should have correct path");
        assert_eq!(single_model.size_bytes, 19, "Single model should have correct size");
    }

    #[test]
    fn test_non_sharded_files_not_grouped() {
        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let models_dir = temp_dir.path().join("models");
        fs::create_dir(&models_dir).unwrap();

        // Create non-sharded model files
        let files = vec![
            "model1.gguf",
            "model2.safetensors",
            "another-model.gguf",
        ];

        let mut model_files = Vec::new();
        for filename in &files {
            let file_path = models_dir.join(filename);
            fs::write(&file_path, "dummy content").unwrap();
            model_files.push(file_path);
        }

        // Initialize auto-discovery
        let discovery = ModelAutoDiscovery::new();

        // Test the grouping logic
        let grouped_models = discovery.group_sharded_models(&models_dir, &model_files).unwrap();

        // All files should be treated as individual models (no sharding detected)
        assert_eq!(grouped_models.len(), 3, "Should have 3 individual model entries");

        // Verify each model has its own entry
        let model_names: Vec<String> = grouped_models.iter().map(|m| m.name.clone()).collect();
        assert!(model_names.contains(&"model1".to_string()));
        assert!(model_names.contains(&"model2".to_string()));
        assert!(model_names.contains(&"another-model".to_string()));
    }

    #[test]
    fn test_mixed_sharded_and_single_files() {
        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let models_dir = temp_dir.path().join("models");
        fs::create_dir(&models_dir).unwrap();

        // Create mixed files: some sharded, some single
        let shard_files = vec![
            "llama-00001-of-00002.gguf",
            "llama-00002-of-00002.gguf",
        ];
        let single_files = vec![
            "phi3.gguf",
            "mistral.safetensors",
        ];

        let mut model_files = Vec::new();

        // Create sharded files
        for filename in &shard_files {
            let file_path = models_dir.join(filename);
            fs::write(&file_path, "dummy content").unwrap();
            model_files.push(file_path);
        }

        // Create single files
        for filename in &single_files {
            let file_path = models_dir.join(filename);
            fs::write(&file_path, "dummy content").unwrap();
            model_files.push(file_path);
        }

        // Initialize auto-discovery
        let discovery = ModelAutoDiscovery::new();

        // Test the grouping logic
        let grouped_models = discovery.group_sharded_models(&models_dir, &model_files).unwrap();

        // Should have 3 entries: 1 grouped sharded + 2 single
        assert_eq!(grouped_models.len(), 3, "Should have 3 model entries: 1 grouped + 2 single");

        // Verify sharded model
        let sharded_model = grouped_models.iter().find(|m| m.name == "models").unwrap();
        assert!(sharded_model.path.to_string_lossy().contains("(+1 more files)"));

        // Verify single models
        let single_model_names: Vec<String> = grouped_models.iter()
            .filter(|m| m.name != "models")
            .map(|m| m.name.clone())
            .collect();
        assert!(single_model_names.contains(&"phi3".to_string()));
        assert!(single_model_names.contains(&"mistral".to_string()));
    }
}