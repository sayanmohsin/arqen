#[cfg(feature = "http-server")]
use axum::http::StatusCode;
#[cfg(feature = "http-server")]
use axum::response::{IntoResponse, Response};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ErrorKind {
    #[error("not found")]
    NotFound,
    #[error("validation error")]
    Validation,
    #[error("authentication error")]
    Authentication,
    #[error("authorization error")]
    Authorization,
    #[error("internal error")]
    Internal,
    #[error("external error")]
    External,
}

#[derive(Debug, thiserror::Error)]
#[error("{kind}: {message}")]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl AppError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }
}

#[cfg(feature = "http-server")]
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self.kind {
            ErrorKind::NotFound => StatusCode::NOT_FOUND,
            ErrorKind::Validation => StatusCode::BAD_REQUEST,
            ErrorKind::Authentication => StatusCode::UNAUTHORIZED,
            ErrorKind::Authorization => StatusCode::FORBIDDEN,
            ErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorKind::External => StatusCode::BAD_GATEWAY,
        };

        let body = axum::Json(serde_json::json!({
            "error": {
                "kind": format!("{}", self.kind),
                "message": self.message,
            }
        }));

        (status, body).into_response()
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, ()>>> for AppError {
    fn from(e: std::sync::PoisonError<std::sync::MutexGuard<'_, ()>>) -> Self {
        AppError::new(ErrorKind::Internal, format!("mutex poisoned: {e}"))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::new(ErrorKind::Validation, format!("invalid JSON: {e}"))
    }
}

#[cfg(feature = "http-client")]
impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::new(ErrorKind::External, format!("HTTP client error: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_kind_display() {
        assert_eq!(format!("{}", ErrorKind::NotFound), "not found");
        assert_eq!(format!("{}", ErrorKind::Validation), "validation error");
        assert_eq!(format!("{}", ErrorKind::Internal), "internal error");
    }

    #[test]
    fn test_app_error_new() {
        let err = AppError::new(ErrorKind::NotFound, "user not found");
        assert_eq!(err.kind, ErrorKind::NotFound);
        assert_eq!(err.message, "user not found");
        assert!(err.source.is_none());
    }

    #[test]
    fn test_app_error_display() {
        let err = AppError::new(ErrorKind::Validation, "invalid email");
        assert_eq!(format!("{err}"), "validation error: invalid email");
    }

    #[test]
    fn test_app_error_with_source() {
        let source = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = AppError::new(ErrorKind::Internal, "io failure").with_source(source);
        assert!(err.source.is_some());
    }

    #[test]
    fn test_app_error_from_serde_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let app_err: AppError = json_err.into();
        assert_eq!(app_err.kind, ErrorKind::Validation);
    }
}
