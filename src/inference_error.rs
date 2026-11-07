use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
#[derive(Debug)]
#[cfg_attr(not(feature = "airframe"), allow(dead_code))]
struct BackendFailure(Box<dyn std::error::Error + Send + Sync>);
impl std::fmt::Display for BackendFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for BackendFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}
/// Keep the concrete engine error visible in the standard source chain.
#[cfg_attr(not(feature = "airframe"), allow(dead_code))]
pub fn from_boxed(error: Box<dyn std::error::Error + Send + Sync>) -> anyhow::Error {
    anyhow::Error::new(BackendFailure(error))
}
#[derive(Debug)]
pub struct ClassifiedError {
    pub status: StatusCode,
    pub code: String,
    pub kind: &'static str,
    pub message: String,
}
pub fn classify(error: &anyhow::Error) -> ClassifiedError {
    for cause in error.chain() {
        if let Some(e) = cause.downcast_ref::<crate::prepared_prompt::PromptError>() {
            let status = match e {
                crate::prepared_prompt::PromptError::Identity => StatusCode::CONFLICT,
                crate::prepared_prompt::PromptError::Cancelled => {
                    StatusCode::from_u16(499).expect("valid code")
                }
                crate::prepared_prompt::PromptError::StopPolicy => StatusCode::BAD_REQUEST,
                _ => StatusCode::UNPROCESSABLE_ENTITY,
            };
            return ClassifiedError {
                status,
                code: e.code().into(),
                kind: "invalid_request_error",
                message: e.to_string(),
            };
        }
        if cause.is::<crate::engine::ResidentModelConflict>() {
            return ClassifiedError {
                status: StatusCode::CONFLICT,
                code: "model_not_resident".into(),
                kind: "invalid_request_error",
                message: cause.to_string(),
            };
        }
        #[cfg(feature = "airframe")]
        {
            if let Some(e) =
                cause.downcast_ref::<airframe::execution::admission::ModelAdmissionError>()
            {
                let client = e.client_error();
                return ClassifiedError {
                    status: if client {
                        StatusCode::UNPROCESSABLE_ENTITY
                    } else {
                        StatusCode::SERVICE_UNAVAILABLE
                    },
                    code: e.code().into(),
                    kind: if client {
                        "invalid_request_error"
                    } else {
                        "server_error"
                    },
                    message: if client {
                        e.to_string()
                    } else {
                        "Model resources or GPU are unavailable".into()
                    },
                };
            }
            if let Some(e) = cause.downcast_ref::<airframe::execution::admission::ExecutionError>()
            {
                let (status, code) = match e {
                    airframe::execution::admission::ExecutionError::Request { code, .. } => {
                        (StatusCode::BAD_REQUEST, *code)
                    }
                    airframe::execution::admission::ExecutionError::State(
                        airframe::runtime::state::StateError::Busy,
                    ) => (StatusCode::SERVICE_UNAVAILABLE, "state_busy"),
                    airframe::execution::admission::ExecutionError::State(
                        airframe::runtime::state::StateError::Identity,
                    ) => (StatusCode::CONFLICT, "state_identity_mismatch"),
                    airframe::execution::admission::ExecutionError::State(_) => {
                        (StatusCode::INTERNAL_SERVER_ERROR, "state_execution_failed")
                    }
                    airframe::execution::admission::ExecutionError::Scheduler { code, .. } => {
                        (StatusCode::SERVICE_UNAVAILABLE, *code)
                    }
                    airframe::execution::admission::ExecutionError::Gpu(_) => {
                        (StatusCode::SERVICE_UNAVAILABLE, "gpu_execution_failed")
                    }
                };
                return ClassifiedError {
                    status,
                    code: code.into(),
                    kind: if status.is_client_error() {
                        "invalid_request_error"
                    } else {
                        "server_error"
                    },
                    message: if status.is_client_error() {
                        e.to_string()
                    } else {
                        "Generation resources are unavailable".into()
                    },
                };
            }
        }
    }
    ClassifiedError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        code: "generation_failed".into(),
        kind: "server_error",
        message: "Generation failed".into(),
    }
}
pub fn body(error: &anyhow::Error) -> serde_json::Value {
    let e = classify(error);
    serde_json::json!({"error":{"message":e.message,"type":e.kind,"param":null,"code":e.code}})
}
pub fn response(error: &anyhow::Error) -> Response {
    let status = classify(error).status;
    (status, Json(body(error))).into_response()
}
