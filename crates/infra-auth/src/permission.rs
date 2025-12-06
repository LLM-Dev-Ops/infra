//! Permission types.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A resource that can be accessed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resource {
    /// Resource type
    pub resource_type: String,
    /// Resource ID (optional, None means all resources of this type)
    pub id: Option<String>,
}

impl Resource {
    /// Create a new resource
    pub fn new(resource_type: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            id: None,
        }
    }

    /// Create a resource with a specific ID
    pub fn with_id(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            id: Some(id.into()),
        }
    }

    /// Check if this resource matches another (considering wildcards)
    pub fn matches(&self, other: &Resource) -> bool {
        if self.resource_type != other.resource_type {
            return false;
        }

        match (&self.id, &other.id) {
            (None, _) => true, // Wildcard matches everything
            (Some(a), Some(b)) => a == b,
            (Some(_), None) => false, // Specific doesn't match wildcard
        }
    }
}

/// An action that can be performed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Read,
    Write,
    Create,
    Delete,
    Execute,
    Admin,
    All,
}

impl Action {
    /// Check if this action matches another (considering All as wildcard)
    pub fn matches(&self, other: &Action) -> bool {
        *self == Action::All || *self == *other
    }
}

/// A permission
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// The resource
    pub resource: Resource,
    /// The action
    pub action: Action,
}

impl Permission {
    /// Create a new permission
    pub fn new(resource: Resource, action: Action) -> Self {
        Self { resource, action }
    }

    /// Check if this permission grants access to another permission
    pub fn grants(&self, other: &Permission) -> bool {
        self.resource.matches(&other.resource) && self.action.matches(&other.action)
    }
}

/// A set of permissions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionSet {
    permissions: HashSet<Permission>,
}

impl PermissionSet {
    /// Create a new empty permission set
    pub fn new() -> Self {
        Self::default()
    }

    /// Grant a permission
    pub fn grant(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Revoke a permission
    pub fn revoke(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }

    /// Check if a permission is granted
    pub fn has(&self, required: &Permission) -> bool {
        self.permissions.iter().any(|p| p.grants(required))
    }

    /// Check if all permissions are granted
    pub fn has_all(&self, required: &[Permission]) -> bool {
        required.iter().all(|p| self.has(p))
    }

    /// Check if any permission is granted
    pub fn has_any(&self, required: &[Permission]) -> bool {
        required.iter().any(|p| self.has(p))
    }

    /// Get all permissions
    pub fn permissions(&self) -> impl Iterator<Item = &Permission> {
        self.permissions.iter()
    }

    /// Get the number of permissions
    pub fn len(&self) -> usize {
        self.permissions.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_matching() {
        let wildcard = Resource::new("users");
        let specific = Resource::with_id("users", "123");

        assert!(wildcard.matches(&specific));
        assert!(!specific.matches(&wildcard));
    }

    #[test]
    fn test_permission_grants() {
        let admin = Permission::new(Resource::new("users"), Action::All);
        let read = Permission::new(Resource::new("users"), Action::Read);

        assert!(admin.grants(&read));
        assert!(!read.grants(&admin));
    }

    #[test]
    fn test_permission_set() {
        let mut perms = PermissionSet::new();
        perms.grant(Permission::new(Resource::new("users"), Action::Read));
        perms.grant(Permission::new(Resource::new("posts"), Action::All));

        // Direct permission
        assert!(perms.has(&Permission::new(Resource::new("users"), Action::Read)));

        // Wildcard action
        assert!(perms.has(&Permission::new(Resource::new("posts"), Action::Write)));

        // Not granted
        assert!(!perms.has(&Permission::new(Resource::new("users"), Action::Delete)));
    }
}
