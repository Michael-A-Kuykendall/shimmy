use axum::{response::Html, Json};
use serde_json::Value;

const OPENAPI_JSON_TEMPLATE: &str = r##"
{
  "openapi": "3.0.3",
  "info": {
    "title": "Shimmy API",
    "description": "OpenAI-compatible local inference server with native, Ollama-compatible, and Anthropic-compatible endpoints.",
    "version": "VERSION"
  },
  "servers": [{"url": "/"}],
  "paths": {
    "/health": {"get": {"summary": "Health check", "responses": {"200": {"description": "Service health and loaded-model counts."}}}},
    "/metrics": {"get": {"summary": "Runtime metrics", "responses": {"200": {"description": "Runtime and model metrics."}}}},
    "/diag": {"get": {"summary": "Diagnostics", "responses": {"200": {"description": "Diagnostic information."}}}},
    "/api/generate": {"post": {"summary": "Generate text", "requestBody": {"required": true, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/GenerateRequest"}}}}, "responses": {"200": {"description": "Generated text or an SSE stream."}}}},
    "/api/models": {"get": {"summary": "List native models", "responses": {"200": {"description": "Available models."}}}},
    "/api/models/discover": {"post": {"summary": "Rediscover models", "responses": {"200": {"description": "Discovery result."}}}},
    "/api/models/{name}/load": {"post": {"summary": "Load a model", "parameters": [{"$ref": "#/components/parameters/ModelName"}], "responses": {"200": {"description": "Model load result."}}}},
    "/api/models/{name}/unload": {"post": {"summary": "Unload a model", "parameters": [{"$ref": "#/components/parameters/ModelName"}], "responses": {"200": {"description": "Model unload result."}}}},
    "/api/models/{name}/status": {"get": {"summary": "Get model status", "parameters": [{"$ref": "#/components/parameters/ModelName"}], "responses": {"200": {"description": "Model status."}}}},
    "/api/tools": {"get": {"summary": "List tools", "responses": {"200": {"description": "Available tools."}}}},
    "/api/tools/{name}/execute": {"post": {"summary": "Execute a tool", "parameters": [{"$ref": "#/components/parameters/ToolName"}], "requestBody": {"required": false, "content": {"application/json": {"schema": {"type": "object", "additionalProperties": true}}}}, "responses": {"200": {"description": "Tool result."}}}},
    "/api/workflows/execute": {"post": {"summary": "Execute a workflow", "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "additionalProperties": true}}}}, "responses": {"200": {"description": "Workflow result."}}}},
    "/v1/models": {"get": {"summary": "List OpenAI-compatible models", "responses": {"200": {"description": "OpenAI model list.", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ModelsResponse"}}}}}}},
    "/v1/chat/completions": {"post": {"summary": "Create a chat completion", "requestBody": {"required": true, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ChatCompletionRequest"}}}}, "responses": {"200": {"description": "A completion or an SSE stream."}, "400": {"description": "Invalid request."}, "404": {"description": "Model not found."}}}},
    "/v1/completions": {"post": {"summary": "Create a text completion", "requestBody": {"required": true, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/CompletionRequest"}}}}, "responses": {"200": {"description": "A text completion."}}}},
    "/api/tags": {"get": {"summary": "List Ollama-compatible models", "responses": {"200": {"description": "Ollama model list."}}}},
    "/v1/messages": {"post": {"summary": "Create an Anthropic-compatible message", "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "additionalProperties": true}}}}, "responses": {"200": {"description": "A message response."}}}},
    "/ws/generate": {"get": {"summary": "Generate over WebSocket", "description": "Upgrade this request to a WebSocket connection and exchange GenerateRequest/token frames.", "responses": {"101": {"description": "WebSocket protocol switch."}}}}
  },
  "components": {
    "parameters": {
      "ModelName": {"name": "name", "in": "path", "required": true, "schema": {"type": "string"}},
      "ToolName": {"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}
    },
    "schemas": {
      "GenerateRequest": {"type": "object", "required": ["model", "prompt"], "properties": {"model": {"type": "string"}, "prompt": {"type": "string"}, "messages": {"type": "array", "items": {"type": "object"}}, "system": {"type": "string"}, "temperature": {"type": "number", "format": "float"}, "top_p": {"type": "number", "format": "float"}, "top_k": {"type": "integer"}, "max_tokens": {"type": "integer", "minimum": 1}, "stream": {"type": "boolean"}, "raw_prompt": {"type": "boolean"}}},
      "ChatCompletionRequest": {"type": "object", "required": ["model", "messages"], "properties": {"model": {"type": "string"}, "messages": {"type": "array", "items": {"$ref": "#/components/schemas/ChatMessage"}}, "stream": {"type": "boolean"}, "temperature": {"type": "number", "format": "float"}, "max_tokens": {"type": "integer", "minimum": 1}, "top_p": {"type": "number", "format": "float"}, "stop": {"oneOf": [{"type": "string"}, {"type": "array", "items": {"type": "string"}}]}, "frequency_penalty": {"type": "number", "format": "float"}, "presence_penalty": {"type": "number", "format": "float"}}},
      "ChatMessage": {"type": "object", "required": ["role", "content"], "properties": {"role": {"type": "string"}, "content": {"oneOf": [{"type": "string"}, {"type": "array", "items": {"type": "object"}}]}}},
      "CompletionRequest": {"type": "object", "required": ["model", "prompt"], "properties": {"model": {"type": "string"}, "prompt": {"type": "string"}, "temperature": {"type": "number", "format": "float"}, "max_tokens": {"type": "integer", "minimum": 1}, "top_p": {"type": "number", "format": "float"}, "stream": {"type": "boolean"}}},
      "ModelsResponse": {"type": "object", "required": ["object", "data"], "properties": {"object": {"type": "string", "example": "list"}, "data": {"type": "array", "items": {"type": "object", "required": ["id", "object", "created", "owned_by"], "properties": {"id": {"type": "string"}, "object": {"type": "string"}, "created": {"type": "integer"}, "owned_by": {"type": "string"}}}}}}
    }
  }
}
"##;

pub async fn openapi_json() -> Json<Value> {
    let document =
        OPENAPI_JSON_TEMPLATE.replace("\"VERSION\"", &format!("\"{}\"", env!("CARGO_PKG_VERSION")));
    Json(serde_json::from_str(&document).expect("embedded OpenAPI document must be valid JSON"))
}

pub async fn docs() -> Html<&'static str> {
    Html(
        r##"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Shimmy API Documentation</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
  </head>
  <body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>window.ui = SwaggerUIBundle({ url: "/openapi.json", dom_id: "#swagger-ui" });</script>
  </body>
</html>"##,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_spec_is_valid_openapi_document() {
        let document: Value = serde_json::from_str(OPENAPI_JSON_TEMPLATE).unwrap();
        assert_eq!(document["openapi"], "3.0.3");
        assert!(document["paths"]["/v1/chat/completions"].is_object());
        assert!(document["paths"]["/ws/generate"].is_object());
    }

    #[tokio::test]
    async fn docs_page_points_to_openapi_spec() {
        let Html(page) = docs().await;
        assert!(page.contains("swagger-ui-bundle.js"));
        assert!(page.contains("/openapi.json"));
    }
}
