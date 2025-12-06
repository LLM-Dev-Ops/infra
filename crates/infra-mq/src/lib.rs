//! Message queue abstraction for LLM-Dev-Ops infrastructure.
//!
//! This crate provides a unified interface for message queues with
//! pluggable backends (in-memory, Redis, RabbitMQ).

mod message;
mod queue;
mod publisher;
mod subscriber;

#[cfg(feature = "memory")]
mod memory;

pub use message::{Message, MessageBuilder, MessageHeaders};
pub use queue::{Queue, QueueConfig};
pub use publisher::Publisher;
pub use subscriber::{Subscriber, MessageHandler};

#[cfg(feature = "memory")]
pub use memory::MemoryQueue;

use infra_errors::InfraResult;
use std::sync::Arc;

/// Create an in-memory queue
#[cfg(feature = "memory")]
pub fn memory_queue(name: &str) -> Arc<dyn Queue> {
    Arc::new(MemoryQueue::new(name))
}

/// Message acknowledgment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ack {
    /// Message processed successfully
    Ok,
    /// Message should be requeued
    Requeue,
    /// Message should be rejected (dead-letter)
    Reject,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "memory")]
    #[tokio::test]
    async fn test_memory_queue() {
        let queue = MemoryQueue::new("test");

        let msg = MessageBuilder::new()
            .body(b"Hello, World!".to_vec())
            .build();

        queue.publish(msg).await.unwrap();

        let received = queue.receive().await.unwrap();
        assert!(received.is_some());
        assert_eq!(received.unwrap().body(), b"Hello, World!");
    }
}
