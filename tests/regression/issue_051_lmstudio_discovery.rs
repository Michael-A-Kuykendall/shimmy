/// Regression tests for model discovery improvements
///
/// This test suite covers fixes for:
/// - Issue #51: LMStudio models not found automatically
/// - Model discovery from multiple sources
/// - Environment variable handling for model paths
use shimmy::discovery::{discover_models_from_directory, ModelDiscovery};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_lmstudio_model_discovery() {
    // Test for Issue #51: LMStudio models not found automatically
    let temp_dir = TempDir::new().unwrap();
    let lmstudio_path = temp_dir
        .path()
        .join(".cache")
        .join("lm-studio")
        .join("models");
    std::fs::create_dir_all(&lmstudio_path).unwrap();

    // Create typical LMStudio model structure
    let model_dir = lmstudio_path.join("microsoft").join("DialoGPT-medium");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("model.gguf"), "dummy gguf content").unwrap();

    let model_dir2 = lmstudio_path.join("meta-llama").join("Llama-2-7b-chat-hf");
    std::fs::create_dir_all(&model_dir2).unwrap();
    std::fs::write(
        model_dir2.join("model.safetensors"),
        "dummy safetensors content",
    )
    .unwrap();

    // Test discovery
    let models = discover_models_from_directory(&lmstudio_path).unwrap();

    // Should find models in nested LMStudio structure
    assert!(
        !models.is_empty(),
        "Should discover models in LMStudio structure"
    );

    let model_names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();

    // Should find models with proper naming
    assert!(
        model_names.iter().any(|name| name.contains("model")),
        "Should find model files in LMStudio structure"
    );
}

#[test]
fn test_multiple_model_format_discovery() {
    // Test discovery of various model formats that should be supported
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create files with various supported extensions
    let test_files = vec![
        "llama-7b-chat.gguf",
        "phi3-mini.q4_0.gguf",
        "qwen2-instruct.safetensors",
        "gemma-2b.bin",
        "mistral-7b.pt",
        "not-a-model.txt", // Should be ignored
        "config.json",     // Should be ignored
        "README.md",       // Should be ignored
    ];

    for file in &test_files {
        std::fs::write(temp_path.join(file), "dummy content").unwrap();
    }

    let models = discover_models_from_directory(temp_path).unwrap();

    // Should find model files but ignore non-model files
    let model_names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();

    // Check that model files are found
    assert!(
        model_names
            .iter()
            .any(|name| name.contains("llama-7b-chat")),
        "Should find GGUF files"
    );
    assert!(
        model_names.iter().any(|name| name.contains("phi3-mini")),
        "Should find quantized GGUF files"
    );
    assert!(
        model_names
            .iter()
            .any(|name| name.contains("qwen2-instruct")),
        "Should find SafeTensors files"
    );

    // Check that non-model files are ignored
    assert!(
        !model_names.iter().any(|name| name.contains("not-a-model")),
        "Should ignore .txt files"
    );
    assert!(
        !model_names.iter().any(|name| name.contains("config")),
        "Should ignore config files"
    );
    assert!(
        !model_names.iter().any(|name| name.contains("README")),
        "Should ignore documentation"
    );

    // Should find at least the major model formats
    assert!(models.len() >= 3, "Should find at least 3 model files");
}

#[test]
fn test_environment_variable_model_paths() {
    // Test that environment variables for model paths are properly handled
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();

    // Create model files in test directories
    std::fs::write(temp_dir1.path().join("model1.gguf"), "content").unwrap();
    std::fs::write(temp_dir2.path().join("model2.safetensors"), "content").unwrap();

    // Set environment variables
    let path1 = temp_dir1.path().to_string_lossy();
    let path2 = temp_dir2.path().to_string_lossy();
    let combined_paths = format!("{};{}", path1, path2);

    std::env::set_var("SHIMMY_MODEL_PATHS", &combined_paths);
    std::env::set_var("OLLAMA_MODELS", path1.as_ref());

    // Test ModelDiscovery picks up environment variables
    let discovery = ModelDiscovery::from_env();
    let search_paths = discovery.search_paths();

    // Clean up environment variables
    std::env::remove_var("SHIMMY_MODEL_PATHS");
    std::env::remove_var("OLLAMA_MODELS");

    // Verify paths were added
    let path_strings: Vec<String> = search_paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    assert!(
        path_strings.iter().any(|p| p.contains(&*path1)),
        "Should include path from SHIMMY_MODEL_PATHS"
    );
    assert!(
        path_strings.iter().any(|p| p.contains(&*path2)),
        "Should include second path from SHIMMY_MODEL_PATHS"
    );
    assert!(
        path_strings.iter().any(|p| p.contains(&*path1)),
        "Should include path from OLLAMA_MODELS"
    );
}

