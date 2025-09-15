# Shimmy Metrics Collection

## Overview

Shimmy includes an **optional usage analytics system** to understand how it's used in real-world scenarios. This data helps prioritize development and may inform future enterprise tooling decisions.

## Our Commitment

**Core Shimmy stays free forever** - this analytics supports potential future enterprise add-ons while keeping the base product completely free and open source.

## Philosophy: Honest & Transparent

- **Opt-in only**: Analytics disabled by default, your choice entirely
- **Anonymous**: Zero personal information, code, or prompts collected
- **Transparent**: Full disclosure of data collection and usage
- **Respectful**: Easy to disable, clear purpose, no dark patterns
- **Aligned**: Supports keeping Shimmy free through sustainable business model

## What Data Is Collected

### Usage Analytics
```json
{
  "session_id": "anonymous-uuid-v4",
  "timestamp": "2025-01-15T10:30:00Z",
  "version": "1.3.2",
  "platform": "windows-x64",
  "metrics": {
    // Business intelligence
    "daily_active_usage": true,
    "session_duration_minutes": 120,
    "requests_per_session": 47,
    "total_tokens_generated": 2350,
    "models_used_count": 2,
    "api_endpoints_used": ["/v1/chat/completions", "/v1/models"],
    "integration_type": "vscode",
    
    // Market segmentation
    "deployment_type": "development",
    "hardware_tier": "workstation", 
    "region_indicator": "americas",
    
    // Performance metrics
    "avg_response_time_ms": 1250,
    "peak_requests_per_hour": 15,
    "gpu_detected": true,
    "gpu_vendor": "nvidia"
  }
}
```

### Business Intelligence Metrics
- **User Engagement**: Session duration, daily active usage, requests per session
- **Market Adoption**: Integration types (VS Code, Cursor, etc.), deployment patterns
- **Scale Indicators**: Peak request rates, concurrent model usage, token generation volume
- **Market Segmentation**: Hardware tiers, regional distribution, usage patterns

### System Information
- Operating system and architecture
- Available memory and CPU cores
- GPU vendor (nvidia/amd/intel/apple) - no specific model info
- Shimmy version and build information

## What Is NOT Collected

❌ **Never collected:**
- Model names or content
- User prompts or AI responses
- File paths or directory structures
- IP addresses or network information
- Personal identification information
- API keys or authentication tokens
- Detailed system specifications (exact CPU/GPU models)

## How It Works

### First Run Experience
```
🚀 Welcome to Shimmy!

📊 Help Improve Shimmy
Shimmy can collect anonymous usage metrics to help improve performance 
and reliability. This data helps prioritize features and fix issues.

What's collected: Performance metrics, error rates, feature usage
What's NOT collected: Your prompts, responses, or personal data

Enable metrics? [Y/n]: 
```

### Settings Management
```bash
# Enable metrics
shimmy config --metrics enable

# Disable metrics  
shimmy config --metrics disable

# View current setting
shimmy config --metrics status

# See what data would be sent (dry-run)
shimmy config --metrics preview
```

### Configuration File
Location: `~/.config/shimmy/config.json` (Linux/macOS) or `%APPDATA%\shimmy\config.json` (Windows)

```json
{
  "metrics": {
    "enabled": false,
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "last_updated": "2025-01-15T10:30:00Z"
  }
}
```

## Data Usage

### Core Product Improvements
- **Performance optimization**: Focus development on real bottlenecks
- **Platform priorities**: Support the platforms users actually use  
- **Feature development**: Build what developers need most
- **Bug fixes**: Target issues affecting the most users
- **Compatibility**: Optimize for popular hardware configurations

### Business Intelligence
- **Market understanding**: Gauge adoption patterns and use cases
- **Enterprise planning**: Inform potential future premium tooling
- **Investor relations**: Demonstrate product-market fit and growth
- **Strategic decisions**: Data-driven product roadmap planning

### Community Benefits  
- **Public insights**: Anonymized usage trends shared with community
- **Performance data**: Real-world benchmarks and recommendations
- **Hardware guidance**: "Tested best on..." recommendations based on data
- **Ecosystem growth**: Help other developers understand AI inference market

## Data Handling

### Privacy Protection
- **No personal data**: Only anonymous technical metrics
- **Aggregated analysis**: Individual sessions not identifiable
- **Secure transmission**: HTTPS encryption for all data
- **Minimal retention**: Data deleted after analysis completion
- **No third parties**: Data not shared with external companies

### Technical Implementation
- **Lightweight**: <1KB data per session, sent at shutdown only
- **Non-blocking**: Never impacts Shimmy performance
- **Fail-safe**: Metrics failures don't affect core functionality
- **Batched**: Multiple sessions combined to reduce network overhead

### Data Endpoint
- **URL**: `https://metrics.shimmy-ai.dev/v1/usage`
- **Method**: POST with JSON payload
- **Frequency**: Once per session (at shutdown)
- **Timeout**: 5 seconds maximum
- **Fallback**: Silent failure if endpoint unavailable

## Benefits to Users

### Direct Improvements
- **Better performance**: Optimizations based on real usage
- **Platform support**: Focus on most-used platforms
- **Bug fixes**: Faster resolution of common issues
- **Feature priorities**: Develop what users actually need

### Community Benefits
- **Public benchmarks**: Real-world performance data
- **Hardware recommendations**: "Best tested on..." guidance
- **Usage insights**: Understanding how Shimmy is used

## Control and Privacy

### Easy Opt-out
```bash
# Disable permanently
shimmy config --metrics disable

# Or delete config file entirely
rm ~/.config/shimmy/config.json  # Linux/macOS
del %APPDATA%\shimmy\config.json  # Windows
```

### Transparency Tools
```bash
# See exactly what would be sent
shimmy config --metrics preview

# Export all your metrics data
shimmy config --metrics export > my_metrics.json

# Validate no personal data
shimmy config --metrics validate
```

## Example Data Flow

1. **User enables metrics** (opt-in on first run)
2. **Shimmy tracks performance** (memory usage, response times, errors)
3. **Session ends** (user stops Shimmy)
4. **Anonymous data sent** (performance metrics only, <1KB)
5. **Analysis improves Shimmy** (performance optimizations, bug fixes)
6. **User benefits** (faster, more reliable Shimmy)

## Questions & Answers

### Q: Can I see what data is being sent?
**A:** Yes! Use `shimmy config --metrics preview` to see the exact JSON payload.

### Q: How do I know my data isn't being misused?
**A:** The code is open source - you can verify exactly what's collected and sent.

### Q: What if I change my mind?
**A:** Easy to disable anytime with `shimmy config --metrics disable`.

### Q: Does this slow down Shimmy?
**A:** No. Data is collected passively and sent only at shutdown. Zero performance impact.

### Q: Is this required to use Shimmy?
**A:** Absolutely not. Shimmy works perfectly with metrics disabled (the default).

### Q: Who has access to this data?
**A:** Only the Shimmy maintainer (Michael A. Kuykendall) for product improvement analysis.

## Commitment

**We pledge:**
- Metrics remain opt-in forever
- No personal data will ever be collected
- Data is used solely for product improvement
- Users maintain full control over their data
- Complete transparency in collection and usage

---

*This metrics system helps make Shimmy better for everyone while respecting your privacy and choice.*