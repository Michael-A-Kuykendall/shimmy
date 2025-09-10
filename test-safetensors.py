#!/usr/bin/env python3
"""
Create a minimal SafeTensors file for testing Shimmy's native SafeTensors support
"""

import json
import struct
from pathlib import Path

def create_minimal_safetensors_file(output_path: str):
    """Create a minimal SafeTensors file for testing"""
    
    # Define a simple tensor with metadata
    tensor_data = b'\x00\x00\x80\x3f' * 4  # Four float32 values (1.0)
    
    metadata = {
        "embed_tokens.weight": {
            "dtype": "F32",
            "shape": [2, 2],
            "data_offsets": [0, 16]
        }
    }
    
    # Serialize metadata as JSON
    metadata_json = json.dumps(metadata, separators=(',', ':'))
    metadata_bytes = metadata_json.encode('utf-8')
    
    # Create SafeTensors format:
    # 8-byte header (metadata length) + metadata + tensor data
    header = struct.pack('<Q', len(metadata_bytes))
    
    # Write the file
    with open(output_path, 'wb') as f:
        f.write(header)
        f.write(metadata_bytes)
        f.write(tensor_data)
    
    print(f"Created minimal SafeTensors file: {output_path}")
    print(f"Metadata: {metadata}")
    print(f"File size: {len(header) + len(metadata_bytes) + len(tensor_data)} bytes")

def create_config_json(output_dir: str):
    """Create a config.json file for the model"""
    config = {
        "model_type": "test_model",
        "vocab_size": 1000,
        "hidden_size": 64,
        "num_hidden_layers": 2,
        "max_position_embeddings": 128
    }
    
    config_path = Path(output_dir) / "config.json"
    with open(config_path, 'w') as f:
        json.dump(config, f, indent=2)
    
    print(f"Created config file: {config_path}")

def create_tokenizer_json(output_dir: str):
    """Create a simple tokenizer.json file"""
    tokenizer = {
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
    }
    
    tokenizer_path = Path(output_dir) / "tokenizer.json"  
    with open(tokenizer_path, 'w') as f:
        json.dump(tokenizer, f, indent=2)
    
    print(f"Created tokenizer file: {tokenizer_path}")

if __name__ == "__main__":
    # Create test directory
    test_dir = Path("test-safetensors-model")
    test_dir.mkdir(exist_ok=True)
    
    # Create files
    create_minimal_safetensors_file(test_dir / "model.safetensors")
    create_config_json(test_dir)
    create_tokenizer_json(test_dir)
    
    print(f"\nTest model created in: {test_dir.absolute()}")
    print("You can now test with:")
    print(f"  cargo run -- probe test-model")
    print(f"  where test-model points to {test_dir / 'model.safetensors'}")