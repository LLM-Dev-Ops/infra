//! HTTP request utilities.

use crate::Method;
use std::collections::HashMap;

/// HTTP request
#[derive(Debug, Clone)]
pub struct Request {
    /// HTTP method
    pub method: Method,
    /// URL
    pub url: String,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Body
    pub body: Option<Vec<u8>>,
}

impl Request {
    /// Create a new request
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Create a GET request
    pub fn get(url: impl Into<String>) -> Self {
        Self::new(Method::Get, url)
    }

    /// Create a POST request
    pub fn post(url: impl Into<String>) -> Self {
        Self::new(Method::Post, url)
    }

    /// Create a PUT request
    pub fn put(url: impl Into<String>) -> Self {
        Self::new(Method::Put, url)
    }

    /// Create a DELETE request
    pub fn delete(url: impl Into<String>) -> Self {
        Self::new(Method::Delete, url)
    }
}

/// Request builder
pub struct RequestBuilder {
    request: Request,
}

impl RequestBuilder {
    /// Create a new builder
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            request: Request::new(method, url),
        }
    }

    /// Add a header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.request.headers.insert(name.into(), value.into());
        self
    }

    /// Set the body
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.request.body = Some(body.into());
        self
    }

    /// Set JSON body
    pub fn json<T: serde::Serialize>(self, body: &T) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_vec(body)?;
        Ok(self.header("Content-Type", "application/json").body(json))
    }

    /// Build the request
    pub fn build(self) -> Request {
        self.request
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder() {
        let request = RequestBuilder::new(Method::Post, "http://example.com/api")
            .header("Authorization", "Bearer token")
            .body(b"test".to_vec())
            .build();

        assert_eq!(request.method, Method::Post);
        assert_eq!(request.url, "http://example.com/api");
        assert_eq!(
            request.headers.get("Authorization"),
            Some(&"Bearer token".to_string())
        );
    }
}
