# Shimmy Cloud Deployment

One-click deployment configurations for popular cloud platforms.

## Quick Deploy Buttons

### Railway
[![Deploy on Railway](https://railway.app/button.svg)](https://railway.app/template/shimmy)

### Render
[![Deploy to Render](https://render.com/images/deploy-to-render-button.svg)](https://render.com/deploy)

### Fly.io
```bash
# Install flyctl and deploy
curl -L https://fly.io/install.sh | sh
fly deploy
```

### Docker (Any Platform)
```bash
# Local development
docker-compose up

# Production with Nginx
docker-compose --profile production up
```

## Platform-Specific Instructions

### Railway.app
1. Click the "Deploy on Railway" button above
2. Connect your GitHub account
3. Fork this repository
4. Railway will automatically build and deploy
5. Your Shimmy instance will be available at `https://your-app.railway.app`

### Render.com
1. Click the "Deploy to Render" button above
2. Connect your GitHub repository
3. Render will use the `render.yaml` configuration
4. Your service will be available with automatic HTTPS

### Fly.io
1. Install the Fly CLI: `curl -L https://fly.io/install.sh | sh`
2. Clone this repository: `git clone https://github.com/Michael-A-Kuykendall/shimmy.git`
3. Navigate to the project: `cd shimmy`
4. Create and deploy: `fly deploy`
5. Access your app: `fly open`

### Google Cloud Run
```bash
# Build and deploy to Cloud Run
gcloud builds submit --tag gcr.io/PROJECT-ID/shimmy
gcloud run deploy --image gcr.io/PROJECT-ID/shimmy --platform managed
```

### AWS App Runner
1. Create `apprunner.yaml` in your repository root:
```yaml
version: 1.0
runtime: docker
build:
  commands:
    build:
      - echo "Building Shimmy with Docker"
run:
  runtime-version: latest
  command: shimmy serve --bind 0.0.0.0:8080
  network:
    port: 8080
```

### DigitalOcean App Platform
1. Create app via DigitalOcean control panel
2. Connect your GitHub repository
3. DigitalOcean will detect the Dockerfile automatically
4. Set environment variables as needed

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `11434` | Port to bind the server |
| `RUST_LOG` | `info` | Log level (error, warn, info, debug, trace) |
| `SHIMMY_BIND` | `0.0.0.0:11434` | Full bind address |

## Resource Requirements

### Minimal
- **CPU**: 0.5 vCPU
- **Memory**: 512MB RAM
- **Storage**: 100MB (binary only)

### Recommended
- **CPU**: 1 vCPU
- **Memory**: 1GB RAM  
- **Storage**: 1GB+ (for model caching)

### High Performance
- **CPU**: 2+ vCPU
- **Memory**: 4GB+ RAM
- **Storage**: 10GB+ SSD

## Security Considerations

1. **Authentication**: Shimmy doesn't include built-in authentication. Use a reverse proxy (Nginx, Cloudflare) for auth.

2. **Rate Limiting**: The included Nginx configuration has basic rate limiting. Adjust as needed.

3. **HTTPS**: Most cloud platforms provide automatic HTTPS. For self-hosted deployments, configure SSL certificates.

4. **Firewall**: Only expose port 11434 (or your configured port) to the public internet.

## Monitoring

### Health Checks
All configurations include health checks at `/health` endpoint.

### Logs
Set `RUST_LOG=debug` for detailed logging. Most platforms provide log aggregation.

### Metrics
For production deployments, consider adding:
- Prometheus metrics
- Jaeger tracing
- Custom monitoring dashboards

## Scaling

### Horizontal Scaling
Shimmy is stateless and can be horizontally scaled. Use a load balancer to distribute requests.

### Vertical Scaling
For better performance with large models:
- Increase memory for model caching
- Add more CPU cores for parallel processing
- Use SSD storage for faster model loading

## Troubleshooting

### Common Issues

1. **Out of Memory**: Increase memory allocation or use memory-mapped loading
2. **Slow Startup**: Enable model caching and use persistent storage
3. **Connection Timeout**: Increase proxy timeout settings for large model inference

### Debug Mode
```bash
# Enable debug logging
RUST_LOG=debug shimmy serve
```

### Container Debugging
```bash
# Access running container
docker exec -it shimmy-container /bin/bash

# Check logs
docker logs shimmy-container
```