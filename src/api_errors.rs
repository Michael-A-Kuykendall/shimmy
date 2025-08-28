// Improved API error handling
use axum::{http::StatusCode, response::Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub enum ApiError {
    ModelNotFound(String),
    GenerationFailed(String),
    InvalidRequest(String),
}

impl From<ApiError> for (StatusCode, Json<ErrorResponse>) {
    fn from(err: ApiError) -> Self {
        match err {
            ApiError::ModelNotFound(model) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Model '{}' not found", model),
                }),
            ),
            ApiError::GenerationFailed(msg) => (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: format!("Generation failed: {}", msg),
                }),
            ),
            ApiError::InvalidRequest(msg) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: msg }),
            ),
        }
    }
}
