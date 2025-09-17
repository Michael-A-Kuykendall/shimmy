# Shimmy Integration Template

*Use this template when creating specifications for integrating Shimmy into applications*

## Integration Specification Template

### Project Information
- **Application Name**: [Your application name]
- **Integration Type**: [REST API, Library, CLI, WebSocket, etc.]
- **Expected Traffic**: [Requests per second/minute/hour]
- **Model Requirements**: [Specific models or model types needed]

### Constitutional Compliance Check
- [ ] **5MB Constraint**: Integration preserves Shimmy's lightweight nature
- [ ] **Startup Speed**: Integration doesn't impact sub-2-second startup
- [ ] **Zero Python Dependencies**: No Python runtime requirements
- [ ] **API Compatibility**: Uses standard OpenAI API endpoints
- [ ] **CLI Access**: Programmatic access via command-line interface

### Integration Architecture

#### Connection Pattern
```
[Your Application] -> [Integration Layer] -> [Shimmy Instance] -> [Model Backend]
```

#### Configuration Template
```yaml
shimmy_config:
  bind: "127.0.0.1:11435"  # Or auto-allocated port
  model_dirs: "path/to/your/models"
  features: ["huggingface", "llama"]  # Optional: specify required features
```

#### Environment Variables
```bash
SHIMMY_PORT=11435                    # Optional: override default port
SHIMMY_MODEL_DIRS="/path/to/models"  # Optional: additional model directories
SHIMMY_LOG_LEVEL=info               # Optional: logging level
```

### API Integration Patterns

#### Standard OpenAI API Usage
```bash
# List available models
curl http://localhost:11435/v1/models

# Generate completion
curl -X POST http://localhost:11435/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "your-model-name",
    "messages": [{"role": "user", "content": "Hello!"}],
    "max_tokens": 100
  }'
```

#### CLI Integration
```bash
# List models programmatically
shimmy list --short | grep "model-pattern"

# Health check
shimmy serve --bind 127.0.0.1:0 &  # Auto-allocate port
SHIMMY_PID=$!

# Cleanup
kill $SHIMMY_PID
```

### Performance Considerations

#### Resource Planning
- **Memory**: Base 5MB + model size + context buffer
- **CPU**: [Your CPU requirements]
- **GPU**: [Optional GPU requirements for acceleration]
- **Network**: [Bandwidth requirements for your use case]

#### Scaling Patterns
- **Single Instance**: Direct API calls for low-traffic applications
- **Load Balanced**: Multiple Shimmy instances behind load balancer
- **Containerized**: Docker deployment with resource constraints
- **Serverless**: Lambda/Function-as-a-Service integration

### Testing Strategy

#### Integration Tests
```bash
# Basic connectivity test
curl -f http://localhost:11435/health || exit 1

# Model availability test
MODEL_COUNT=$(curl -s http://localhost:11435/v1/models | jq '.data | length')
[ "$MODEL_COUNT" -gt 0 ] || exit 1

# Generation test
RESPONSE=$(curl -s -X POST http://localhost:11435/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model": "test-model", "messages": [{"role": "user", "content": "test"}], "max_tokens": 10}')
echo "$RESPONSE" | jq -e '.choices[0].message.content' || exit 1
```

#### Performance Validation
```bash
# Startup time test
time shimmy serve --bind 127.0.0.1:0 &
# Should complete in <2 seconds

# Memory usage test
ps -o pid,vsz,rss -p $SHIMMY_PID
# VSZ should be reasonable for your constraints
```

### Error Handling

#### Common Error Scenarios
- **Model Not Found**: Verify model is in discovery path
- **Port Conflicts**: Use auto-allocation or check port availability
- **Memory Limits**: Monitor resource usage, especially with large models
- **GPU Issues**: Check GPU detection and driver compatibility

#### Graceful Degradation
```bash
# Fallback to CPU if GPU fails
shimmy serve --bind 127.0.0.1:11435
# Shimmy automatically handles GPU fallback

# Model fallback hierarchy
# 1. Requested specific model
# 2. Similar model in same family
# 3. Default available model
# 4. Error with available alternatives
```

### Deployment Checklist

#### Pre-Deployment
- [ ] **Models Available**: Required models discoverable in configured paths
- [ ] **Ports Configured**: Network ports available and firewall configured
- [ ] **Resource Limits**: Memory and CPU limits appropriate for model size
- [ ] **Dependencies Met**: Rust runtime and required libraries available

#### Post-Deployment Validation
- [ ] **Health Check**: `/health` endpoint responding correctly
- [ ] **Model Discovery**: Models appearing in `/v1/models` endpoint
- [ ] **Generation Test**: Successful completion generation
- [ ] **Performance Metrics**: Startup time and memory usage within limits

### Monitoring Integration

#### Key Metrics
- **Startup Time**: Should be <2 seconds
- **Memory Usage**: Base 5MB + model overhead
- **Request Latency**: Time to first token and total generation time
- **Error Rate**: Failed requests as percentage of total
- **Model Load Time**: Time to load models on demand

#### Alerting Thresholds
```yaml
alerts:
  startup_time: >2s
  memory_usage: >expected_model_size + 100MB
  error_rate: >5%
  request_latency_p95: >5s  # Adjust based on your SLA
```

### Security Considerations

#### Network Security
- **Bind Address**: Use 127.0.0.1 for local-only access
- **Firewall Rules**: Restrict access to required ports only
- **TLS/HTTPS**: Consider reverse proxy for HTTPS termination
- **Authentication**: Implement application-level auth as needed

#### Model Security
- **Model Validation**: Verify model integrity before loading
- **Access Controls**: Limit model directory access permissions
- **Input Sanitization**: Validate prompts in your application layer
- **Output Filtering**: Review generated content as appropriate

---

*This template ensures constitutional compliance while providing practical integration guidance.*

*Customize sections based on your specific integration requirements.*

*Last Updated: September 17, 2025*
*Version: 1.0*