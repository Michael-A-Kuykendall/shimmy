# Features and boundaries — Shimmy 2.6.6

- Local CLI/server with default Airframe 0.4.5 dense GPU engine.
- Checked GGUF metadata/tensor admission shared with ShimmyTok 0.8.5.
- Exact supported template/tokenizer preparation and request-isolated state.
- Selected OpenAI-compatible model listing, JSON chat/text completions and
  explicit chat SSE with usage/finish/stop accounting.
- Non-loopback API-key requirement and explicit browser-origin allowlists.
- Optional selected-model warm-up and scalar diagnostic timing.

Each statement has limits: see [API](API.md), [configuration](CONFIGURATION.md),
[templates](CHAT_TEMPLATES.md) and the [exact fixture matrix](SUPPORTED_MODELS.md).
Discovery and parser support do not certify an arbitrary model.
Management/tool placeholder routes are not a complete model service.

Not enabled/certified: Gemma2, MoE, MLA, recurrent/SSM/multimodal executors,
INT4 and experimental kernels. Metal and other unexecuted native platforms
are NOT_RUN; WSL device-loss issues remain open.
The fast profile is non-inference, and compatibility flags do not restore
removed engines. A crates.io-only release does not publish other distributions.
