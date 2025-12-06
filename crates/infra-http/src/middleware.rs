//! HTTP middleware.

use crate::request::Request;
use crate::response::Response;
use async_trait::async_trait;
use infra_errors::InfraResult;
use std::sync::Arc;

/// Middleware trait
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Process a request before sending
    async fn before(&self, request: Request) -> InfraResult<Request> {
        Ok(request)
    }

    /// Process a response after receiving
    async fn after(&self, response: Response) -> InfraResult<Response> {
        Ok(response)
    }

    /// Middleware name for debugging
    fn name(&self) -> &str {
        "anonymous"
    }
}

/// Stack of middleware
pub struct MiddlewareStack {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareStack {
    /// Create a new middleware stack
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    /// Add a middleware
    pub fn add<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(Arc::new(middleware));
        self
    }

    /// Process a request through all middleware
    pub async fn process_request(&self, mut request: Request) -> InfraResult<Request> {
        for middleware in &self.middlewares {
            request = middleware.before(request).await?;
        }
        Ok(request)
    }

    /// Process a response through all middleware (in reverse order)
    pub async fn process_response(&self, mut response: Response) -> InfraResult<Response> {
        for middleware in self.middlewares.iter().rev() {
            response = middleware.after(response).await?;
        }
        Ok(response)
    }
}

impl Default for MiddlewareStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Logging middleware
pub struct LoggingMiddleware;

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn before(&self, request: Request) -> InfraResult<Request> {
        tracing::info!(
            method = ?request.method,
            url = %request.url,
            "Sending request"
        );
        Ok(request)
    }

    async fn after(&self, response: Response) -> InfraResult<Response> {
        tracing::info!(
            status = response.status,
            "Received response"
        );
        Ok(response)
    }

    fn name(&self) -> &str {
        "logging"
    }
}

/// Authorization header middleware
pub struct AuthMiddleware {
    token: String,
}

impl AuthMiddleware {
    /// Create with bearer token
    pub fn bearer(token: impl Into<String>) -> Self {
        Self {
            token: format!("Bearer {}", token.into()),
        }
    }
}

#[async_trait]
impl Middleware for AuthMiddleware {
    async fn before(&self, mut request: Request) -> InfraResult<Request> {
        request
            .headers
            .insert("Authorization".to_string(), self.token.clone());
        Ok(request)
    }

    fn name(&self) -> &str {
        "auth"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Method;

    #[tokio::test]
    async fn test_middleware_stack() {
        let stack = MiddlewareStack::new()
            .add(LoggingMiddleware)
            .add(AuthMiddleware::bearer("test-token"));

        let request = Request::new(Method::Get, "http://example.com");
        let processed = stack.process_request(request).await.unwrap();

        assert_eq!(
            processed.headers.get("Authorization"),
            Some(&"Bearer test-token".to_string())
        );
    }
}
