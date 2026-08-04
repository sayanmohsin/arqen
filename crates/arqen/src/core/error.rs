use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "http-server")]
use axum::http::StatusCode;
#[cfg(feature = "http-server")]
use axum::response::{IntoResponse, Response};

/// Stable error codes for API responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    NotFound,
    Validation,
    Authentication,
    Authorization,
    Conflict,
    RateLimited,
    Timeout,
    Dependency,
    Internal,
    External,
    Unavailable,
}

impl ErrorCode {
    /// Convert to HTTP status code.
    ///
    /// Requires the `http-server` feature.
    #[cfg(feature = "http-server")]
    pub fn status_code(&self) -> StatusCode {
        match self {
            ErrorCode::NotFound => StatusCode::NOT_FOUND,
            ErrorCode::Validation => StatusCode::BAD_REQUEST,
            ErrorCode::Authentication => StatusCode::UNAUTHORIZED,
            ErrorCode::Authorization => StatusCode::FORBIDDEN,
            ErrorCode::Conflict => StatusCode::CONFLICT,
            ErrorCode::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            ErrorCode::Timeout => StatusCode::GATEWAY_TIMEOUT,
            ErrorCode::Dependency => StatusCode::BAD_GATEWAY,
            ErrorCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::External => StatusCode::BAD_GATEWAY,
            ErrorCode::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::NotFound => write!(f, "not_found"),
            ErrorCode::Validation => write!(f, "validation"),
            ErrorCode::Authentication => write!(f, "authentication"),
            ErrorCode::Authorization => write!(f, "authorization"),
            ErrorCode::Conflict => write!(f, "conflict"),
            ErrorCode::RateLimited => write!(f, "rate_limited"),
            ErrorCode::Timeout => write!(f, "timeout"),
            ErrorCode::Dependency => write!(f, "dependency"),
            ErrorCode::Internal => write!(f, "internal"),
            ErrorCode::External => write!(f, "external"),
            ErrorCode::Unavailable => write!(f, "unavailable"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorrelationId(pub String);

impl CorrelationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub correlation_id: CorrelationId,
    pub path: String,
    pub method: String,
}

impl ErrorContext {
    pub fn new(
        correlation_id: CorrelationId,
        path: impl Into<String>,
        method: impl Into<String>,
    ) -> Self {
        Self {
            correlation_id,
            path: path.into(),
            method: method.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    pub correlation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

impl ErrorResponse {
    pub fn new(
        code: ErrorCode,
        message: impl Into<String>,
        correlation_id: impl Into<String>,
    ) -> Self {
        Self {
            error: ErrorBody {
                code,
                message: message.into(),
                correlation_id: correlation_id.into(),
                details: None,
            },
        }
    }

    pub fn with_details(
        code: ErrorCode,
        message: impl Into<String>,
        correlation_id: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            error: ErrorBody {
                code,
                message: message.into(),
                correlation_id: correlation_id.into(),
                details: Some(details),
            },
        }
    }

    pub fn redacted(correlation_id: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::Internal,
            "An internal error occurred",
            correlation_id,
        )
    }
}

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
    #[error("conflict")]
    Conflict,
    #[error("rate limited")]
    RateLimited,
    #[error("timeout")]
    Timeout,
    #[error("dependency error")]
    Dependency,
    #[error("internal error")]
    Internal,
    #[error("external error")]
    External,
    #[error("unavailable")]
    Unavailable,
}

impl ErrorKind {
    pub fn to_code(&self) -> ErrorCode {
        match self {
            ErrorKind::NotFound => ErrorCode::NotFound,
            ErrorKind::Validation => ErrorCode::Validation,
            ErrorKind::Authentication => ErrorCode::Authentication,
            ErrorKind::Authorization => ErrorCode::Authorization,
            ErrorKind::Conflict => ErrorCode::Conflict,
            ErrorKind::RateLimited => ErrorCode::RateLimited,
            ErrorKind::Timeout => ErrorCode::Timeout,
            ErrorKind::Dependency => ErrorCode::Dependency,
            ErrorKind::Internal => ErrorCode::Internal,
            ErrorKind::External => ErrorCode::External,
            ErrorKind::Unavailable => ErrorCode::Unavailable,
        }
    }

    /// Check if this error kind should be redacted in responses.
    pub fn should_redact(&self) -> bool {
        matches!(self, ErrorKind::Internal)
    }
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

    pub fn to_response(&self, correlation_id: &CorrelationId) -> ErrorResponse {
        ErrorResponse::new(self.kind.to_code(), &self.message, correlation_id.0.clone())
    }

    pub fn to_redacted_response(&self, correlation_id: &CorrelationId) -> ErrorResponse {
        if self.kind.should_redact() {
            ErrorResponse::redacted(correlation_id.0.clone())
        } else {
            self.to_response(correlation_id)
        }
    }

    pub fn is_internal(&self) -> bool {
        self.kind == ErrorKind::Internal
    }
}

#[cfg(feature = "http-server")]
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let correlation_id = CorrelationId::new();
        let status = self.kind.to_code().status_code();
        let response = self.to_redacted_response(&correlation_id);
        let body = axum::Json(response);
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

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => {
                AppError::new(ErrorKind::NotFound, format!("file not found: {e}"))
            }
            std::io::ErrorKind::PermissionDenied => {
                AppError::new(ErrorKind::Authorization, format!("permission denied: {e}"))
            }
            _ => AppError::new(ErrorKind::Internal, format!("IO error: {e}")),
        }
    }
}

