# OpenAPI and Swagger UI

Shimmy exposes an OpenAPI contract and an interactive Swagger UI whenever the
HTTP server is running:

- `GET /openapi.json` returns the OpenAPI 3.0.3 document.
- `GET /docs` loads Swagger UI and points it at `/openapi.json`.

For a local server bound to port `11435`, open

```bash
curl http://127.0.0.1:11435/openapi.json
```

## What It Documents

The contract covers the complete HTTP surface registered by Shimmy:

- OpenAI-compatible models, chat completions, and text completions
- Native generation, model management, tools, workflows, health, metrics, and diagnostics
- Ollama-compatible model tags
- Anthropic-compatible messages
- The `/ws/generate` WebSocket upgrade and its purpose

Request schemas describe the fields accepted by the existing handlers. The
contract does not change API behavior or add an alternate inference path.

## How It Works

The contract is kept as a static document in `src/openapi.rs` and embedded in
the binary. This keeps the dependency footprint small and makes the published
contract explicit instead of requiring annotations on every inference handler.
The package version is inserted into the document when `/openapi.json` is
requested.

Swagger UI is served as a small HTML page that loads the pinned major version
of `swagger-ui-dist` from the public unpkg CDN. It uses the local
`/openapi.json` endpoint as its only API definition source. The UI therefore
does not expose model data or credentials; it only provides a browser for the
documented routes.

The source of truth for route registration remains `src/server.rs`. When a
route is added or changed, update `src/openapi.rs` and the human-readable
reference in `docs/API.md` in the same change.

## Issue

This feature closes GitHub issue [#153](https://github.com/Michael-A-Kuykendall/shimmy/issues/153).
