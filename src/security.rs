//! Authentication and explicit browser-origin policy for the HTTP server.
use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::{net::SocketAddr, sync::Arc};
use subtle::ConstantTimeEq;

/// Deliberately does not implement Debug: the API credential must not be logged.
pub struct ApiSecurity {
    api_key: Option<String>,
    allowed_origins: Vec<HeaderValue>,
}

impl ApiSecurity {
    pub fn new(api_key: Option<String>, origins: Vec<String>) -> anyhow::Result<Self> {
        if let Some(key) = &api_key {
            anyhow::ensure!(
                key.len() >= 16 && key.bytes().all(|b| b.is_ascii_graphic()),
                "SHIMMY_API_KEY must contain at least 16 printable ASCII characters without spaces"
            );
        }
        let mut allowed_origins = Vec::new();
        for origin in origins {
            let url = reqwest::Url::parse(&origin).map_err(|_| {
                anyhow::anyhow!("SHIMMY_ALLOWED_ORIGINS must contain HTTP(S) origins")
            })?;
            let canonical = url.origin().ascii_serialization();
            anyhow::ensure!(
                matches!(url.scheme(), "http" | "https")
                    && origin.trim_end_matches('/') == canonical,
                "SHIMMY_ALLOWED_ORIGINS must contain exact HTTP(S) origins, without paths or wildcards"
            );
            allowed_origins.push(HeaderValue::from_str(&canonical)?);
        }
        Ok(Self {
            api_key,
            allowed_origins,
        })
    }

    pub fn from_env() -> anyhow::Result<Self> {
        let api_key = match std::env::var("SHIMMY_API_KEY") {
            Ok(value) => Some(value),
            Err(std::env::VarError::NotPresent) => None,
            Err(_) => anyhow::bail!("SHIMMY_API_KEY must be valid Unicode"),
        };
        let origins = match std::env::var("SHIMMY_ALLOWED_ORIGINS") {
            Ok(value) => value
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect(),
            Err(std::env::VarError::NotPresent) => Vec::new(),
            Err(_) => anyhow::bail!("SHIMMY_ALLOWED_ORIGINS must be valid Unicode"),
        };
        Self::new(api_key, origins)
    }

    pub fn validate_bind(&self, addr: SocketAddr) -> anyhow::Result<()> {
        anyhow::ensure!(
            addr.ip().is_loopback() || self.api_key.is_some(),
            "Non-loopback binds require SHIMMY_API_KEY; use 127.0.0.1 for local unauthenticated access"
        );
        Ok(())
    }

    fn authenticates(&self, req: &Request) -> bool {
        let Some(expected) = &self.api_key else {
            return true;
        };
        let bearer = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split_once(' '))
            .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("Bearer"))
            .map(|(_, key)| key);
        let anthropic = req
            .headers()
            .get("x-api-key")
            .and_then(|value| value.to_str().ok());
        [bearer, anthropic]
            .into_iter()
            .flatten()
            .any(|key| bool::from(expected.as_bytes().ct_eq(key.as_bytes())))
    }
}

pub async fn protect(State(policy): State<Arc<ApiSecurity>>, req: Request, next: Next) -> Response {
    let origin = req.headers().get(header::ORIGIN).cloned();
    if origin
        .as_ref()
        .is_some_and(|origin| !policy.allowed_origins.contains(origin))
    {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": {"message": "Browser origin is not allowed", "type": "permission_error"}
            })),
        )
            .into_response();
    }

    let mut response = if req.method() == Method::OPTIONS {
        if origin.is_none() {
            return StatusCode::BAD_REQUEST.into_response();
        }
        StatusCode::NO_CONTENT.into_response()
    } else if !(req.method() == Method::GET && req.uri().path() == "/health")
        && !policy.authenticates(&req)
    {
        let mut response = (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
            "error": {"message": "Valid API credentials required", "type": "authentication_error"}
        }))).into_response();
        response
            .headers_mut()
            .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        response
    } else {
        next.run(req).await
    };

    if let Some(origin) = origin {
        let headers = response.headers_mut();
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        headers.append(header::VARY, HeaderValue::from_static("Origin"));
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, POST, OPTIONS"),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("Content-Type, Authorization, X-Api-Key, Anthropic-Version"),
        );
        headers.insert(
            header::ACCESS_CONTROL_MAX_AGE,
            HeaderValue::from_static("600"),
        );
    }
    response
}
