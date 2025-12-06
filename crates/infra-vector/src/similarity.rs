//! Similarity metrics.

use crate::vector::Vector;
use infra_errors::{InfraError, InfraResult, VectorOperation};

/// Similarity metric
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Similarity {
    /// Cosine similarity
    Cosine,
    /// Euclidean distance
    Euclidean,
    /// Dot product
    DotProduct,
    /// Manhattan distance (L1)
    Manhattan,
}

impl Similarity {
    /// Compute similarity/distance between two vectors
    pub fn compute(&self, a: &Vector, b: &Vector) -> InfraResult<f32> {
        match self {
            Self::Cosine => cosine_similarity(a, b),
            Self::Euclidean => euclidean_distance(a, b),
            Self::DotProduct => dot_product(a, b),
            Self::Manhattan => manhattan_distance(a, b),
        }
    }
}

/// Compute cosine similarity between two vectors
pub fn cosine_similarity(a: &Vector, b: &Vector) -> InfraResult<f32> {
    if !a.same_dim(b) {
        return Err(InfraError::Vector {
            operation: VectorOperation::Search,
            message: format!("Dimension mismatch: {} vs {}", a.dim(), b.dim()),
            dimensions: Some(a.dim()),
            context: None,
        });
    }

    let dot = a.dot(b)?;
    let norm_a = a.norm();
    let norm_b = b.norm();

    if norm_a == 0.0 || norm_b == 0.0 {
        return Ok(0.0);
    }

    Ok(dot / (norm_a * norm_b))
}

/// Compute Euclidean distance between two vectors
pub fn euclidean_distance(a: &Vector, b: &Vector) -> InfraResult<f32> {
    if !a.same_dim(b) {
        return Err(InfraError::Vector {
            operation: VectorOperation::Search,
            message: format!("Dimension mismatch: {} vs {}", a.dim(), b.dim()),
            dimensions: Some(a.dim()),
            context: None,
        });
    }

    let sum: f32 = a
        .data()
        .iter()
        .zip(b.data().iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum();

    Ok(sum.sqrt())
}

/// Compute dot product between two vectors
pub fn dot_product(a: &Vector, b: &Vector) -> InfraResult<f32> {
    a.dot(b)
}

/// Compute Manhattan distance (L1) between two vectors
pub fn manhattan_distance(a: &Vector, b: &Vector) -> InfraResult<f32> {
    if !a.same_dim(b) {
        return Err(InfraError::Vector {
            operation: VectorOperation::Search,
            message: format!("Dimension mismatch: {} vs {}", a.dim(), b.dim()),
            dimensions: Some(a.dim()),
            context: None,
        });
    }

    let sum: f32 = a
        .data()
        .iter()
        .zip(b.data().iter())
        .map(|(x, y)| (x - y).abs())
        .sum();

    Ok(sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = Vector::new(vec![1.0, 0.0]);
        let b = Vector::new(vec![1.0, 0.0]);
        let sim = cosine_similarity(&a, &b).unwrap();
        assert!((sim - 1.0).abs() < 1e-6);

        let c = Vector::new(vec![0.0, 1.0]);
        let sim = cosine_similarity(&a, &c).unwrap();
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = Vector::new(vec![0.0, 0.0]);
        let b = Vector::new(vec![3.0, 4.0]);
        let dist = euclidean_distance(&a, &b).unwrap();
        assert!((dist - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_manhattan_distance() {
        let a = Vector::new(vec![0.0, 0.0]);
        let b = Vector::new(vec![3.0, 4.0]);
        let dist = manhattan_distance(&a, &b).unwrap();
        assert!((dist - 7.0).abs() < 1e-6);
    }
}
