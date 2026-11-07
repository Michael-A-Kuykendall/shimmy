# Prompt preparation architecture — Shimmy 2.6.6

src/prompt_render.rs centralizes chat/completion rendering. HTTP handlers obtain
the loaded model's template and special-token extras, render the ordered turns
once, then pass a prepared prompt to the engine. The strict Airframe path uses
its validated prompt contract; compatibility family fallbacks do not expand
admitted inference support.

ShimmyJinja 0.5.0 handles the supported Jinja subset. ShimmyTok 0.8.5 tokenizes
prepared text/control markers; Airframe preserves those IDs for generation.
Raw mode is an explicit separate input policy, not permission to prepend another
chat wrapper. Missing required metadata/template behavior should error rather
than invent a default.

Input templating, output grammar constraints and GPU inference are separate
contracts. A renderer accepting Gemma-style text does not enable a Gemma
executor. A grammar option does not prove schema-constrained generation.

Test exact rendered bytes and IDs, multi-turn conversations, JSON/chat SSE
integration and long-to-short isolation on the actual combined candidate.
See [template usage](CHAT_TEMPLATES.md), [certification](CERTIFICATION.md)
and [fixture coverage](SUPPORTED_MODELS.md). No universal template/model parity
claim follows from these components.
