//! Audit sinks.

use crate::event::AuditEvent;
use async_trait::async_trait;
use infra_errors::InfraResult;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Audit sink trait
#[async_trait]
pub trait AuditSink: Send + Sync {
    /// Write an event
    async fn write(&self, event: &AuditEvent) -> InfraResult<()>;

    /// Flush pending events
    async fn flush(&self) -> InfraResult<()> {
        Ok(())
    }

    /// Sink name
    fn name(&self) -> &str;
}

/// Console sink (logs to stdout/tracing)
pub struct ConsoleSink {
    json_format: bool,
}

impl ConsoleSink {
    /// Create a new console sink
    pub fn new() -> Self {
        Self { json_format: false }
    }

    /// Enable JSON formatted output
    pub fn json(mut self) -> Self {
        self.json_format = true;
        self
    }
}

impl Default for ConsoleSink {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditSink for ConsoleSink {
    async fn write(&self, event: &AuditEvent) -> InfraResult<()> {
        if self.json_format {
            let json = serde_json::to_string(event).unwrap_or_default();
            tracing::info!(target: "audit", "{}", json);
        } else {
            tracing::info!(
                target: "audit",
                id = %event.id(),
                event_type = ?event.event_type(),
                action = %event.action(),
                outcome = ?event.outcome(),
                "Audit event"
            );
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "console"
    }
}

/// In-memory sink (for testing)
pub struct MemorySink {
    events: Arc<RwLock<Vec<AuditEvent>>>,
    max_events: usize,
}

impl MemorySink {
    /// Create a new memory sink
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events: 10000,
        }
    }

    /// Create with a maximum event limit
    pub fn with_limit(max: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events: max,
        }
    }

    /// Get all events
    pub async fn events(&self) -> Vec<AuditEvent> {
        self.events.read().await.clone()
    }

    /// Clear all events
    pub async fn clear(&self) {
        self.events.write().await.clear();
    }

    /// Get event count
    pub async fn count(&self) -> usize {
        self.events.read().await.len()
    }
}

impl Default for MemorySink {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditSink for MemorySink {
    async fn write(&self, event: &AuditEvent) -> InfraResult<()> {
        let mut events = self.events.write().await;

        // Enforce max events limit
        if events.len() >= self.max_events {
            events.remove(0);
        }

        events.push(event.clone());
        Ok(())
    }

    fn name(&self) -> &str {
        "memory"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{AuditEventBuilder, EventType, Outcome};

    #[tokio::test]
    async fn test_memory_sink() {
        let sink = MemorySink::new();

        let event = AuditEventBuilder::new(EventType::System)
            .action("test")
            .outcome(Outcome::Success)
            .build();

        sink.write(&event).await.unwrap();
        assert_eq!(sink.count().await, 1);

        sink.clear().await;
        assert_eq!(sink.count().await, 0);
    }

    #[tokio::test]
    async fn test_memory_sink_limit() {
        let sink = MemorySink::with_limit(2);

        for i in 0..5 {
            let event = AuditEventBuilder::new(EventType::System)
                .action(format!("action-{i}"))
                .outcome(Outcome::Success)
                .build();
            sink.write(&event).await.unwrap();
        }

        // Should only have last 2 events
        assert_eq!(sink.count().await, 2);
    }
}
