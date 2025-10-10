# 🛠️ Building with Shimmy: Developer Guide

*Everything you need to build reliable applications with Shimmy as your foundation*

## 🚀 What Are You Building with Shimmy?

Whether you're forking Shimmy for your application or integrating it as a service, this guide provides the tools and specifications you need to build systematically and reliably.

## 🎯 Quick Start: Two Powerful Ways to Build with Shimmy

### 1. 🔧 **Integrate Shimmy into Your Application**
Perfect for: Adding local AI capabilities to existing applications

```bash
# Start Shimmy server
shimmy serve --bind 127.0.0.1:11435

# Use OpenAI-compatible API
curl -X POST http://localhost:11435/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "your-model",
    "messages": [{"role": "user", "content": "Hello!"}],
    "max_tokens": 100
  }'
```

**Documentation**: See [`templates/integration_template.md`](templates/integration_template.md) for complete integration guide.

### 2. 🍴 **Fork Shimmy for Custom Solutions**
Perfect for: Building specialized AI inference tools tailored to your needs

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/shimmy.git
cd shimmy

# Review architectural principles
cat memory/constitution.md

# Plan your features with Spec-Kit methodology
# See "Feature Development Workflow" below
```

## 📋 Feature Development Workflow

Shimmy uses GitHub Spec-Kit methodology for systematic feature development. Here's how to plan and implement features:

### Step 1: Specify Your Feature (`/specify`)
Create a detailed specification that focuses on WHAT and WHY, not HOW.

**Template**: Use [`templates/spec-template.md`](templates/spec-template.md)

**Example**: Planning MLX support for Apple Silicon
```markdown
# Feature Specification: MLX GPU Acceleration

**Feature Branch**: `041-mlx-support`
**Created**: 2025-09-17
**Status**: Draft

## User Scenarios & Testing
- **Primary User**: Developer with Apple Silicon Mac running Shimmy locally
- **Scenario**: User runs `shimmy serve` and expects automatic GPU acceleration
- **Success Criteria**: Model inference uses Metal GPU instead of CPU

## Functional Requirements
- FR-001: Shimmy shall auto-detect Apple Silicon architecture
- FR-002: Shimmy shall prefer MLX backend when available
- FR-003: Shimmy shall fallback to CPU if MLX fails
```

### Step 2: Plan Implementation (`/plan`)
Generate technical implementation plan from your specification.

**Template**: Use [`templates/plan-template.md`](templates/plan-template.md)

**Constitutional Check**: Ensure your plan complies with Shimmy's principles:
- ✅ Maintains 5MB binary size limit
- ✅ Preserves sub-2-second startup
- ✅ No new Python dependencies
- ✅ Maintains OpenAI API compatibility

### Step 3: Break Into Tasks (`/tasks`)
Create actionable task list for implementation.

**Template**: Use [`templates/tasks-template.md`](templates/tasks-template.md)

**Example Task Breakdown**:
```markdown
## Tasks: MLX GPU Acceleration

- T001: Add MLX feature flag to Cargo.toml
- T002: Create MLX detection module in src/gpu/
- T003: [P] Write integration tests for MLX backend
- T004: [P] Write unit tests for GPU detection
- T005: Implement MLX model loading
- T006: Add MLX to engine adapter selection logic
- T007: Update documentation and examples
```

## 🛡️ Constitutional Compliance

Every feature must comply with Shimmy's architectural principles:

### **Immutable Constraints**
- **5MB Binary Limit**: Core binary cannot exceed 5MB
- **Sub-2-Second Startup**: Performance must be maintained
- **Zero Python Dependencies**: Pure Rust implementation only

### **Development Requirements**
- **Library-First**: Features start as standalone libraries
- **CLI Interface**: All functionality accessible via command line
- **Test-First**: Comprehensive tests before implementation
- **API Compatibility**: Maintain OpenAI API compatibility

### **Quality Gates**
Before any feature is merged:
- [ ] Constitutional compliance verified
- [ ] All tests pass: `cargo test --all-features`
- [ ] Integration tests pass
- [ ] Startup time < 2 seconds
- [ ] Binary size < 5MB

## 🔧 Integration Templates

### REST API Integration
```typescript
import OpenAI from "openai";

