//! Core trait definition for LLM providers.

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use crate::error::Result;
use crate::types::{EmbeddingRequest, EmbeddingResponse, LlmRequest, LlmResponse, StreamChunk};

/// A trait for LLM provider implementations.
///
/// This trait defines the core interface that all LLM providers must implement.
/// It supports both completion and embedding operations, with support for streaming.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Performs a text completion request.
    ///
    /// # Arguments
    ///
    /// * `request` - The completion request parameters.
    ///
    /// # Returns
    ///
    /// Returns the completion response on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the provider is unavailable,
    /// or if authentication/authorization fails.
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse>;

    /// Performs a streaming text completion request.
    ///
    /// # Arguments
    ///
    /// * `request` - The completion request parameters.
    ///
    /// # Returns
    ///
    /// Returns a stream of completion chunks on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the provider is unavailable,
    /// or if authentication/authorization fails.
    async fn stream(
        &self,
        request: LlmRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>>;

    /// Generates embeddings for the given text.
    ///
    /// # Arguments
    ///
    /// * `request` - The embedding request parameters.
    ///
    /// # Returns
    ///
    /// Returns the embedding vectors on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the provider is unavailable,
    /// or if authentication/authorization fails.
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse>;

    /// Returns the name of this provider (e.g., "openai", "anthropic").
    fn provider_name(&self) -> &str;
}
