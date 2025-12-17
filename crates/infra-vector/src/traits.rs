//! Vector store trait definitions.
//!
//! This module defines the `VectorStore` trait as specified in SPARC Phase 2 pseudocode.
//! All vector store implementations (RuVectorStore, MockVectorStore) implement this trait.

use crate::types::{
    BatchInsertResult, CollectionStats, MetadataFilter, SearchResult, VectorId, VectorRecord,
};
use async_trait::async_trait;
use infra_errors::InfraResult;
use serde_json::Value as Json;

/// Vector store trait for similarity search operations.
///
/// From SPARC spec: This trait defines the interface for all vector store implementations.
/// The primary implementation wraps ruvector-core capabilities with:
/// - Unified error handling
/// - OpenTelemetry instrumentation (when `otel` feature enabled)
/// - Configuration via infra-config
/// - WASM support via ruvector-gnn-wasm
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Insert a vector with associated metadata.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the vector
    /// * `vector` - The vector data (f32 array)
    /// * `metadata` - Optional JSON metadata to store with the vector
    ///
    /// # Errors
    /// Returns `InfraError::Vector` if:
    /// - Dimension mismatch with collection
    /// - Duplicate ID (depending on implementation)
    /// - Storage failure
    async fn insert(
        &self,
        id: VectorId,
        vector: Vec<f32>,
        metadata: Option<Json>,
    ) -> InfraResult<()>;

    /// Batch insert multiple vectors.
    ///
    /// More efficient than individual inserts for large datasets.
    /// Uses ruvector's batch API when available.
    ///
    /// # Arguments
    /// * `vectors` - Vec of (id, vector, optional metadata) tuples
    ///
    /// # Returns
    /// `BatchInsertResult` with counts of successful and failed inserts
    async fn insert_batch(
        &self,
        vectors: Vec<(VectorId, Vec<f32>, Option<Json>)>,
    ) -> InfraResult<BatchInsertResult>;

    /// Search for similar vectors.
    ///
    /// # Arguments
    /// * `query` - Query vector for similarity search
    /// * `k` - Number of results to return
    /// * `filter` - Optional metadata filter to apply
    ///
    /// # Returns
    /// Vec of `SearchResult` ordered by similarity (highest first for cosine/dot product)
    async fn search(
        &self,
        query: Vec<f32>,
        k: usize,
        filter: Option<MetadataFilter>,
    ) -> InfraResult<Vec<SearchResult>>;

    /// Get a vector by ID.
    ///
    /// # Arguments
    /// * `id` - The vector ID to retrieve
    ///
    /// # Returns
    /// `Some(VectorRecord)` if found, `None` if not found
    async fn get(&self, id: &VectorId) -> InfraResult<Option<VectorRecord>>;

    /// Delete a vector by ID.
    ///
    /// # Arguments
    /// * `id` - The vector ID to delete
    ///
    /// # Returns
    /// `true` if vector was found and deleted, `false` if not found
    async fn delete(&self, id: &VectorId) -> InfraResult<bool>;

    /// Update vector metadata.
    ///
    /// # Arguments
    /// * `id` - The vector ID to update
    /// * `metadata` - New metadata to set (replaces existing)
    ///
    /// # Errors
    /// Returns error if vector ID not found
    async fn update_metadata(&self, id: &VectorId, metadata: Json) -> InfraResult<()>;

    /// Get collection statistics.
    ///
    /// # Returns
    /// `CollectionStats` with total vectors, dimensions, index size, etc.
    async fn stats(&self) -> InfraResult<CollectionStats>;

    /// Check if a vector exists by ID.
    ///
    /// Default implementation uses `get()`, but implementations may override
    /// for better performance.
    async fn exists(&self, id: &VectorId) -> InfraResult<bool> {
        Ok(self.get(id).await?.is_some())
    }

    /// Clear all vectors from the collection.
    ///
    /// # Warning
    /// This operation is destructive and cannot be undone.
    async fn clear(&self) -> InfraResult<()>;

    /// Get the collection name.
    fn collection_name(&self) -> &str;

    /// Get the vector dimensions for this collection.
    fn dimensions(&self) -> usize;
}

// Tests for VectorStore trait implementations are in their respective modules (e.g., store.rs)
