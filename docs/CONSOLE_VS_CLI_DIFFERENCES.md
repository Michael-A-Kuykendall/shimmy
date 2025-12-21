# Console vs CLI Behavior Differences

This document tracks intentional differences between console WebSocket inference and CLI inference to prevent regressions.

## Generation Parameters

### Common Settings (Both Paths)
- `temperature`: 0.7
- `top_p`: 0.9 (non-streaming only, streaming uses default)
- `top_k`: 40 (non-streaming only, streaming uses default)
- Stop tokens: Derived from `TemplateFamily::stop_tokens()` for detected model
- Templates: Both use `TemplateFamily::detect()` for consistent prompt formatting

### Intentional Differences

#### Repeat Penalty
- **Non-streaming (console/CLI)**: `repeat_penalty = 1.1`
- **Streaming (console)**: `repeat_penalty = 1.2`
- **Rationale**: Stronger penalty in streaming helps prevent infinite loops since user sees output in real-time

#### Max Tokens
- **Both paths**: No hardcoded `max_tokens` cap
- **Rationale**: Rely on templates and stop tokens for natural stopping, avoiding arbitrary truncation
- **Previous behavior**: Had `max_tokens = 256` cap that was removed per surgical checklist

## Protocol Differences

### CLI
- Outputs plain text to stdout
- No streaming protocol structure
- Stop tokens handled by engine only

### Console WebSocket
- Streaming: `{"token": "..."}` per token, `{"done": true}` on completion
- Non-streaming: Direct JSON responses with `type` field
- Additional sanitization: `sanitize_stream_payload()` strips leaked role markers
- Repetition guard: Collapses pathological repeats (8+ char phrases repeated 5+ times)

## Model Detection
- **Both**: Use `TemplateFamily::detect()` from model name
- **Both**: Use `model_entry.template` if available in registry
- **Both**: Same fallback logic if detection fails

## Stop Token Handling
- **Both**: Stop tokens passed to `GenOptions`
- **Console**: Additional detection in `sanitize_stream_payload()` for `<|end|>` and `<|im_end|>`
- **Console**: Repetition guard can trigger early stop

## Error Handling
- **CLI**: Errors propagate to stderr
- **Console**: Errors emitted as `{"token": "Error: ..."}` + `{"done": true}` in canonical protocol
