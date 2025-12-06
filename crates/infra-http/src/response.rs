//! HTTP response utilities.

use infra_errors::{InfraError, InfraResult};

/// HTTP response wrapper
#[derive(Debug)]
pub struct Response {
    /// Status code
    pub status: u16,
    /// Headers
    pub headers: std::collections::HashMap<String, String>,
    /// Body
    pub body: Vec<u8>,
}

impl Response {
    /// Create a new response
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: std::collections::HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Create an OK response
    pub fn ok() -> Self {
        Self::new(200)
    }

    /// Create a created response
    pub fn created() -> Self {
        Self::new(201)
    }

    /// Create a no content response
    pub fn no_content() -> Self {
        Self::new(204)
    }

    /// Create a bad request response
    pub fn bad_request() -> Self {
        Self::new(400)
    }

    /// Create an unauthorized response
    pub fn unauthorized() -> Self {
        Self::new(401)
    }

    /// Create a forbidden response
    pub fn forbidden() -> Self {
        Self::new(403)
    }

    /// Create a not found response
    pub fn not_found() -> Self {
        Self::new(404)
    }

    /// Create an internal server error response
    pub fn internal_error() -> Self {
        Self::new(500)
    }

    /// Check if the response is successful
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Check if the response is a client error
    pub fn is_client_error(&self) -> bool {
        self.status >= 400 && self.status < 500
    }

    /// Check if the response is a server error
    pub fn is_server_error(&self) -> bool {
        self.status >= 500
    }

    /// Set a header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set the body
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Set JSON body
    pub fn json<T: serde::Serialize>(self, body: &T) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_vec(body)?;
        Ok(self
            .header("Content-Type", "application/json")
            .body(json))
    }

    /// Parse body as JSON
    pub fn parse_json<T: serde::de::DeserializeOwned>(&self) -> InfraResult<T> {
        serde_json::from_slice(&self.body).map_err(|e| InfraError::Http {
            status: None,
            message: format!("Failed to parse JSON: {e}"),
            url: None,
            context: None,
        })
    }

    /// Get body as string
    pub fn text(&self) -> InfraResult<String> {
        String::from_utf8(self.body.clone()).map_err(|e| InfraError::Http {
            status: None,
            message: format!("Invalid UTF-8: {e}"),
            url: None,
            context: None,
        })
    }
}

/// Extension trait for responses
pub trait ResponseExt {
    /// Convert to InfraResult
    fn into_result(self) -> InfraResult<Response>;
}

impl ResponseExt for Response {
    fn into_result(self) -> InfraResult<Response> {
        if self.is_success() {
            Ok(self)
        } else {
            Err(InfraError::Http {
                status: Some(self.status),
                message: format!("HTTP error: {}", self.status),
                url: None,
                context: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_status() {
        assert!(Response::ok().is_success());
        assert!(Response::bad_request().is_client_error());
        assert!(Response::internal_error().is_server_error());
    }
}
