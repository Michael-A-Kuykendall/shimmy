# Configuration Guide

This guide covers all configuration options for Shimmy.

## Environment Variables

### Required

- **`SHIMMY_BASE_GGUF`**: Path to the base GGUF model file
  ```bash
  export SHIMMY_BASE_GGUF=/path/to/your/model.gguf
  ```

### Optional

- **`SHIMMY_LORA_GGUF`**: Path to LoRA adapter file
  ```bash
  export SHIMMY_LORA_GGUF=/path/to/your/lora.gguf
  ```

- **`SHIMMY_LOG_LEVEL`**: Logging level (error, warn, info, debug, trace)
  ```bash
  export SHIMMY_LOG_LEVEL=info
  ```

- **`SHIMMY_BIND_ADDRESS`**: Default bind address for server
  ```bash
  export SHIMMY_BIND_ADDRESS=127.0.0.1:11435
  ```

## Command Line Options

### Server Configuration

```bash
shimmy serve [OPTIONS]
```

**Options:**
- `--bind <ADDRESS>`: Bind address (default: 127.0.0.1:11435)
- `--port <PORT>`: Port number (overrides port in bind address)
- `--workers <N>`: Number of worker threads (default: auto-detected)
- `--max-connections <N>`: Maximum concurrent connections (default: 100)

### Model Configuration

```bash
shimmy generate [OPTIONS]
```

**Options:**
- `--model <NAME>`: Model name to use (default: "default")
- `--prompt <TEXT>`: Input prompt
- `--max-tokens <N>`: Maximum tokens to generate (default: 100)
- `--temperature <F>`: Sampling temperature (default: 0.7)
- `--top-p <F>`: Top-p sampling (default: 0.9)
- `--top-k <N>`: Top-k sampling (default: 40)

## Model Setup

### GGUF Models

Place your GGUF model files in a accessible location and set the environment variable:

```bash
# Example model locations
export SHIMMY_BASE_GGUF=~/.cache/models/phi3-mini.gguf
export SHIMMY_BASE_GGUF=/models/llama2-7b.gguf
export SHIMMY_BASE_GGUF=./models/mistral-7b.gguf
```

### LoRA Adapters

If using LoRA adapters, ensure they are compatible with your base model:

```bash
export SHIMMY_LORA_GGUF=~/.cache/adapters/coding-adapter.gguf
```

## Templates

Shimmy supports multiple prompt templates:

### Available Templates

- **`chatml`**: ChatML format for chat-based models
- **`llama3`**: Llama 3 instruction format
- **`openchat`**: OpenChat conversation format

### Template Selection

Templates are automatically selected based on model detection, but can be overridden:

```bash
shimmy generate --template chatml --prompt "Hello"
```

## Performance Tuning

### CPU Optimization

```bash
# Set number of threads for inference
export OMP_NUM_THREADS=8

# Enable CPU optimizations
export SHIMMY_CPU_THREADS=8
```

### Memory Management

```bash
# Limit memory usage (in MB)
export SHIMMY_MAX_MEMORY=4096

# Enable memory mapping for large models
export SHIMMY_MMAP=true
```

### GPU Support

Currently, shimmy uses CPU-only inference. GPU support is planned for future releases.

## Security Considerations

### Network Security

- Bind to localhost (`127.0.0.1`) for local-only access
- Use a reverse proxy (nginx, caddy) for external access
- Consider authentication middleware for production use

### Model Security

- Verify model file integrity before loading
- Use trusted model sources
- Monitor resource usage for potential abuse

## Logging Configuration

### Log Levels

```bash
# Minimal logging (errors only)
export SHIMMY_LOG_LEVEL=error

# Standard logging (info and above)
export SHIMMY_LOG_LEVEL=info

# Debug logging (all messages)
export SHIMMY_LOG_LEVEL=debug
```

### Log Output

```bash
# Log to file
shimmy serve 2>&1 | tee shimmy.log

# Structured JSON logging
export SHIMMY_LOG_FORMAT=json
```

## Troubleshooting

### Common Issues

1. **Model not loading**
   - Check file path and permissions
   - Verify GGUF format compatibility
   - Check available memory

2. **Server not starting**
   - Verify port is not in use
   - Check bind address format
   - Review log output for errors

3. **Slow inference**
   - Increase CPU thread count
   - Verify model size vs available memory
   - Consider model quantization

### Debug Mode

Enable verbose logging for troubleshooting:

```bash
SHIMMY_LOG_LEVEL=debug shimmy serve --verbose
```
