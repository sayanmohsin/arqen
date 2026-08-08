//! Typed request identity shared by handlers, repositories, jobs, and logs.

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::{Extensions, request::Parts};

use crate::auth::AuthContext;
use crate::core::error::CorrelationId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestContext {
    pub subject: Option<String>,
    pub tenant_id: Option<String>,
    pub instance_id: Option<String>,
    pub scopes: Vec<String>,
    pub roles: Vec<String>,
    pub correlation_id: String,
}

impl RequestContext {
    pub fn anonymous(correlation_id: impl Into<String>) -> Self {
        Self {
            subject: None,
            tenant_id: None,
            instance_id: None,
            scopes: Vec::new(),
            roles: Vec::new(),
            correlation_id: correlation_id.into(),
        }
    }

    pub fn from_auth(auth: &AuthContext, correlation_id: impl Into<String>) -> Self {
        Self {
            subject: Some(auth.subject.clone()),
            tenant_id: claim_string(auth, &["tenant_id", "tenant"]),
            instance_id: claim_string(auth, &["instance_id", "instance"]),
            scopes: claim_strings(auth, "scopes"),
            roles: claim_strings(auth, "roles"),
            correlation_id: correlation_id.into(),
        }
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|value| value == scope)
    }
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|value| value == role)
    }
}

fn claim_string(auth: &AuthContext, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        auth.claims
            .get(*key)
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned)
    })
}

fn claim_strings(auth: &AuthContext, key: &str) -> Vec<String> {
    match auth.claims.get(key) {
        Some(value) if value.is_string() => value
            .as_str()
            .map(|v| v.split_whitespace().map(ToOwned::to_owned).collect())
            .unwrap_or_default(),
        Some(value) => value
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                    .collect()
            })
            .unwrap_or_default(),
        None => Vec::new(),
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = crate::http::middleware_auth::AuthRejection;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<RequestContext>()
            .cloned()
            .ok_or(crate::http::middleware_auth::AuthRejection::Missing)
    }
}

pub(crate) fn from_extensions(extensions: &Extensions) -> RequestContext {
    let correlation_id = extensions
        .get::<CorrelationId>()
        .cloned()
        .unwrap_or_default();
    extensions
        .get::<AuthContext>()
        .map(|auth| RequestContext::from_auth(auth, correlation_id.0.clone()))
        .unwrap_or_else(|| RequestContext::anonymous(correlation_id.0))
}
