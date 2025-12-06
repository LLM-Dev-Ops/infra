//! Message subscriber.

use crate::message::Message;
use crate::queue::Queue;
use crate::Ack;
use async_trait::async_trait;
use infra_errors::InfraResult;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Message handler trait
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle a message
    async fn handle(&self, message: &Message) -> Ack;
}

/// Simple function handler
pub struct FnHandler<F>
where
    F: Fn(&Message) -> Ack + Send + Sync,
{
    handler: F,
}

impl<F> FnHandler<F>
where
    F: Fn(&Message) -> Ack + Send + Sync,
{
    /// Create a new function handler
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

#[async_trait]
impl<F> MessageHandler for FnHandler<F>
where
    F: Fn(&Message) -> Ack + Send + Sync,
{
    async fn handle(&self, message: &Message) -> Ack {
        (self.handler)(message)
    }
}

/// Message subscriber
pub struct Subscriber {
    queue: Arc<dyn Queue>,
    handler: Arc<dyn MessageHandler>,
    poll_interval: Duration,
    shutdown_rx: Option<mpsc::Receiver<()>>,
}

impl Subscriber {
    /// Create a new subscriber
    pub fn new(queue: Arc<dyn Queue>, handler: Arc<dyn MessageHandler>) -> Self {
        Self {
            queue,
            handler,
            poll_interval: Duration::from_millis(100),
            shutdown_rx: None,
        }
    }

    /// Create with a function handler
    pub fn with_fn<F>(queue: Arc<dyn Queue>, handler: F) -> Self
    where
        F: Fn(&Message) -> Ack + Send + Sync + 'static,
    {
        Self::new(queue, Arc::new(FnHandler::new(handler)))
    }

    /// Set the poll interval
    pub fn poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Set the shutdown channel
    pub fn with_shutdown(mut self, rx: mpsc::Receiver<()>) -> Self {
        self.shutdown_rx = Some(rx);
        self
    }

    /// Start consuming messages
    pub async fn start(mut self) -> InfraResult<()> {
        tracing::info!(queue = %self.queue.name(), "Starting subscriber");

        loop {
            // Check for shutdown
            if let Some(ref mut rx) = self.shutdown_rx {
                if rx.try_recv().is_ok() {
                    tracing::info!("Subscriber shutting down");
                    break;
                }
            }

            // Try to receive a message
            match self.queue.receive_timeout(self.poll_interval).await {
                Ok(Some(message)) => {
                    tracing::debug!(message_id = %message.id(), "Received message");

                    let ack = self.handler.handle(&message).await;
                    self.queue.ack(message.id(), ack).await?;

                    tracing::debug!(
                        message_id = %message.id(),
                        ack = ?ack,
                        "Message acknowledged"
                    );
                }
                Ok(None) => {
                    // No message, continue polling
                }
                Err(e) => {
                    tracing::error!(error = %e, "Error receiving message");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }

        Ok(())
    }

    /// Process a single message (for testing)
    pub async fn process_one(&self) -> InfraResult<Option<Ack>> {
        if let Some(message) = self.queue.receive().await? {
            let ack = self.handler.handle(&message).await;
            self.queue.ack(message.id(), ack).await?;
            Ok(Some(ack))
        } else {
            Ok(None)
        }
    }
}
