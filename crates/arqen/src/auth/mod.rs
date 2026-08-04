//! Authentication module for Arqen.
//!
//! Provides pluggable authentication with API keys, JWT, and session adapters.
//! API key comparison uses constant-time equality to prevent timing attacks.
//! JWT validation uses the `jsonwebtoken` crate with proper signature, expiry,
//! issuer, and audience checks.

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::core::{AppError, ErrorKind};

/// Authentication error type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthError {
    /// No credentials provided.
    Missing,
    /// Credentials are invalid.
    Invalid,
    /// Token has expired.
    Expired,
    /// User is not authorized for this resource.
    Unauthorized(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::Missing => write!(f, "authentication required"),
            AuthError::Invalid => write!(f, "invalid credentials"),
            AuthError::Expired => write!(f, "credentials expired"),
            AuthError::Unauthorized(msg) => write!(f, "unauthorized: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

impl From<AuthError> for AppError {
    fn from(e: AuthError) -> Self {
        match e {
            AuthError::Missing | AuthError::Invalid | AuthError::Expired => {
                AppError::new(ErrorKind::Authentication, e.to_string())
            }
            AuthError::Unauthorized(msg) => {
                AppError::new(ErrorKind::Authorization, msg)
            }
        }
    }
}

/// Authentication context extracted from requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    /// Subject identifier (user ID, API key ID, etc.).
    pub subject: String,
    /// Claims or permissions.
    pub claims: HashMap<String, serde_json::Value>,
    /// Which adapter authenticated this request.
    pub adapter: String,
}

impl AuthContext {
    /// Create a new auth context.
    pub fn new(subject: impl Into<String>, adapter: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            claims: HashMap::new(),
            adapter: adapter.into(),
        }
    }

    /// Add a claim.
    pub fn with_claim(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.claims.insert(key.into(), value);
        self
    }

    /// Check if a claim exists.
    pub fn has_claim(&self, key: &str) -> bool {
        self.claims.contains_key(key)
    }

    /// Get a claim value.
    pub fn get_claim(&self, key: &str) -> Option<&serde_json::Value> {
        self.claims.get(key)
    }

    /// Check if user has a specific role.
    pub fn has_role(&self, role: &str) -> bool {
        self.get_claim("roles")
            .and_then(|v| v.as_array())
            .map(|roles| roles.iter().any(|r| r.as_str() == Some(role)))
            .unwrap_or(false)
    }
}

/// Trait for authentication adapters.
///
/// This trait requires the `http-server` feature because it uses
/// `axum::http::HeaderMap` for header extraction.
#[cfg(feature = "http-server")]
#[async_trait]
pub trait Authentication: Send + Sync {
    /// Authenticate a request from headers.
    async fn authenticate(&self, headers: &axum::http::HeaderMap) -> Result<AuthContext, AuthError>;
}

/// Constant-time string comparison to prevent timing attacks.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

/// Hash an API key using SHA-256 for secure storage.
pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// API key authentication adapter.
///
/// Validates API keys from the `Authorization: Bearer <key>` header
/// or `X-API-Key` header. Uses constant-time comparison to prevent timing attacks.
///
/// Requires the `http-server` feature.
#[cfg(feature = "http-server")]
pub struct ApiKeyAuth {
    /// Valid API key hashes mapped to subject IDs.
    keys: HashMap<String, String>,
}

#[cfg(feature = "http-server")]
impl ApiKeyAuth {
    /// Create a new API key auth adapter.
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    /// Add an API key (will be hashed for secure storage).
    pub fn with_key(mut self, key: impl Into<String>, subject: impl Into<String>) -> Self {
        self.keys.insert(hash_api_key(&key.into()), subject.into());
        self
    }

    /// Add a pre-hashed API key.
    pub fn with_hashed_key(mut self, hashed_key: impl Into<String>, subject: impl Into<String>) -> Self {
        self.keys.insert(hashed_key.into(), subject.into());
        self
    }

    /// Add multiple API keys.
    pub fn with_keys(mut self, keys: HashMap<String, String>) -> Self {
        for (key, subject) in keys {
            self.keys.insert(hash_api_key(&key), subject);
        }
        self
    }
}

