//! Request handlers.

use async_trait::async_trait;
use infra_errors::InfraResult;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Handler result
#[derive(Debug, Clone)]
pub struct HandlerResult {
    /// Status code
    pub status: u16,
    /// Response body
    pub body: Vec<u8>,
    /// Response headers
    pub headers: HashMap<String, String>,
}

impl HandlerResult {
    /// Create an OK response
    pub fn ok(body: impl Into<Vec<u8>>) -> Self {
        Self {
            status: 200,
            body: body.into(),
            headers: HashMap::new(),
        }
    }

    /// Create a JSON response
    pub fn json<T: serde::Serialize>(data: &T) -> Result<Self, serde_json::Error> {
        let body = serde_json::to_vec(data)?;
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        Ok(Self {
            status: 200,
            body,
            headers,
        })
    }

    /// Create an error response
    pub fn error(status: u16, message: &str) -> Self {
        Self {
            status,
            body: message.as_bytes().to_vec(),
            headers: HashMap::new(),
        }
    }

    /// Create a not found response
    pub fn not_found() -> Self {
        Self::error(404, "Not Found")
    }

    /// Create a bad request response
    pub fn bad_request(message: &str) -> Self {
        Self::error(400, message)
    }

    /// Create an internal error response
    pub fn internal_error(message: &str) -> Self {
        Self::error(500, message)
    }

    /// Set a header
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set status code
    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }
}

/// Request context
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Request path
    pub path: String,
    /// Path parameters
    pub params: HashMap<String, String>,
    /// Query parameters
    pub query: HashMap<String, String>,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body
    pub body: Vec<u8>,
}

impl RequestContext {
    /// Create a new context
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            params: HashMap::new(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Get a path parameter
    pub fn param(&self, name: &str) -> Option<&String> {
        self.params.get(name)
    }

    /// Get a query parameter
    pub fn query_param(&self, name: &str) -> Option<&String> {
        self.query.get(name)
    }

    /// Get a header
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    /// Parse body as JSON
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }
}

/// Handler trait
#[async_trait]
pub trait Handler: Send + Sync {
    /// Handle a request
    async fn handle(&self, ctx: RequestContext) -> InfraResult<HandlerResult>;
}

/// Function handler wrapper
pub struct HandlerFn<F, Fut>
where
    F: Fn(RequestContext) -> Fut + Send + Sync,
    Fut: Future<Output = InfraResult<HandlerResult>> + Send,
{
    f: F,
}

impl<F, Fut> HandlerFn<F, Fut>
where
    F: Fn(RequestContext) -> Fut + Send + Sync,
    Fut: Future<Output = InfraResult<HandlerResult>> + Send,
{
    /// Create a new handler from a function
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

#[async_trait]
impl<F, Fut> Handler for HandlerFn<F, Fut>
where
    F: Fn(RequestContext) -> Fut + Send + Sync,
    Fut: Future<Output = InfraResult<HandlerResult>> + Send,
{
    async fn handle(&self, ctx: RequestContext) -> InfraResult<HandlerResult> {
        (self.f)(ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_result() {
        let result = HandlerResult::ok(b"Hello".to_vec())
            .with_header("x-custom", "value")
            .with_status(201);

        assert_eq!(result.status, 201);
        assert_eq!(result.headers.get("x-custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_json_result() {
        let data = serde_json::json!({"message": "Hello"});
        let result = HandlerResult::json(&data).unwrap();

        assert_eq!(result.status, 200);
        assert_eq!(
            result.headers.get("content-type"),
            Some(&"application/json".to_string())
        );
    }
}
