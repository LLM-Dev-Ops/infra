//! Span utilities.

use tracing::{Span, Level};
use std::collections::HashMap;

/// Span builder for creating customized spans
pub struct SpanBuilder {
    name: String,
    level: Level,
    attributes: HashMap<String, String>,
}

impl SpanBuilder {
    /// Create a new span builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            level: Level::INFO,
            attributes: HashMap::new(),
        }
    }

    /// Set span level
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Add an attribute
    pub fn attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Build the span
    pub fn build(self) -> Span {
        match self.level {
            Level::TRACE => tracing::trace_span!("span", name = %self.name),
            Level::DEBUG => tracing::debug_span!("span", name = %self.name),
            Level::INFO => tracing::info_span!("span", name = %self.name),
            Level::WARN => tracing::warn_span!("span", name = %self.name),
            Level::ERROR => tracing::error_span!("span", name = %self.name),
        }
    }
}

/// Extension trait for spans
pub trait SpanExt {
    /// Record a key-value attribute
    fn record_attribute(&self, key: &str, value: &str);

    /// Record an error
    fn record_error(&self, error: &dyn std::error::Error);

    /// Record success status
    fn record_ok(&self);
}

impl SpanExt for Span {
    fn record_attribute(&self, key: &str, value: &str) {
        self.record(key, value);
    }

    fn record_error(&self, error: &dyn std::error::Error) {
        self.record("error", true);
        self.record("error.message", error.to_string().as_str());
    }

    fn record_ok(&self) {
        self.record("error", false);
    }
}

/// Create a span for an HTTP request
pub fn http_span(method: &str, path: &str) -> Span {
    tracing::info_span!(
        "http_request",
        http.method = %method,
        http.path = %path,
        http.status_code = tracing::field::Empty,
    )
}

/// Create a span for a database operation
pub fn db_span(operation: &str, table: &str) -> Span {
    tracing::info_span!(
        "db_operation",
        db.operation = %operation,
        db.table = %table,
        db.rows_affected = tracing::field::Empty,
    )
}

/// Create a span for an external service call
pub fn external_span(service: &str, operation: &str) -> Span {
    tracing::info_span!(
        "external_call",
        service.name = %service,
        service.operation = %operation,
        service.response_time_ms = tracing::field::Empty,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_builder() {
        let span = SpanBuilder::new("test-span")
            .level(Level::DEBUG)
            .attribute("key", "value")
            .build();

        // Just verify it doesn't panic
        let _guard = span.enter();
    }
}
