# Chat Templating Architecture

**Canonical reference for how shimmy renders chat/completion prompts.**
Read this before touching any templating code. This supersedes the old
scattered `TemplateFamily` match sites and the retired `shimmy_server_gpu`
renderer.

## 1. The single renderer

**`shimmy/src/prompt_render.rs`** is the ONE place prompts are built. It renders
the model's **real GGUF `tokenizer.chat_template`** (Jinja, via the `shimmyjinja`
crate), with a family fallback and a raw-completion escape hatch.

Decision priority (`prompt_render::decide`, mode `Auto`):

1. **GGUF Jinja** — if the model has a `chat_template`, render it via shimmyjinja
   (full HF-subset: for/if/set/slice-step/filters/bos/eos/add_generation_prompt).
2. **Family fallback** — if no GGUF template but a known family (ChatML/Llama3/Gemma),
   use `TemplateFamily::render` (the fallback writers in `src/templates.rs`).
3. **Raw** — base/completion models with no template (starcoder2, phi-2) pass the
   prompt through unchanged (NOT an OpenChat wrap).

`RenderMode::{Auto, ForceRaw, ForceTemplate}` lets callers override.

## 2. The three layers

- **Templating (INPUT shape)** — `prompt_render` shapes the model input from the
  conversation. This is a pre-tokenization concern.
- **Grammar (OUTPUT constraint)** — `airframe::grammar` / schoolmarm constrains
  generated tokens at sample time. ORTHOGONAL to templating; do not conflate them.
- **Engine (inference)** — `engine/airframe.rs` applies templating before
  `rt.generate`, then runs the forward pass.

## 3. The paths that use it

| Path | File | Uses |
|------|------|------|
| HTTP `/api/generate`, ws | `api.rs` | `render_chat_prompt_with_extras` / `render_completion_prompt_with_extras` |
| OpenAI `/v1/chat/completions`, `/v1/completions` | `openai_compat/mod.rs` | same |
| Anthropic `/v1/messages` | `anthropic_compat.rs` | same |
| CLI `generate` (cert path) | `engine/airframe.rs` | `render_completion_prompt_with_extras` |

The GGUF `chat_template` is sourced at registry load
(`model_registry::read_chat_template` → `ModelSpec.chat_template`).

## 4. `--raw` / completion policy

- **Auto-raw**: a model with no GGUF template AND no chat family (base/code) is
  raw by default.
- **Explicit `--raw`** (`GenOptions.raw_prompt`, CLI `--raw`): forces raw even when
  a template exists (escape hatch).
- Instruct models with a template always get it applied.

## 5. The regularized gate

- **`tests/gguf_routing.rs`** (shimmy, `--features airframe`): reads every GGUF
  header and asserts `decide()` routes it correctly (Jinja / Raw / family). Runs
  offline (no GPU), ~14s. This is the cheap per-model "screen test".
- **shimmyjinja tests** (`tests/real_model_templates.rs`, `edge_cases.rs`): embed
  verbatim real templates (TinyLlama, Gemma-2, Phi, Qwen2, DeepSeek, Qwen3) and
  assert exact rendered output — the renderer correctness contract.

## 6. Grammar orthogonality (do not break)

`airframe::grammar::grammar_hooks` is a sample-time logits mask, independent of
prompt templating. Keep them separate.

## 7. Decommissioned (do not resurrect)

- `airframe/src/bin/shimmy_server_gpu.rs` — legacy standalone GPU server with its
  own `ChatTemplateFamily`; **deleted** (2026-08-27). The product server is
  `shimmy/src/server.rs`.
- Coarse `match spec.template { Some("chatml")... }` selection — **removed**; all
  paths use `family_from_spec` + the single renderer.
- `shimmy_server_gpu`'s `make_prompt_renderer` — superseded by `prompt_render`.
