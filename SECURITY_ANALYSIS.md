# Shimmy Security Analysis & Posture

## Executive Summary

Shimmy is designed as a **local-first inference shim** with minimal attack surface. This analysis covers security implications, compliance considerations, and recommendations for safe deployment.

## Threat Model

### Primary Use Case
- **Local development environment** serving LLM inference
- **Single-user or trusted team** access patterns
- **Development/prototyping** workloads, not production data

### Attack Vectors
1. **Network-based attacks** via HTTP/WebSocket endpoints
2. **Model file tampering** via malicious GGUF files
3. **Resource exhaustion** through excessive requests
4. **Dependency vulnerabilities** in Rust crates

## Current Security Posture

### ✅ Security Strengths

**1. Minimal Attack Surface**
- Single binary (5.1MB) with minimal dependencies
- No persistent storage or user accounts
- Stateless request processing
- CPU-only inference (no GPU drivers)

**2. Network Security**
- Defaults to localhost binding (`127.0.0.1:11435`)
- No authentication required for local access
- Standard HTTP/WebSocket protocols (no custom protocols)

**3. Code Safety**
- Rust memory safety by default
- Limited unsafe code (only in llama.cpp FFI boundary)
- Structured error handling with anyhow

**4. Input Validation**
- JSON request deserialization with serde
- Model path validation in discovery
- File extension validation for model loading

### ⚠️ Security Considerations

**1. No Authentication/Authorization**
- **By Design**: Local development tool, not production service
- **Risk**: If bound to 0.0.0.0, accessible to network
- **Mitigation**: Documentation emphasizes localhost binding

**2. No Rate Limiting**
- **By Design**: Shim should be "invisible infrastructure"
- **Risk**: Resource exhaustion via request spam
- **Mitigation**: OS-level process limits, monitoring

**3. Model File Trust**
- **Assumption**: Users control their model files
- **Risk**: Malicious GGUF files could exploit llama.cpp
- **Mitigation**: Use trusted model sources, verify checksums

**4. FFI Safety Boundary**
- **Limited Scope**: Only in `engine/llama.rs` transmute
- **Risk**: Memory safety violations in llama.cpp
- **Mitigation**: Minimal FFI surface, proper lifetime management

## Compliance Implications

### GDPR/Privacy
- **✅ Local Processing**: No data sent to external services
- **✅ No Data Collection**: No telemetry or user tracking
- **✅ User Control**: Complete data sovereignty

### Enterprise Use
- **✅ Offline Operation**: No internet required after model download
- **✅ Audit Trail**: Request/response logging available
- **⚠️ Access Control**: No built-in user authentication

### Industry Standards
- **ISO 27001**: Supports "data minimization" principle
- **SOX Compliance**: Audit logs for request tracing
- **HIPAA**: Local processing reduces data exposure risk

## Recommended Security Measures

### For Development Use (Current)
```bash
# Bind to localhost only (default)
shimmy serve --bind 127.0.0.1:11435

# Enable logging for audit trail
SHIMMY_LOG_LEVEL=info shimmy serve 2>&1 | tee shimmy.log
```

### For Team/Network Use
```bash
# Use reverse proxy with authentication
nginx → shimmy (localhost)
# Configure nginx with:
# - SSL/TLS termination
# - Basic auth or OAuth
# - Rate limiting
# - Request logging
```

### For Production-like Use
```bash
# Container isolation
docker run -p 127.0.0.1:11435:11435 shimmy

# Process monitoring
systemd service with:
# - Resource limits (memory, CPU)
# - Restart policies
# - Log aggregation
```

## Security Roadmap (If Needed)

### Phase 1: Optional Hardening
- [ ] Add `--auth-token` flag for simple bearer token auth
- [ ] Add `--max-requests-per-minute` rate limiting
- [ ] Add `--max-request-size` input validation
- [ ] Add model file checksum verification

### Phase 2: Enterprise Features (Optional)
- [ ] JWT token validation
- [ ] Request audit logging (structured JSON)
- [ ] Metrics endpoint with Prometheus format
- [ ] Health check with dependency status

**Note**: These would be **optional flags** to maintain the "zero-config shim" philosophy.

## Risk Assessment

| Risk Category | Likelihood | Impact | Mitigation |
|---------------|------------|--------|------------|
| Network attack via localhost | LOW | LOW | Default localhost binding |
| Model file tampering | MEDIUM | MEDIUM | Use trusted sources |
| Resource exhaustion | MEDIUM | LOW | OS limits, monitoring |
| Dependency vulnerability | LOW | MEDIUM | Regular cargo audit |
| FFI memory safety | LOW | HIGH | Minimal unsafe code |

## Recommendations

### For Users
1. **Keep default localhost binding** unless network access needed
2. **Use trusted model sources** (HuggingFace, official repos)
3. **Monitor resource usage** in production environments
4. **Update regularly** for dependency security patches

### For Contributors
1. **Minimize unsafe code** - justify any new unsafe blocks
2. **Validate all inputs** from HTTP requests
3. **Add security tests** for edge cases
4. **Document security assumptions** in code comments

### For Deployment
1. **Use reverse proxy** for network access (nginx, caddy)
2. **Enable logging** for audit trails
3. **Set resource limits** via systemd/docker
4. **Regular security updates** via package managers

## Conclusion

Shimmy's security posture aligns with its mission as a **local development shim**:

- **Minimal attack surface** by design
- **Appropriate for development use** with current security measures
- **Clear upgrade path** for production-like deployments
- **Compliance-friendly** for privacy-sensitive environments

The "shim" philosophy means security features should be **optional and additive**, not mandatory. Users requiring enterprise security can layer it via infrastructure (reverse proxy, containers, monitoring) rather than building it into the core binary.
