//! Queue trait and configuration.

use crate::message::Message;
use crate::Ack;
use async_trait::async_trait;
use infra_errors::InfraResult;
use std::time::Duration;

/// Queue configuration
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Queue name
    pub name: String,
    /// Whether the queue is durable
    pub durable: bool,
    /// Maximum queue length
    pub max_length: Option<u32>,
    /// Message TTL
    pub message_ttl: Option<Duration>,
    /// Dead letter queue name
    pub dead_letter_queue: Option<String>,
    /// Maximum retries before dead-lettering
    pub max_retries: u32,
}

impl QueueConfig {
    /// Create a new queue configuration
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            durable: true,
            max_length: None,
            message_ttl: None,
            dead_letter_queue: None,
            max_retries: 3,
        }
    }

    /// Set durability
    pub fn durable(mut self, durable: bool) -> Self {
        self.durable = durable;
        self
    }

    /// Set maximum length
    pub fn max_length(mut self, max: u32) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Set message TTL
    pub fn message_ttl(mut self, ttl: Duration) -> Self {
        self.message_ttl = Some(ttl);
        self
    }

    /// Set dead letter queue
    pub fn dead_letter_queue(mut self, name: impl Into<String>) -> Self {
        self.dead_letter_queue = Some(name.into());
        self
    }

    /// Set maximum retries
    pub fn max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }
}

/// Queue trait
#[async_trait]
pub trait Queue: Send + Sync {
    /// Get the queue name
    fn name(&self) -> &str;

    /// Publish a message to the queue
    async fn publish(&self, message: Message) -> InfraResult<()>;

    /// Receive a message from the queue
    async fn receive(&self) -> InfraResult<Option<Message>>;

    /// Receive a message with timeout
    async fn receive_timeout(&self, timeout: Duration) -> InfraResult<Option<Message>>;

    /// Acknowledge a message
    async fn ack(&self, message_id: &str, ack: Ack) -> InfraResult<()>;

    /// Get the current queue length
    async fn len(&self) -> InfraResult<usize>;

    /// Check if the queue is empty
    async fn is_empty(&self) -> InfraResult<bool> {
        Ok(self.len().await? == 0)
    }

    /// Purge all messages from the queue
    async fn purge(&self) -> InfraResult<usize>;
}
