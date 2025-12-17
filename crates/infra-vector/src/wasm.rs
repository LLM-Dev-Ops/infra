//! WASM bindings for vector operations.
//!
//! This module provides WebAssembly bindings for the infra-vector crate,
//! as specified in SPARC Phase 2 pseudocode.
//!
//! When the `wasm` feature is enabled, these bindings allow using vector
//! operations from JavaScript/TypeScript in browser or Node.js environments.

#![cfg(feature = "wasm")]

use wasm_bindgen::prelude::*;
use crate::types::{Distance, HnswConfig, VectorStoreConfig};
use crate::similarity;
use crate::vector::Vector;

/// JavaScript-compatible vector store for browser/WASM usage.
///
/// From SPARC spec: WASM support via ruvector-gnn-wasm.
/// This provides a simplified interface for vector operations in JavaScript.
#[wasm_bindgen]
pub struct JsVectorStore {
    dimensions: usize,
    distance: Distance,
    vectors: std::collections::HashMap<String, JsStoredVector>,
}

#[derive(Clone)]
struct JsStoredVector {
    data: Vec<f32>,
    metadata: Option<String>,
}

#[wasm_bindgen]
impl JsVectorStore {
    /// Create a new vector store.
    ///
    /// # Arguments
    /// * `dimensions` - Vector dimensionality
    ///
    /// # Returns
    /// A new JsVectorStore instance
    #[wasm_bindgen(constructor)]
    pub fn new(dimensions: usize) -> Result<JsVectorStore, JsValue> {
        if dimensions == 0 {
            return Err(JsValue::from_str("Dimensions must be greater than 0"));
        }
        if dimensions > 65536 {
            return Err(JsValue::from_str("Dimensions exceeds maximum of 65536"));
        }

        Ok(Self {
            dimensions,
            distance: Distance::Cosine,
            vectors: std::collections::HashMap::new(),
        })
    }

    /// Set the distance metric.
    ///
    /// # Arguments
    /// * `metric` - One of: "cosine", "euclidean", "dot_product", "manhattan"
    #[wasm_bindgen(js_name = setDistance)]
    pub fn set_distance(&mut self, metric: &str) -> Result<(), JsValue> {
        self.distance = match metric {
            "cosine" => Distance::Cosine,
            "euclidean" => Distance::Euclidean,
            "dot_product" | "dot" => Distance::DotProduct,
            "manhattan" | "l1" => Distance::Manhattan,
            _ => return Err(JsValue::from_str(&format!("Unknown distance metric: {}", metric))),
        };
        Ok(())
    }

    /// Insert a vector into the store.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the vector
    /// * `vector` - Float32Array of vector data
    /// * `metadata` - Optional JSON string of metadata
    pub fn insert(
        &mut self,
        id: &str,
        vector: &[f32],
        metadata: Option<String>,
    ) -> Result<(), JsValue> {
        if vector.len() != self.dimensions {
            return Err(JsValue::from_str(&format!(
                "Dimension mismatch: expected {}, got {}",
                self.dimensions,
                vector.len()
            )));
        }

        self.vectors.insert(
            id.to_string(),
            JsStoredVector {
                data: vector.to_vec(),
                metadata,
            },
        );

        Ok(())
    }

