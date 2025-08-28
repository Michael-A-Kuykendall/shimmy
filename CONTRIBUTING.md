# Contributing to Shimmy

We welcome contributions to Shimmy! This document provides guidelines for contributing to the project.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Create a new branch for your feature or bugfix
4. Make your changes
5. Test your changes thoroughly
6. Submit a pull request

## Development Setup

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/yourusername/shimmy.git
cd shimmy

# Build with llama feature
cargo build --features llama

# Run tests
cargo test

# Run with sample model
export SHIMMY_BASE_GGUF=/path/to/model.gguf
cargo run --features llama -- serve
```

## Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Use `cargo clippy` to catch common issues
- Add documentation for public APIs
- Include tests for new functionality

## Pull Request Process

1. Ensure your code builds and tests pass
2. Update documentation as needed
3. Add tests for new features
4. Follow the existing code style
5. Write a clear pull request description

## Types of Contributions

### Bug Reports
- Use the GitHub issue template
- Include minimal reproduction steps
- Provide system information (OS, Rust version, etc.)

### Feature Requests
- Describe the use case
- Explain why the feature would be valuable
- Consider implementation complexity

### Code Contributions
- Bug fixes are always welcome
- New features should be discussed in an issue first
- Performance improvements should include benchmarks

## Areas Needing Help

- OpenAI API compatibility layer
- Model auto-discovery system
- Performance optimizations
- Documentation improvements
- Integration examples
- Testing across different platforms

## Testing

```bash
# Run all tests
cargo test

# Run tests with llama feature
cargo test --features llama

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

## Documentation

- API documentation should be comprehensive
- Include code examples
- Update relevant documentation files
- Use clear, concise language

## Community Guidelines

- Be respectful and inclusive
- Help others learn and contribute
- Focus on constructive feedback
- Follow the code of conduct

## Questions?

- Open an issue for technical questions
- Use discussions for general questions
- Join our community chat (link TBD)

Thank you for contributing to Shimmy!
