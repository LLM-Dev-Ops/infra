//! Load balancing.

use infra_errors::{InfraError, InfraResult};
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Backend server
#[derive(Debug, Clone)]
pub struct Backend {
    /// Backend URL
    pub url: String,
    /// Weight for weighted load balancing
    pub weight: u32,
    /// Whether the backend is healthy
    pub healthy: bool,
}

impl Backend {
    /// Create a new backend
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            weight: 1,
            healthy: true,
        }
    }

    /// Set weight
    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }
}

/// Load balancing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// Round-robin
    RoundRobin,
    /// Random selection
    Random,
    /// Weighted round-robin
    Weighted,
    /// Least connections (not fully implemented)
    LeastConnections,
}

/// Load balancer
pub struct LoadBalancer {
    backends: Arc<RwLock<Vec<Backend>>>,
    strategy: Strategy,
    counter: AtomicUsize,
}

impl LoadBalancer {
    /// Create a new load balancer
    pub fn new(strategy: Strategy) -> Self {
        Self {
            backends: Arc::new(RwLock::new(Vec::new())),
            strategy,
            counter: AtomicUsize::new(0),
        }
    }

    /// Create with round-robin strategy
    pub fn round_robin() -> Self {
        Self::new(Strategy::RoundRobin)
    }

    /// Create with random strategy
    pub fn random() -> Self {
        Self::new(Strategy::Random)
    }

    /// Add a backend
    pub async fn add_backend(&self, backend: Backend) {
        let mut backends = self.backends.write().await;
        backends.push(backend);
    }

    /// Remove a backend by URL
    pub async fn remove_backend(&self, url: &str) {
        let mut backends = self.backends.write().await;
        backends.retain(|b| b.url != url);
    }

    /// Mark a backend as unhealthy
    pub async fn mark_unhealthy(&self, url: &str) {
        let mut backends = self.backends.write().await;
        if let Some(backend) = backends.iter_mut().find(|b| b.url == url) {
            backend.healthy = false;
        }
    }

    /// Mark a backend as healthy
    pub async fn mark_healthy(&self, url: &str) {
        let mut backends = self.backends.write().await;
        if let Some(backend) = backends.iter_mut().find(|b| b.url == url) {
            backend.healthy = true;
        }
    }

    /// Get the next backend
    pub async fn next(&self) -> InfraResult<Backend> {
        let backends = self.backends.read().await;
        let healthy: Vec<_> = backends.iter().filter(|b| b.healthy).collect();

        if healthy.is_empty() {
            return Err(InfraError::External {
                service: "load_balancer".to_string(),
                operation: "next".to_string(),
                message: "No healthy backends available".to_string(),
                retry_after: None,
                context: None,
            });
        }

        match self.strategy {
            Strategy::RoundRobin => {
                let idx = self.counter.fetch_add(1, Ordering::Relaxed) % healthy.len();
                Ok(healthy[idx].clone())
            }
            Strategy::Random => {
                let idx = rand::thread_rng().gen_range(0..healthy.len());
                Ok(healthy[idx].clone())
            }
            Strategy::Weighted => {
                let total_weight: u32 = healthy.iter().map(|b| b.weight).sum();
                if total_weight == 0 {
                    return Ok(healthy[0].clone());
                }

                let mut rand_weight = rand::thread_rng().gen_range(0..total_weight);
                for backend in &healthy {
                    if rand_weight < backend.weight {
                        return Ok((*backend).clone());
                    }
                    rand_weight -= backend.weight;
                }

                Ok(healthy[0].clone())
            }
            Strategy::LeastConnections => {
                // Simplified: just use round-robin for now
                let idx = self.counter.fetch_add(1, Ordering::Relaxed) % healthy.len();
                Ok(healthy[idx].clone())
            }
        }
    }

    /// Get all backends
    pub async fn backends(&self) -> Vec<Backend> {
        self.backends.read().await.clone()
    }

    /// Get healthy backend count
    pub async fn healthy_count(&self) -> usize {
        self.backends.read().await.iter().filter(|b| b.healthy).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_round_robin() {
        let balancer = LoadBalancer::round_robin();
        balancer.add_backend(Backend::new("http://server1")).await;
        balancer.add_backend(Backend::new("http://server2")).await;
        balancer.add_backend(Backend::new("http://server3")).await;

        let b1 = balancer.next().await.unwrap();
        let b2 = balancer.next().await.unwrap();
        let b3 = balancer.next().await.unwrap();
        let b4 = balancer.next().await.unwrap();

        assert_eq!(b1.url, "http://server1");
        assert_eq!(b2.url, "http://server2");
        assert_eq!(b3.url, "http://server3");
        assert_eq!(b4.url, "http://server1");
    }

    #[tokio::test]
    async fn test_unhealthy_backend() {
        let balancer = LoadBalancer::round_robin();
        balancer.add_backend(Backend::new("http://server1")).await;
        balancer.add_backend(Backend::new("http://server2")).await;

        balancer.mark_unhealthy("http://server1").await;

        // Should only return server2
        for _ in 0..5 {
            let backend = balancer.next().await.unwrap();
            assert_eq!(backend.url, "http://server2");
        }
    }

    #[tokio::test]
    async fn test_no_healthy_backends() {
        let balancer = LoadBalancer::round_robin();
        balancer.add_backend(Backend::new("http://server1")).await;
        balancer.mark_unhealthy("http://server1").await;

        let result = balancer.next().await;
        assert!(result.is_err());
    }
}
