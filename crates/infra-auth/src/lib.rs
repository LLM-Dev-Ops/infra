//! Authentication and authorization for LLM-Dev-Ops infrastructure.
//!
//! This crate provides authentication (identity verification) and
//! authorization (permission checking) utilities.

mod identity;
mod session;
mod permission;
mod policy;
mod middleware;

pub use identity::{Identity, IdentityProvider, TokenIdentity};
pub use session::{Session, SessionStore, MemorySessionStore};
pub use permission::{Permission, PermissionSet, Action, Resource};
pub use policy::{Policy, PolicyEngine, PolicyDecision, Effect};
pub use middleware::{AuthContext, AuthError};

#[cfg(feature = "axum")]
pub mod axum_integration;

use infra_errors::InfraResult;

/// Verify a bearer token
pub fn verify_bearer_token(token: &str, secret: &[u8]) -> InfraResult<TokenIdentity> {
    TokenIdentity::from_token(token, secret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_set() {
        let mut perms = PermissionSet::new();
        perms.grant(Permission::new(Resource::new("users"), Action::Read));
        perms.grant(Permission::new(Resource::new("users"), Action::Write));

        assert!(perms.has(&Permission::new(Resource::new("users"), Action::Read)));
        assert!(!perms.has(&Permission::new(Resource::new("posts"), Action::Read)));
    }
}
