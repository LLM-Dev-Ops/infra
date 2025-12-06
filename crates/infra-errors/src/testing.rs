//! Test utilities for infra-errors.

use crate::{InfraError, VectorOperation, AuthErrorKind, CryptoOperation, IoOperation};
use std::path::PathBuf;
use std::time::Duration;

/// Create a mock config error for testing
#[must_use]
pub fn mock_config_error(message: &str) -> InfraError {
    InfraError::Config {
        message: message.to_string(),
        key: Some("test.key".to_string()),
        context: None,
    }
}

/// Create a mock HTTP error for testing
#[must_use]
pub fn mock_http_error(status: u16) -> InfraError {
    InfraError::Http {
        status: Some(status),
        message: format!("HTTP {status}"),
        url: Some("http://test.example.com".to_string()),
        context: None,
    }
}

/// Create a mock vector error for testing
#[must_use]
pub fn mock_vector_error(operation: VectorOperation) -> InfraError {
    InfraError::Vector {
        operation,
        message: format!("Mock {operation} error"),
        dimensions: Some(128),
        context: None,
    }
}

/// Create a mock auth error for testing
#[must_use]
pub fn mock_auth_error(kind: AuthErrorKind) -> InfraError {
    InfraError::Auth {
        kind,
        message: format!("Mock auth error: {kind}"),
        identity: Some("test@example.com".to_string()),
        context: None,
    }
}

/// Create a mock crypto error for testing
#[must_use]
pub fn mock_crypto_error(operation: CryptoOperation) -> InfraError {
    InfraError::Crypto {
        operation,
        message: format!("Mock crypto {operation} error"),
        context: None,
    }
}

/// Create a mock I/O error for testing
#[must_use]
pub fn mock_io_error(operation: IoOperation) -> InfraError {
    InfraError::Io {
        operation,
        path: Some(PathBuf::from("/test/path")),
        message: format!("Mock I/O {operation} error"),
        context: None,
    }
}

/// Create a mock timeout error for testing
#[must_use]
pub fn mock_timeout_error() -> InfraError {
    InfraError::Timeout {
        operation: "test_operation".to_string(),
        duration: Duration::from_secs(30),
        context: None,
    }
}

/// Create a mock not found error for testing
#[must_use]
pub fn mock_not_found_error(resource_type: &str, resource_id: &str) -> InfraError {
    InfraError::NotFound {
        resource_type: resource_type.to_string(),
        resource_id: resource_id.to_string(),
        context: None,
    }
}

/// Create a mock validation error for testing
#[must_use]
pub fn mock_validation_error(field: &str, message: &str) -> InfraError {
    InfraError::Validation {
        field: Some(field.to_string()),
        message: message.to_string(),
        expected: Some("valid value".to_string()),
        actual: Some("invalid value".to_string()),
        context: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_errors_are_valid() {
        let config_err = mock_config_error("test");
        assert_eq!(config_err.error_type(), "config");

        let http_err = mock_http_error(404);
        assert_eq!(http_err.error_type(), "http");

        let vector_err = mock_vector_error(VectorOperation::Search);
        assert_eq!(vector_err.error_type(), "vector");

        let auth_err = mock_auth_error(AuthErrorKind::InvalidToken);
        assert_eq!(auth_err.error_type(), "auth");
        assert!(!auth_err.is_retryable());

        let timeout_err = mock_timeout_error();
        assert!(timeout_err.is_retryable());
    }
}
