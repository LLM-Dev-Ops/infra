//! Vector type and operations.

use infra_errors::{InfraError, InfraResult, VectorOperation};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Vector error
#[derive(Debug, Clone, thiserror::Error)]
pub enum VectorError {
    #[error("Dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },

    #[error("Zero norm: cannot normalize zero vector")]
    ZeroNorm,

    #[error("Empty vector")]
    Empty,
}

/// A dense vector
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vector {
    data: Vec<f32>,
}

impl Vector {
    /// Create a new vector from data
    pub fn new(data: Vec<f32>) -> Self {
        Self { data }
    }

    /// Create a zero vector
    pub fn zeros(dim: usize) -> Self {
        Self {
            data: vec![0.0; dim],
        }
    }

    /// Create a ones vector
    pub fn ones(dim: usize) -> Self {
        Self {
            data: vec![1.0; dim],
        }
    }

    /// Create a random vector (uniform distribution)
    pub fn random(dim: usize) -> Self {
        let mut rng = rand::thread_rng();
        let data: Vec<f32> = (0..dim).map(|_| rng.gen::<f32>() * 2.0 - 1.0).collect();
        Self { data }
    }

    /// Create a random unit vector
    pub fn random_unit(dim: usize) -> Self {
        let v = Self::random(dim);
        v.normalize().unwrap_or_else(|_| Self::random_unit(dim))
    }

    /// Get the dimension
    pub fn dim(&self) -> usize {
        self.data.len()
    }

    /// Get the data
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// Get mutable data
    pub fn data_mut(&mut self) -> &mut [f32] {
        &mut self.data
    }

    /// Check if vectors have the same dimension
    pub fn same_dim(&self, other: &Vector) -> bool {
        self.data.len() == other.data.len()
    }

    /// Compute the L2 norm
    pub fn norm(&self) -> f32 {
        self.data.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    /// Compute the squared L2 norm
    pub fn norm_squared(&self) -> f32 {
        self.data.iter().map(|x| x * x).sum()
    }

    /// Normalize to unit length
    pub fn normalize(&self) -> InfraResult<Vector> {
        let norm = self.norm();
        if norm == 0.0 {
            return Err(InfraError::Vector {
                operation: VectorOperation::Update,
                message: "Cannot normalize zero vector".to_string(),
                dimensions: Some(self.dim()),
                context: None,
            });
        }
        Ok(Self {
            data: self.data.iter().map(|x| x / norm).collect(),
        })
    }

    /// Scale the vector
    pub fn scale(&self, factor: f32) -> Vector {
        Self {
            data: self.data.iter().map(|x| x * factor).collect(),
        }
    }

    /// Dot product with another vector
    pub fn dot(&self, other: &Vector) -> InfraResult<f32> {
        if !self.same_dim(other) {
            return Err(InfraError::Vector {
                operation: VectorOperation::Search,
                message: format!(
                    "Dimension mismatch: {} vs {}",
                    self.dim(),
                    other.dim()
                ),
                dimensions: Some(self.dim()),
                context: None,
            });
        }

        Ok(self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a * b)
            .sum())
    }

    /// Element-wise addition
    pub fn add(&self, other: &Vector) -> InfraResult<Vector> {
        if !self.same_dim(other) {
            return Err(InfraError::Vector {
                operation: VectorOperation::Update,
                message: format!(
                    "Dimension mismatch: {} vs {}",
                    self.dim(),
                    other.dim()
                ),
                dimensions: Some(self.dim()),
                context: None,
            });
        }

        Ok(Self {
            data: self
                .data
                .iter()
                .zip(other.data.iter())
                .map(|(a, b)| a + b)
                .collect(),
        })
    }

    /// Element-wise subtraction
    pub fn sub(&self, other: &Vector) -> InfraResult<Vector> {
        if !self.same_dim(other) {
            return Err(InfraError::Vector {
                operation: VectorOperation::Update,
                message: format!(
                    "Dimension mismatch: {} vs {}",
                    self.dim(),
                    other.dim()
                ),
                dimensions: Some(self.dim()),
                context: None,
            });
        }

        Ok(Self {
            data: self
                .data
                .iter()
                .zip(other.data.iter())
                .map(|(a, b)| a - b)
                .collect(),
        })
    }

    /// Compute the mean of vector elements
    pub fn mean(&self) -> f32 {
        if self.data.is_empty() {
            return 0.0;
        }
        self.data.iter().sum::<f32>() / self.data.len() as f32
    }

    /// Compute the sum of vector elements
    pub fn sum(&self) -> f32 {
        self.data.iter().sum()
    }

    /// Get the minimum element
    pub fn min(&self) -> Option<f32> {
        self.data.iter().cloned().reduce(f32::min)
    }

    /// Get the maximum element
    pub fn max(&self) -> Option<f32> {
        self.data.iter().cloned().reduce(f32::max)
    }
}

impl From<Vec<f32>> for Vector {
    fn from(data: Vec<f32>) -> Self {
        Self::new(data)
    }
}

impl From<&[f32]> for Vector {
    fn from(data: &[f32]) -> Self {
        Self::new(data.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_creation() {
        let v = Vector::new(vec![1.0, 2.0, 3.0]);
        assert_eq!(v.dim(), 3);
        assert_eq!(v.data(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_vector_norm() {
        let v = Vector::new(vec![3.0, 4.0]);
        assert!((v.norm() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_vector_normalize() {
        let v = Vector::new(vec![3.0, 4.0]);
        let n = v.normalize().unwrap();
        assert!((n.norm() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vector_dot() {
        let a = Vector::new(vec![1.0, 2.0, 3.0]);
        let b = Vector::new(vec![4.0, 5.0, 6.0]);
        let dot = a.dot(&b).unwrap();
        assert!((dot - 32.0).abs() < 1e-6);
    }

    #[test]
    fn test_vector_add() {
        let a = Vector::new(vec![1.0, 2.0]);
        let b = Vector::new(vec![3.0, 4.0]);
        let c = a.add(&b).unwrap();
        assert_eq!(c.data(), &[4.0, 6.0]);
    }
}
