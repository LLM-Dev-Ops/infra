//! HTTP client and server utilities for LLM-Dev-Ops infrastructure.
//!
//! This crate provides a unified HTTP client with retry, circuit breaker,
//! and observability integration.

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "server")]
mod server;
mod request;
mod response;
mod middleware;

#[cfg(feature = "client")]
pub use client::{HttpClient, HttpClientBuilder};
#[cfg(feature = "server")]
pub use server::{ServerBuilder, Router};
pub use request::{Request, RequestBuilder};
pub use response::{Response, ResponseExt};
pub use middleware::{Middleware, MiddlewareStack};

use std::time::Duration;

/// HTTP method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl From<Method> for http::Method {
    fn from(method: Method) -> Self {
        match method {
            Method::Get => http::Method::GET,
            Method::Post => http::Method::POST,
            Method::Put => http::Method::PUT,
            Method::Delete => http::Method::DELETE,
            Method::Patch => http::Method::PATCH,
            Method::Head => http::Method::HEAD,
            Method::Options => http::Method::OPTIONS,
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before opening
    pub failure_threshold: u32,
    /// Success threshold to close
    pub success_threshold: u32,
    /// Duration to stay open
    pub open_duration: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            open_duration: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_conversion() {
        assert_eq!(http::Method::from(Method::Get), http::Method::GET);
        assert_eq!(http::Method::from(Method::Post), http::Method::POST);
    }
}
