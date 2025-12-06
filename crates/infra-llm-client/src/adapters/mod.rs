//! Adapters for various LLM providers.
//!
//! This module contains adapter implementations for different LLM providers.
//! Currently, these are placeholder structs that will be fully implemented in the future.

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use crate::error::{LlmClientError, Result};
use crate::provider::LlmProvider;
use crate::types::{EmbeddingRequest, EmbeddingResponse, LlmRequest, LlmResponse, StreamChunk};

/// Placeholder adapter for OpenAI's API.
///
/// This struct will implement the `LlmProvider` trait to interact with OpenAI's
/// completion and embedding endpoints.
#[derive(Debug, Clone)]
pub struct OpenAiAdapter {
    /// API key for authentication (placeholder).
    pub api_key: String,
    /// Base URL for the API (placeholder).
    pub base_url: String,
}

impl OpenAiAdapter {
    /// Creates a new OpenAI adapter (placeholder implementation).
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    /// Creates a new OpenAI adapter with a custom base URL (placeholder implementation).
    #[must_use]
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self { api_key, base_url }
    }
}

#[async_trait]
impl LlmProvider for OpenAiAdapter {
    async fn complete(&self, _request: LlmRequest) -> Result<LlmResponse> {
        // TODO: Implement OpenAI completion API
        Err(LlmClientError::Unsupported(
            "OpenAI adapter not yet implemented".to_string(),
        ))
    }

    async fn stream(
        &self,
        _request: LlmRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>> {
        // TODO: Implement OpenAI streaming API
        Err(LlmClientError::Unsupported(
            "OpenAI streaming not yet implemented".to_string(),
        ))
    }

    async fn embed(&self, _request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        // TODO: Implement OpenAI embeddings API
        Err(LlmClientError::Unsupported(
            "OpenAI embeddings not yet implemented".to_string(),
        ))
    }

    fn provider_name(&self) -> &str {
        "openai"
    }
}

/// Placeholder adapter for Anthropic's API.
///
/// This struct will implement the `LlmProvider` trait to interact with Anthropic's
/// Claude API for completion and other operations.
#[derive(Debug, Clone)]
pub struct AnthropicAdapter {
    /// API key for authentication (placeholder).
    pub api_key: String,
    /// Base URL for the API (placeholder).
    pub base_url: String,
}

impl AnthropicAdapter {
    /// Creates a new Anthropic adapter (placeholder implementation).
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }

    /// Creates a new Anthropic adapter with a custom base URL (placeholder implementation).
    #[must_use]
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self { api_key, base_url }
    }
}

#[async_trait]
impl LlmProvider for AnthropicAdapter {
    async fn complete(&self, _request: LlmRequest) -> Result<LlmResponse> {
        // TODO: Implement Anthropic completion API
        Err(LlmClientError::Unsupported(
            "Anthropic adapter not yet implemented".to_string(),
        ))
    }

    async fn stream(
        &self,
        _request: LlmRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>> {
        // TODO: Implement Anthropic streaming API
        Err(LlmClientError::Unsupported(
            "Anthropic streaming not yet implemented".to_string(),
        ))
    }

    async fn embed(&self, _request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        // TODO: Implement Anthropic embeddings API (when available)
        Err(LlmClientError::Unsupported(
            "Anthropic embeddings not yet available".to_string(),
        ))
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }
}
