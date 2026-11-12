# OpenAPI and interactive docs — Shimmy 2.6.7

A running server provides GET /openapi.json and GET /docs. Configured credentials
protect these routes just like inference; browser Origin checks also apply.
Use an authenticated client to retrieve the schema. A browser UI may need
explicit allowed-origin and credential handling.

The router and request/response implementation define actual behavior; schema
availability alone does not prove every external SDK feature is implemented.
Consult [API](API.md) and [OpenAI compatibility](OPENAI_COMPAT.md), especially
chat-only SSE, usage, typed errors and placeholder management/tool routes.

Model numerical/platform certification is separate from protocol documentation.
See [exact fixture scope](SUPPORTED_MODELS.md). No generated schema or UI
authorizes public publication.
