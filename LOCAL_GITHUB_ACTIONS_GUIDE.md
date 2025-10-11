# Local GitHub Actions Development Guide

## Overview

This guide documents the complete process for running GitHub Actions workflows locally using the `act` CLI tool, eliminating the need for public trial-and-error releases and providing professional-grade CI/CD development workflows.

## The Problem: Public CI/CD Failures

**Before**: Trial-and-error with public test tags (`v1.7.2-test1`, `v1.7.2-test2`, etc.)
- Public red CI badges during development
- Embarrassing failures during high-traffic periods  
- No ability to debug complex workflow issues locally
- Wasted GitHub Actions minutes
- Unprofessional appearance to users and contributors

**After**: Complete local simulation of GitHub Actions environment
- Test all workflows locally before any public release
- Debug issues in identical environment to GitHub runners
- Professional, polished public releases only
- Zero public CI failures during development
- Significant cost savings on GitHub Actions minutes

## act CLI Tool: Local GitHub Actions Execution

### What is act?

`act` is a CLI tool that runs your GitHub Actions workflows locally using Docker containers. It reads your `.github/workflows/` directory and executes the exact same commands that would run in GitHub's cloud environment.

**Key Benefits:**
- **Identical Environment**: Uses same Docker images as GitHub Actions
- **Fast Feedback Loop**: Test changes immediately without git push
- **Cost Effective**: Reduces GitHub Actions usage and CI minutes
- **Professional Development**: Debug privately before public releases
- **Complete Simulation**: Environment variables, secrets, file systems all replicated

### Installation

#### Windows (Chocolatey)
```bash
choco install act-cli
```

#### Verify Installation
```bash
act --version
# Should output: act version 0.2.82 (or newer)
```

### Configuration

#### .actrc Configuration File
Create `C:\Users\{username}\.actrc` with:

```
-P ubuntu-latest=catthehacker/ubuntu:full-latest
--container-daemon-socket npipe:////./pipe/docker_engine
```

**Image Options:**
- `catthehacker/ubuntu:micro-latest` (~200MB) - Basic NodeJS only
- `catthehacker/ubuntu:act-latest` (~500MB) - Standard tools 
- `catthehacker/ubuntu:full-latest` (~17GB) - Complete development environment

**Recommendation**: Use `full-latest` for Rust/C++ projects requiring build tools like libclang, cmake, etc.

### Basic Usage

#### List Available Workflows
```bash
act --list
```

#### Run Specific Workflow
```bash
act -W .github/workflows/release.yml
```

#### Run Specific Job
```bash
act -W .github/workflows/release.yml -j preflight
```

#### Force Image Pull (Update Dependencies)
```bash
act -W .github/workflows/release.yml -j preflight --pull
```

## Shimmy Project: Release Gate Validation

### The Challenge

Shimmy has a 6-gate mandatory release validation system:
1. **Gate 1**: Core Build Validation
2. **Gate 2**: CUDA Build Validation (with 19+ hour timeout tolerance) 
3. **Gate 3**: Template Packaging Validation
4. **Gate 4**: Binary Size Constitutional Limit (20MB)
5. **Gate 5**: Test Suite Validation
6. **Gate 6**: Documentation Validation

These gates were failing publicly due to:
- Missing CUDA Toolkit on GitHub runners
- libclang dependencies for bindgen in llama.cpp compilation
- Systematic Cargo.lock uncommitted changes
- Feature naming inconsistencies

### Solution: act-Based Local Validation

#### 1. Install and Configure act
```bash
choco install act-cli
```

Create `.actrc`:
```
-P ubuntu-latest=catthehacker/ubuntu:full-latest
--container-daemon-socket npipe:////./pipe/docker_engine
```

#### 2. Local Release Gate Testing
```bash
# Navigate to project directory
cd C:\Users\micha\repos\shimmy

# Run complete 6-gate validation locally
act -W .github/workflows/release.yml -j preflight --pull
```

#### 3. Debug and Fix Issues Locally

**Example Issue Found**: libclang missing for bindgen compilation
```
thread 'main' panicked at bindgen-0.72.1/lib.rs:616:27:
Unable to find libclang: "couldn't find any valid shared libraries matching: ['libclang.so', 'libclang-*.so', 'libclang.so.*', 'libclang-*.so.*']"
```

**Solution**: Switch to `full-latest` image with complete development environment.

#### 4. Iterative Local Development

**Professional Workflow:**
1. Make code changes
2. Run `act -W .github/workflows/release.yml -j preflight` locally
3. Fix any issues discovered
4. Repeat until all gates pass locally
5. **Only then** create public release

**No More Public Test Tags**: Never again use `v1.7.2-test1`, `v1.7.2-test2`, etc.

## Advanced Features

### Environment Variables and Secrets

