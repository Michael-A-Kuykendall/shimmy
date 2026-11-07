# Bounded Metal follow-up

The first manual run (42416723884, control 88038a8d7c8bed68260348b12d256588975538f5) passed 74 complete Qwen3-0.6B FP32-reference comparisons but timed out before producing Qwen3-1.7B logits. Its actual consumed embedding path logged zero-valued first-four components.

The manual apple-metal-bounded workflow (also available through the registered candidate-platforms workflow on this feature branch) checks the exact 33 unique original prompt embeddings using the same published Airframe dequant_token_embd operator. Independent expected row hashes come from the frozen F32 model; the CPU Q4_K arithmetic must match those hashes before GPU output is compared. The original 0.00002 operator bound is unchanged. No model weights are stored in this repository or uploaded artifacts.

The separate existing state probe covers reset, cancellation, invalid-input recovery and concurrent serialization on Qwen3-0.6B. It does not replace full-vector numerical certification.

Local commands:
- python3 .github/certification/bounded.py --self-test
- cargo clippy --locked --manifest-path .github/certification/probes/Cargo.toml --all-targets -- -D warnings
- actionlint .github/workflows/apple-metal-bounded.yml .github/workflows/candidate-platforms.yml

Native-only execution:
- python3 .github/certification/bounded.py --stage prepare
- python3 .github/certification/bounded.py --stage embedding
- python3 .github/certification/bounded.py --stage state
- python3 .github/certification/bounded.py --stage verify

Each test records its actual command, exit, duration and log hash. Timeouts are BLOCKED; numerical mismatches are FAIL. Artifact upload always runs and includes no model assets. Neither a passing embedding test nor a passing state test closes the original 198-comparison contract. Physical M4 remains NOT_RUN. The full unchanged contract remains in apple-certification.yml.
