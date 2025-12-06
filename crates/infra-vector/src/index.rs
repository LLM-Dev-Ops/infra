//! Vector index for similarity search.

use crate::similarity::Similarity;
use crate::vector::Vector;
use infra_errors::{InfraError, InfraResult, VectorOperation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// ID of the vector
    pub id: String,
    /// Similarity score
    pub score: f32,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Index configuration
#[derive(Debug, Clone)]
pub struct IndexConfig {
    /// Vector dimension
    pub dimension: usize,
    /// Similarity metric
    pub similarity: Similarity,
    /// Whether to normalize vectors
    pub normalize: bool,
}

impl IndexConfig {
    /// Create a new config
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            similarity: Similarity::Cosine,
            normalize: true,
        }
    }

    /// Set similarity metric
    pub fn similarity(mut self, similarity: Similarity) -> Self {
        self.similarity = similarity;
        self
    }

    /// Set normalization
    pub fn normalize(mut self, normalize: bool) -> Self {
        self.normalize = normalize;
        self
    }
}

/// Vector entry in the index
#[derive(Debug, Clone)]
struct VectorEntry {
    vector: Vector,
    metadata: Option<serde_json::Value>,
}

/// Simple vector index (brute-force)
pub struct VectorIndex {
    config: IndexConfig,
    vectors: HashMap<String, VectorEntry>,
}

impl VectorIndex {
    /// Create a new index
    pub fn new(config: IndexConfig) -> Self {
        Self {
            config,
            vectors: HashMap::new(),
        }
    }

    /// Get the dimension
    pub fn dimension(&self) -> usize {
        self.config.dimension
    }

    /// Get the number of vectors
    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    /// Insert a vector
    pub fn insert(
        &mut self,
        id: impl Into<String>,
        vector: Vector,
        metadata: Option<serde_json::Value>,
    ) -> InfraResult<()> {
        if vector.dim() != self.config.dimension {
            return Err(InfraError::Vector {
                operation: VectorOperation::Index,
                message: format!(
                    "Dimension mismatch: expected {}, got {}",
                    self.config.dimension,
                    vector.dim()
                ),
                dimensions: Some(self.config.dimension),
                context: None,
            });
        }

        let vector = if self.config.normalize {
            vector.normalize()?
        } else {
            vector
        };

        self.vectors.insert(
            id.into(),
            VectorEntry { vector, metadata },
        );

        Ok(())
    }

    /// Remove a vector
    pub fn remove(&mut self, id: &str) -> bool {
        self.vectors.remove(id).is_some()
    }

    /// Get a vector by ID
    pub fn get(&self, id: &str) -> Option<&Vector> {
        self.vectors.get(id).map(|e| &e.vector)
    }

    /// Check if a vector exists
    pub fn contains(&self, id: &str) -> bool {
        self.vectors.contains_key(id)
    }

    /// Search for similar vectors
    pub fn search(&self, query: &Vector, k: usize) -> InfraResult<Vec<SearchResult>> {
        if query.dim() != self.config.dimension {
            return Err(InfraError::Vector {
                operation: VectorOperation::Search,
                message: format!(
                    "Dimension mismatch: expected {}, got {}",
                    self.config.dimension,
                    query.dim()
                ),
                dimensions: Some(self.config.dimension),
                context: None,
            });
        }

        let query = if self.config.normalize {
            query.normalize()?
        } else {
            query.clone()
        };

        let mut results: Vec<SearchResult> = self
            .vectors
            .iter()
            .map(|(id, entry)| {
                let score = self
                    .config
                    .similarity
                    .compute(&query, &entry.vector)
                    .unwrap_or(f32::NEG_INFINITY);
                SearchResult {
                    id: id.clone(),
                    score,
                    metadata: entry.metadata.clone(),
                }
            })
            .collect();

        // Sort by score (descending for similarity, ascending for distance)
        match self.config.similarity {
            Similarity::Cosine | Similarity::DotProduct => {
                results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            }
            Similarity::Euclidean | Similarity::Manhattan => {
                results.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
            }
        }

        results.truncate(k);
        Ok(results)
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.vectors.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_index_insert_search() {
        let config = IndexConfig::new(3);
        let mut index = VectorIndex::new(config);

        index
            .insert("a", Vector::new(vec![1.0, 0.0, 0.0]), None)
            .unwrap();
        index
            .insert("b", Vector::new(vec![0.0, 1.0, 0.0]), None)
            .unwrap();
        index
            .insert(
                "c",
                Vector::new(vec![1.0, 1.0, 0.0]),
                Some(json!({"label": "c"})),
            )
            .unwrap();

        let query = Vector::new(vec![1.0, 0.0, 0.0]);
        let results = index.search(&query, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "a");
    }

    #[test]
    fn test_index_remove() {
        let config = IndexConfig::new(3);
        let mut index = VectorIndex::new(config);

        index
            .insert("a", Vector::new(vec![1.0, 0.0, 0.0]), None)
            .unwrap();

        assert!(index.contains("a"));
        assert!(index.remove("a"));
        assert!(!index.contains("a"));
    }
}
