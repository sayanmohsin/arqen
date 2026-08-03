//! Request validation module for Arqen.
//!
//! Provides typed extractors and validation for request data.

use async_trait::async_trait;
use axum::extract::FromRequest;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::core::error::{ErrorCode, ErrorResponse};

/// Validation error for a single field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldError {
    /// Field name.
    pub field: String,
    /// Error code (e.g., "required", "min_length").
    pub code: String,
    /// Human-readable error message.
    pub message: String,
}

impl FieldError {
    pub fn new(field: impl Into<String>, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
        }
    }
}

/// Collection of validation errors.
#[derive(Debug, Clone)]
pub struct ValidationErrors {
    pub errors: Vec<FieldError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn push(&mut self, error: FieldError) {
        self.errors.push(error);
    }

    /// Convert to error response.
    pub fn to_response(&self, correlation_id: &str) -> ErrorResponse {
        let details = serde_json::to_value(&self.errors).unwrap_or_default();
        ErrorResponse::with_details(
            ErrorCode::Validation,
            "Validation failed",
            correlation_id,
            details,
        )
    }
}

impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let messages: Vec<String> = self.errors.iter().map(|e| format!("{}: {}", e.field, e.message)).collect();
        write!(f, "Validation failed: {}", messages.join(", "))
    }
}

impl std::error::Error for ValidationErrors {}

/// Rejection type for validation errors.
#[derive(Debug)]
pub struct ValidationRejection(pub ValidationErrors);

impl From<ValidationErrors> for ValidationRejection {
    fn from(errors: ValidationErrors) -> Self {
        ValidationRejection(errors)
    }
}

impl IntoResponse for ValidationRejection {
    fn into_response(self) -> Response {
        let correlation_id = uuid::Uuid::new_v4().to_string();
        let response = self.0.to_response(&correlation_id);
        (StatusCode::BAD_REQUEST, axum::Json(response)).into_response()
    }
}

/// Trait for validating data.
pub trait Validate {
    fn validate(&self) -> Result<(), ValidationErrors>;
}

/// Axum extractor that validates request data.
pub struct Validated<T>(pub T);

impl<T> Validated<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[async_trait]
impl<T, S> FromRequest<S> for Validated<T>
where
    T: Validate + DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ValidationRejection;

    async fn from_request(
        req: axum::http::Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract the body using axum's Json extractor
        let body = axum::extract::Json::<T>::from_request(req, state)
            .await
            .map_err(|e| {
                ValidationRejection(ValidationErrors {
                    errors: vec![FieldError::new("_body", "invalid_json", e.to_string())],
                })
            })?;

        // Validate the extracted data
        body.0.validate()?;
        Ok(Validated(body.0))
    }
}

/// Validator functions for common patterns.
pub mod validators {
    use super::*;

    /// Check if a value is present (not None).
    pub fn required<T>(field: &str, value: &Option<T>) -> Result<(), FieldError> {
        if value.is_none() {
            Err(FieldError::new(field, "required", "field is required"))
        } else {
            Ok(())
        }
    }

    /// Check string minimum length.
    pub fn min_length(field: &str, value: &str, min: usize) -> Result<(), FieldError> {
        if value.len() < min {
            Err(FieldError::new(
                field,
                "min_length",
                format!("must be at least {} characters", min),
            ))
        } else {
            Ok(())
        }
    }

    /// Check string maximum length.
    pub fn max_length(field: &str, value: &str, max: usize) -> Result<(), FieldError> {
        if value.len() > max {
            Err(FieldError::new(
                field,
                "max_length",
                format!("must be at most {} characters", max),
            ))
        } else {
            Ok(())
        }
    }

    /// Check if string is a valid email.
    pub fn email(field: &str, value: &str) -> Result<(), FieldError> {
        if value.contains('@') && value.contains('.') {
            Ok(())
        } else {
            Err(FieldError::new(field, "email", "invalid email address"))
        }
    }

    /// Check if string is a valid URL.
    pub fn url(field: &str, value: &str) -> Result<(), FieldError> {
        if value.starts_with("http://") || value.starts_with("https://") {
            Ok(())
        } else {
            Err(FieldError::new(field, "url", "invalid URL"))
        }
    }

    /// Check numeric minimum value.
    pub fn min_value<T: PartialOrd + std::fmt::Debug>(field: &str, value: &T, min: &T) -> Result<(), FieldError> {
        if value < min {
            Err(FieldError::new(
                field,
                "min_value",
                format!("must be at least {:?}", min),
            ))
        } else {
            Ok(())
        }
    }

