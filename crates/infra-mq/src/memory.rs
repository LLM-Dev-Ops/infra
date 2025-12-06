//! In-memory queue implementation.

use crate::message::Message;
use crate::queue::Queue;
use crate::Ack;
use async_trait::async_trait;
use infra_errors::{InfraError, InfraResult, MqOperation};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// In-memory queue implementation
pub struct MemoryQueue {
    name: String,
    messages: Arc<Mutex<VecDeque<Message>>>,
    pending: Arc<Mutex<Vec<Message>>>,
}

impl MemoryQueue {
    /// Create a new in-memory queue
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            messages: Arc::new(Mutex::new(VecDeque::new())),
            pending: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl Queue for MemoryQueue {
    fn name(&self) -> &str {
        &self.name
    }

    async fn publish(&self, message: Message) -> InfraResult<()> {
        let mut messages = self.messages.lock().await;
        messages.push_back(message);
        Ok(())
    }

    async fn receive(&self) -> InfraResult<Option<Message>> {
        let mut messages = self.messages.lock().await;
        if let Some(mut message) = messages.pop_front() {
            message.increment_delivery();

            // Move to pending
            let mut pending = self.pending.lock().await;
            pending.push(message.clone());

            Ok(Some(message))
        } else {
            Ok(None)
        }
    }

    async fn receive_timeout(&self, timeout: Duration) -> InfraResult<Option<Message>> {
        let start = std::time::Instant::now();

        loop {
            if let Some(message) = self.receive().await? {
                return Ok(Some(message));
            }

            if start.elapsed() >= timeout {
                return Ok(None);
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn ack(&self, message_id: &str, ack: Ack) -> InfraResult<()> {
        let mut pending = self.pending.lock().await;
        let pos = pending.iter().position(|m| m.id() == message_id);

        match pos {
            Some(index) => {
                let message = pending.remove(index);

                match ack {
                    Ack::Ok => {
                        // Message processed, remove from pending
                    }
                    Ack::Requeue => {
                        // Put back in queue
                        let mut messages = self.messages.lock().await;
                        messages.push_front(message);
                    }
                    Ack::Reject => {
                        // Message rejected, could go to dead letter queue
                        tracing::warn!(message_id = %message_id, "Message rejected");
                    }
                }

                Ok(())
            }
            None => Err(InfraError::MessageQueue {
                operation: MqOperation::Acknowledge,
                queue: self.name.clone(),
                message: format!("Message not found: {message_id}"),
                context: None,
            }),
        }
    }

    async fn len(&self) -> InfraResult<usize> {
        let messages = self.messages.lock().await;
        Ok(messages.len())
    }

    async fn purge(&self) -> InfraResult<usize> {
        let mut messages = self.messages.lock().await;
        let count = messages.len();
        messages.clear();
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::MessageBuilder;

    #[tokio::test]
    async fn test_memory_queue_publish_receive() {
        let queue = MemoryQueue::new("test");

        let msg = MessageBuilder::new()
            .body_string("Hello")
            .build();

        queue.publish(msg).await.unwrap();
        assert_eq!(queue.len().await.unwrap(), 1);

        let received = queue.receive().await.unwrap().unwrap();
        assert_eq!(received.body_string(), Some("Hello".to_string()));
        assert_eq!(received.delivery_count(), 1);
    }

    #[tokio::test]
    async fn test_memory_queue_ack() {
        let queue = MemoryQueue::new("test");

        let msg = MessageBuilder::new()
            .body_string("Hello")
            .build();

        queue.publish(msg).await.unwrap();
        let received = queue.receive().await.unwrap().unwrap();

        queue.ack(received.id(), Ack::Ok).await.unwrap();
    }

    #[tokio::test]
    async fn test_memory_queue_requeue() {
        let queue = MemoryQueue::new("test");

        let msg = MessageBuilder::new()
            .body_string("Hello")
            .build();

        queue.publish(msg).await.unwrap();

        let received = queue.receive().await.unwrap().unwrap();
        queue.ack(received.id(), Ack::Requeue).await.unwrap();

        // Message should be back in queue
        assert_eq!(queue.len().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_memory_queue_purge() {
        let queue = MemoryQueue::new("test");

        for i in 0..5 {
            let msg = MessageBuilder::new()
                .body_string(&format!("Message {i}"))
                .build();
            queue.publish(msg).await.unwrap();
        }

        let count = queue.purge().await.unwrap();
        assert_eq!(count, 5);
        assert!(queue.is_empty().await.unwrap());
    }
}
