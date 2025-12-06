//! Error context for enhanced debugging and tracing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context that can be attached to any InfraError
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Unique error instance ID for correlation
    pub error_id: String,

    /// Timestamp when error occurred
    pub timestamp: DateTime<Utc>,

    /// Source location (file, line, column)
    pub location: Option<SourceLocation>,

    /// Related span IDs for distributed tracing
    pub trace_ids: TraceIds,

    /// Key-value pairs for additional context
    pub attributes: HashMap<String, String>,

    /// Suggested remediation steps
    pub remediation: Option<Vec<String>>,
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            error_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            location: None,
            trace_ids: TraceIds::default(),
            attributes: HashMap::new(),
            remediation: None,
        }
    }
}

impl ErrorContext {
    /// Create a new error context with a unique ID
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an attribute to the context
    #[must_use]
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Set the source location
    #[must_use]
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Set remediation steps
    #[must_use]
    pub fn with_remediation(mut self, steps: Vec<String>) -> Self {
        self.remediation = Some(steps);
        self
    }

    /// Set trace IDs from current span
    #[must_use]
    pub fn with_trace_ids(mut self, trace_ids: TraceIds) -> Self {
        self.trace_ids = trace_ids;
        self
    }
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub function: Option<String>,
}

impl SourceLocation {
    /// Create a new source location
    #[must_use]
    pub fn new(file: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            file: file.into(),
            line,
            column,
            function: None,
        }
    }

    /// Add function name
    #[must_use]
    pub fn with_function(mut self, function: impl Into<String>) -> Self {
        self.function = Some(function.into());
        self
    }
}

/// Trace IDs for distributed tracing correlation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraceIds {
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub parent_span_id: Option<String>,
}

impl TraceIds {
    /// Create new trace IDs
    #[must_use]
    pub fn new(trace_id: Option<String>, span_id: Option<String>) -> Self {
        Self {
            trace_id,
            span_id,
            parent_span_id: None,
        }
    }

    /// Set parent span ID
    #[must_use]
    pub fn with_parent(mut self, parent_span_id: impl Into<String>) -> Self {
        self.parent_span_id = Some(parent_span_id.into());
        self
    }
}

/// Macro for capturing source location
#[macro_export]
macro_rules! source_location {
    () => {
        $crate::SourceLocation::new(file!(), line!(), column!())
    };
}

/// Macro for creating an error with context
#[macro_export]
macro_rules! infra_error {
    ($error:expr) => {{
        let mut err = $error;
        err.set_context($crate::ErrorContext::new().with_location($crate::source_location!()));
        err
    }};
    ($error:expr, $($key:expr => $value:expr),+ $(,)?) => {{
        let mut ctx = $crate::ErrorContext::new().with_location($crate::source_location!());
        $(
            ctx = ctx.with_attribute($key, $value);
        )+
        let mut err = $error;
        err.set_context(ctx);
        err
    }};
}
