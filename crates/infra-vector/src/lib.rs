//! Vector database operations for LLM-Dev-Ops infrastructure.
//!
//! This crate wraps ruvector-core capabilities with:
//! - Unified error handling via `infra-errors`
//! - OpenTelemetry instrumentation (when `otel` feature enabled)
//! - Configuration via `infra-config`
//! - WASM support via `ruvector-gnn-wasm`
//!
//! # Features
//!
//! - `default` - Includes `std` and `ruvector` features
//! - `std` - Standard library support
//! - `ruvector` - RuvVector integration (ruvector-core)
//! - `wasm` - WebAssembly bindings via ruvector-gnn-wasm
//! - `otel` - OpenTelemetry tracing instrumentation
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use infra_vector::{RuVectorStore, VectorStore, VectorStoreConfig, Distance, VectorId};
//!
//! # async fn example() -> infra_errors::InfraResult<()> {
//! // Create a vector store
//! let config = VectorStoreConfig::new("embeddings", 1536)
//!     .with_distance(Distance::Cosine);
//! let store = RuVectorStore::new(config).await?;
//!
//! // Insert a vector
//! let id = VectorId::new("doc-1");
//! let vector = vec![0.1; 1536]; // Your embedding
//! store.insert(id, vector, None).await?;
//!
//! // Search for similar vectors
//! let query = vec![0.1; 1536];
//! let results = store.search(query, 10, None).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Docker Setup
//!
//! The vector store uses PostgreSQL with pgvector for production use.
//! See docker-compose.yml in the repository root for the recommended setup:
//!
//! ```bash
//! docker compose up -d postgres redis
//! ```
//!
//! # Environment Variables
//!
//! - `INFRA_VECTOR_COLLECTION` - Collection name (default: "default")
//! - `INFRA_VECTOR_DIMENSIONS` - Vector dimensions (default: 1536)
//! - `INFRA_VECTOR_DISTANCE` - Distance metric: cosine, euclidean, dot_product, manhattan
//! - `INFRA_RUVECTOR_URL` - RuvVector HTTP endpoint (default: http://localhost:8100)

// Core modules (always available)
mod vector;
mod similarity;
mod index;
mod embedding;

// New modules from SPARC spec
mod types;
mod traits;
mod store;

// WASM module (feature-gated)
#[cfg(feature = "wasm")]
mod wasm;

// Re-export core vector operations
pub use vector::{Vector, VectorError};
pub use similarity::{Similarity, cosine_similarity, euclidean_distance, dot_product, manhattan_distance};
pub use index::{VectorIndex, IndexConfig, SearchResult as IndexSearchResult};
pub use embedding::{Embedding, EmbeddingModel, EmbeddingBatch};

// Re-export SPARC types and traits
pub use types::{
    VectorId,
    Distance,
    HnswConfig,
    CompressionConfig,
    TierThresholds,
    VectorStoreConfig,
    VectorRecord,
    SearchResult,
    BatchInsertResult,
    CollectionStats,
    MetadataFilter,
};
pub use traits::VectorStore;
pub use store::RuVectorStore;

// Re-export WASM bindings when enabled
#[cfg(feature = "wasm")]
pub use wasm::*;

use infra_errors::InfraResult;

/// Create a zero vector of the given dimension.
///
/// # Example
/// ```
/// let v = infra_vector::zeros(10);
/// assert_eq!(v.dim(), 10);
/// ```
pub fn zeros(dim: usize) -> Vector {
    Vector::zeros(dim)
}

/// Create a random unit vector of the given dimension.
///
/// # Example
/// ```
/// let v = infra_vector::random_unit(100);
/// assert!((v.norm() - 1.0).abs() < 1e-6);
/// ```
pub fn random_unit(dim: usize) -> Vector {
    Vector::random_unit(dim)
}

/// Normalize a vector to unit length.
///
/// # Errors
/// Returns error if the vector has zero norm.
pub fn normalize(v: &Vector) -> InfraResult<Vector> {
    v.normalize()
}

/// Create a RuVectorStore from environment configuration.
///
/// This is a convenience function that reads configuration from environment
/// variables and creates a configured store.
///
/// # Environment Variables
/// - `INFRA_VECTOR_COLLECTION` - Collection name
/// - `INFRA_VECTOR_DIMENSIONS` - Vector dimensions
/// - `INFRA_VECTOR_DISTANCE` - Distance metric
/// - `INFRA_RUVECTOR_URL` - RuvVector endpoint URL
///
/// # Example
/// ```rust,no_run
/// # async fn example() -> infra_errors::InfraResult<()> {
/// let store = infra_vector::create_store_from_env().await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_store_from_env() -> InfraResult<RuVectorStore> {
    RuVectorStore::from_env().await
}

/// Embedding normalizer utility.
///
/// Provides batch normalization for embedding vectors.
pub struct EmbeddingNormalizer;

impl EmbeddingNormalizer {
    /// L2 normalize a vector in place.
    pub fn normalize(vector: &mut [f32]) {
        let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for x in vector.iter_mut() {
                *x /= magnitude;
            }
        }
    }

    /// Batch normalize vectors.
    pub fn normalize_batch(vectors: &mut [Vec<f32>]) {
        for vector in vectors {
            Self::normalize(vector);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeros() {
        let v = zeros(10);
        assert_eq!(v.dim(), 10);
        assert!(v.data().iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_random_unit() {
        let v = random_unit(100);
        assert_eq!(v.dim(), 100);

        let norm = v.norm();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_embedding_normalizer() {
        let mut v = vec![3.0, 4.0];
        EmbeddingNormalizer::normalize(&mut v);

        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_embedding_normalizer_batch() {
        let mut vectors = vec![
            vec![3.0, 4.0],
            vec![1.0, 0.0],
            vec![0.0, 5.0],
        ];

        EmbeddingNormalizer::normalize_batch(&mut vectors);

        for v in &vectors {
            let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 1e-6);
        }
    }

    #[tokio::test]
    async fn test_ruvector_store_integration() {
        let config = VectorStoreConfig::new("test_integration", 128)
            .with_distance(Distance::Cosine);

        let store = RuVectorStore::new(config).await.unwrap();

        // Insert vectors
        for i in 0..5 {
            let id = VectorId::new(format!("vec-{}", i));
            let vector: Vec<f32> = (0..128).map(|j| (i * 128 + j) as f32 / 1000.0).collect();
            store.insert(id, vector, None).await.unwrap();
        }

        // Search
        let query: Vec<f32> = (0..128).map(|j| j as f32 / 1000.0).collect();
        let results = store.search(query, 3, None).await.unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].id.as_str(), "vec-0");

        // Stats
        let stats = store.stats().await.unwrap();
        assert_eq!(stats.total_vectors, 5);
        assert_eq!(stats.dimensions, 128);
    }
}