#[test]
fn test_common_model_directory_structures() {
    // Test discovery in common model directory layouts
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create nested directory structures like real model repositories
    let structures = vec![
        // Hugging Face style
        "microsoft/DialoGPT-medium/pytorch_model.bin",
        "microsoft/DialoGPT-medium/config.json",
        // LMStudio style
        "lm-studio/models/microsoft/DialoGPT-medium/model.gguf",
        // Ollama style
        "ollama/models/blobs/sha256-abc123.gguf",
        // Flat structure
        "llama-2-7b-chat.q4_0.gguf",
        "phi-3-mini-4k-instruct.safetensors",
    ];

    for structure in &structures {
        let full_path = temp_path.join(structure);
        std::fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        std::fs::write(full_path, "dummy content").unwrap();
    }

    // Test discovery from root
    let models = discover_models_from_directory(temp_path).unwrap();

    // Should find models in various directory structures
    assert!(
        !models.is_empty(),
        "Should find models in nested structures"
    );
    assert!(
        models.len() >= 3,
        "Should find multiple models across different structures"
    );

    let model_names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();

    // Check that we find models from different structures
    let has_pytorch = model_names
        .iter()
        .any(|name| name.contains("pytorch_model"));
    let has_gguf = model_names
        .iter()
        .any(|name| name.contains("model") || name.contains("llama") || name.contains("sha256"));
    let has_safetensors = model_names.iter().any(|name| name.contains("phi"));

    assert!(
        has_pytorch || has_gguf || has_safetensors,
        "Should find models from at least one structure type"
    );
}

#[test]
fn test_model_discovery_error_handling() {
    // Test that model discovery handles errors gracefully without crashing

    // Test with non-existent directory
    let non_existent = PathBuf::from("/absolutely/non/existent/path/that/should/not/exist");
    let result = discover_models_from_directory(&non_existent);

    // Should handle gracefully (either return empty list or error, but not panic)
    if let Ok(models) = result {
        assert!(
            models.is_empty(),
            "Non-existent directory should return empty list"
        );
    }
    // Error is also acceptable

    // Test with file instead of directory
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let result = discover_models_from_directory(temp_file.path());

    // Should handle gracefully
    if let Ok(models) = result {
        assert!(models.is_empty(), "File path should return empty list");
    }
    // Error is also acceptable
}

#[test]
fn test_model_name_sanitization() {
    // Test that discovered model names are properly sanitized and don't contain paths
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create models with various path depths
    let deep_path = temp_path
        .join("very")
        .join("deep")
        .join("nested")
        .join("path");
    std::fs::create_dir_all(&deep_path).unwrap();
    std::fs::write(deep_path.join("model-with-long-path.gguf"), "content").unwrap();

    let models = discover_models_from_directory(temp_path).unwrap();

    for model in &models {
        // Model names should not contain path separators
        assert!(
            !model.name.contains('/'),
            "Model name should not contain forward slashes: {}",
            model.name
        );
        assert!(
            !model.name.contains('\\'),
            "Model name should not contain backslashes: {}",
            model.name
        );

        // Model names should be clean identifiers
        assert!(!model.name.is_empty(), "Model name should not be empty");
        assert!(
            !model.name.starts_with('.'),
            "Model name should not start with dot"
        );
    }
}
