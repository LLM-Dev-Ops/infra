//! Vector operations and embeddings for LLM-Dev-Ops infrastructure.
//!
//! This crate provides vector operations, similarity calculations,
//! and embedding utilities for LLM applications.

mod vector;
mod similarity;
mod index;
mod embedding;

pub use vector::{Vector, VectorError};
pub use similarity::{Similarity, cosine_similarity, euclidean_distance, dot_product};
pub use index::{VectorIndex, IndexConfig, SearchResult};
pub use embedding::{Embedding, EmbeddingModel};

#[cfg(feature = "wasm")]
mod wasm;

#[cfg(feature = "wasm")]
pub use wasm::*;

use infra_errors::InfraResult;

/// Create a zero vector of the given dimension
pub fn zeros(dim: usize) -> Vector {
    Vector::zeros(dim)
}

/// Create a random unit vector of the given dimension
pub fn random_unit(dim: usize) -> Vector {
    Vector::random_unit(dim)
}

/// Normalize a vector to unit length
pub fn normalize(v: &Vector) -> InfraResult<Vector> {
    v.normalize()
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
}
