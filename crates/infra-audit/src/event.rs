//! Audit event types.

use crate::context::{Actor, AuditContext};
use chrono::{DateTime, Utc};
use infra_id::{IdGenerator, UlidGenerator};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Authentication events
    Authentication,
    /// Authorization events
    Authorization,
    /// Data access events
    DataAccess,
    /// Data modification events
    DataModification,
    /// Configuration changes
    ConfigChange,
    /// System events
    System,
    /// Security events
    Security,
    /// Custom event type
    Custom,
}

/// Event outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Operation succeeded
    Success,
    /// Operation failed
    Failure,
    /// Operation was denied
    Denied,
    /// Unknown outcome
    Unknown,
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    id: String,
    /// Event timestamp
    timestamp: DateTime<Utc>,
    /// Event type
    event_type: EventType,
    /// Action performed
    action: String,
    /// Event outcome
    outcome: Outcome,
    /// Actor who performed the action
    actor: Option<Actor>,
    /// Resource affected
    resource: Option<String>,
    /// Resource ID
    resource_id: Option<String>,
    /// Additional context
    context: AuditContext,
    /// Custom metadata
    metadata: HashMap<String, serde_json::Value>,
    /// Error message (if outcome is failure)
    error: Option<String>,
}

impl AuditEvent {
    /// Get the event ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Get the event type
    pub fn event_type(&self) -> EventType {
        self.event_type
    }

    /// Get the action
    pub fn action(&self) -> &str {
        &self.action
    }

    /// Get the outcome
    pub fn outcome(&self) -> Outcome {
        self.outcome
    }

    /// Get the actor
    pub fn actor(&self) -> Option<&Actor> {
        self.actor.as_ref()
    }

    /// Get the resource
    pub fn resource(&self) -> Option<&str> {
        self.resource.as_deref()
    }

    /// Get metadata
    pub fn metadata(&self) -> &HashMap<String, serde_json::Value> {
        &self.metadata
    }
}

/// Audit event builder
pub struct AuditEventBuilder {
    event_type: EventType,
    action: String,
    outcome: Outcome,
    actor: Option<Actor>,
    resource: Option<String>,
    resource_id: Option<String>,
    context: AuditContext,
    metadata: HashMap<String, serde_json::Value>,
    error: Option<String>,
}

impl AuditEventBuilder {
    /// Create a new builder
    pub fn new(event_type: EventType) -> Self {
        Self {
            event_type,
            action: String::new(),
            outcome: Outcome::Unknown,
            actor: None,
            resource: None,
            resource_id: None,
            context: AuditContext::default(),
            metadata: HashMap::new(),
            error: None,
        }
    }

    /// Set the action
    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.action = action.into();
        self
    }

    /// Set the outcome
    pub fn outcome(mut self, outcome: Outcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Set the actor
    pub fn actor(mut self, actor: Actor) -> Self {
        self.actor = Some(actor);
        self
    }

    /// Set the resource
    pub fn resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Set the resource ID
    pub fn resource_id(mut self, id: impl Into<String>) -> Self {
        self.resource_id = Some(id.into());
        self
    }

    /// Set the context
    pub fn context(mut self, context: AuditContext) -> Self {
        self.context = context;
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set error message
    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Build the event
    pub fn build(self) -> AuditEvent {
        AuditEvent {
            id: UlidGenerator.generate(),
            timestamp: Utc::now(),
            event_type: self.event_type,
            action: self.action,
            outcome: self.outcome,
            actor: self.actor,
            resource: self.resource,
            resource_id: self.resource_id,
            context: self.context,
            metadata: self.metadata,
            error: self.error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_builder() {
        let event = AuditEventBuilder::new(EventType::DataAccess)
            .action("read")
            .outcome(Outcome::Success)
            .resource("users")
            .resource_id("123")
            .actor(Actor::user("admin"))
            .metadata("query", "SELECT * FROM users")
            .build();

        assert_eq!(event.event_type(), EventType::DataAccess);
        assert_eq!(event.action(), "read");
        assert_eq!(event.outcome(), Outcome::Success);
        assert_eq!(event.resource(), Some("users"));
    }
}
