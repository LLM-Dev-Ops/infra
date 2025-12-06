//! Common types for LLM requests and responses.

use serde::{Deserialize, Serialize};

/// The role of a message in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// A message from the system (instructions, context).
    System,
    /// A message from the user.
    User,
    /// A message from the assistant (LLM).
    Assistant,
}

/// A message in a conversation with an LLM.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender.
    pub role: Role,
    /// The content of the message.
    pub content: String,
}

/// A request to an LLM for text completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    /// The model to use for completion (e.g., "gpt-4", "claude-3-opus-20240229").
    pub model: String,
    /// The conversation messages.
    pub messages: Vec<Message>,
    /// The sampling temperature (0.0 to 2.0). Higher values make output more random.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// The maximum number of tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Nucleus sampling parameter. An alternative to temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Number of completions to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Whether to stream the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Sequences where the API will stop generating further tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

/// A response from an LLM completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// The generated content.
    pub content: String,
    /// The model used for completion.
    pub model: String,
    /// The reason the generation stopped (e.g., "stop", "length", "content_filter").
    pub finish_reason: Option<String>,
    /// Usage statistics for the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Token usage statistics for an LLM request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    /// Number of tokens in the prompt.
    pub prompt_tokens: u32,
    /// Number of tokens in the completion.
    pub completion_tokens: u32,
    /// Total number of tokens used (prompt + completion).
    pub total_tokens: u32,
}

/// A request for text embeddings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    /// The model to use for embeddings (e.g., "text-embedding-ada-002").
    pub model: String,
    /// The input text(s) to embed.
    pub input: EmbeddingInput,
}

/// Input for an embedding request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    /// A single text string.
    Single(String),
    /// Multiple text strings.
    Multiple(Vec<String>),
}

/// A response from an embedding request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// The model used for embeddings.
    pub model: String,
    /// The embedding vectors.
    pub embeddings: Vec<Embedding>,
    /// Usage statistics for the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// A single embedding vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    /// The embedding vector.
    pub embedding: Vec<f32>,
    /// The index of this embedding in the input.
    pub index: usize,
}

/// A chunk of a streaming response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// The incremental content in this chunk.
    pub content: String,
    /// The model used for completion.
    pub model: String,
    /// The reason the generation stopped, if this is the final chunk.
    pub finish_reason: Option<String>,
}
