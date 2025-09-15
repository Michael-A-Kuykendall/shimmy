# Security Policy

## Supported Versions

We actively support the following versions of Shimmy with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 1.3.x   | :white_check_mark: |
| 1.2.x   | :white_check_mark: |
| 1.1.x   | :x:                |
| < 1.1   | :x:                |

## Reporting a Vulnerability

We take the security of Shimmy seriously. If you discover a security vulnerability, please follow these guidelines:

### 🔒 Private Disclosure Process

**DO NOT** create a public GitHub issue for security vulnerabilities.

Instead, please report security issues privately using one of these methods:

1. **GitHub Security Advisories (Preferred)**
   - Go to the [Security tab](https://github.com/Michael-A-Kuykendall/shimmy/security) of this repository
   - Click "Report a vulnerability"
   - Fill out the advisory form with details

2. **Direct Email**
   - Send details to: security@shimmy-ai.dev
   - Use PGP encryption if possible (key available on request)
   - Include "SECURITY" in the subject line

### 📋 What to Include

Please provide the following information in your report:

- **Description**: Clear description of the vulnerability
- **Impact**: What could an attacker accomplish?
- **Reproduction**: Step-by-step instructions to reproduce the issue
- **Environment**: 
  - Shimmy version
  - Operating system
  - Rust version (if building from source)
  - Any relevant configuration
- **Proof of Concept**: Code, screenshots, or logs demonstrating the issue
- **Suggested Fix**: If you have ideas for remediation

### 🕒 Response Timeline

We aim to respond to security reports according to the following timeline:

- **Initial Response**: Within 48 hours of report
- **Triage**: Within 7 days - confirm/deny vulnerability 
- **Resolution**: Within 30 days for critical issues, 90 days for others
- **Disclosure**: Public disclosure after fix is released and users have time to update

### 🛡️ Vulnerability Severity Guidelines

We use the following criteria to classify vulnerabilities:

#### Critical
- Remote code execution
- Privilege escalation to system level
- Authentication bypass allowing admin access

#### High  
- Local privilege escalation
- SQL injection or similar injection attacks
- Authentication bypass for user accounts
- Sensitive data exposure

#### Medium
- Cross-site scripting (XSS)
- Cross-site request forgery (CSRF) 
- Information disclosure (non-sensitive)
- Denial of service attacks

#### Low
- Issues requiring local access
- Minor information leaks
- Issues with minimal security impact

### 🎁 Recognition

We believe in recognizing security researchers who help keep Shimmy secure:

- **Hall of Fame**: Public recognition in our security acknowledgments
- **CVE Assignment**: For qualifying vulnerabilities
- **Early Access**: Beta access to new features
- **Swag**: Shimmy stickers and merchandise

*Note: We currently do not offer monetary bug bounties, but we deeply appreciate responsible disclosure.*

### 🚨 Emergency Contact

For critical vulnerabilities that are being actively exploited:

- **Email**: security-urgent@shimmy-ai.dev  
- **Subject**: "URGENT SECURITY - [Brief Description]"
- **Response**: Within 12 hours

## Security Best Practices

### For Users

1. **Keep Updated**: Always use the latest supported version
2. **Network Security**: 
   - Run Shimmy behind a firewall in production
   - Use HTTPS/TLS for external access
   - Limit network access to necessary ports only
3. **Model Security**:
   - Only use models from trusted sources
   - Scan downloaded models with antivirus
   - Be cautious with user-uploaded models
4. **Container Security**:
   - Use official Shimmy Docker images
   - Keep base images updated
   - Run containers as non-root when possible

### For Developers

1. **Dependencies**: 
   - Regularly audit and update dependencies
   - Use `cargo audit` to check for known vulnerabilities
2. **Input Validation**:
   - Validate all user inputs
   - Sanitize file paths and model names
   - Implement rate limiting
3. **Error Handling**:
   - Don't expose internal errors to users
   - Log security events for monitoring

## Security Features

Shimmy includes several built-in security features:

- **Input Sanitization**: All API inputs are validated and sanitized
- **Path Traversal Protection**: File access is restricted to configured directories  
- **Memory Safety**: Built with Rust for memory-safe execution
- **Sandboxed Execution**: Model inference runs in isolated contexts
- **Audit Logging**: Security events are logged for monitoring

## Scope

This security policy covers:

- **In Scope**:
  - Shimmy server binary and source code
  - Official Docker images and containers
  - API endpoints and interfaces
  - Configuration and deployment scripts
  - Dependencies and third-party integrations

- **Out of Scope**:
  - Third-party models (GGUF files)
  - User-provided configurations or scripts
  - Issues in unsupported versions
  - General Rust language or standard library issues
  - Infrastructure or hosting environment issues

## Legal

- We will not pursue legal action against security researchers who follow this policy
- We ask that researchers not access, modify, or delete data without explicit permission
- Please do not perform testing on production systems without prior authorization

## Contact

For non-security related issues, please use:
- GitHub Issues: https://github.com/Michael-A-Kuykendall/shimmy/issues
- General Questions: hello@shimmy-ai.dev

---

*This security policy is effective as of September 14, 2025 and may be updated periodically.*