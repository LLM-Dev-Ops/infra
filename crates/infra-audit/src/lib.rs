//! Audit logging for LLM-Dev-Ops infrastructure.
//!
//! This crate provides structured audit logging for security-sensitive
//! operations with support for multiple backends.

mod event;
mod logger;
mod sink;
mod context;

pub use event::{AuditEvent, AuditEventBuilder, EventType, Outcome};
pub use logger::{AuditLogger, LoggerConfig};
pub use sink::{AuditSink, ConsoleSink, MemorySink};
pub use context::{AuditContext, Actor};

use infra_errors::InfraResult;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global audit logger
static GLOBAL_LOGGER: RwLock<Option<Arc<AuditLogger>>> = RwLock::const_new(None);

/// Initialize the global audit logger
pub async fn init(logger: AuditLogger) {
    let mut global = GLOBAL_LOGGER.write().await;
    *global = Some(Arc::new(logger));
}

/// Log an audit event
pub async fn log(event: AuditEvent) -> InfraResult<()> {
    let global = GLOBAL_LOGGER.read().await;
    if let Some(logger) = global.as_ref() {
        logger.log(event).await
    } else {
        tracing::warn!("Audit logger not initialized");
        Ok(())
    }
}

/// Create a quick audit event and log it
pub async fn audit(
    event_type: EventType,
    action: impl Into<String>,
    outcome: Outcome,
) -> InfraResult<()> {
    let event = AuditEventBuilder::new(event_type)
        .action(action)
        .outcome(outcome)
        .build();
    log(event).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_event() {
        let event = AuditEventBuilder::new(EventType::Authentication)
            .action("login")
            .outcome(Outcome::Success)
            .actor(Actor::user("user123"))
            .build();

        assert_eq!(event.event_type(), EventType::Authentication);
        assert_eq!(event.action(), "login");
    }
}
