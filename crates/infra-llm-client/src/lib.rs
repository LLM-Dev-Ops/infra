//! # infra-llm-client
//!
//! Unified LLM provider abstraction for LLM-Dev-Ops infrastructure.
//!
//! This crate provides a common interface for interacting with various LLM providers
//! (OpenAI, Anthropic, etc.) through a unified `LlmProvider` trait. It includes:
//!
//! - Core `LlmProvider` trait for implementing provider-specific adapters
//! - Common types for LLM requests, responses, and messages
//! - Error handling for LLM operations
//! - Placeholder adapters for future provider implementations
//!
//! ## Features
//!
//! - `std` (default): Enable standard library support
//!
//! ## Example
//!
//! ```rust,ignore
//! use infra_llm_client::{LlmProvider, LlmRequest, Message, Role};
//!
//! async fn example(provider: impl LlmProvider) {
//!     let request = LlmRequest {
//!         model: "gpt-4".to_string(),
//!         messages: vec![
//!             Message {
//!                 role: Role::User,
//!                 content: "Hello, world!".to_string(),
//!             }
//!         ],
//!         temperature: Some(0.7),
//!         max_tokens: Some(100),
//!     };
//!
//!     let response = provider.complete(request).await.unwrap();
//!     println!("Response: {}", response.content);
//! }
//! ```

pub mod adapters;
pub mod error;
pub mod provider;
pub mod types;

// Re-export commonly used items
pub use error::LlmClientError;
pub use provider::LlmProvider;
pub use types::{
    EmbeddingRequest, EmbeddingResponse, LlmRequest, LlmResponse, Message, Role,
};