const shimmy = new OpenAI({
  baseURL: "http://localhost:11435/v1",
  apiKey: "sk-local", // placeholder
});

const response = await shimmy.chat.completions.create({
  model: "your-model",
  messages: [{ role: "user", content: "Hello!" }],
  max_tokens: 100,
});
```

### CLI Integration
```bash
# Programmatic model listing
MODELS=$(shimmy list --short)

# Health check
if curl -f http://localhost:11435/health; then
  echo "Shimmy is running"
fi

# Generation with error handling
shimmy generate --name "model" --prompt "test" --max-tokens 50 || {
  echo "Generation failed"
  exit 1
}
```

### Docker Integration
```dockerfile
FROM rust:1.89 as builder
COPY . /app
WORKDIR /app
RUN cargo build --release --features huggingface

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/shimmy /usr/local/bin/
EXPOSE 11435
CMD ["shimmy", "serve", "--bind", "0.0.0.0:11435"]
```

## 📊 Performance Monitoring

### Key Metrics to Track
- **Startup Time**: Should be < 2 seconds
- **Memory Usage**: Base 5MB + model size
- **Request Latency**: Time to first token
- **Error Rate**: Failed requests percentage

### Health Check Integration
```bash
# Basic health check
curl -f http://localhost:11435/health

# Detailed monitoring
curl http://localhost:11435/v1/models | jq '.data | length'
```

## 🚀 Deployment Patterns

### Single Instance (Development)
```bash
shimmy serve --bind 127.0.0.1:11435
```

### Load Balanced (Production)
```yaml
# docker-compose.yml
version: '3.8'
services:
  shimmy-1:
    image: shimmy:latest
    ports: ["11435:11435"]
  shimmy-2:
    image: shimmy:latest
    ports: ["11436:11435"]
  nginx:
    image: nginx
    # Load balance between instances
```

### Serverless (AWS Lambda)
```bash
# Package for Lambda deployment
cargo lambda build --release
```

## 💡 Best Practices

### For Application Developers
1. **Health Checks**: Always verify Shimmy is running before requests
2. **Error Handling**: Implement graceful degradation
3. **Resource Limits**: Monitor memory usage with large models
4. **Security**: Bind to localhost for local-only access

### For Fork Maintainers
1. **Read Constitution**: Understand architectural principles first
2. **Spec-First**: Use `/specify` → `/plan` → `/tasks` workflow
3. **Test Coverage**: Write tests before implementation
4. **Performance**: Validate startup time and binary size
5. **Stay Updated**: Regularly sync with upstream
6. **Document Changes**: Maintain clear changelog
7. **Constitutional Respect**: Preserve core architectural principles

## 🎯 Success Stories

### "I integrated Shimmy into my web app"
*"The OpenAI API compatibility meant zero code changes. Just pointed my existing client to localhost:11435 and it worked perfectly."*

### "I forked Shimmy for our enterprise needs"
*"The constitutional principles gave us confidence the architecture wouldn't drift. We added our custom auth layer while preserving the 5MB advantage."*

### "I contributed MLX support"
*"The Spec-Kit workflow made it easy to plan the feature systematically. The constitutional checks caught potential performance issues early."*

---

## 🔗 Resources

- **Integration Templates**: [`templates/integration_template.md`](templates/integration_template.md)
- **Constitutional Principles**: [`memory/constitution.md`](memory/constitution.md)
- **Spec-Kit Templates**: [`.internal/spec-template.md`](.internal/spec-template.md)
- **GitHub Issues**: [Report bugs or request features](https://github.com/Michael-A-Kuykendall/shimmy/issues)
- **Discussions**: [Community Q&A](https://github.com/Michael-A-Kuykendall/shimmy/discussions)

**Building something cool with Shimmy?** We'd love to hear about it! Share your project in [GitHub Discussions](https://github.com/Michael-A-Kuykendall/shimmy/discussions).

---

*Shimmy: Free forever, built to be your reliable foundation for local AI.*
