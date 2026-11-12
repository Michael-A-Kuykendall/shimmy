# Integration — Shimmy 2.6.7

Start the [authenticated or loopback server](quickstart.md), query /v1/models,
and use an actual returned model ID. Configure an OpenAI-compatible client's
base URL to http://127.0.0.1:11435/v1 and its API key to the configured private key.
Only an unconfigured-key loopback server may use a client-side placeholder.

Use chat completions for messages and SSE; omit stream or set false for JSON.
Text completions use prompt and reject stream:true. Set
stream_options.include_usage when final chat streaming usage is needed.
Do not assume functions, embeddings, vision or every SDK option is implemented.
See [exact compatibility](OPENAI_COMPAT.md) and [API](API.md).

For a discovered CLI model:
```bash
shimmy list
shimmy generate MODEL_NAME --prompt "Hello" --max-tokens 8
```
Replace MODEL_NAME with a registered name. --model-path belongs to serve, not
generate; additional search directories use --model-dirs.

Browser origins must be explicitly allowed, independently of authentication.
Use TLS and appropriate network controls for remote clients.
Model files, templates, prompts and diagnostics retain their own privacy and
licensing requirements.

Generated init templates are historical scaffolds, not certified deployment
integrations; see [deployment scope](../deploy/README.md). No new Docker,
The released 2.6.7 installation channels include verified native binaries, versioned GHCR, Homebrew and Scoop; npm is not provided and `latest` is unchanged.