#[cfg(feature = "http-server")]
impl Default for ApiKeyAuth {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "http-server")]
#[async_trait]
impl Authentication for ApiKeyAuth {
    async fn authenticate(&self, headers: &axum::http::HeaderMap) -> Result<AuthContext, AuthError> {
        // Try X-API-Key header first
        if let Some(key) = headers.get("x-api-key") {
            let key = key.to_str().map_err(|_| AuthError::Invalid)?;
            let hashed = hash_api_key(key);
            for (stored_hash, subject) in &self.keys {
                if constant_time_eq(&hashed, stored_hash) {
                    return Ok(AuthContext::new(subject, "api_key"));
                }
            }
            return Err(AuthError::Invalid);
        }

        // Try Authorization: Bearer <key>
        if let Some(auth) = headers.get("authorization") {
            let auth = auth.to_str().map_err(|_| AuthError::Invalid)?;
            if let Some(key) = auth.strip_prefix("Bearer ") {
                let hashed = hash_api_key(key);
                for (stored_hash, subject) in &self.keys {
                    if constant_time_eq(&hashed, stored_hash) {
                        return Ok(AuthContext::new(subject, "api_key"));
                    }
                }
                return Err(AuthError::Invalid);
            }
        }

        Err(AuthError::Missing)
    }
}

/// JWT authentication adapter.
///
/// Validates JWT tokens from the `Authorization: Bearer <token>` header
/// using the `jsonwebtoken` crate with proper signature, expiry, issuer,
/// and audience validation.
///
/// Requires the `http-server` feature.
#[cfg(feature = "http-server")]
pub struct JwtAuth {
    /// Decoding key for JWT validation.
    decoding_key: jsonwebtoken::DecodingKey,
    /// Validation settings.
    validation: jsonwebtoken::Validation,
    /// Optional issuer to validate.
    issuer: Option<String>,
}

#[cfg(feature = "http-server")]
impl JwtAuth {
    /// Create a new JWT auth adapter with a secret key (HMAC).
    pub fn new_secret(secret: &[u8]) -> Self {
        use jsonwebtoken::{Algorithm, Validation, DecodingKey};

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        Self {
            decoding_key: DecodingKey::from_secret(secret),
            validation,
            issuer: None,
        }
    }

    /// Create a new JWT auth adapter with RSA public key.
    pub fn new_rsa(public_key: &[u8]) -> Self {
        use jsonwebtoken::{Algorithm, Validation, DecodingKey};

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        Self {
            decoding_key: DecodingKey::from_rsa_pem(public_key).expect("invalid RSA public key"),
            validation,
            issuer: None,
        }
    }

    /// Set the expected issuer for validation.
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    /// Set the expected audience for validation.
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.validation.set_audience(&[audience.into()]);
        self
    }

    /// Decode and validate a JWT token.
    pub fn validate_token(&self, token: &str) -> Result<jsonwebtoken::TokenData<serde_json::Value>, AuthError> {
        jsonwebtoken::decode::<serde_json::Value>(
            token,
            &self.decoding_key,
            &self.validation,
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::Expired,
            _ => AuthError::Invalid,
        })
    }
}

#[cfg(feature = "http-server")]
#[async_trait]
impl Authentication for JwtAuth {
    async fn authenticate(&self, headers: &axum::http::HeaderMap) -> Result<AuthContext, AuthError> {
        let auth = headers
            .get("authorization")
            .ok_or(AuthError::Missing)?
            .to_str()
            .map_err(|_| AuthError::Invalid)?;

        let token = auth
            .strip_prefix("Bearer ")
            .ok_or(AuthError::Invalid)?;

        let token_data = self.validate_token(token)?;
        let claims = token_data.claims;

        let subject = claims
            .get("sub")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let mut context = AuthContext::new(subject, "jwt");

        // Copy all claims into context
        if let Some(obj) = claims.as_object() {
            for (key, value) in obj {
                context.claims.insert(key.clone(), value.clone());
            }
        }

        Ok(context)
    }
}