    /// Check numeric maximum value.
    pub fn max_value<T: PartialOrd + std::fmt::Debug>(field: &str, value: &T, max: &T) -> Result<(), FieldError> {
        if value > max {
            Err(FieldError::new(
                field,
                "max_value",
                format!("must be at most {:?}", max),
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct CreateUser {
        email: Option<String>,
        name: Option<String>,
        age: Option<u32>,
    }

    impl Validate for CreateUser {
        fn validate(&self) -> Result<(), ValidationErrors> {
            let mut errors = ValidationErrors::new();

            if let Err(e) = validators::required("email", &self.email) {
                errors.push(e);
            } else if let Some(ref email) = self.email {
                if let Err(e) = validators::email("email", email) {
                    errors.push(e);
                }
            }

            if let Some(ref name) = self.name {
                if let Err(e) = validators::min_length("name", name, 3) {
                    errors.push(e);
                }
                if let Err(e) = validators::max_length("name", name, 50) {
                    errors.push(e);
                }
            }

            if let Some(age) = self.age {
                if let Err(e) = validators::min_value("age", &age, &18) {
                    errors.push(e);
                }
            }

            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
    }

    #[test]
    fn test_field_error_new() {
        let err = FieldError::new("email", "required", "field is required");
        assert_eq!(err.field, "email");
        assert_eq!(err.code, "required");
        assert_eq!(err.message, "field is required");
    }

    #[test]
    fn test_validation_errors_new() {
        let errors = ValidationErrors::new();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validation_errors_push() {
        let mut errors = ValidationErrors::new();
        errors.push(FieldError::new("email", "required", "required"));
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_validation_errors_display() {
        let errors = ValidationErrors {
            errors: vec![
                FieldError::new("email", "required", "required"),
                FieldError::new("name", "min_length", "too short"),
            ],
        };
        let display = format!("{}", errors);
        assert!(display.contains("email: required"));
        assert!(display.contains("name: too short"));
    }

    #[test]
    fn test_validators_required() {
        assert!(validators::required("field", &Some("value")).is_ok());
        assert!(validators::required("field", &None::<String>).is_err());
    }

    #[test]
    fn test_validators_min_length() {
        assert!(validators::min_length("field", "hello", 3).is_ok());
        assert!(validators::min_length("field", "hi", 3).is_err());
    }

    #[test]
    fn test_validators_max_length() {
        assert!(validators::max_length("field", "hello", 10).is_ok());
        assert!(validators::max_length("field", "hello world!", 10).is_err());
    }

    #[test]
    fn test_validators_email() {
        assert!(validators::email("field", "user@example.com").is_ok());
        assert!(validators::email("field", "invalid").is_err());
    }

    #[test]
    fn test_validators_url() {
        assert!(validators::url("field", "https://example.com").is_ok());
        assert!(validators::url("field", "http://example.com").is_ok());
        assert!(validators::url("field", "invalid").is_err());
    }

    #[test]
    fn test_validators_min_value() {
        assert!(validators::min_value("field", &10, &5).is_ok());
        assert!(validators::min_value("field", &5, &10).is_err());
    }

    #[test]
    fn test_validators_max_value() {
        assert!(validators::max_value("field", &5, &10).is_ok());
        assert!(validators::max_value("field", &10, &5).is_err());
    }

    #[test]
    fn test_create_user_valid() {
        let user = CreateUser {
            email: Some("user@example.com".to_string()),
            name: Some("Alice".to_string()),
            age: Some(25),
        };
        assert!(user.validate().is_ok());
    }

    #[test]
    fn test_create_user_missing_email() {
        let user = CreateUser {
            email: None,
            name: Some("Alice".to_string()),
            age: Some(25),
        };
        let errors = user.validate().unwrap_err();
        assert_eq!(errors.errors.len(), 1);
        assert_eq!(errors.errors[0].field, "email");
    }

    #[test]
    fn test_create_user_invalid_email() {
        let user = CreateUser {
            email: Some("invalid".to_string()),
            name: Some("Alice".to_string()),
            age: Some(25),
        };
        let errors = user.validate().unwrap_err();
        assert_eq!(errors.errors.len(), 1);
        assert_eq!(errors.errors[0].code, "email");
    }

    #[test]
    fn test_create_user_short_name() {
        let user = CreateUser {
            email: Some("user@example.com".to_string()),
            name: Some("Al".to_string()),
            age: Some(25),
        };
        let errors = user.validate().unwrap_err();
        assert_eq!(errors.errors.len(), 1);
        assert_eq!(errors.errors[0].code, "min_length");
    }

    #[test]
    fn test_create_user_underage() {
        let user = CreateUser {
            email: Some("user@example.com".to_string()),
            name: Some("Alice".to_string()),
            age: Some(16),
        };
        let errors = user.validate().unwrap_err();
        assert_eq!(errors.errors.len(), 1);
        assert_eq!(errors.errors[0].code, "min_value");
    }

    #[test]
    fn test_create_user_multiple_errors() {
        let user = CreateUser {
            email: None,
            name: Some("Al".to_string()),
            age: Some(16),
        };
        let errors = user.validate().unwrap_err();
        assert_eq!(errors.errors.len(), 3);
    }

    #[test]
    fn test_validation_errors_to_response() {
        let errors = ValidationErrors {
            errors: vec![FieldError::new("email", "required", "required")],
        };
        let response = errors.to_response("req-123");
        assert_eq!(response.error.code, ErrorCode::Validation);
        assert_eq!(response.error.correlation_id, "req-123");
    }
}
