# Shimmy Constitutional Principles

*These principles are immutable and govern all development decisions for the Shimmy inference engine.*

## Article I: The Lightweight Binary Mandate
**IMMUTABLE**: Shimmy's core binary shall remain lightweight and focused, never exceeding 20MB. This preserves our fundamental competitive advantage over bloated alternatives (Ollama: 680MB+). Even at 20MB, Shimmy remains 34x smaller than Ollama. Target range: 10-20MB for full-featured deployment. All features must be designed for minimal binary impact while delivering maximum value.

## Article II: Library-First Architecture
**REQUIREMENT**: Every feature must begin as a standalone, reusable library before integration into Shimmy core. This ensures modularity, testability, and prevents architectural debt.

## Article III: CLI Interface Mandate
**REQUIREMENT**: All functionality must be accessible via command-line interface. Shimmy serves as a universal shim - CLI access ensures programmatic integration and automation capabilities.

## Article IV: Test-First Imperative
**REQUIREMENT**: No implementation shall proceed without comprehensive test specifications. Use `cargo test --all-features` as the validation standard. Integration tests must pass before any feature is considered complete.

## Article V: Startup Speed Supremacy
**IMMUTABLE**: Shimmy must maintain sub-2-second startup time. This 2-5x speed advantage over alternatives is core to our value proposition. Any feature that degrades startup performance is rejected.

## Article VI: Zero Python Dependencies
**IMMUTABLE**: Shimmy's core shall remain free of Python runtime dependencies. Native Rust implementations only. Python integrations may exist as optional, external components but never as required dependencies.

## Article VII: API Compatibility Preservation
**REQUIREMENT**: OpenAI API compatibility must be maintained. Shimmy serves as a drop-in replacement - breaking this compatibility breaks our fundamental promise to users.

## Article VIII: Integration-First Testing
**REQUIREMENT**: Testing must prioritize real-world scenarios over mocks. Use actual model files, real HTTP requests, and genuine client integrations wherever possible.

## Article IX: Specification-Driven Development
**REQUIREMENT**: All new features must follow GitHub Spec-Kit methodology:
1. `/specify` - Create detailed specification
2. `/plan` - Generate implementation plan  
3. `/tasks` - Break into actionable items
4. Implementation with continuous validation

## Constitutional Enforcement

### Version Control Integration
- All pull requests must reference constitutional compliance
- Breaking changes require constitutional amendment process
- Major version bumps require constitutional review

### Feature Acceptance Criteria
1. ✅ Preserves lightweight binary constraint (≤20MB)
2. ✅ Maintains sub-2-second startup
3. ✅ Zero new Python dependencies
4. ✅ CLI interface provided
5. ✅ Comprehensive test coverage
6. ✅ OpenAI API compatibility maintained
7. ✅ Specification-driven development followed

### Emergency Constitutional Overrides
In exceptional circumstances, constitutional principles may be temporarily suspended only by:
1. Explicit human approval from project maintainer
2. Documented justification for the override
3. Clear remediation timeline
4. Constitutional compliance restoration plan

## Architectural Principles

### Performance Hierarchy
1. **Startup Speed** (non-negotiable)
2. **Binary Efficiency** (lightweight constraint ≤20MB)
3. **Inference Throughput** (optimize within constraints)
4. **Feature Richness** (only if compatible with above)

### Technology Stack Constraints
- **Core Language**: Rust (immutable)
- **HTTP Framework**: Axum (current standard)
- **Model Formats**: SafeTensors (native), GGUF (via llama.cpp), HuggingFace (optional)
- **GPU Support**: Multiple vendors (NVIDIA, AMD, Intel)
- **Platform Support**: Cross-platform (Windows, Linux, macOS)

### Development Workflow
- **Methodology**: GitHub Spec-Kit driven
- **Testing**: `cargo test --all-features`
- **Documentation**: Specification-first
- **Integration**: Library-first modular design

---

*Constitutional violations will result in immediate development halt and architectural review.*

*Last Updated: September 18, 2025*
*Version: 1.1 - Developer Expansion Update*

## Amendment History
- **v1.1 (Sept 18, 2025)**: Updated binary size constraint from 5MB to ≤20MB to support developer expansion features while maintaining 34x competitive advantage over Ollama
- **v1.0 (Sept 17, 2025)**: Initial constitutional framework