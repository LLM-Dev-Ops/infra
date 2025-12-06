//! Core InfraError type definition.

use crate::context::ErrorContext;
use crate::kinds::{
    AuthErrorKind, CryptoOperation, IoOperation, MqOperation,
    SerializationFormat, VectorOperation,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Primary error type for all infra operations
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum InfraError {
    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config {
        message: String,
        key: Option<String>,
        #[serde(skip)]
        context: Option<ErrorContext>,
    },

    /// HTTP/Network errors
    #[error("HTTP error: {message}")]
    Http {
        status: Option<u16>,
        message: String,
        url: Option<String>,
        #[serde(skip)]
        context: Option<ErrorContext>,
    },

    /// Vector operation errors
    #[error("Vector {operation} error: {message}")]
    Vector {
        operation: VectorOperation,
        message: String,
        dimensions: Option<usize>,
        #[serde(skip)]
        context: Option<ErrorContext>,
    },

    /// Authentication/Authorization errors
    #[error("Authentication error ({kind}): {message}")]
    Auth {
        kind: AuthErrorKind,
        message: String,
        identity: Option<String>,
        #[serde(skip)]
        context: Option<ErrorContext>,
    },

    /// Cryptographic errors
    #[error("Crypto {operation} error: {message}")]
    Crypto {
        operation: CryptoOperation,
        message: String,
        #[serde(skip)]
        context: Option<ErrorContext>,
    },

    /// I/O errors
    #[error("I/O {operation} error: {message}")]
    Io {
        operation: IoOperation,
        path: Option<PathBuf>,
        message: String,
        #[serde(skip)]
        context: Option<ErrorContext>,
    },

    /// Serialization errors
    #[error("Serialization error ({format}): {message}")]
    Serialization {
        format: SerializationFormat,
        message: String,
        location: Option<String>,
        #[serde(skip)]
        context: Option<ErrorContext>,
    },

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation {
        field: Option<String>,
        message: String,
        expected: Option<String>,
        actual: Option<String>,
        #[serde(skip)]
        context: Option<ErrorContext>,
    },

    /// External service errors
    #[error("External service error ({service}): {message}")]
    External {
        service: String,
        operation: String,
        message: String,
        #[serde(with = "duration_option_serde")]
        retry_after: Option<Duration>,
        #[serde(skip)]
        context: Option<ErrorContext>,
    },

    /// Message queue errors
    #[error("Message queue error ({operation}): {message}")]
    MessageQueue {
        queue: String,
        operation: MqOperation,
        message: String,
        #[serde(skip)]
        context: Option<ErrorContext>,
    },

    /// Schema errors
    #[error("Schema error: {message}")]
    Schema {
        schema_id: Option<String>,
        path: Option<String>,
        message: String,
        #[serde(skip)]
        context: Option<ErrorContext>,
    },

    /// Timeout errors
    #[error("Operation timed out after {duration:?}")]
    Timeout {
        operation: String,
        #[serde(with = "duration_serde")]
        duration: Duration,
        #[serde(skip)]
        context: Option<ErrorContext>,
    },

    /// Resource not found
    #[error("Resource not found: {resource_type}/{resource_id}")]
    NotFound {
        resource_type: String,
        resource_id: String,
        #[serde(skip)]
        context: Option<ErrorContext>,
    },

    /// Resource already exists
    #[error("Resource already exists: {resource_type}/{resource_id}")]
    AlreadyExists {
        resource_type: String,
        resource_id: String,
        #[serde(skip)]
        context: Option<ErrorContext>,
    },
}

mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

mod duration_option_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.map(|d| d.as_millis() as u64).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis: Option<u64> = Option::deserialize(deserializer)?;
        Ok(millis.map(Duration::from_millis))
    }
}