    /// Search for similar vectors.
    ///
    /// # Arguments
    /// * `query` - Query vector (Float32Array)
    /// * `k` - Number of results to return
    ///
    /// # Returns
    /// JSON string array of results: [{id, score, metadata}, ...]
    pub fn search(&self, query: &[f32], k: usize) -> Result<String, JsValue> {
        if query.len() != self.dimensions {
            return Err(JsValue::from_str(&format!(
                "Dimension mismatch: expected {}, got {}",
                self.dimensions,
                query.len()
            )));
        }

        let query_vec = Vector::new(query.to_vec());
        let mut results: Vec<(String, f32, Option<String>)> = self
            .vectors
            .iter()
            .map(|(id, stored)| {
                let stored_vec = Vector::new(stored.data.clone());
                let score = match self.distance {
                    Distance::Cosine => {
                        similarity::cosine_similarity(&query_vec, &stored_vec).unwrap_or(0.0)
                    }
                    Distance::Euclidean => {
                        -similarity::euclidean_distance(&query_vec, &stored_vec).unwrap_or(f32::MAX)
                    }
                    Distance::DotProduct => {
                        similarity::dot_product(&query_vec, &stored_vec).unwrap_or(0.0)
                    }
                    Distance::Manhattan => {
                        -similarity::manhattan_distance(&query_vec, &stored_vec).unwrap_or(f32::MAX)
                    }
                };
                (id.clone(), score, stored.metadata.clone())
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);

        // Convert to JSON
        let json_results: Vec<serde_json::Value> = results
            .into_iter()
            .map(|(id, score, metadata)| {
                serde_json::json!({
                    "id": id,
                    "score": score,
                    "metadata": metadata.and_then(|m| serde_json::from_str::<serde_json::Value>(&m).ok())
                })
            })
            .collect();

        serde_json::to_string(&json_results)
            .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
    }

    /// Delete a vector by ID.
    ///
    /// # Arguments
    /// * `id` - Vector ID to delete
    ///
    /// # Returns
    /// true if vector was found and deleted
    pub fn delete(&mut self, id: &str) -> bool {
        self.vectors.remove(id).is_some()
    }

    /// Get the number of vectors in the store.
    #[wasm_bindgen(getter)]
    pub fn count(&self) -> usize {
        self.vectors.len()
    }

    /// Get the vector dimensions.
    #[wasm_bindgen(getter)]
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Clear all vectors from the store.
    pub fn clear(&mut self) {
        self.vectors.clear();
    }

    /// Check if a vector exists.
    pub fn exists(&self, id: &str) -> bool {
        self.vectors.contains_key(id)
    }

    /// Get a vector by ID.
    ///
    /// # Returns
    /// JSON string of the vector record, or null if not found
    pub fn get(&self, id: &str) -> Option<String> {
        self.vectors.get(id).map(|stored| {
            serde_json::json!({
                "id": id,
                "vector": stored.data,
                "metadata": stored.metadata.as_ref().and_then(|m| serde_json::from_str::<serde_json::Value>(m).ok())
            })
            .to_string()
        })
    }
}

/// Compute cosine similarity between two vectors.
///
/// # Arguments
/// * `a` - First vector (Float32Array)
/// * `b` - Second vector (Float32Array)
///
/// # Returns
/// Cosine similarity score (-1 to 1)
#[wasm_bindgen(js_name = cosineSimilarity)]
pub fn js_cosine_similarity(a: &[f32], b: &[f32]) -> Result<f32, JsValue> {
    if a.len() != b.len() {
        return Err(JsValue::from_str(&format!(
            "Dimension mismatch: {} vs {}",
            a.len(),
            b.len()
        )));
    }

    let vec_a = Vector::new(a.to_vec());
    let vec_b = Vector::new(b.to_vec());

    similarity::cosine_similarity(&vec_a, &vec_b)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Compute Euclidean distance between two vectors.
///
/// # Arguments
/// * `a` - First vector (Float32Array)
/// * `b` - Second vector (Float32Array)
///
/// # Returns
/// Euclidean (L2) distance
#[wasm_bindgen(js_name = euclideanDistance)]
pub fn js_euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32, JsValue> {
    if a.len() != b.len() {
        return Err(JsValue::from_str(&format!(
            "Dimension mismatch: {} vs {}",
            a.len(),
            b.len()
        )));
    }

    let vec_a = Vector::new(a.to_vec());
    let vec_b = Vector::new(b.to_vec());

    similarity::euclidean_distance(&vec_a, &vec_b)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Compute dot product between two vectors.
///
/// # Arguments
/// * `a` - First vector (Float32Array)
/// * `b` - Second vector (Float32Array)
///
/// # Returns
/// Dot product (inner product)
#[wasm_bindgen(js_name = dotProduct)]
pub fn js_dot_product(a: &[f32], b: &[f32]) -> Result<f32, JsValue> {
    if a.len() != b.len() {
        return Err(JsValue::from_str(&format!(
            "Dimension mismatch: {} vs {}",
            a.len(),
            b.len()
        )));
    }

    let vec_a = Vector::new(a.to_vec());
    let vec_b = Vector::new(b.to_vec());

    similarity::dot_product(&vec_a, &vec_b)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Normalize a vector to unit length.
///
/// # Arguments
/// * `vector` - Input vector (Float32Array)
///
/// # Returns
/// Normalized vector (Float32Array)
#[wasm_bindgen(js_name = normalizeVector)]
pub fn js_normalize_vector(vector: &[f32]) -> Result<Vec<f32>, JsValue> {
    let vec = Vector::new(vector.to_vec());
    vec.normalize()
        .map(|v| v.data().to_vec())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Create a random unit vector.
///
/// # Arguments
/// * `dimensions` - Vector dimensionality
///
/// # Returns
/// Random unit vector (Float32Array)
#[wasm_bindgen(js_name = randomUnitVector)]
pub fn js_random_unit_vector(dimensions: usize) -> Vec<f32> {
    Vector::random_unit(dimensions).data().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_js_vector_store_creation() {
        let store = JsVectorStore::new(128).unwrap();
        assert_eq!(store.dimensions(), 128);
        assert_eq!(store.count(), 0);
    }

    #[wasm_bindgen_test]
    fn test_js_vector_store_insert_search() {
        let mut store = JsVectorStore::new(3).unwrap();

        store.insert("a", &[1.0, 0.0, 0.0], None).unwrap();
        store.insert("b", &[0.0, 1.0, 0.0], None).unwrap();

        assert_eq!(store.count(), 2);

        let results = store.search(&[1.0, 0.0, 0.0], 1).unwrap();
        assert!(results.contains("\"id\":\"a\""));
    }

    #[wasm_bindgen_test]
    fn test_js_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];

        let sim = js_cosine_similarity(&a, &b).unwrap();
        assert!((sim - 1.0).abs() < 0.001);
    }
}
