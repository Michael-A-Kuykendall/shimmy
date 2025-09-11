// Create a test SafeTensors file for testing native loading

use std::fs;
use std::path::Path;

fn create_minimal_safetensors() -> Vec<u8> {
    // Create a minimal valid SafeTensors format
    // SafeTensors format: 8-byte header (length) + JSON metadata + tensor data
    let metadata = r#"{"embed_tokens.weight":{"dtype":"F32","shape":[2,2],"data_offsets":[0,16]}}"#;
    let metadata_bytes = metadata.as_bytes();
    let metadata_len = metadata_bytes.len() as u64;

    let mut data = Vec::new();
    data.extend_from_slice(&metadata_len.to_le_bytes());
    data.extend_from_slice(metadata_bytes);

    // Add tensor data (4 x 4 bytes for 2x2 F32 matrix)
    let tensor_data = [1.0f32, 0.5f32, 0.25f32, 0.125f32];
    for value in tensor_data {
        data.extend_from_slice(&value.to_le_bytes());
    }

    data
}

fn create_config_json() -> String {
    r#"{
  "model_type": "test_model",
  "vocab_size": 1000,
  "hidden_size": 64,
  "num_hidden_layers": 2,
  "max_position_embeddings": 128
}"#
    .to_string()
}

fn create_tokenizer_json() -> String {
    r#"{
  "model": {
    "type": "BPE",
    "vocab": {
      "<s>": 0,
      "</s>": 1,
      "<unk>": 2,
      "hello": 3,
      "world": 4,
      "test": 5,
      " ": 6
    }
  }
}"#
    .to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = Path::new("test-safetensors-model");
    fs::create_dir_all(test_dir)?;

    // Create SafeTensors file
    let safetensors_data = create_minimal_safetensors();
    fs::write(test_dir.join("model.safetensors"), safetensors_data)?;
    println!("Created: {}/model.safetensors", test_dir.display());

    // Create config.json
    fs::write(test_dir.join("config.json"), create_config_json())?;
    println!("Created: {}/config.json", test_dir.display());

    // Create tokenizer.json
    fs::write(test_dir.join("tokenizer.json"), create_tokenizer_json())?;
    println!("Created: {}/tokenizer.json", test_dir.display());

    println!("\nTest SafeTensors model created successfully!");
    println!("You can now test with:");
    println!("  cargo run -- probe test-safetensors-model");
    println!("  cargo run -- generate test-safetensors-model --prompt \"Hello world\"");

    Ok(())
}