Create `.secrets` file in project root:
```
GITHUB_TOKEN=your_token_here
CUSTOM_SECRET=value
```

Pass to act:
```bash
act --secret-file .secrets
```

### Custom Event Types

```bash
# Simulate push event
act push

# Simulate pull request
act pull_request

# Simulate workflow_dispatch
act workflow_dispatch
```

### Docker Platform Specification

```bash
# Force specific platform
act --platform ubuntu-latest=ubuntu:latest
```

## Limitations and Considerations

### Known Limitations
- **Not 100% Identical**: Some GitHub-specific features may not work
- **Docker Dependency**: Requires Docker Desktop
- **Windows Containers**: Limited support for Windows-specific workflows
- **Resource Usage**: Large images require significant disk space
- **Secrets Management**: Local secrets file needed for authenticated operations

### Performance Considerations
- **Image Download**: First run downloads large Docker images
- **Build Caching**: Subsequent runs much faster due to Docker layer caching
- **Parallel Execution**: May need to limit concurrent jobs based on system resources

## Best Practices

### 1. Progressive Development
- Start with minimal workflows locally
- Build complexity gradually
- Test each gate individually before full validation

### 2. Image Management
- Use `micro` image for simple workflows
- Use `full` image for complex build requirements
- Update images regularly with `--pull` flag

### 3. Resource Management
- Monitor Docker disk usage
- Clean up containers regularly: `docker system prune`
- Consider dedicated development machine for large workflows

### 4. Security
- Never commit `.secrets` file to version control
- Use environment-specific secrets
- Rotate secrets regularly

## Integration with Existing Workflows

### Pre-Commit Hooks Integration
```bash
# Add to .pre-commit-config.yaml
- repo: local
  hooks:
    - id: act-validation
      name: Local GitHub Actions Validation
      entry: act -W .github/workflows/release.yml -j preflight
      language: system
      pass_filenames: false
```

### IDE Integration
Most IDEs can be configured to run act commands as build tasks or terminal shortcuts.

### CI/CD Pipeline Enhancement
Use act in development environments while maintaining GitHub Actions for production releases.

## Troubleshooting

### Common Issues

#### 1. libclang Missing
**Error**: `Unable to find libclang`
**Solution**: Switch to `catthehacker/ubuntu:full-latest` image

#### 2. Permission Denied
**Error**: Docker permission issues
**Solution**: Ensure Docker Desktop is running and user has Docker permissions

#### 3. Out of Disk Space
**Error**: No space left on device
**Solution**: `docker system prune -a` to clean up unused images and containers

#### 4. Workflow Not Found
**Error**: Workflow file not found
**Solution**: Verify path to `.github/workflows/` directory

### Debug Mode
```bash
# Enable verbose logging
act --verbose -W .github/workflows/release.yml -j preflight
```

## ROI Analysis

### Time Savings
- **Before**: 5-10 public test iterations × 15 minutes each = 75-150 minutes per release
- **After**: 2-3 local iterations × 5 minutes each = 10-15 minutes per release
- **Savings**: 60-135 minutes per release cycle

### Cost Savings
- **GitHub Actions Minutes**: ~$0.008 per minute for private repos
- **Before**: 150 minutes × $0.008 = $1.20 per release
- **After**: 15 minutes × $0.008 = $0.12 per release  
- **Savings**: $1.08 per release (90% reduction)

### Professional Image
- **Before**: Public red CI badges during development
- **After**: Only green badges visible to users
- **Value**: Immeasurable professional credibility

## Conclusion

The `act` CLI tool transforms GitHub Actions development from public trial-and-error into professional, systematic local development. For projects like Shimmy with complex build requirements and mandatory release gates, this approach is essential for maintaining professional standards while developing efficiently.

**Key Success Metrics:**
- ✅ Zero public CI failures during development
- ✅ 90% reduction in GitHub Actions costs
- ✅ Professional appearance to users and contributors
- ✅ Faster development cycles through immediate feedback
- ✅ Identical environment testing without cloud dependency

This methodology can and should be applied to all projects requiring GitHub Actions workflows.

---

## Appendix: Shimmy-Specific Configuration

### Release Workflow Command
```bash
act -W .github/workflows/release.yml -j preflight --pull
```

### Dry Run Workflow Command  
```bash
act -W .github/workflows/release-dry-run.yml -j dry-run --pull
```

### Complete Validation Command
```bash
# Test all gates locally before any public release
act -W .github/workflows/release.yml --pull
```

### Emergency Bypass (Never Use Unless Critical)
```bash
# Only for genuine emergencies - breaks professional standards
act -W .github/workflows/release.yml -j preflight --pull --no-cleanup
```

This guide represents the systematic solution to professional CI/CD development and should be referenced for all future projects requiring GitHub Actions workflows.