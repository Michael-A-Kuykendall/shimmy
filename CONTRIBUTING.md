# Contributing to Shimmy

Thanks for your interest in contributing to Shimmy!

## How to Contribute
1. Fork the repo and create a branch (`git checkout -b feature/foo`)
2. Make your changes with clear commits and tests if applicable
3. **Sign off your commits** (required): `git commit -s -m "Your message"`
4. Run existing tests to ensure nothing breaks (`cargo test`)
5. Ensure code quality (`cargo fmt && cargo clippy`)
6. Open a Pull Request against `main`

### Developer Certificate of Origin (DCO)
All contributions must be signed off with the Developer Certificate of Origin. This certifies that you have the right to contribute your code. See [DCO.md](DCO.md) for details.

**Quick setup:**
```bash
git config format.signoff true  # Auto sign-off all commits
```

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

## Maintainer Process

### Current Maintainer Structure
- **Lead Maintainer**: Michael A. Kuykendall (@Michael-A-Kuykendall)
- **Additional Maintainers**: Currently none (solo-maintained project)

### Becoming a Maintainer
Currently, Shimmy is maintained by a single maintainer to ensure consistent vision and rapid iteration. If you're interested in becoming a maintainer in the future:

1. **Demonstrate Expertise**: Contribute several high-quality PRs over time
2. **Private Application**: Email `maintainer-request@shimmy-ai.dev` with:
   - Your GitHub username and contribution history
   - Area of expertise (e.g., performance, platform support, API design)
   - Time commitment you can provide
   - Why you'd like to help maintain Shimmy

3. **Evaluation Process**: Applications are reviewed when additional maintainers are needed

### Maintainer Responsibilities
When additional maintainers are added, they will:
- Review and approve pull requests
- Triage and respond to issues
- Maintain code quality standards
- Participate in release planning
- Uphold the Shimmy philosophy and project direction

*Note: The project may remain solo-maintained until contribution volume requires additional help.*

## Recognition
Contributors are acknowledged in `AUTHORS.md` after a merged PR.

## Questions?
Open a GitHub Discussion or ping @Michael-A-Kuykendall in your PR.