//! Message publisher.

use crate::message::{Message, MessageBuilder};
use crate::queue::Queue;
use infra_errors::InfraResult;
use serde::Serialize;
use std::sync::Arc;

/// Message publisher
pub struct Publisher {
    queue: Arc<dyn Queue>,
}

impl Publisher {
    /// Create a new publisher
    pub fn new(queue: Arc<dyn Queue>) -> Self {
        Self { queue }
    }

    /// Publish a message
    pub async fn publish(&self, message: Message) -> InfraResult<()> {
        self.queue.publish(message).await
    }

    /// Publish a raw byte message
    pub async fn publish_bytes(&self, body: Vec<u8>) -> InfraResult<()> {
        let message = MessageBuilder::new().body(body).build();
        self.publish(message).await
    }

    /// Publish a string message
    pub async fn publish_string(&self, body: &str) -> InfraResult<()> {
        let message = MessageBuilder::new().body_string(body).build();
        self.publish(message).await
    }

    /// Publish a JSON message
    pub async fn publish_json<T: Serialize>(&self, body: &T) -> InfraResult<()> {
        let message = MessageBuilder::new()
            .body_json(body)
            .map_err(|e| infra_errors::InfraError::Serialization {
                format: infra_errors::SerializationFormat::Json,
                message: e.to_string(),
                location: None,
                context: None,
            })?
            .build();
        self.publish(message).await
    }

    /// Publish with a correlation ID (for request-response)
    pub async fn publish_with_correlation(
        &self,
        body: Vec<u8>,
        correlation_id: &str,
        reply_to: Option<&str>,
    ) -> InfraResult<()> {
        let mut builder = MessageBuilder::new()
            .body(body)
            .correlation_id(correlation_id);

        if let Some(reply_queue) = reply_to {
            builder = builder.reply_to(reply_queue);
        }

        self.publish(builder.build()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryQueue;

    #[tokio::test]
    async fn test_publisher() {
        let queue = Arc::new(MemoryQueue::new("test"));
        let publisher = Publisher::new(queue.clone());

        publisher.publish_string("Hello").await.unwrap();

        let msg = queue.receive().await.unwrap().unwrap();
        assert_eq!(msg.body_string(), Some("Hello".to_string()));
    }
}
