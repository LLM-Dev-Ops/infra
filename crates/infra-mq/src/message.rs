//! Message types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Message headers
pub type MessageHeaders = HashMap<String, String>;

/// A message in the queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID
    id: String,
    /// Message body
    body: Vec<u8>,
    /// Headers
    headers: MessageHeaders,
    /// Correlation ID for request-response patterns
    correlation_id: Option<String>,
    /// Reply-to queue name
    reply_to: Option<String>,
    /// Message timestamp
    timestamp: u64,
    /// Time-to-live in milliseconds
    ttl: Option<u64>,
    /// Delivery count (for retry tracking)
    delivery_count: u32,
}

impl Message {
    /// Create a new message with a body
    pub fn new(body: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            body,
            headers: HashMap::new(),
            correlation_id: None,
            reply_to: None,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            ttl: None,
            delivery_count: 0,
        }
    }

    /// Get the message ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the message body
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Get the body as a string
    pub fn body_string(&self) -> Option<String> {
        String::from_utf8(self.body.clone()).ok()
    }

    /// Parse the body as JSON
    pub fn body_json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }

    /// Get headers
    pub fn headers(&self) -> &MessageHeaders {
        &self.headers
    }

    /// Get a specific header
    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    /// Get correlation ID
    pub fn correlation_id(&self) -> Option<&str> {
        self.correlation_id.as_deref()
    }

    /// Get reply-to queue
    pub fn reply_to(&self) -> Option<&str> {
        self.reply_to.as_deref()
    }

    /// Get timestamp
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Get TTL
    pub fn ttl(&self) -> Option<Duration> {
        self.ttl.map(Duration::from_millis)
    }

    /// Check if the message has expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            now > self.timestamp + ttl
        } else {
            false
        }
    }

    /// Get delivery count
    pub fn delivery_count(&self) -> u32 {
        self.delivery_count
    }

    /// Increment delivery count
    pub fn increment_delivery(&mut self) {
        self.delivery_count += 1;
    }
}

/// Message builder
pub struct MessageBuilder {
    message: Message,
}

impl MessageBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            message: Message::new(Vec::new()),
        }
    }

    /// Set the body
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.message.body = body;
        self
    }

    /// Set the body from a string
    pub fn body_string(mut self, body: &str) -> Self {
        self.message.body = body.as_bytes().to_vec();
        self
    }

    /// Set the body from JSON
    pub fn body_json<T: Serialize>(mut self, body: &T) -> Result<Self, serde_json::Error> {
        self.message.body = serde_json::to_vec(body)?;
        self.message.headers.insert(
            "content-type".to_string(),
            "application/json".to_string(),
        );
        Ok(self)
    }

    /// Add a header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.message.headers.insert(key.into(), value.into());
        self
    }

    /// Set correlation ID
    pub fn correlation_id(mut self, id: impl Into<String>) -> Self {
        self.message.correlation_id = Some(id.into());
        self
    }

    /// Set reply-to queue
    pub fn reply_to(mut self, queue: impl Into<String>) -> Self {
        self.message.reply_to = Some(queue.into());
        self
    }

    /// Set TTL
    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.message.ttl = Some(ttl.as_millis() as u64);
        self
    }

    /// Build the message
    pub fn build(self) -> Message {
        self.message
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_builder() {
        let msg = MessageBuilder::new()
            .body_string("Hello")
            .header("x-custom", "value")
            .correlation_id("corr-123")
            .ttl(Duration::from_secs(60))
            .build();

        assert_eq!(msg.body_string(), Some("Hello".to_string()));
        assert_eq!(msg.header("x-custom"), Some(&"value".to_string()));
        assert_eq!(msg.correlation_id(), Some("corr-123"));
        assert!(msg.ttl().is_some());
    }

    #[test]
    fn test_message_expiry() {
        let mut msg = MessageBuilder::new()
            .body(vec![])
            .ttl(Duration::from_millis(0))
            .build();

        // Force the message to be old
        msg.timestamp = 0;

        assert!(msg.is_expired());
    }
}
