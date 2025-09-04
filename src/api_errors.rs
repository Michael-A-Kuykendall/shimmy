// Improved API error handling
use axum::{http::StatusCode, response::Json};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::Json;

    #[test]
    fn test_error_response_creation() {
        let response = ErrorResponse {
            error: "Test error message".to_string(),
        };
        
        assert_eq!(response.error, "Test error message");
    }

    #[test]
    fn test_error_response_serialization() {
        let response = ErrorResponse {
            error: "Serialization test".to_string(),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Serialization test"));
        
        let parsed: ErrorResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.error, "Serialization test");
    }

    #[test]
    fn test_api_error_model_not_found() {
        let error = ApiError::ModelNotFound("test-model".to_string());
        let (status, json_response) = <(StatusCode, Json<ErrorResponse>)>::from(error);
        
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json_response.0.error, "Model 'test-model' not found");
    }

    #[test]
    fn test_api_error_generation_failed() {
        let error = ApiError::GenerationFailed("Out of memory".to_string());
        let (status, json_response) = <(StatusCode, Json<ErrorResponse>)>::from(error);
        
        assert_eq!(status, StatusCode::BAD_GATEWAY);
        assert_eq!(json_response.0.error, "Generation failed: Out of memory");
    }

    #[test]
    fn test_api_error_invalid_request() {
        let error = ApiError::InvalidRequest("Missing required field".to_string());
        let (status, json_response) = <(StatusCode, Json<ErrorResponse>)>::from(error);
        
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json_response.0.error, "Missing required field");
    }

    #[test]
    fn test_api_error_empty_model_name() {
        let error = ApiError::ModelNotFound("".to_string());
        let (status, json_response) = <(StatusCode, Json<ErrorResponse>)>::from(error);
        
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json_response.0.error, "Model '' not found");
    }

    #[test]
    fn test_api_error_special_characters() {
        let error = ApiError::ModelNotFound("model/with/slashes".to_string());
        let (status, json_response) = <(StatusCode, Json<ErrorResponse>)>::from(error);
        
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(json_response.0.error.contains("model/with/slashes"));
    }

    #[test]
    fn test_api_error_unicode_content() {
        let error = ApiError::GenerationFailed("Erreur Unicode: éñ¡".to_string());
        let (status, json_response) = <(StatusCode, Json<ErrorResponse>)>::from(error);
        
        assert_eq!(status, StatusCode::BAD_GATEWAY);
        assert!(json_response.0.error.contains("éñ¡"));
    }

    #[test]
    fn test_api_error_long_messages() {
        let long_message = "A".repeat(1000);
        let error = ApiError::InvalidRequest(long_message.clone());
        let (status, json_response) = <(StatusCode, Json<ErrorResponse>)>::from(error);
        
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json_response.0.error, long_message);
    }

    #[test]
    fn test_api_error_debug_format() {
        let error1 = ApiError::ModelNotFound("test".to_string());
        let error2 = ApiError::GenerationFailed("test".to_string());
        let error3 = ApiError::InvalidRequest("test".to_string());
        
        let debug1 = format!("{:?}", error1);
        let debug2 = format!("{:?}", error2);
        let debug3 = format!("{:?}", error3);
        
        assert!(debug1.contains("ModelNotFound"));
        assert!(debug2.contains("GenerationFailed"));
        assert!(debug3.contains("InvalidRequest"));
        assert!(debug1.contains("test"));
        assert!(debug2.contains("test"));
        assert!(debug3.contains("test"));
    }

    #[test]
    fn test_error_response_json_structure() {
        let error = ApiError::ModelNotFound("my-model".to_string());
        let (_, json_response) = <(StatusCode, Json<ErrorResponse>)>::from(error);
        
        // Test that the structure can be serialized correctly
        let serialized = serde_json::to_value(&json_response.0).unwrap();
        assert!(serialized.is_object());
        assert!(serialized["error"].is_string());
        assert_eq!(serialized["error"], "Model 'my-model' not found");
    }

    #[test]
    fn test_multiple_error_conversions() {
        let errors = vec![
            ApiError::ModelNotFound("model1".to_string()),
            ApiError::GenerationFailed("GPU error".to_string()),
            ApiError::InvalidRequest("Bad JSON".to_string()),
            ApiError::ModelNotFound("model2".to_string()),
        ];
        
        let responses: Vec<(StatusCode, Json<ErrorResponse>)> = errors
            .into_iter()
            .map(|e| <(StatusCode, Json<ErrorResponse>)>::from(e))
            .collect();
        
        assert_eq!(responses.len(), 4);
        assert_eq!(responses[0].0, StatusCode::NOT_FOUND);
        assert_eq!(responses[1].0, StatusCode::BAD_GATEWAY);
        assert_eq!(responses[2].0, StatusCode::BAD_REQUEST);
        assert_eq!(responses[3].0, StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_error_response_fields() {
        let response = ErrorResponse {
            error: "Test".to_string(),
        };
        
        // Verify the struct has only the expected fields
        let json = serde_json::to_value(&response).unwrap();
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 1);
        assert!(obj.contains_key("error"));
    }

    #[test]
    fn test_status_code_mapping() {
        // Verify all status codes are as expected
        assert_eq!(StatusCode::NOT_FOUND.as_u16(), 404);
        assert_eq!(StatusCode::BAD_GATEWAY.as_u16(), 502);
        assert_eq!(StatusCode::BAD_REQUEST.as_u16(), 400);
    }
}