impl From<std::net::AddrParseError> for AppError {
    fn from(e: std::net::AddrParseError) -> Self {
        AppError::new(ErrorKind::Validation, format!("invalid address: {e}"))
    }
}

impl From<toml::de::Error> for AppError {
    fn from(e: toml::de::Error) -> Self {
        AppError::new(ErrorKind::Validation, format!("invalid TOML: {e}"))
    }
}

impl From<std::env::VarError> for AppError {
    fn from(e: std::env::VarError) -> Self {
        AppError::new(
            ErrorKind::Internal,
            format!("environment variable error: {e}"),
        )
    }
}

#[cfg(feature = "http-client")]
impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            AppError::new(ErrorKind::Timeout, format!("request timed out: {e}"))
        } else if e.is_connect() {
            AppError::new(ErrorKind::Dependency, format!("connection failed: {e}"))
        } else {
            AppError::new(ErrorKind::External, format!("HTTP client error: {e}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(format!("{}", ErrorCode::NotFound), "not_found");
        assert_eq!(format!("{}", ErrorCode::Validation), "validation");
        assert_eq!(format!("{}", ErrorCode::Timeout), "timeout");
        assert_eq!(format!("{}", ErrorCode::Dependency), "dependency");
        assert_eq!(format!("{}", ErrorCode::Internal), "internal");
    }

    #[cfg(feature = "http-server")]
    #[test]
    fn test_error_code_status_codes() {
        assert_eq!(ErrorCode::NotFound.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(ErrorCode::Validation.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(
            ErrorCode::Authentication.status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ErrorCode::Authorization.status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(ErrorCode::Conflict.status_code(), StatusCode::CONFLICT);
        assert_eq!(
            ErrorCode::RateLimited.status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            ErrorCode::Timeout.status_code(),
            StatusCode::GATEWAY_TIMEOUT
        );
        assert_eq!(ErrorCode::Dependency.status_code(), StatusCode::BAD_GATEWAY);
        assert_eq!(
            ErrorCode::Internal.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(ErrorCode::External.status_code(), StatusCode::BAD_GATEWAY);
        assert_eq!(
            ErrorCode::Unavailable.status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[test]
    fn test_correlation_id_new() {
        let id = CorrelationId::new();
        assert!(!id.0.is_empty());
    }

    #[test]
    fn test_correlation_id_default() {
        let id = CorrelationId::default();
        assert!(!id.0.is_empty());
    }

    #[test]
    fn test_error_response_new() {
        let response = ErrorResponse::new(ErrorCode::NotFound, "not found", "req-123");
        assert_eq!(response.error.code, ErrorCode::NotFound);
        assert_eq!(response.error.message, "not found");
        assert_eq!(response.error.correlation_id, "req-123");
        assert!(response.error.details.is_none());
    }

    #[test]
    fn test_error_response_with_details() {
        let details = serde_json::json!({"field": "email"});
        let response = ErrorResponse::with_details(
            ErrorCode::Validation,
            "invalid email",
            "req-123",
            details.clone(),
        );
        assert_eq!(response.error.details, Some(details));
    }

    #[test]
    fn test_error_response_redacted() {
        let response = ErrorResponse::redacted("req-123");
        assert_eq!(response.error.code, ErrorCode::Internal);
        assert_eq!(response.error.message, "An internal error occurred");
    }

    #[test]
    fn test_error_kind_to_code() {
        assert_eq!(ErrorKind::NotFound.to_code(), ErrorCode::NotFound);
        assert_eq!(ErrorKind::Validation.to_code(), ErrorCode::Validation);
        assert_eq!(ErrorKind::Timeout.to_code(), ErrorCode::Timeout);
        assert_eq!(ErrorKind::Dependency.to_code(), ErrorCode::Dependency);
        assert_eq!(ErrorKind::Internal.to_code(), ErrorCode::Internal);
    }

    #[test]
    fn test_error_kind_should_redact() {
        assert!(ErrorKind::Internal.should_redact());
        assert!(!ErrorKind::NotFound.should_redact());
        assert!(!ErrorKind::Validation.should_redact());
        assert!(!ErrorKind::Timeout.should_redact());
        assert!(!ErrorKind::Dependency.should_redact());
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

    #[test]
    fn test_app_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let app_err: AppError = io_err.into();
        assert_eq!(app_err.kind, ErrorKind::NotFound);
    }

    #[test]
    fn test_app_error_from_io_permission() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let app_err: AppError = io_err.into();
        assert_eq!(app_err.kind, ErrorKind::Authorization);
    }

    #[test]
    fn test_app_error_from_addr_parse() {
        let addr_err = "not-an-address"
            .parse::<std::net::SocketAddr>()
            .unwrap_err();
        let app_err: AppError = addr_err.into();
        assert_eq!(app_err.kind, ErrorKind::Validation);
    }

    #[test]
    fn test_app_error_to_response() {
        let err = AppError::new(ErrorKind::NotFound, "not found");
        let correlation_id = CorrelationId::new();
        let response = err.to_response(&correlation_id);
        assert_eq!(response.error.code, ErrorCode::NotFound);
        assert_eq!(response.error.message, "not found");
        assert_eq!(response.error.correlation_id, correlation_id.0);
    }

    #[test]
    fn test_app_error_to_redacted_response_internal() {
        let err = AppError::new(ErrorKind::Internal, "database connection failed");
        let correlation_id = CorrelationId::new();
        let response = err.to_redacted_response(&correlation_id);
        assert_eq!(response.error.code, ErrorCode::Internal);
        assert_eq!(response.error.message, "An internal error occurred");
    }

    #[test]
    fn test_app_error_to_redacted_response_non_internal() {
        let err = AppError::new(ErrorKind::NotFound, "user not found");
        let correlation_id = CorrelationId::new();
        let response = err.to_redacted_response(&correlation_id);
        assert_eq!(response.error.code, ErrorCode::NotFound);
        assert_eq!(response.error.message, "user not found");
    }

    #[test]
    fn test_app_error_is_internal() {
        let err = AppError::new(ErrorKind::Internal, "internal error");
        assert!(err.is_internal());

        let err = AppError::new(ErrorKind::NotFound, "not found");
        assert!(!err.is_internal());
    }

    #[test]
    fn test_error_response_json_format() {
        let response = ErrorResponse::new(ErrorCode::NotFound, "not found", "req-123");
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("not_found"));
        assert!(json.contains("not found"));
        assert!(json.contains("req-123"));
    }

    #[test]
    fn test_error_response_json_no_details_when_none() {
        let response = ErrorResponse::new(ErrorCode::NotFound, "not found", "req-123");
        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("details"));
    }

    #[test]
    fn test_error_response_json_with_details() {
        let response = ErrorResponse::with_details(
            ErrorCode::Validation,
            "invalid",
            "req-123",
            serde_json::json!({"field": "email"}),
        );
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("details"));
        assert!(json.contains("email"));
    }

    #[cfg(feature = "http-server")]
    #[test]
    fn test_timeout_error_mapping() {
        let err = AppError::new(ErrorKind::Timeout, "request timed out");
        let correlation_id = CorrelationId::new();
        let response = err.to_response(&correlation_id);
        assert_eq!(response.error.code, ErrorCode::Timeout);
        assert_eq!(
            response.error.code.status_code(),
            StatusCode::GATEWAY_TIMEOUT
        );
    }

    #[cfg(feature = "http-server")]
    #[test]
    fn test_dependency_error_mapping() {
        let err = AppError::new(ErrorKind::Dependency, "thingd unavailable");
        let correlation_id = CorrelationId::new();
        let response = err.to_response(&correlation_id);
        assert_eq!(response.error.code, ErrorCode::Dependency);
        assert_eq!(response.error.code.status_code(), StatusCode::BAD_GATEWAY);
    }
}
