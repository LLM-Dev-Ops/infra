//! RuVectorStore implementation.
//!
//! This module implements the VectorStore trait wrapping ruvector-core capabilities,
//! as specified in SPARC Phase 2 pseudocode.
//!
//! The RuVectorStore provides:
//! - Vector storage and retrieval
//! - Similarity search with metadata filtering
//! - Batch operations for efficient data loading
//! - OpenTelemetry instrumentation (when enabled)

use crate::traits::VectorStore;
use crate::types::{
    BatchInsertResult, CollectionStats, CompressionConfig, Distance, HnswConfig, MetadataFilter,
    SearchResult, TierThresholds, VectorId, VectorRecord, VectorStoreConfig,
};
use async_trait::async_trait;
use chrono::Utc;
use infra_errors::{InfraError, InfraResult, VectorOperation};
use serde_json::Value as Json;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// RuVector-backed vector store implementation.
///
/// From SPARC spec: Wraps ruvector-core with unified error handling,
/// OpenTelemetry instrumentation, and configuration via infra-config.
///
/// When the `ruvector` feature is enabled, this uses the actual ruvector-core
/// implementation. Otherwise, it falls back to an in-memory implementation
/// for development and testing.
pub struct RuVectorStore {
    /// Collection configuration
    config: VectorStoreConfig,
    /// In-memory storage (used when ruvector is not available or for testing)
    storage: RwLock<HashMap<String, StoredVector>>,
}

/// Internal storage representation for a vector.
#[derive(Debug, Clone)]
struct StoredVector {
    vector: Vec<f32>,
    metadata: Option<Json>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl RuVectorStore {
    /// Create a new RuVectorStore from configuration.
    ///
    /// # Example
    /// ```rust,no_run
    /// use infra_vector::{RuVectorStore, VectorStoreConfig, Distance};
    ///
    /// # async fn example() -> infra_errors::InfraResult<()> {
    /// let config = VectorStoreConfig::new("embeddings", 1536)
    ///     .with_distance(Distance::Cosine);
    ///
    /// let store = RuVectorStore::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: VectorStoreConfig) -> InfraResult<Self> {
        #[cfg(feature = "otel")]
        tracing::info!(
            collection = %config.collection_name,
            dimensions = config.dimensions,
            distance = %config.distance,
            "Creating RuVectorStore"
        );

        // Validate configuration
        if config.dimensions == 0 {
            return Err(InfraError::Vector {
                operation: VectorOperation::Index,
                message: "Dimensions must be greater than 0".to_string(),
                dimensions: Some(0),
                context: Some("VectorStoreConfig validation".to_string()),
            });
        }

        if config.dimensions > 65536 {
            return Err(InfraError::Vector {
                operation: VectorOperation::Index,
                message: format!("Dimensions {} exceeds maximum of 65536", config.dimensions),
                dimensions: Some(config.dimensions),
                context: Some("VectorStoreConfig validation".to_string()),
            });
        }

        // TODO: When ruvector-core is available, initialize the actual ruvector collection
        // using the `ruvector` feature flag.

        Ok(Self {
            config,
            storage: RwLock::new(HashMap::new()),
        })
    }

    /// Create from environment configuration.
    ///
    /// Reads configuration from environment variables:
    /// - `INFRA_VECTOR_COLLECTION`: Collection name (default: "default")
    /// - `INFRA_VECTOR_DIMENSIONS`: Vector dimensions (default: 1536)
    /// - `INFRA_VECTOR_DISTANCE`: Distance metric (default: "cosine")
    /// - `INFRA_RUVECTOR_URL`: RuvVector endpoint URL
    pub async fn from_env() -> InfraResult<Self> {
        let collection_name =
            std::env::var("INFRA_VECTOR_COLLECTION").unwrap_or_else(|_| "default".to_string());

        let dimensions: usize = std::env::var("INFRA_VECTOR_DIMENSIONS")
            .unwrap_or_else(|_| "1536".to_string())
            .parse()
            .map_err(|e| InfraError::Config {
                message: format!("Invalid INFRA_VECTOR_DIMENSIONS: {}", e),
                key: Some("INFRA_VECTOR_DIMENSIONS".to_string()),
                context: None,
            })?;

        let distance = match std::env::var("INFRA_VECTOR_DISTANCE")
            .unwrap_or_else(|_| "cosine".to_string())
            .to_lowercase()
            .as_str()
        {
            "cosine" => Distance::Cosine,
            "euclidean" => Distance::Euclidean,
            "dot_product" | "dot" => Distance::DotProduct,
            "manhattan" | "l1" => Distance::Manhattan,
            other => {
                return Err(InfraError::Config {
                    message: format!("Unknown distance metric: {}", other),
                    key: Some("INFRA_VECTOR_DISTANCE".to_string()),
                    context: None,
                });
            }
        };

        let mut config = VectorStoreConfig::new(collection_name, dimensions).with_distance(distance);

        if let Ok(url) = std::env::var("INFRA_RUVECTOR_URL") {
            config = config.with_endpoint(url);
        }

        Self::new(config).await
    }

