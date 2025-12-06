//! Embedding utilities.

use crate::vector::Vector;
use infra_errors::InfraResult;
use serde::{Deserialize, Serialize};

/// An embedding with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    /// The embedding vector
    pub vector: Vector,
    /// Model used to generate the embedding
    pub model: Option<String>,
    /// Original text (if applicable)
    pub text: Option<String>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

impl Embedding {
    /// Create a new embedding
    pub fn new(vector: Vector) -> Self {
        Self {
            vector,
            model: None,
            text: None,
            metadata: None,
        }
    }

    /// Set the model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the original text
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Get the dimension
    pub fn dim(&self) -> usize {
        self.vector.dim()
    }
}

/// Embedding model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModel {
    /// Model name
    pub name: String,
    /// Model dimension
    pub dimension: usize,
    /// Maximum input length
    pub max_input_length: usize,
    /// Model provider
    pub provider: Option<String>,
}

impl EmbeddingModel {
    /// Create a new model config
    pub fn new(name: impl Into<String>, dimension: usize) -> Self {
        Self {
            name: name.into(),
            dimension,
            max_input_length: 8192,
            provider: None,
        }
    }

    /// OpenAI text-embedding-3-small
    pub fn openai_small() -> Self {
        Self {
            name: "text-embedding-3-small".to_string(),
            dimension: 1536,
            max_input_length: 8191,
            provider: Some("openai".to_string()),
        }
    }

    /// OpenAI text-embedding-3-large
    pub fn openai_large() -> Self {
        Self {
            name: "text-embedding-3-large".to_string(),
            dimension: 3072,
            max_input_length: 8191,
            provider: Some("openai".to_string()),
        }
    }

    /// Cohere embed-v3
    pub fn cohere_v3() -> Self {
        Self {
            name: "embed-v3".to_string(),
            dimension: 1024,
            max_input_length: 512,
            provider: Some("cohere".to_string()),
        }
    }

    /// Set max input length
    pub fn max_input_length(mut self, length: usize) -> Self {
        self.max_input_length = length;
        self
    }

    /// Set provider
    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }
}

/// Batch of embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingBatch {
    /// The embeddings
    pub embeddings: Vec<Embedding>,
    /// Model used
    pub model: Option<String>,
    /// Total tokens used
    pub total_tokens: Option<usize>,
}

impl EmbeddingBatch {
    /// Create a new batch
    pub fn new(embeddings: Vec<Embedding>) -> Self {
        Self {
            embeddings,
            model: None,
            total_tokens: None,
        }
    }

    /// Get the number of embeddings
    pub fn len(&self) -> usize {
        self.embeddings.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.embeddings.is_empty()
    }

    /// Get the vectors
    pub fn vectors(&self) -> Vec<&Vector> {
        self.embeddings.iter().map(|e| &e.vector).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding() {
        let vector = Vector::new(vec![0.1, 0.2, 0.3]);
        let embedding = Embedding::new(vector.clone())
            .with_model("test-model")
            .with_text("Hello, world!");

        assert_eq!(embedding.dim(), 3);
        assert_eq!(embedding.model, Some("test-model".to_string()));
        assert_eq!(embedding.text, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_embedding_model() {
        let model = EmbeddingModel::openai_small();
        assert_eq!(model.dimension, 1536);
        assert_eq!(model.provider, Some("openai".to_string()));
    }
}
