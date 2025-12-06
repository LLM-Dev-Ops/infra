//! Authentication middleware.

use crate::identity::Identity;
use infra_errors::{AuthErrorKind, InfraError, InfraResult};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Authentication error
#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthError {
    #[error("Missing authentication")]
    Missing,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Authentication failed: {0}")]
    Other(String),
}

impl From<AuthError> for InfraError {
    fn from(err: AuthError) -> Self {
        let kind = match &err {
            AuthError::Missing => AuthErrorKind::MissingCredentials,
            AuthError::InvalidToken => AuthErrorKind::InvalidToken,
            AuthError::TokenExpired => AuthErrorKind::TokenExpired,
            AuthError::InsufficientPermissions => AuthErrorKind::InsufficientPermissions,
            AuthError::Other(_) => AuthErrorKind::InvalidToken,
        };

        InfraError::Auth {
            kind,
            message: err.to_string(),
            identity: None,
            context: None,
        }
    }
}

/// Authentication context
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Current identity
    identity: Option<Identity>,
    /// Token (if authenticated via token)
    token: Option<String>,
}

impl AuthContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            identity: None,
            token: None,
        }
    }

    /// Create with an identity
    pub fn with_identity(identity: Identity) -> Self {
        Self {
            identity: Some(identity),
            token: None,
        }
    }

    /// Create with a token
    pub fn with_token(token: String, identity: Identity) -> Self {
        Self {
            identity: Some(identity),
            token: Some(token),
        }
    }

    /// Get the identity
    pub fn identity(&self) -> Option<&Identity> {
        self.identity.as_ref()
    }

    /// Get the identity, returning error if not authenticated
    pub fn require_identity(&self) -> InfraResult<&Identity> {
        self.identity.as_ref().ok_or_else(|| AuthError::Missing.into())
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.identity.is_some()
    }

    /// Check if the user has a role
    pub fn has_role(&self, role: &str) -> bool {
        self.identity
            .as_ref()
            .map(|i| i.has_role(role))
            .unwrap_or(false)
    }

    /// Get the token
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Request-scoped authentication context
pub struct RequestAuthContext {
    inner: Arc<RwLock<AuthContext>>,
}

impl RequestAuthContext {
    /// Create a new context
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(AuthContext::new())),
        }
    }

    /// Set the identity
    pub async fn set_identity(&self, identity: Identity) {
        let mut ctx = self.inner.write().await;
        ctx.identity = Some(identity);
    }

    /// Get the identity
    pub async fn identity(&self) -> Option<Identity> {
        self.inner.read().await.identity.clone()
    }

    /// Get the context
    pub async fn context(&self) -> AuthContext {
        self.inner.read().await.clone()
    }
}

impl Default for RequestAuthContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RequestAuthContext {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_context() {
        let identity = Identity::user("user123").with_role("admin");
        let ctx = AuthContext::with_identity(identity);

        assert!(ctx.is_authenticated());
        assert!(ctx.has_role("admin"));
        assert!(!ctx.has_role("guest"));
    }

    #[tokio::test]
    async fn test_request_auth_context() {
        let ctx = RequestAuthContext::new();
        let identity = Identity::user("user123");

        ctx.set_identity(identity).await;

        let retrieved = ctx.identity().await.unwrap();
        assert_eq!(retrieved.id, "user123");
    }
}
