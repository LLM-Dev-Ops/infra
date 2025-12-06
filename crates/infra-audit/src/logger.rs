//! Audit logger.

use crate::event::AuditEvent;
use crate::sink::AuditSink;
use infra_errors::InfraResult;
use std::sync::Arc;

/// Logger configuration
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Buffer size for async logging
    pub buffer_size: usize,
    /// Whether to log synchronously
    pub sync_mode: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            sync_mode: false,
        }
    }
}

/// Audit logger
pub struct AuditLogger {
    sinks: Vec<Arc<dyn AuditSink>>,
    config: LoggerConfig,
}

impl AuditLogger {
    /// Create a new logger with a sink
    pub fn new(sink: Arc<dyn AuditSink>) -> Self {
        Self {
            sinks: vec![sink],
            config: LoggerConfig::default(),
        }
    }

    /// Create a new logger with configuration
    pub fn with_config(sink: Arc<dyn AuditSink>, config: LoggerConfig) -> Self {
        Self {
            sinks: vec![sink],
            config,
        }
    }

    /// Add a sink
    pub fn add_sink(&mut self, sink: Arc<dyn AuditSink>) {
        self.sinks.push(sink);
    }

    /// Log an event to all sinks
    pub async fn log(&self, event: AuditEvent) -> InfraResult<()> {
        for sink in &self.sinks {
            sink.write(&event).await?;
        }
        Ok(())
    }

    /// Flush all sinks
    pub async fn flush(&self) -> InfraResult<()> {
        for sink in &self.sinks {
            sink.flush().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::MemorySink;
    use crate::event::{AuditEventBuilder, EventType, Outcome};

    #[tokio::test]
    async fn test_logger() {
        let sink = Arc::new(MemorySink::new());
        let logger = AuditLogger::new(sink.clone());

        let event = AuditEventBuilder::new(EventType::System)
            .action("test")
            .outcome(Outcome::Success)
            .build();

        logger.log(event).await.unwrap();

        let events = sink.events().await;
        assert_eq!(events.len(), 1);
    }
}
