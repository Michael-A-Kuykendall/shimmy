# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report security vulnerabilities by emailing [michaelallenkuykendall@gmail.com](mailto:michaelallenkuykendall@gmail.com) with:

- A description of the vulnerability
- Steps to reproduce the issue
- Potential impact
- Any suggested mitigations

### What to expect

- **Response time**: Within 48 hours for initial acknowledgment
- **Updates**: Regular updates on investigation progress
- **Resolution**: Security patches released as priority updates
- **Credit**: Public recognition for responsible disclosure (if desired)

### Security Considerations

Shimmy is designed with security in mind:

- **Local only**: No external network calls by design
- **Minimal attack surface**: Small binary, limited functionality
- **Memory safety**: Written in Rust
- **No persistent state**: Stateless operation by default

### Scope

This security policy covers:
- The core Shimmy binary
- Official extensions and integrations
- Build and distribution infrastructure

Out of scope:
- Third-party models or model files
- User configurations or custom modifications
- Issues in dependencies (report to respective maintainers)

## Security Best Practices

When using Shimmy:

1. **Network exposure**: Only bind to localhost unless necessary
2. **Model sources**: Only use trusted model files
3. **File permissions**: Ensure proper file system permissions
4. **Updates**: Keep Shimmy updated to the latest version

## Vulnerability Disclosure Timeline

1. **Day 0**: Vulnerability reported privately
2. **Day 1-2**: Initial assessment and acknowledgment
3. **Day 3-14**: Investigation and patch development
4. **Day 15**: Patch released and public disclosure
5. **Day 16+**: Post-disclosure monitoring

Thank you for helping keep Shimmy and the community safe!