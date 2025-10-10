# Quick HuggingFace Upload Commands

## 1. Login to HuggingFace
```bash
hf auth login
# Enter your HuggingFace token when prompted
```

## 2. Create the repository and upload
```bash
# Create the repo and upload the model file
huggingface-cli upload Michael-A-Kuykendall/gpt-oss-20b-moe-cpu-offload-gguf /home/ubuntu/shimmy/models/gpt-oss-20b-f16.gguf --repo-type model

# Upload the README
huggingface-cli upload Michael-A-Kuykendall/gpt-oss-20b-moe-cpu-offload-gguf /home/ubuntu/shimmy/models/MOE-GGUF-README.md README.md --repo-type model
```

## Alternative: Create repo first
```bash
# Create empty repo
huggingface-cli create-repo Michael-A-Kuykendall/gpt-oss-20b-moe-cpu-offload-gguf --type model

# Then upload files
huggingface-cli upload Michael-A-Kuykendall/gpt-oss-20b-moe-cpu-offload-gguf /home/ubuntu/shimmy/models/gpt-oss-20b-f16.gguf
huggingface-cli upload Michael-A-Kuykendall/gpt-oss-20b-moe-cpu-offload-gguf /home/ubuntu/shimmy/models/MOE-GGUF-README.md README.md
```

## Model Details
- **File**: `/home/ubuntu/shimmy/models/gpt-oss-20b-f16.gguf` (13GB)
- **Type**: F16 GGUF with MoE CPU offloading support
- **Special Feature**: Works with shimmy feat/moe-cpu-offload branch
<<<<<<< HEAD
<<<<<<< HEAD
- **Memory Savings**: 99.9% VRAM reduction (2MB vs 15GB)
=======
- **Memory Savings**: 99.9% VRAM reduction (2MB vs 15GB)
>>>>>>> main
=======
- **Memory Savings**: 99.9% VRAM reduction (2MB vs 15GB)
>>>>>>> main