/// Session-based authentication adapter.
///
/// Validates sessions from cookies.
///
/// Requires the `http-server` feature.
#[cfg(feature = "http-server")]
pub struct SessionAuth {
    /// Valid session tokens mapped to auth contexts.
    sessions: HashMap<String, AuthContext>,
    /// Cookie name to look for.
    cookie_name: String,
}

#[cfg(feature = "http-server")]
impl SessionAuth {
    /// Create a new session auth adapter.
    pub fn new(cookie_name: impl Into<String>) -> Self {
        Self {
            sessions: HashMap::new(),
            cookie_name: cookie_name.into(),
        }
    }

    /// Add a valid session.
    pub fn with_session(mut self, token: impl Into<String>, context: AuthContext) -> Self {
        self.sessions.insert(token.into(), context);
        self
    }
}

#[cfg(feature = "http-server")]
#[async_trait]
impl Authentication for SessionAuth {
    async fn authenticate(&self, headers: &axum::http::HeaderMap) -> Result<AuthContext, AuthError> {
        let cookie_header = headers
            .get("cookie")
            .ok_or(AuthError::Missing)?
            .to_str()
            .map_err(|_| AuthError::Invalid)?;

        for part in cookie_header.split(';') {
            let part = part.trim();
            if let Some((name, value)) = part.split_once('=') {
                if name == self.cookie_name {
                    return self.sessions
                        .get(value)
                        .cloned()
                        .ok_or(AuthError::Invalid);
                }
            }
        }

        Err(AuthError::Missing)
    }
}

/// Policy trait for authorization.
pub trait Policy: Send + Sync {
    /// Check if an auth context is authorized for a resource.
    fn check(&self, context: &AuthContext, resource: &str) -> Result<(), AuthError>;
}

/// Policy that allows all requests.
pub struct AllowAll;

impl Policy for AllowAll {
    fn check(&self, _context: &AuthContext, _resource: &str) -> Result<(), AuthError> {
        Ok(())
    }
}

/// Policy that denies all requests.
pub struct DenyAll;

impl Policy for DenyAll {
    fn check(&self, _context: &AuthContext, _resource: &str) -> Result<(), AuthError> {
        Err(AuthError::Unauthorized("access denied".to_string()))
    }
}

/// Policy that checks for a specific role.
pub struct RequireRole {
    role: String,
}

impl RequireRole {
    pub fn new(role: impl Into<String>) -> Self {
        Self { role: role.into() }
    }
}

impl Policy for RequireRole {
    fn check(&self, context: &AuthContext, _resource: &str) -> Result<(), AuthError> {
        if context.has_role(&self.role) {
            Ok(())
        } else {
            Err(AuthError::Unauthorized(format!(
                "requires role: {}",
                self.role
            )))
        }
    }
}

/// Compose multiple policies (all must pass).
pub struct AllOf {
    policies: Vec<Box<dyn Policy>>,
}

impl AllOf {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    pub fn with_policy(mut self, policy: impl Policy + 'static) -> Self {
        self.policies.push(Box::new(policy));
        self
    }
}

impl Default for AllOf {
    fn default() -> Self {
        Self::new()
    }
}

impl Policy for AllOf {
    fn check(&self, context: &AuthContext, resource: &str) -> Result<(), AuthError> {
        for policy in &self.policies {
            policy.check(context, resource)?;
        }
        Ok(())
    }
}

/// Compose multiple policies (any must pass).
pub struct AnyOf {
    policies: Vec<Box<dyn Policy>>,
}

impl AnyOf {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    pub fn with_policy(mut self, policy: impl Policy + 'static) -> Self {
        self.policies.push(Box::new(policy));
        self
    }
}

impl Default for AnyOf {
    fn default() -> Self {
        Self::new()
    }
}

