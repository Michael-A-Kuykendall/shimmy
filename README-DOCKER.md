# 🐳 Docker Deployment

Easy deployment with Docker Compose - just mount your models directory and go!

## Quick Start

1. **Create your models directory:**
```bash
mkdir models
```

2. **Download some models:**
```bash
# Example: Download a small model
curl -L "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf" -o models/phi-3-mini.gguf
```

3. **Start Shimmy:**
```bash
docker-compose up -d
```

4. **Test the API:**
```bash
curl http://localhost:11434/v1/models
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SHIMMY_PORT` | `11434` | Server port |
| `SHIMMY_HOST` | `0.0.0.0` | Listen address |
| `SHIMMY_BASE_GGUF` | `/app/models` | Models directory |

### Volumes

- `./models:/app/models` - Mount your local models directory
- `shimmy-cache:/root/.cache` - Persistent cache for downloads

### GPU Support

For NVIDIA GPU support, ensure you have:
- [NVIDIA Container Toolkit](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/install-guide.html) installed
- Docker Compose v2.3+ with GPU support

GPU access is automatically configured in the provided `docker-compose.yml`.

## Usage Examples

### Basic Usage
```bash
# Start server
docker-compose up -d

# Check logs
docker-compose logs -f shimmy

# Stop server
docker-compose down
```

### Custom Configuration
```yaml
# docker-compose.override.yml
services:
  shimmy:
    ports:
      - "8080:11434"  # Use port 8080 instead
    environment:
      - SHIMMY_PORT=11434
      - SHIMMY_LOG_LEVEL=debug
```

### Multiple Models
```bash
# Your models directory structure
models/
├── phi-3-mini.gguf
├── llama-2-7b.gguf
└── mistral-7b.gguf
```

Shimmy will automatically discover and serve all `.gguf` models in the mounted directory.

## API Endpoints

Once running, Shimmy provides OpenAI-compatible endpoints:

- `GET /v1/models` - List available models
- `POST /v1/chat/completions` - Chat completions
- `POST /v1/completions` - Text completions
- `GET /health` - Health check

## Troubleshooting

### Container won't start
```bash
# Check logs
docker-compose logs shimmy

# Check if port is available
netstat -tulpn | grep 11434
```

### Models not loading
```bash
# Verify models directory is mounted
docker-compose exec shimmy ls -la /app/models

# Check file permissions
ls -la models/
```

### GPU not detected
```bash
# Check NVIDIA runtime
docker run --rm --gpus all nvidia/cuda:11.0-base nvidia-smi

# Verify Docker Compose GPU config
docker-compose config
```

## Building from Source

To build your own image:

```bash
# Build the image
docker build -t shimmy:local .

# Use local image in docker-compose.yml
# Replace: image: ghcr.io/michael-a-kuykendall/shimmy:latest
# With:    image: shimmy:local
```