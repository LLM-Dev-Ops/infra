//! Identity types.

use chrono::{DateTime, Utc};
use infra_crypto::jwt::{Claims, JwtSigner};
use infra_errors::{AuthErrorKind, InfraError, InfraResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User or service identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    /// Unique identifier
    pub id: String,
    /// Identity type
    pub identity_type: IdentityType,
    /// Display name
    pub name: Option<String>,
    /// Email (for users)
    pub email: Option<String>,
    /// Roles
    pub roles: Vec<String>,
    /// Additional attributes
    pub attributes: HashMap<String, serde_json::Value>,
}

impl Identity {
    /// Create a new user identity
    pub fn user(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            identity_type: IdentityType::User,
            name: None,
            email: None,
            roles: Vec::new(),
            attributes: HashMap::new(),
        }
    }

    /// Create a new service identity
    pub fn service(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            identity_type: IdentityType::Service,
            name: None,
            email: None,
            roles: Vec::new(),
            attributes: HashMap::new(),
        }
    }

    /// Create an anonymous identity
    pub fn anonymous() -> Self {
        Self {
            id: "anonymous".to_string(),
            identity_type: IdentityType::Anonymous,
            name: None,
            email: None,
            roles: Vec::new(),
            attributes: HashMap::new(),
        }
    }

    /// Set name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set email
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Add a role
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    /// Add roles
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles.extend(roles);
        self
    }

    /// Add an attribute
    pub fn with_attribute(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Check if the identity has a role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if the identity is anonymous
    pub fn is_anonymous(&self) -> bool {
        matches!(self.identity_type, IdentityType::Anonymous)
    }
}

/// Identity type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityType {
    /// Human user
    User,
    /// Service account
    Service,
    /// Anonymous
    Anonymous,
}

/// Identity provider trait
pub trait IdentityProvider: Send + Sync {
    /// Verify a token and return the identity
    fn verify(&self, token: &str) -> InfraResult<Identity>;
}

/// Token-based identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenIdentity {
    /// The underlying identity
    pub identity: Identity,
    /// Token expiration
    pub expires_at: DateTime<Utc>,
    /// Token ID (jti)
    pub token_id: Option<String>,
}

impl TokenIdentity {
    /// Create from a JWT token
    pub fn from_token(token: &str, secret: &[u8]) -> InfraResult<Self> {
        let signer = JwtSigner::hs256(secret);
        let claims: Claims<TokenPayload> = signer.verify(token)?;

        let identity = Identity {
            id: claims.sub.unwrap_or_default(),
            identity_type: claims.payload.identity_type.unwrap_or(IdentityType::User),
            name: claims.payload.name,
            email: claims.payload.email,
            roles: claims.payload.roles.unwrap_or_default(),
            attributes: claims.payload.attributes.unwrap_or_default(),
        };

        Ok(Self {
            identity,
            expires_at: DateTime::from_timestamp(claims.exp, 0)
                .unwrap_or_else(|| Utc::now()),
            token_id: claims.jti,
        })
    }

    /// Create a JWT token
    pub fn to_token(&self, secret: &[u8], expiry: chrono::Duration) -> InfraResult<String> {
        let signer = JwtSigner::hs256(secret);

        let payload = TokenPayload {
            identity_type: Some(self.identity.identity_type),
            name: self.identity.name.clone(),
            email: self.identity.email.clone(),
            roles: Some(self.identity.roles.clone()),
            attributes: Some(self.identity.attributes.clone()),
        };

        let claims = Claims::with_payload(payload, expiry)
            .with_subject(&self.identity.id);

        signer.sign(&claims)
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TokenPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    identity_type: Option<IdentityType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    roles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attributes: Option<HashMap<String, serde_json::Value>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let identity = Identity::user("user123")
            .with_name("John Doe")
            .with_email("john@example.com")
            .with_role("admin")
            .with_role("user");

        assert_eq!(identity.id, "user123");
        assert!(identity.has_role("admin"));
        assert!(!identity.has_role("guest"));
    }

    #[test]
    fn test_token_roundtrip() {
        let identity = Identity::user("user123")
            .with_name("John")
            .with_role("user");

        let token_identity = TokenIdentity {
            identity,
            expires_at: Utc::now() + chrono::Duration::hours(1),
            token_id: None,
        };

        let secret = b"super_secret_key_at_least_32_bytes!";
        let token = token_identity
            .to_token(secret, chrono::Duration::hours(1))
            .unwrap();

        let decoded = TokenIdentity::from_token(&token, secret).unwrap();
        assert_eq!(decoded.identity.id, "user123");
    }
}
