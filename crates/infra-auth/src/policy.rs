//! Policy-based authorization.

use crate::identity::Identity;
use crate::permission::{Action, Permission, Resource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Policy effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effect {
    /// Allow the action
    Allow,
    /// Deny the action
    Deny,
}

/// Policy decision
#[derive(Debug, Clone)]
pub struct PolicyDecision {
    /// The effect
    pub effect: Effect,
    /// The policy that made the decision
    pub policy_id: Option<String>,
    /// Reason for the decision
    pub reason: Option<String>,
}

impl PolicyDecision {
    /// Create an allow decision
    pub fn allow() -> Self {
        Self {
            effect: Effect::Allow,
            policy_id: None,
            reason: None,
        }
    }

    /// Create a deny decision
    pub fn deny() -> Self {
        Self {
            effect: Effect::Deny,
            policy_id: None,
            reason: None,
        }
    }

    /// Set the policy ID
    pub fn with_policy(mut self, id: impl Into<String>) -> Self {
        self.policy_id = Some(id.into());
        self
    }

    /// Set the reason
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Check if allowed
    pub fn is_allowed(&self) -> bool {
        matches!(self.effect, Effect::Allow)
    }
}

/// A policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Policy ID
    pub id: String,
    /// Policy name
    pub name: Option<String>,
    /// Effect
    pub effect: Effect,
    /// Required roles (any of these)
    pub roles: Option<Vec<String>>,
    /// Required attributes
    pub attributes: Option<HashMap<String, serde_json::Value>>,
    /// Resources this policy applies to
    pub resources: Option<Vec<String>>,
    /// Actions this policy applies to
    pub actions: Option<Vec<Action>>,
    /// Priority (higher = evaluated first)
    pub priority: i32,
}

impl Policy {
    /// Create a new allow policy
    pub fn allow(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: None,
            effect: Effect::Allow,
            roles: None,
            attributes: None,
            resources: None,
            actions: None,
            priority: 0,
        }
    }

    /// Create a new deny policy
    pub fn deny(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: None,
            effect: Effect::Deny,
            roles: None,
            attributes: None,
            resources: None,
            actions: None,
            priority: 0,
        }
    }

    /// Set required roles
    pub fn for_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = Some(roles);
        self
    }

    /// Set resources
    pub fn on_resources(mut self, resources: Vec<String>) -> Self {
        self.resources = Some(resources);
        self
    }

    /// Set actions
    pub fn for_actions(mut self, actions: Vec<Action>) -> Self {
        self.actions = Some(actions);
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Check if this policy applies to the given request
    fn applies(&self, identity: &Identity, resource: &str, action: Action) -> bool {
        // Check roles
        if let Some(required_roles) = &self.roles {
            if !required_roles.iter().any(|r| identity.has_role(r)) {
                return false;
            }
        }

        // Check resources
        if let Some(resources) = &self.resources {
            if !resources.iter().any(|r| r == "*" || r == resource) {
                return false;
            }
        }

        // Check actions
        if let Some(actions) = &self.actions {
            if !actions.iter().any(|a| *a == Action::All || *a == action) {
                return false;
            }
        }

        true
    }
}

/// Policy engine
pub struct PolicyEngine {
    policies: Vec<Policy>,
    default_effect: Effect,
}

impl PolicyEngine {
    /// Create a new policy engine with deny-by-default
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
            default_effect: Effect::Deny,
        }
    }

    /// Create with allow-by-default (not recommended for production)
    pub fn allow_by_default() -> Self {
        Self {
            policies: Vec::new(),
            default_effect: Effect::Allow,
        }
    }

    /// Add a policy
    pub fn add_policy(&mut self, policy: Policy) {
        self.policies.push(policy);
        // Sort by priority (descending)
        self.policies.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Evaluate a request
    pub fn evaluate(
        &self,
        identity: &Identity,
        resource: &str,
        action: Action,
    ) -> PolicyDecision {
        for policy in &self.policies {
            if policy.applies(identity, resource, action) {
                return PolicyDecision {
                    effect: policy.effect,
                    policy_id: Some(policy.id.clone()),
                    reason: policy.name.clone(),
                };
            }
        }

        // Return default decision
        PolicyDecision {
            effect: self.default_effect,
            policy_id: None,
            reason: Some("No matching policy".to_string()),
        }
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_engine() {
        let mut engine = PolicyEngine::new();

        // Admins can do anything
        engine.add_policy(
            Policy::allow("admin-all")
                .for_roles(vec!["admin".to_string()])
                .priority(100),
        );

        // Users can read
        engine.add_policy(
            Policy::allow("user-read")
                .for_roles(vec!["user".to_string()])
                .for_actions(vec![Action::Read])
                .priority(50),
        );

        let admin = Identity::user("admin1").with_role("admin");
        let user = Identity::user("user1").with_role("user");

        // Admin can do anything
        assert!(engine.evaluate(&admin, "users", Action::Delete).is_allowed());

        // User can read
        assert!(engine.evaluate(&user, "posts", Action::Read).is_allowed());

        // User cannot delete
        assert!(!engine.evaluate(&user, "posts", Action::Delete).is_allowed());
    }
}
