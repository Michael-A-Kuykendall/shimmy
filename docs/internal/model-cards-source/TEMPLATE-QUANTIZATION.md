---
quantized_by: MikeKuykendall
pipeline_tag: text-generation
license: {LICENSE}
license_link: {LICENSE_LINK}
base_model: {BASE_MODEL}
tags:
- {TAG1}
- {TAG2}
- moe
- mixture-of-experts
- gguf
- quantized
language:
- {LANGUAGE}
---

# {MODEL_NAME} - GGUF Quantization

Quantized GGUF version of [{BASE_MODEL}](https://huggingface.co/{BASE_MODEL})

Using <a href="https://github.com/ggerganov/llama.cpp/">llama.cpp</a> release <a href="{LLAMACPP_RELEASE_URL}">{LLAMACPP_VERSION}</a> for quantization.

## Model Details

- **Base Model**: [{BASE_MODEL_SHORT}](https://huggingface.co/{BASE_MODEL})
- **Quantization**: {QUANT_METHOD}
- **File Size**: {FILE_SIZE}
- **Original Size**: {ORIGINAL_SIZE} (F16)
- **Compression Ratio**: {COMPRESSION_PCT}%
- **Quantized by**: [MikeKuykendall](https://huggingface.co/MikeKuykendall)

## Quantization Details

This model has been quantized using llama.cpp's `{QUANT_METHOD}` quantization method:

{QUANT_DESCRIPTION}

### Why this quantization?

{QUANT_RATIONALE}

## Download

**Single file download**:
```bash
huggingface-cli download MikeKuykendall/{REPO_NAME} --include "{FILENAME}" --local-dir ./
```

**Using with llama.cpp**:
```bash
# Clone llama.cpp
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp && make

# Download model
huggingface-cli download MikeKuykendall/{REPO_NAME} --include "{FILENAME}" --local-dir ./models

# Run inference
./llama-cli -m ./models/{FILENAME} -p "Your prompt here" -n 128
```

**Using with Shimmy** (MoE CPU Offloading Support):
```bash
# Install Shimmy
cargo install --git https://github.com/Michael-A-Kuykendall/shimmy --features llama

# Run with MoE CPU offloading (saves VRAM)
shimmy serve --model ./models/{FILENAME} --cpu-moe

# Query the API
curl http://localhost:11435/api/generate \
  -d '{"model":"{MODEL_NAME}","prompt":"Your prompt","stream":false}'
```

## Prompt Format

```
{PROMPT_FORMAT}
```

## Model Architecture

{MODEL_ARCHITECTURE_DESCRIPTION}

## Usage Examples

<details>
  <summary>llama.cpp CLI</summary>

```bash
./llama-cli \
  -m {FILENAME} \
  -p "{EXAMPLE_PROMPT}" \
  -n 256 \
  -c 4096
```

</details>

<details>
  <summary>llama.cpp Server</summary>

```bash
# Start server
./llama-server -m {FILENAME} -c 4096 --port 8080

# Query server
curl http://localhost:8080/v1/completions \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "{EXAMPLE_PROMPT}",
    "n_predict": 256
  }'
```

</details>

<details>
  <summary>Shimmy with MoE CPU Offloading</summary>

```bash
# Start Shimmy server with CPU offloading
shimmy serve --model {FILENAME} --cpu-moe --bind 0.0.0.0:11435

# Generate text
curl http://localhost:11435/api/generate \
  -d '{
    "model": "{MODEL_NAME}",
    "prompt": "{EXAMPLE_PROMPT}",
    "max_tokens": 256,
    "stream": false
  }'
```

**MoE CPU Offloading**: Shimmy supports offloading MoE expert tensors to CPU RAM, reducing VRAM usage by 80-95% at the cost of 3-7x slower generation. Perfect for VRAM-constrained scenarios.

</details>

## Performance Characteristics

{PERFORMANCE_NOTES}

## Original Model Info

{ORIGINAL_MODEL_SUMMARY}

**Links**:
- Original Model: [{BASE_MODEL}](https://huggingface.co/{BASE_MODEL})
- {ADDITIONAL_LINKS}

## License

This model inherits the license from the original model: [{LICENSE}]({LICENSE_LINK})

## Citation

```bibtex
{CITATION}
```

---

*Quantized by [MikeKuykendall](https://huggingface.co/MikeKuykendall) using llama.cpp*
