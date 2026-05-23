# Chat Templates Reference

How Shimmy formats conversations for each model family, what stop tokens terminate generation, and how to configure or override templates per request.

---

## What Is a Chat Template?

A **chat template** transforms a structured conversation (system + messages) into the raw text string the model actually sees as input. Language models don't know about "system prompts" or "roles" — they're text predictors. The template encodes the structure into the token stream in whatever format the model was fine-tuned to expect.

Get the template wrong and the model receives a string it wasn't trained on. Outputs will range from slightly degraded to completely broken. Getting it right is critical for instruction-following behavior.

---

## Template Auto-Detection

Shimmy automatically selects a template for each model based on its name:

```
infer_template(model_name):
  if name contains "llama-3" | "llama3" | "meta-llama-3" → Llama3
  else → ChatML (default for everything else)
```

You can override this per-request via the `template` field in the API if Shimmy detects the wrong family for a model.

---

## Supported Template Families

### ChatML — Default for Most Models

**Used by:** TinyLlama, Phi-2, Phi-3 (planned), Gemma-2, StarCoder2, Qwen2, Mistral, and any model not explicitly recognized as another family.

**Format:**

```
<|im_start|>system
{system_prompt}<|im_end|>
<|im_start|>user
{user_message}<|im_end|>
<|im_start|>assistant
{assistant_message}<|im_end|>
<|im_start|>user
{next_user_message}<|im_end|>
<|im_start|>assistant
```
_(The final `<|im_start|>assistant\n` is the prompt prefix — the model continues from here.)_

**System prompt:** Supported. Appears as the first turn before any user/assistant messages.

**Stop tokens:**
- `<|im_end|>` — primary stop token, terminates the assistant turn
- `<|im_start|>` — secondary stop, in case the model starts a new turn header

**Example full conversation:**
```
<|im_start|>system
You are a helpful coding assistant.<|im_end|>
<|im_start|>user
Write a Python function to sum a list.<|im_end|>
<|im_start|>assistant
def sum_list(nums):
    return sum(nums)<|im_end|>
<|im_start|>user
Can you add type hints?<|im_end|>
<|im_start|>assistant

```

**curl example:**
```bash
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "tinyllama-1.1b",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "Say hello in Spanish."}
    ],
    "max_tokens": 32
  }'
```

---

### Llama3 — Llama 3.x Models

**Used by:** Llama-3.2-1B-Instruct, Llama-3.2-3B-Instruct, Llama-3.1 family, Llama-3 family.

**Format:**

```
<|begin_of_text|><|start_header_id|>system<|end_header_id|>
{system_prompt}<|eot_id|><|start_header_id|>user<|end_header_id|>
{user_message}<|eot_id|><|start_header_id|>assistant<|end_header_id|>

```

_(Note the blank line after `<|end_header_id|>` for the assistant — the model outputs into this blank space.)_

**System prompt:** Supported. Required if you want the model to follow a persona or constraints.

**Stop tokens:**
- `<|eot_id|>` — primary stop token ("end of turn id"), terminates every turn
- `<|end_of_text|>` — document end, secondary stop

**Example full conversation:**
```
<|begin_of_text|><|start_header_id|>system<|end_header_id|>
You are a helpful coding assistant.<|eot_id|><|start_header_id|>user<|end_header_id|>
Write a Rust function to reverse a string.<|eot_id|><|start_header_id|>assistant<|end_header_id|>

```

**Key difference from ChatML:** The `<|eot_id|>` token (not `<|eos|>`) is the stop token for Llama-3 models. Without it, generation would run to `max_tokens`. Shimmy adds `<|eot_id|>` and `<|end_of_text|>` to `extra_stop_tokens` automatically when the Llama3 template is active.

**curl example:**
```bash
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "llama-3.2-1b-instruct",
    "messages": [
      {"role": "user", "content": "Explain recursion briefly."}
    ],
    "max_tokens": 128
  }'
```

---

### OpenChat — Simple Completion Format

**Used by:** Models that don't match ChatML or Llama3 detection and have no fine-tuning with structured turn markers.

**Format:**

```
user: {user_message}
assistant: 
```

**System prompt:** Not supported — ignored if provided.

