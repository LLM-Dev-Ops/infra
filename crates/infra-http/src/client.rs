//! HTTP client with retry and circuit breaker.

use crate::{CircuitBreakerConfig, RetryConfig};
use infra_errors::{InfraError, InfraResult};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker
struct CircuitBreaker {
    state: RwLock<CircuitState>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: AtomicU64,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: AtomicU64::new(0),
            config,
        }
    }

    async fn allow_request(&self) -> bool {
        let state = *self.state.read().await;
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let last_failure = self.last_failure_time.load(Ordering::Relaxed);
                let now = Instant::now().elapsed().as_secs();
                if now - last_failure > self.config.open_duration.as_secs() {
                    let mut state = self.state.write().await;
                    *state = CircuitState::HalfOpen;
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    async fn record_success(&self) {
        let state = *self.state.read().await;
        if state == CircuitState::HalfOpen {
            let count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
            if count >= self.config.success_threshold {
                let mut state = self.state.write().await;
                *state = CircuitState::Closed;
                self.failure_count.store(0, Ordering::Relaxed);
                self.success_count.store(0, Ordering::Relaxed);
            }
        }
    }

    async fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        self.last_failure_time.store(
            Instant::now().elapsed().as_secs(),
            Ordering::Relaxed,
        );

        if count >= self.config.failure_threshold {
            let mut state = self.state.write().await;
            *state = CircuitState::Open;
        }
    }
}

/// HTTP client builder
pub struct HttpClientBuilder {
    base_url: Option<String>,
    timeout: Duration,
    retry_config: RetryConfig,
    circuit_breaker_config: Option<CircuitBreakerConfig>,
    default_headers: HashMap<String, String>,
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClientBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            base_url: None,
            timeout: Duration::from_secs(30),
            retry_config: RetryConfig::default(),
            circuit_breaker_config: None,
            default_headers: HashMap::new(),
        }
    }

    /// Set base URL
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set retry configuration
    pub fn retry(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Enable circuit breaker
    pub fn circuit_breaker(mut self, config: CircuitBreakerConfig) -> Self {
        self.circuit_breaker_config = Some(config);
        self
    }

    /// Add a default header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(name.into(), value.into());
        self
    }

    /// Build the client
    pub fn build(self) -> InfraResult<HttpClient> {
        let mut headers = HeaderMap::new();
        for (name, value) in &self.default_headers {
            let header_name = HeaderName::try_from(name.as_str()).map_err(|e| {
                InfraError::Http {
                    status: None,
                    message: format!("Invalid header name: {e}"),
                    url: None,
                    context: None,
                }
            })?;
            let header_value = HeaderValue::try_from(value.as_str()).map_err(|e| {
                InfraError::Http {
                    status: None,
                    message: format!("Invalid header value: {e}"),
                    url: None,
                    context: None,
                }
            })?;
            headers.insert(header_name, header_value);
        }

        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .default_headers(headers)
            .build()
            .map_err(|e| InfraError::Http {
                status: None,
                message: format!("Failed to build HTTP client: {e}"),
                url: None,
                context: None,
            })?;

        let circuit_breaker = self
            .circuit_breaker_config
            .map(|config| Arc::new(CircuitBreaker::new(config)));

        Ok(HttpClient {
            client,
            base_url: self.base_url,
            retry_config: self.retry_config,
            circuit_breaker,
        })
    }
}

/// HTTP client with retry and circuit breaker
pub struct HttpClient {
    client: reqwest::Client,
    base_url: Option<String>,
    retry_config: RetryConfig,
    circuit_breaker: Option<Arc<CircuitBreaker>>,
}