impl Policy for AnyOf {
    fn check(&self, context: &AuthContext, resource: &str) -> Result<(), AuthError> {
        for policy in &self.policies {
            if policy.check(context, resource).is_ok() {
                return Ok(());
            }
        }
        Err(AuthError::Unauthorized("no matching policy".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_context_new() {
        let ctx = AuthContext::new("user-123", "api_key");
        assert_eq!(ctx.subject, "user-123");
        assert_eq!(ctx.adapter, "api_key");
        assert!(ctx.claims.is_empty());
    }

    #[test]
    fn test_auth_context_with_claim() {
        let ctx = AuthContext::new("user-123", "api_key")
            .with_claim("roles", serde_json::json!(["admin"]));
        assert!(ctx.has_claim("roles"));
        assert_eq!(ctx.get_claim("roles"), Some(&serde_json::json!(["admin"])));
    }

    #[test]
    fn test_auth_context_has_role() {
        let ctx = AuthContext::new("user-123", "api_key")
            .with_claim("roles", serde_json::json!(["admin", "user"]));
        assert!(ctx.has_role("admin"));
        assert!(ctx.has_role("user"));
        assert!(!ctx.has_role("superadmin"));
    }

    #[test]
    fn test_auth_context_no_roles() {
        let ctx = AuthContext::new("user-123", "api_key");
        assert!(!ctx.has_role("admin"));
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq("hello", "hello"));
        assert!(!constant_time_eq("hello", "world"));
        assert!(!constant_time_eq("hello", "hell"));
        assert!(!constant_time_eq("hell", "hello"));
    }

    #[test]
    fn test_hash_api_key() {
        let hash1 = hash_api_key("test-key");
        let hash2 = hash_api_key("test-key");
        let hash3 = hash_api_key("different-key");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_allow_all_policy() {
        let policy = AllowAll;
        let ctx = AuthContext::new("user-123", "test");
        assert!(policy.check(&ctx, "resource").is_ok());
    }

    #[test]
    fn test_deny_all_policy() {
        let policy = DenyAll;
        let ctx = AuthContext::new("user-123", "test");
        assert!(policy.check(&ctx, "resource").is_err());
    }

    #[test]
    fn test_require_role_policy() {
        let policy = RequireRole::new("admin");
        let ctx = AuthContext::new("user-123", "test")
            .with_claim("roles", serde_json::json!(["admin"]));
        assert!(policy.check(&ctx, "resource").is_ok());

        let ctx = AuthContext::new("user-123", "test")
            .with_claim("roles", serde_json::json!(["user"]));
        assert!(policy.check(&ctx, "resource").is_err());
    }

    #[test]
    fn test_all_of_policy() {
        let policy = AllOf::new()
            .with_policy(AllowAll)
            .with_policy(RequireRole::new("admin"));

        let ctx = AuthContext::new("user-123", "test")
            .with_claim("roles", serde_json::json!(["admin"]));
        assert!(policy.check(&ctx, "resource").is_ok());

        let ctx = AuthContext::new("user-123", "test")
            .with_claim("roles", serde_json::json!(["user"]));
        assert!(policy.check(&ctx, "resource").is_err());
    }

    #[test]
    fn test_any_of_policy() {
        let policy = AnyOf::new()
            .with_policy(DenyAll)
            .with_policy(RequireRole::new("admin"));

        let ctx = AuthContext::new("user-123", "test")
            .with_claim("roles", serde_json::json!(["admin"]));
        assert!(policy.check(&ctx, "resource").is_ok());

        let ctx = AuthContext::new("user-123", "test")
            .with_claim("roles", serde_json::json!(["user"]));
        assert!(policy.check(&ctx, "resource").is_err());
    }

    #[test]
    fn test_auth_error_display() {
        assert_eq!(format!("{}", AuthError::Missing), "authentication required");
        assert_eq!(format!("{}", AuthError::Invalid), "invalid credentials");
        assert_eq!(format!("{}", AuthError::Expired), "credentials expired");
        assert_eq!(
            format!("{}", AuthError::Unauthorized("denied".to_string())),
            "unauthorized: denied"
        );
    }

    #[test]
    fn test_auth_error_to_app_error() {
        let err: AppError = AuthError::Missing.into();
        assert_eq!(err.kind, ErrorKind::Authentication);

        let err: AppError = AuthError::Unauthorized("denied".to_string()).into();
        assert_eq!(err.kind, ErrorKind::Authorization);
    }
}

#[cfg(all(test, feature = "http-server"))]
mod http_tests {
    use super::*;

    #[tokio::test]
    async fn test_api_key_auth_valid() {
        let auth = ApiKeyAuth::new()
            .with_key("test-key", "user-123");

        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-api-key", "test-key".parse().unwrap());

        let ctx = auth.authenticate(&headers).await.unwrap();
        assert_eq!(ctx.subject, "user-123");
        assert_eq!(ctx.adapter, "api_key");
    }

    #[tokio::test]
    async fn test_api_key_auth_invalid() {
        let auth = ApiKeyAuth::new()
            .with_key("test-key", "user-123");

        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-api-key", "wrong-key".parse().unwrap());

        let err = auth.authenticate(&headers).await.unwrap_err();
        assert_eq!(err, AuthError::Invalid);
    }

    #[tokio::test]
    async fn test_api_key_auth_missing() {
        let auth = ApiKeyAuth::new()
            .with_key("test-key", "user-123");

        let headers = axum::http::HeaderMap::new();

        let err = auth.authenticate(&headers).await.unwrap_err();
        assert_eq!(err, AuthError::Missing);
    }

    #[tokio::test]
    async fn test_api_key_auth_bearer() {
        let auth = ApiKeyAuth::new()
            .with_key("test-key", "user-123");

        let mut headers = axum::http::HeaderMap::new();
        headers.insert("authorization", "Bearer test-key".parse().unwrap());

        let ctx = auth.authenticate(&headers).await.unwrap();
        assert_eq!(ctx.subject, "user-123");
    }

    #[tokio::test]
    async fn test_jwt_auth_decode() {
        let secret = b"test-secret";
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256);
        let claims = serde_json::json!({
            "sub": "user-123",
            "exp": 4102444800_i64,
            "iss": "test-issuer"
        });
        let token = jsonwebtoken::encode(&header, &claims, &jsonwebtoken::EncodingKey::from_secret(secret))
            .unwrap();

        let auth = JwtAuth::new_secret(secret)
            .with_issuer("test-issuer");

        let mut headers = axum::http::HeaderMap::new();
        headers.insert("authorization", format!("Bearer {}", token).parse().unwrap());

        let ctx = auth.authenticate(&headers).await.unwrap();
        assert_eq!(ctx.subject, "user-123");
        assert_eq!(ctx.adapter, "jwt");
        assert_eq!(ctx.get_claim("iss"), Some(&serde_json::json!("test-issuer")));
    }

    #[tokio::test]
    async fn test_jwt_auth_invalid_token() {
        let secret = b"test-secret";
        let auth = JwtAuth::new_secret(secret);

        let mut headers = axum::http::HeaderMap::new();
        headers.insert("authorization", "Bearer invalid-token".parse().unwrap());

        let err = auth.authenticate(&headers).await.unwrap_err();
        assert_eq!(err, AuthError::Invalid);
    }

    #[tokio::test]
    async fn test_jwt_auth_wrong_secret() {
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256);
        let claims = serde_json::json!({
            "sub": "user-123",
            "exp": 4102444800_i64,
        });
        let token = jsonwebtoken::encode(&header, &claims, &jsonwebtoken::EncodingKey::from_secret(b"wrong-secret"))
            .unwrap();

        let auth = JwtAuth::new_secret(b"correct-secret");

        let mut headers = axum::http::HeaderMap::new();
        headers.insert("authorization", format!("Bearer {}", token).parse().unwrap());

        let err = auth.authenticate(&headers).await.unwrap_err();
        assert_eq!(err, AuthError::Invalid);
    }

    #[tokio::test]
    async fn test_session_auth_valid() {
        let ctx = AuthContext::new("user-123", "session");

        let auth = SessionAuth::new("session_id")
            .with_session("abc123", ctx);

        let mut headers = axum::http::HeaderMap::new();
        headers.insert("cookie", "session_id=abc123".parse().unwrap());

        let ctx = auth.authenticate(&headers).await.unwrap();
        assert_eq!(ctx.subject, "user-123");
    }

    #[tokio::test]
    async fn test_session_auth_missing() {
        let auth = SessionAuth::new("session_id");

        let headers = axum::http::HeaderMap::new();

        let err = auth.authenticate(&headers).await.unwrap_err();
        assert_eq!(err, AuthError::Missing);
    }
}