**Stop tokens:** None configured by default. Generation runs to `max_tokens`. Pass an explicit `stop` token in the request if needed.

This template is most appropriate for GPT-2 and raw completion models that aren't instruction-tuned.

---

## Per-Model Template Notes

### TinyLlama 1.1B Chat v1.0

TinyLlama was fine-tuned on ChatML. Uses `<|im_start|>/<|im_end|>` exactly. System prompts are supported and encouraged — the model follows them reliably.

Typical system prompt:
```
You are a helpful, respectful and honest assistant.
```

### Llama-3.2-1B / 3B Instruct

Meta's Llama-3.2 uses the Llama3 template. These models have a 131072-token native context window. For practical consumer use, set `SHIMMY_MAX_CTX=8192` to keep VRAM usage manageable.

**Note on the system prompt:** Llama-3 responds well to a concise system prompt. Very long system prompts (>500 tokens) can dilute instruction following on smaller variants.

### Phi-2 (2.7B)

Phi-2 was trained on a mix of completion and some instruction data. ChatML works but the model wasn't heavily fine-tuned on it. For Phi-2, keep system prompts short or omit them. Best results come from concrete, direct user messages.

**Known limitation:** Phi-2 doesn't have a strong system-prompt boundary — a well-crafted user message that ignores the system prompt can override it. This is a model limitation, not a Shimmy issue.

### StarCoder2 3B

StarCoder2 is a **code completion** model, not a chat model. It generates code continuations from raw prefixes, not from conversations.

For StarCoder2, use the `/v1/completions` endpoint instead of `/v1/chat/completions`:

```bash
curl -s http://127.0.0.1:11435/v1/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "starcoder2-3b",
    "prompt": "def fibonacci(n):\n    ",
    "max_tokens": 128
  }'
```

### GPT-2 (117M)

GPT-2 is a raw text completion model with no instruction tuning. Use `/v1/completions` with a plain text prefix. Context window is 1024 tokens.

---

## Custom Stop Tokens

You can add stop tokens per-request via the `stop` field:

```json
{
  "model": "your-model",
  "messages": [...],
  "stop": ["###", "\n\n", "<custom_end>"],
  "max_tokens": 256
}
```

Shimmy adds these to `extra_stop_tokens` on top of the template's default stops. Useful for:
- Models with non-standard stop markers not covered by auto-detection
- Controlling generation end in completion-style tasks
- Preventing the model from starting a new turn unprompted

**Single-token constraint:** Stop token strings must encode to a single token to be effective. Multi-token strings (e.g., `"END OF RESPONSE"`) are silently skipped — the model would have to produce all the tokens of the string at exactly that point, which is unreliable. Use single special tokens like `"<|end|>"`, `"###"`, or `"\n\n"`.

---

## Template Override via API

If Shimmy assigns the wrong template (e.g., a ChatML model named with "llama3" in the filename), you can override it:

> **Note:** Direct template override in the `POST /v1/chat/completions` body is not currently exposed as an API field. Template selection is driven by model name detection at registration time.

**Workaround:** Register the model with an explicit template at load time using `SHIMMY_BASE_GGUF` plus the model name, or use the CLI `--template` flag if running manual generation:

```bash
shimmy generate --name my-model --template chatml --prompt "..."
```

---

## Debugging Template Issues

**Symptom: Model echoes the turn markers**

If the model outputs `<|im_end|>` or `<|eot_id|>` as text rather than stopping, the stop token isn't being recognized. This usually means the tokenizer tokenizes the stop string differently than expected.

**Fix:** Pass the stop token explicitly in the request and verify it's a single token in this model's vocabulary.

**Symptom: Model starts with the user's message repeated**

The template isn't being applied at all, and the raw message text is being sent as the prompt. Check that the model spec has a `template` field set.

**Symptom: System prompt is being ignored**

- Verify you're using ChatML or Llama3 (both support system prompts)
- OpenChat ignores system prompts by design
- For Phi-2, the model may not follow system prompts strongly — this is a model limitation

---

## Further Reading

- [ARCHITECTURE.md](ARCHITECTURE.md) — how the template feeds into the inference pipeline
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) — fixing garbled output and stop token failures
- [OPENAI_COMPAT.md](OPENAI_COMPAT.md) — full OpenAI API compatibility matrix