impl HttpClient {
    /// Create a new builder
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::new()
    }

    /// Create a simple client
    pub fn new() -> InfraResult<Self> {
        Self::builder().build()
    }

    /// Build the full URL
    fn build_url(&self, path: &str) -> String {
        if let Some(base) = &self.base_url {
            format!("{}{}", base.trim_end_matches('/'), path)
        } else {
            path.to_string()
        }
    }

    /// Execute a request with retry
    async fn execute_with_retry(
        &self,
        request_builder: reqwest::RequestBuilder,
    ) -> InfraResult<reqwest::Response> {
        // Check circuit breaker
        if let Some(cb) = &self.circuit_breaker {
            if !cb.allow_request().await {
                return Err(InfraError::Http {
                    status: Some(503),
                    message: "Circuit breaker is open".to_string(),
                    url: None,
                    context: None,
                });
            }
        }

        let mut attempts = 0;
        let mut delay = self.retry_config.initial_delay;

        loop {
            attempts += 1;

            // Clone the request builder for retry
            let request = request_builder
                .try_clone()
                .ok_or_else(|| InfraError::Http {
                    status: None,
                    message: "Request body cannot be cloned for retry".to_string(),
                    url: None,
                    context: None,
                })?;

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Some(cb) = &self.circuit_breaker {
                            cb.record_success().await;
                        }
                        return Ok(response);
                    }

                    let status = response.status().as_u16();

                    // Don't retry client errors (4xx)
                    if status >= 400 && status < 500 {
                        return Err(InfraError::Http {
                            status: Some(status),
                            message: format!("HTTP error: {}", response.status()),
                            url: None,
                            context: None,
                        });
                    }

                    // Retry server errors (5xx)
                    if attempts > self.retry_config.max_retries {
                        if let Some(cb) = &self.circuit_breaker {
                            cb.record_failure().await;
                        }
                        return Err(InfraError::Http {
                            status: Some(status),
                            message: format!("HTTP error after {} retries: {}", attempts, response.status()),
                            url: None,
                            context: None,
                        });
                    }
                }
                Err(e) => {
                    if attempts > self.retry_config.max_retries {
                        if let Some(cb) = &self.circuit_breaker {
                            cb.record_failure().await;
                        }
                        return Err(InfraError::Http {
                            status: None,
                            message: format!("Request failed after {} retries: {}", attempts, e),
                            url: None,
                            context: None,
                        });
                    }
                }
            }

            // Wait before retry
            tokio::time::sleep(delay).await;
            delay = std::cmp::min(
                Duration::from_secs_f64(delay.as_secs_f64() * self.retry_config.multiplier),
                self.retry_config.max_delay,
            );
        }
    }

    /// Send a GET request
    pub async fn get(&self, path: &str) -> InfraResult<reqwest::Response> {
        let url = self.build_url(path);
        let request = self.client.get(&url);
        self.execute_with_retry(request).await
    }

    /// Send a POST request with JSON body
    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> InfraResult<reqwest::Response> {
        let url = self.build_url(path);
        let request = self.client.post(&url).json(body);
        self.execute_with_retry(request).await
    }

    /// Send a PUT request with JSON body
    pub async fn put<T: Serialize>(&self, path: &str, body: &T) -> InfraResult<reqwest::Response> {
        let url = self.build_url(path);
        let request = self.client.put(&url).json(body);
        self.execute_with_retry(request).await
    }

    /// Send a DELETE request
    pub async fn delete(&self, path: &str) -> InfraResult<reqwest::Response> {
        let url = self.build_url(path);
        let request = self.client.delete(&url);
        self.execute_with_retry(request).await
    }

    /// Send a GET request and parse JSON response
    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> InfraResult<T> {
        let response = self.get(path).await?;
        response.json().await.map_err(|e| InfraError::Http {
            status: None,
            message: format!("Failed to parse JSON response: {e}"),
            url: Some(self.build_url(path)),
            context: None,
        })
    }

    /// Send a POST request and parse JSON response
    pub async fn post_json<B: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> InfraResult<R> {
        let response = self.post(path, body).await?;
        response.json().await.map_err(|e| InfraError::Http {
            status: None,
            message: format!("Failed to parse JSON response: {e}"),
            url: Some(self.build_url(path)),
            context: None,
        })
    }
}
