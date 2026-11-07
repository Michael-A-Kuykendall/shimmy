# Migration to Shimmy 2.6.6

The product uses Airframe 0.4.5 as its default engine. Earlier llama.cpp, MLX,
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

Use cargo install shimmy --version 2.6.6 --locked. Crates.io publication does not create new binary assets, Docker
images or package-manager distributions. Review [quickstart](quickstart.md),
[configuration](CONFIGURATION.md), [API](API.md) and
[2.6.6 changes](../CHANGELOG.md).

Shimmy is Apache-2.0; the Airframe runtime is MIT, while libfse has separate
noncommercial/evaluation and commercial-permission terms. Deployment must
respect all applicable component/model licenses.
