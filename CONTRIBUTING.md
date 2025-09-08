# Contributing to Shimmy

Thanks for your interest in contributing to Shimmy!

## How to Contribute
1. Fork the repo and create a branch (`git checkout -b feature/foo`)
2. Make your changes with clear commits and tests if applicable
3. Run existing tests to ensure nothing breaks (`cargo test`)
4. Ensure code quality (`cargo fmt && cargo clippy`)
5. Open a Pull Request against `main`

## Code Style
- Rust 2021 edition
- Use `cargo fmt` and `cargo clippy` before submitting
- Keep PRs small and focused - large refactors may be rejected
- Add tests for new functionality
- Document public APIs with rustdoc comments

## Contribution Scope
Features should align with the **Shimmy philosophy**:
- **Lightweight**: ~5MB binary target
- **Zero-config**: No setup, just works
- **OpenAI API compatibility**: Drop-in replacement
- **Invisible infrastructure**: Minimal surface area

## What We Welcome
- Bug fixes with test cases
- Performance improvements
- API compatibility enhancements
- Documentation improvements
- Platform-specific fixes
- Test coverage improvements

## What We Generally Reject
- Features that bloat binary size significantly
- Complex configuration systems
- UI/dashboard components (use external tools)
- Breaking changes to established APIs
- Features not aligned with OpenAI compatibility

## Review Process
- All PRs require review and approval from the lead maintainer
- Merge authority is reserved to maintain project direction
- We aim to review PRs within 1-2 business days
- Constructive feedback will be provided for rejected PRs

## Development Setup
```bash
# Clone and setup
git clone https://github.com/Michael-A-Kuykendall/shimmy
cd shimmy
cargo build

# Run tests
cargo test

# Check formatting and linting
cargo fmt --check
cargo clippy -- -D warnings
```

## Recognition
Contributors are acknowledged in `AUTHORS.md` after a merged PR.

## Questions?
Open a GitHub Discussion or ping @Michael-A-Kuykendall in your PR.