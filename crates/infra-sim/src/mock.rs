//! Mock service utilities.

use async_trait::async_trait;
use infra_errors::InfraResult;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock response
#[derive(Debug, Clone)]
pub struct MockResponse {
    /// Response body
    pub body: Vec<u8>,
    /// Response status
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Delay before responding
    pub delay: Option<std::time::Duration>,
}

impl MockResponse {
    /// Create a success response
    pub fn ok(body: impl Into<Vec<u8>>) -> Self {
        Self {
            body: body.into(),
            status: 200,
            headers: HashMap::new(),
            delay: None,
        }
    }

    /// Create an error response
    pub fn error(status: u16, message: &str) -> Self {
        Self {
            body: message.as_bytes().to_vec(),
            status,
            headers: HashMap::new(),
            delay: None,
        }
    }

    /// Create a JSON response
    pub fn json<T: serde::Serialize>(data: &T) -> Result<Self, serde_json::Error> {
        let body = serde_json::to_vec(data)?;
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        Ok(Self {
            body,
            status: 200,
            headers,
            delay: None,
        })
    }

    /// Set a delay
    pub fn with_delay(mut self, delay: std::time::Duration) -> Self {
        self.delay = Some(delay);
        self
    }
}

/// Mock service trait
#[async_trait]
pub trait MockService: Send + Sync {
    /// Handle a request
    async fn handle(&self, method: &str, path: &str, body: &[u8]) -> InfraResult<MockResponse>;
}

/// Mock service builder
pub struct MockBuilder {
    responses: HashMap<(String, String), MockResponse>,
    default_response: Option<MockResponse>,
    call_count: Arc<RwLock<HashMap<(String, String), usize>>>,
}

impl MockBuilder {
    /// Create a new mock builder
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            default_response: None,
            call_count: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a response for a method and path
    pub fn on(mut self, method: &str, path: &str, response: MockResponse) -> Self {
        self.responses
            .insert((method.to_uppercase(), path.to_string()), response);
        self
    }

    /// Add a GET response
    pub fn on_get(self, path: &str, response: MockResponse) -> Self {
        self.on("GET", path, response)
    }

    /// Add a POST response
    pub fn on_post(self, path: &str, response: MockResponse) -> Self {
        self.on("POST", path, response)
    }

    /// Add a PUT response
    pub fn on_put(self, path: &str, response: MockResponse) -> Self {
        self.on("PUT", path, response)
    }

    /// Add a DELETE response
    pub fn on_delete(self, path: &str, response: MockResponse) -> Self {
        self.on("DELETE", path, response)
    }

    /// Set the default response
    pub fn default_response(mut self, response: MockResponse) -> Self {
        self.default_response = Some(response);
        self
    }

    /// Build the mock service
    pub fn build(self) -> BuiltMock {
        BuiltMock {
            responses: self.responses,
            default_response: self.default_response.unwrap_or_else(|| {
                MockResponse::error(404, "Not Found")
            }),
            call_count: self.call_count,
        }
    }
}

impl Default for MockBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Built mock service
pub struct BuiltMock {
    responses: HashMap<(String, String), MockResponse>,
    default_response: MockResponse,
    call_count: Arc<RwLock<HashMap<(String, String), usize>>>,
}

impl BuiltMock {
    /// Get call count for a method and path
    pub async fn call_count(&self, method: &str, path: &str) -> usize {
        let counts = self.call_count.read().await;
        counts
            .get(&(method.to_uppercase(), path.to_string()))
            .copied()
            .unwrap_or(0)
    }

    /// Check if a method and path was called
    pub async fn was_called(&self, method: &str, path: &str) -> bool {
        self.call_count(method, path).await > 0
    }

    /// Reset call counts
    pub async fn reset(&self) {
        self.call_count.write().await.clear();
    }
}

#[async_trait]
impl MockService for BuiltMock {
    async fn handle(&self, method: &str, path: &str, _body: &[u8]) -> InfraResult<MockResponse> {
        let key = (method.to_uppercase(), path.to_string());

        // Increment call count
        {
            let mut counts = self.call_count.write().await;
            *counts.entry(key.clone()).or_insert(0) += 1;
        }

        // Get response
        let response = self
            .responses
            .get(&key)
            .cloned()
            .unwrap_or_else(|| self.default_response.clone());

        // Apply delay if configured
        if let Some(delay) = response.delay {
            tokio::time::sleep(delay).await;
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_builder() {
        let mock = MockBuilder::new()
            .on_get("/api/users", MockResponse::ok(b"[]".to_vec()))
            .on_post("/api/users", MockResponse::ok(b"{\"id\": 1}".to_vec()))
            .build();

        let response = mock.handle("GET", "/api/users", &[]).await.unwrap();
        assert_eq!(response.status, 200);

        assert!(mock.was_called("GET", "/api/users").await);
        assert_eq!(mock.call_count("GET", "/api/users").await, 1);
    }

    #[tokio::test]
    async fn test_mock_default_response() {
        let mock = MockBuilder::new()
            .default_response(MockResponse::error(500, "Internal Error"))
            .build();

        let response = mock.handle("GET", "/unknown", &[]).await.unwrap();
        assert_eq!(response.status, 500);
    }
}