impl InfraError {
    /// Get the error type string for metrics/logging
    #[must_use]
    pub fn error_type(&self) -> &'static str {
        match self {
            Self::Config { .. } => "config",
            Self::Http { .. } => "http",
            Self::Vector { .. } => "vector",
            Self::Auth { .. } => "auth",
            Self::Crypto { .. } => "crypto",
            Self::Io { .. } => "io",
            Self::Serialization { .. } => "serialization",
            Self::Validation { .. } => "validation",
            Self::External { .. } => "external",
            Self::MessageQueue { .. } => "message_queue",
            Self::Schema { .. } => "schema",
            Self::Timeout { .. } => "timeout",
            Self::NotFound { .. } => "not_found",
            Self::AlreadyExists { .. } => "already_exists",
        }
    }

    /// Check if this error is retryable
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Http { status: Some(s), .. } => *s >= 500 || *s == 429,
            Self::External { retry_after, .. } => retry_after.is_some(),
            Self::Auth { kind: AuthErrorKind::RateLimited, .. } => true,
            Self::MessageQueue { .. } => true,
            Self::Timeout { .. } => true,
            Self::Io { operation, .. } => matches!(
                operation,
                IoOperation::Read | IoOperation::Write
            ),
            _ => false,
        }
    }

    /// Get retry delay if applicable
    #[must_use]
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::External { retry_after, .. } => *retry_after,
            Self::Auth { kind: AuthErrorKind::RateLimited, .. } => {
                Some(Duration::from_secs(60))
            }
            Self::Http { status: Some(429), .. } => Some(Duration::from_secs(30)),
            Self::Timeout { .. } => Some(Duration::from_secs(1)),
            _ => None,
        }
    }

    /// Set error context
    pub fn set_context(&mut self, ctx: ErrorContext) {
        match self {
            Self::Config { context, .. }
            | Self::Http { context, .. }
            | Self::Vector { context, .. }
            | Self::Auth { context, .. }
            | Self::Crypto { context, .. }
            | Self::Io { context, .. }
            | Self::Serialization { context, .. }
            | Self::Validation { context, .. }
            | Self::External { context, .. }
            | Self::MessageQueue { context, .. }
            | Self::Schema { context, .. }
            | Self::Timeout { context, .. }
            | Self::NotFound { context, .. }
            | Self::AlreadyExists { context, .. } => {
                *context = Some(ctx);
            }
        }
    }

    /// Get error context
    #[must_use]
    pub fn context(&self) -> Option<&ErrorContext> {
        match self {
            Self::Config { context, .. }
            | Self::Http { context, .. }
            | Self::Vector { context, .. }
            | Self::Auth { context, .. }
            | Self::Crypto { context, .. }
            | Self::Io { context, .. }
            | Self::Serialization { context, .. }
            | Self::Validation { context, .. }
            | Self::External { context, .. }
            | Self::MessageQueue { context, .. }
            | Self::Schema { context, .. }
            | Self::Timeout { context, .. }
            | Self::NotFound { context, .. }
            | Self::AlreadyExists { context, .. } => context.as_ref(),
        }
    }

    /// Create a config error
    #[must_use]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            key: None,
            context: None,
        }
    }

    /// Create a config error with key
    #[must_use]
    pub fn config_with_key(message: impl Into<String>, key: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            key: Some(key.into()),
            context: None,
        }
    }

    /// Create an HTTP error
    #[must_use]
    pub fn http(message: impl Into<String>) -> Self {
        Self::Http {
            status: None,
            message: message.into(),
            url: None,
            context: None,
        }
    }

    /// Create an HTTP error with status
    #[must_use]
    pub fn http_with_status(status: u16, message: impl Into<String>) -> Self {
        Self::Http {
            status: Some(status),
            message: message.into(),
            url: None,
            context: None,
        }
    }

    /// Create a validation error
    #[must_use]
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            field: None,
            message: message.into(),
            expected: None,
            actual: None,
            context: None,
        }
    }

    /// Create a validation error with field
    #[must_use]
    pub fn validation_field(
        field: impl Into<String>,
        message: impl Into<String>,
        expected: Option<String>,
        actual: Option<String>,
    ) -> Self {
        Self::Validation {
            field: Some(field.into()),
            message: message.into(),
            expected,
            actual,
            context: None,
        }
    }

    /// Create a not found error
    #[must_use]
    pub fn not_found(resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            context: None,
        }
    }

    /// Create a timeout error
    #[must_use]
    pub fn timeout(operation: impl Into<String>, duration: Duration) -> Self {
        Self::Timeout {
            operation: operation.into(),
            duration,
            context: None,
        }
    }
}

// Conversion from std::io::Error
impl From<std::io::Error> for InfraError {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            operation: IoOperation::Read,
            path: None,
            message: err.to_string(),
            context: None,
        }
    }
}

// Conversion from serde_json::Error
impl From<serde_json::Error> for InfraError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            format: SerializationFormat::Json,
            message: err.to_string(),
            location: Some(format!("line {}, column {}", err.line(), err.column())),
            context: None,
        }
    }
}

impl Clone for InfraError {
    fn clone(&self) -> Self {
        match self {
            Self::Config { message, key, context } => Self::Config {
                message: message.clone(),
                key: key.clone(),
                context: context.clone(),
            },
            Self::Http { status, message, url, context } => Self::Http {
                status: *status,
                message: message.clone(),
                url: url.clone(),
                context: context.clone(),
            },
            Self::Vector { operation, message, dimensions, context } => Self::Vector {
                operation: *operation,
                message: message.clone(),
                dimensions: *dimensions,
                context: context.clone(),
            },
            Self::Auth { kind, message, identity, context } => Self::Auth {
                kind: *kind,
                message: message.clone(),
                identity: identity.clone(),
                context: context.clone(),
            },
            Self::Crypto { operation, message, context } => Self::Crypto {
                operation: *operation,
                message: message.clone(),
                context: context.clone(),
            },
            Self::Io { operation, path, message, context } => Self::Io {
                operation: *operation,
                path: path.clone(),
                message: message.clone(),
                context: context.clone(),
            },
            Self::Serialization { format, message, location, context } => Self::Serialization {
                format: *format,
                message: message.clone(),
                location: location.clone(),
                context: context.clone(),
            },
            Self::Validation { field, message, expected, actual, context } => Self::Validation {
                field: field.clone(),
                message: message.clone(),
                expected: expected.clone(),
                actual: actual.clone(),
                context: context.clone(),
            },
            Self::External { service, operation, message, retry_after, context } => Self::External {
                service: service.clone(),
                operation: operation.clone(),
                message: message.clone(),
                retry_after: *retry_after,
                context: context.clone(),
            },
            Self::MessageQueue { queue, operation, message, context } => Self::MessageQueue {
                queue: queue.clone(),
                operation: *operation,
                message: message.clone(),
                context: context.clone(),
            },
            Self::Schema { schema_id, path, message, context } => Self::Schema {
                schema_id: schema_id.clone(),
                path: path.clone(),
                message: message.clone(),
                context: context.clone(),
            },
            Self::Timeout { operation, duration, context } => Self::Timeout {
                operation: operation.clone(),
                duration: *duration,
                context: context.clone(),
            },
            Self::NotFound { resource_type, resource_id, context } => Self::NotFound {
                resource_type: resource_type.clone(),
                resource_id: resource_id.clone(),
                context: context.clone(),
            },
            Self::AlreadyExists { resource_type, resource_id, context } => Self::AlreadyExists {
                resource_type: resource_type.clone(),
                resource_id: resource_id.clone(),
                context: context.clone(),
            },
        }
    }
}