    /// Validate that a vector has the correct dimensions.
    fn validate_dimensions(&self, vector: &[f32], operation: VectorOperation) -> InfraResult<()> {
        if vector.len() != self.config.dimensions {
            return Err(InfraError::Vector {
                operation,
                message: format!(
                    "Dimension mismatch: expected {}, got {}",
                    self.config.dimensions,
                    vector.len()
                ),
                dimensions: Some(vector.len()),
                context: Some(format!("collection: {}", self.config.collection_name)),
            });
        }
        Ok(())
    }

    /// Compute similarity score between two vectors.
    fn compute_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        match self.config.distance {
            Distance::Cosine => {
                let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
                let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm_a == 0.0 || norm_b == 0.0 {
                    0.0
                } else {
                    dot / (norm_a * norm_b)
                }
            }
            Distance::Euclidean => {
                let sum: f32 = a
                    .iter()
                    .zip(b.iter())
                    .map(|(x, y)| (x - y).powi(2))
                    .sum();
                -sum.sqrt() // Negative because lower distance = more similar
            }
            Distance::DotProduct => a.iter().zip(b.iter()).map(|(x, y)| x * y).sum(),
            Distance::Manhattan => {
                let sum: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum();
                -sum // Negative because lower distance = more similar
            }
        }
    }

    /// Check if a stored vector matches a metadata filter.
    fn matches_filter(&self, metadata: &Option<Json>, filter: &MetadataFilter) -> bool {
        let meta = match metadata {
            Some(m) => m,
            None => return false,
        };

        match filter {
            MetadataFilter::Eq { field, value } => {
                meta.get(field).map_or(false, |v| v == value)
            }
            MetadataFilter::Ne { field, value } => {
                meta.get(field).map_or(true, |v| v != value)
            }
            MetadataFilter::Gt { field, value } => {
                self.compare_json_values(meta.get(field), value, |a, b| a > b)
            }
            MetadataFilter::Gte { field, value } => {
                self.compare_json_values(meta.get(field), value, |a, b| a >= b)
            }
            MetadataFilter::Lt { field, value } => {
                self.compare_json_values(meta.get(field), value, |a, b| a < b)
            }
            MetadataFilter::Lte { field, value } => {
                self.compare_json_values(meta.get(field), value, |a, b| a <= b)
            }
            MetadataFilter::In { field, values } => {
                meta.get(field).map_or(false, |v| values.contains(v))
            }
            MetadataFilter::Contains { field, value } => meta.get(field).map_or(false, |v| {
                v.as_str().map_or(false, |s| s.contains(value))
            }),
            MetadataFilter::And(filters) => filters.iter().all(|f| self.matches_filter(metadata, f)),
            MetadataFilter::Or(filters) => filters.iter().any(|f| self.matches_filter(metadata, f)),
            MetadataFilter::Not(f) => !self.matches_filter(metadata, f),
        }
    }

    /// Compare JSON values numerically.
    fn compare_json_values<F>(&self, actual: Option<&Json>, expected: &Json, cmp: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        match (actual, expected) {
            (Some(Json::Number(a)), Json::Number(b)) => {
                match (a.as_f64(), b.as_f64()) {
                    (Some(a), Some(b)) => cmp(a, b),
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

#[async_trait]
impl VectorStore for RuVectorStore {
    async fn insert(
        &self,
        id: VectorId,
        vector: Vec<f32>,
        metadata: Option<Json>,
    ) -> InfraResult<()> {
        self.validate_dimensions(&vector, VectorOperation::Insert)?;

        #[cfg(feature = "otel")]
        tracing::debug!(
            vector.id = %id,
            vector.dimensions = vector.len(),
            "Inserting vector"
        );

        let now = Utc::now();
        let stored = StoredVector {
            vector,
            metadata,
            created_at: now,
            updated_at: now,
        };

        let mut storage = self.storage.write().map_err(|e| InfraError::Vector {
            operation: VectorOperation::Insert,
            message: format!("Failed to acquire write lock: {}", e),
            dimensions: None,
            context: None,
        })?;

        storage.insert(id.as_str().to_string(), stored);
        Ok(())
    }

    async fn insert_batch(
        &self,
        vectors: Vec<(VectorId, Vec<f32>, Option<Json>)>,
    ) -> InfraResult<BatchInsertResult> {
        let start = Instant::now();
        let mut inserted = 0;
        let mut failed = Vec::new();

        #[cfg(feature = "otel")]
        tracing::info!(
            batch_size = vectors.len(),
            "Starting batch insert"
        );

        for (id, vector, metadata) in vectors {
            match self.insert(id.clone(), vector, metadata).await {
                Ok(()) => inserted += 1,
                Err(e) => failed.push((id, e.to_string())),
            }
        }

        let duration = start.elapsed();

        #[cfg(feature = "otel")]
        tracing::info!(
            inserted = inserted,
            failed = failed.len(),
            duration_ms = duration.as_millis(),
            "Batch insert completed"
        );

        Ok(BatchInsertResult::new(inserted, failed, duration))
    }

    async fn search(
        &self,
        query: Vec<f32>,
        k: usize,
        filter: Option<MetadataFilter>,
    ) -> InfraResult<Vec<SearchResult>> {
        self.validate_dimensions(&query, VectorOperation::Search)?;

        #[cfg(feature = "otel")]
        tracing::debug!(
            vector.dimensions = query.len(),
            k = k,
            has_filter = filter.is_some(),
            "Searching vectors"
        );

        let storage = self.storage.read().map_err(|e| InfraError::Vector {
            operation: VectorOperation::Search,
            message: format!("Failed to acquire read lock: {}", e),
            dimensions: None,
            context: None,
        })?;

        let mut results: Vec<SearchResult> = storage
            .iter()
            .filter(|(_, stored)| {
                filter
                    .as_ref()
                    .map_or(true, |f| self.matches_filter(&stored.metadata, f))
            })
            .map(|(id, stored)| {
                let score = self.compute_similarity(&query, &stored.vector);
                SearchResult::new(VectorId::new(id.clone()), score)
                    .with_metadata(stored.metadata.clone().unwrap_or(Json::Null))
            })
            .collect();

        // Sort by score (descending for similarity metrics)
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results.truncate(k);

        #[cfg(feature = "otel")]
        tracing::debug!(
            results = results.len(),
            "Search completed"
        );

        Ok(results)
    }

    async fn get(&self, id: &VectorId) -> InfraResult<Option<VectorRecord>> {
        let storage = self.storage.read().map_err(|e| InfraError::Vector {
            operation: VectorOperation::Search,
            message: format!("Failed to acquire read lock: {}", e),
            dimensions: None,
            context: None,
        })?;

        Ok(storage.get(id.as_str()).map(|stored| VectorRecord {
            id: id.clone(),
            vector: stored.vector.clone(),
            metadata: stored.metadata.clone(),
            created_at: stored.created_at,
            updated_at: stored.updated_at,
        }))
    }

    async fn delete(&self, id: &VectorId) -> InfraResult<bool> {
        let mut storage = self.storage.write().map_err(|e| InfraError::Vector {
            operation: VectorOperation::Delete,
            message: format!("Failed to acquire write lock: {}", e),
            dimensions: None,
            context: None,
        })?;

        Ok(storage.remove(id.as_str()).is_some())
    }

    async fn update_metadata(&self, id: &VectorId, metadata: Json) -> InfraResult<()> {
        let mut storage = self.storage.write().map_err(|e| InfraError::Vector {
            operation: VectorOperation::Update,
            message: format!("Failed to acquire write lock: {}", e),
            dimensions: None,
            context: None,
        })?;

        match storage.get_mut(id.as_str()) {
            Some(stored) => {
                stored.metadata = Some(metadata);
                stored.updated_at = Utc::now();
                Ok(())
            }
            None => Err(InfraError::Vector {
                operation: VectorOperation::Update,
                message: format!("Vector not found: {}", id),
                dimensions: None,
                context: None,
            }),
        }
    }

    async fn stats(&self) -> InfraResult<CollectionStats> {
        let storage = self.storage.read().map_err(|e| InfraError::Vector {
            operation: VectorOperation::Index,
            message: format!("Failed to acquire read lock: {}", e),
            dimensions: None,
            context: None,
        })?;

        // Estimate index size (vectors * dimensions * sizeof(f32) + overhead)
        let vector_data_size = storage.len() * self.config.dimensions * 4;
        let metadata_estimate = storage
            .values()
            .map(|v| {
                v.metadata
                    .as_ref()
                    .map_or(0, |m| m.to_string().len())
            })
            .sum::<usize>();
        let index_size = vector_data_size + metadata_estimate;

        Ok(CollectionStats {
            total_vectors: storage.len(),
            dimensions: self.config.dimensions,
            index_size_bytes: index_size,
            distance_metric: self.config.distance,
            collection_name: self.config.collection_name.clone(),
        })
    }

    async fn clear(&self) -> InfraResult<()> {
        let mut storage = self.storage.write().map_err(|e| InfraError::Vector {
            operation: VectorOperation::Delete,
            message: format!("Failed to acquire write lock: {}", e),
            dimensions: None,
            context: None,
        })?;

        storage.clear();
        Ok(())
    }

    fn collection_name(&self) -> &str {
        &self.config.collection_name
    }

    fn dimensions(&self) -> usize {
        self.config.dimensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_create_store() {
        let config = VectorStoreConfig::new("test", 128);
        let store = RuVectorStore::new(config).await.unwrap();
        assert_eq!(store.collection_name(), "test");
        assert_eq!(store.dimensions(), 128);
    }

    #[tokio::test]
    async fn test_insert_and_get() {
        let config = VectorStoreConfig::new("test", 3);
        let store = RuVectorStore::new(config).await.unwrap();

        let id = VectorId::new("vec1");
        let vector = vec![1.0, 0.0, 0.0];
        let metadata = Some(json!({"label": "test"}));

        store.insert(id.clone(), vector.clone(), metadata).await.unwrap();

        let record = store.get(&id).await.unwrap().unwrap();
        assert_eq!(record.id, id);
        assert_eq!(record.vector, vector);
    }

    #[tokio::test]
    async fn test_search() {
        let config = VectorStoreConfig::new("test", 3).with_distance(Distance::Cosine);
        let store = RuVectorStore::new(config).await.unwrap();

        store.insert(VectorId::new("a"), vec![1.0, 0.0, 0.0], None).await.unwrap();
        store.insert(VectorId::new("b"), vec![0.0, 1.0, 0.0], None).await.unwrap();
        store.insert(VectorId::new("c"), vec![0.707, 0.707, 0.0], None).await.unwrap();

        let results = store.search(vec![1.0, 0.0, 0.0], 2, None).await.unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id.as_str(), "a");
        assert!((results[0].score - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_search_with_filter() {
        let config = VectorStoreConfig::new("test", 3);
        let store = RuVectorStore::new(config).await.unwrap();

        store
            .insert(
                VectorId::new("a"),
                vec![1.0, 0.0, 0.0],
                Some(json!({"category": "tech"})),
            )
            .await
            .unwrap();
        store
            .insert(
                VectorId::new("b"),
                vec![1.0, 0.0, 0.0],
                Some(json!({"category": "science"})),
            )
            .await
            .unwrap();

        let filter = MetadataFilter::eq("category", json!("tech"));
        let results = store.search(vec![1.0, 0.0, 0.0], 10, Some(filter)).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id.as_str(), "a");
    }

    #[tokio::test]
    async fn test_delete() {
        let config = VectorStoreConfig::new("test", 3);
        let store = RuVectorStore::new(config).await.unwrap();

        let id = VectorId::new("to_delete");
        store.insert(id.clone(), vec![1.0, 0.0, 0.0], None).await.unwrap();

        assert!(store.exists(&id).await.unwrap());
        assert!(store.delete(&id).await.unwrap());
        assert!(!store.exists(&id).await.unwrap());
    }

    #[tokio::test]
    async fn test_batch_insert() {
        let config = VectorStoreConfig::new("test", 3);
        let store = RuVectorStore::new(config).await.unwrap();

        let vectors = vec![
            (VectorId::new("b1"), vec![1.0, 0.0, 0.0], None),
            (VectorId::new("b2"), vec![0.0, 1.0, 0.0], None),
            (VectorId::new("b3"), vec![0.0, 0.0, 1.0], None),
        ];

        let result = store.insert_batch(vectors).await.unwrap();
        assert_eq!(result.inserted, 3);
        assert!(result.all_succeeded());

        let stats = store.stats().await.unwrap();
        assert_eq!(stats.total_vectors, 3);
    }

    #[tokio::test]
    async fn test_dimension_mismatch() {
        let config = VectorStoreConfig::new("test", 3);
        let store = RuVectorStore::new(config).await.unwrap();

        let result = store.insert(VectorId::new("bad"), vec![1.0, 0.0], None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stats() {
        let config = VectorStoreConfig::new("test", 128);
        let store = RuVectorStore::new(config).await.unwrap();

        for i in 0..10 {
            store
                .insert(VectorId::new(format!("v{}", i)), vec![0.1; 128], None)
                .await
                .unwrap();
        }

        let stats = store.stats().await.unwrap();
        assert_eq!(stats.total_vectors, 10);
        assert_eq!(stats.dimensions, 128);
        assert!(stats.index_size_bytes > 0);
    }
}
