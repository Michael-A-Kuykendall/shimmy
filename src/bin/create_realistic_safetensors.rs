// Create a realistic SafeTensors model that mimics real HuggingFace models

use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️  Creating realistic SafeTensors test model...");

    let test_dir = Path::new("test-huggingface-model");
    fs::create_dir_all(test_dir)?;

    // Create realistic config.json
    create_config_json(test_dir)?;

    // Create realistic tokenizer.json
    create_tokenizer_json(test_dir)?;

    // Create realistic SafeTensors model
    create_realistic_safetensors(test_dir.join("model.safetensors"))?;

    println!("✅ Realistic test model created in: {}", test_dir.display());
    println!("\nYou can now test with:");
    println!("  cargo run --bin shimmy -- discover");
    println!("  cargo run --bin shimmy -- probe <model-name>");

    Ok(())
}

fn create_config_json(test_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let config = r#"{
  "model_type": "distilbert",
  "vocab_size": 30522,
  "hidden_size": 768,
  "num_hidden_layers": 6,
  "num_attention_heads": 12,
  "intermediate_size": 3072,
  "hidden_act": "gelu",
  "hidden_dropout_prob": 0.1,
  "attention_probs_dropout_prob": 0.1,
  "max_position_embeddings": 512,
  "initializer_range": 0.02
}"#;

    let config_path = test_dir.join("config.json");
    fs::write(&config_path, config)?;
    println!("✅ Created: {}", config_path.display());

    Ok(())
}

fn create_tokenizer_json(test_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = r#"{
  "model": {
    "type": "WordPiece",
    "vocab": {
      "[PAD]": 0,
      "[UNK]": 100,
      "[CLS]": 101,
      "[SEP]": 102,
      "[MASK]": 103,
      "hello": 7592,
      "world": 2088,
      "the": 1996,
      "a": 1037,
      "an": 2019,
      "and": 1998,
      "or": 2030,
      "test": 3231,
      "model": 2944,
      "this": 2023,
      "is": 2003,
      "for": 2005,
      "testing": 7667
    }
  },
  "normalizer": {
    "type": "BertNormalizer"
  },
  "pre_tokenizer": {
    "type": "BertPreTokenizer"
  }
}"#;

    let tokenizer_path = test_dir.join("tokenizer.json");
    fs::write(&tokenizer_path, tokenizer)?;
    println!("✅ Created: {}", tokenizer_path.display());

    Ok(())
}

fn create_realistic_safetensors(
    filepath: std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create tensors that mimic a real transformer model structure
    // This is a simplified DistilBERT-like model

    let vocab_size = 30522;
    let hidden_size = 768;
    let max_position = 512;

    // Calculate tensor sizes
    let word_emb_size = vocab_size * hidden_size * 4; // F32 = 4 bytes
    let pos_emb_size = max_position * hidden_size * 4;
    let ln_weight_size = hidden_size * 4;
    let ln_bias_size = hidden_size * 4;

    // Calculate offsets
    let offset_1 = 0;
    let offset_2 = word_emb_size;
    let offset_3 = offset_2 + pos_emb_size;
    let offset_4 = offset_3 + ln_weight_size;
    let offset_5 = offset_4 + ln_bias_size;
    let offset_6 = offset_5 + hidden_size * hidden_size * 4;

    // Build metadata for realistic transformer tensors
    let metadata = format!(
        r#"{{
  "embeddings.word_embeddings.weight": {{
    "dtype": "F32",
    "shape": [{}, {}],
    "data_offsets": [{}, {}]
  }},
  "embeddings.position_embeddings.weight": {{
    "dtype": "F32",
    "shape": [{}, {}],
    "data_offsets": [{}, {}]
  }},
  "embeddings.LayerNorm.weight": {{
    "dtype": "F32",
    "shape": [{}],
    "data_offsets": [{}, {}]
  }},
  "embeddings.LayerNorm.bias": {{
    "dtype": "F32",
    "shape": [{}],
    "data_offsets": [{}, {}]
  }},
  "transformer.layer.0.attention.q_lin.weight": {{
    "dtype": "F32",
    "shape": [{}, {}],
    "data_offsets": [{}, {}]
  }}
}}"#,
        vocab_size,
        hidden_size,
        offset_1,
        offset_2,
        max_position,
        hidden_size,
        offset_2,
        offset_3,
        hidden_size,
        offset_3,
        offset_4,
        hidden_size,
        offset_4,
        offset_5,
        hidden_size,
        hidden_size,
        offset_5,
        offset_6
    );

    let metadata_bytes = metadata.as_bytes();
    let metadata_len = metadata_bytes.len() as u64;

    // Create the SafeTensors file
    let mut data = Vec::new();

    // Write metadata length (8 bytes, little endian)
    data.extend_from_slice(&metadata_len.to_le_bytes());

    // Write metadata
    data.extend_from_slice(metadata_bytes);

    // Write tensor data
    println!(
        "📊 Writing tensor data ({:.1} MB)...",
        (offset_6 as f64) / (1024.0 * 1024.0)
    );

    // Word embeddings (30522 x 768) - realistic small values
    for i in 0..(vocab_size * hidden_size) {
        let value = 0.02 * ((i % 200) as f32 - 100.0) / 100.0; // Small random-ish values
        data.extend_from_slice(&value.to_le_bytes());
    }

    // Position embeddings (512 x 768)
    for i in 0..(max_position * hidden_size) {
        let value = 0.01 * ((i % 100) as f32 - 50.0) / 50.0;
        data.extend_from_slice(&value.to_le_bytes());
    }

    // LayerNorm weight (768) - typically initialized to 1.0
    for _ in 0..hidden_size {
        data.extend_from_slice(&1.0f32.to_le_bytes());
    }

    // LayerNorm bias (768) - typically initialized to 0.0
    for _ in 0..hidden_size {
        data.extend_from_slice(&0.0f32.to_le_bytes());
    }

    // Attention weight (768 x 768)
    for i in 0..(hidden_size * hidden_size) {
        let value = 0.05 * ((i % 50) as f32 - 25.0) / 25.0;
        data.extend_from_slice(&value.to_le_bytes());
    }

    // Write the file
    fs::write(&filepath, data)?;

    let size_mb = fs::metadata(&filepath)?.len() as f64 / (1024.0 * 1024.0);
    println!(
        "✅ Created realistic SafeTensors: {} ({:.1} MB)",
        filepath.display(),
        size_mb
    );

    Ok(())
}
