# Migration to Shimmy 2.6.7

The product uses Airframe 0.4.6 as its default engine. Earlier llama.cpp, MLX,
HuggingFace and RustChain engine claims do not describe this release.
Compatibility flags and empty legacy Cargo features do not restore those
executors. The no-default-features fast profile is not CPU inference.

Use the [exact dense fixture matrix](SUPPORTED_MODELS.md), F32 KV and reference
kernels. INT4, experimental kernels, Gemma/MoE/MLA/recurrent/SSM/multimodal
execution and unexecuted native GPU platforms are not enabled/certified.

Remote binds now require a private API key; browser origins must be explicit.
Chat JSON is default; chat SSE is explicit and text-completion streaming rejects.
Prompt preparation preserves exact IDs; usage/finish/stop metadata comes from
generation instead of text-length estimates.

Use `cargo install shimmy --version 2.6.7 --locked`. The 2.6.7 release also includes verified native binaries and versioned Docker
images or package-manager distributions. Review [quickstart](quickstart.md),
[configuration](CONFIGURATION.md), [API](API.md) and
[2.6.7 changes](../CHANGELOG.md).

Shimmy is Apache-2.0; Airframe and Observe are MIT, and ShimmyTok is dual-licensed. The [additional Apache-2.0 grant for libfse 0.1.4](https://github.com/Michael-A-Kuykendall/airframe/blob/8efb1e7a830b0822cfc5ac1b121e149948d5ffe3/crates/libfse/LICENSE-GRANT-0.1.4.md) permits commercial use, embedding, modification and redistribution without separate permission. Original crate metadata remains unchanged. Deployment must respect applicable component, third-party and model terms; see [stack licensing](../LICENSING.md).
