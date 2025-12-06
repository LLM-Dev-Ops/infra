//! Audit context.

use serde::{Deserialize, Serialize};

/// Actor who performed an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    /// Actor type
    pub actor_type: ActorType,
    /// Actor ID
    pub id: String,
    /// Actor name
    pub name: Option<String>,
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
}

impl Actor {
    /// Create a user actor
    pub fn user(id: impl Into<String>) -> Self {
        Self {
            actor_type: ActorType::User,
            id: id.into(),
            name: None,
            ip_address: None,
            user_agent: None,
        }
    }

    /// Create a service actor
    pub fn service(id: impl Into<String>) -> Self {
        Self {
            actor_type: ActorType::Service,
            id: id.into(),
            name: None,
            ip_address: None,
            user_agent: None,
        }
    }

    /// Create a system actor
    pub fn system() -> Self {
        Self {
            actor_type: ActorType::System,
            id: "system".to_string(),
            name: Some("System".to_string()),
            ip_address: None,
            user_agent: None,
        }
    }

    /// Set the name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the IP address
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set the user agent
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }
}

/// Actor type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    /// Human user
    User,
    /// Service/application
    Service,
    /// System process
    System,
    /// Anonymous
    Anonymous,
}

/// Audit context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditContext {
    /// Request ID
    pub request_id: Option<String>,
    /// Trace ID
    pub trace_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Client IP
    pub client_ip: Option<String>,
    /// Server hostname
    pub server: Option<String>,
    /// Environment
    pub environment: Option<String>,
}

impl AuditContext {
    /// Create a new context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set request ID
    pub fn request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    /// Set trace ID
    pub fn trace_id(mut self, id: impl Into<String>) -> Self {
        self.trace_id = Some(id.into());
        self
    }

    /// Set session ID
    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }

    /// Set client IP
    pub fn client_ip(mut self, ip: impl Into<String>) -> Self {
        self.client_ip = Some(ip.into());
        self
    }

    /// Set server hostname
    pub fn server(mut self, server: impl Into<String>) -> Self {
        self.server = Some(server.into());
        self
    }

    /// Set environment
    pub fn environment(mut self, env: impl Into<String>) -> Self {
        self.environment = Some(env.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor() {
        let actor = Actor::user("user123")
            .with_name("John Doe")
            .with_ip("192.168.1.1");

        assert_eq!(actor.id, "user123");
        assert_eq!(actor.name, Some("John Doe".to_string()));
        assert_eq!(actor.ip_address, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_context() {
        let ctx = AuditContext::new()
            .request_id("req-123")
            .trace_id("trace-456")
            .environment("production");

        assert_eq!(ctx.request_id, Some("req-123".to_string()));
        assert_eq!(ctx.trace_id, Some("trace-456".to_string()));
        assert_eq!(ctx.environment, Some("production".to_string()));
    }
}
