//! Trace context and propagation.

use std::collections::HashMap;

/// Trace context for distributed tracing
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// Trace ID
    pub trace_id: String,
    /// Span ID
    pub span_id: String,
    /// Trace flags
    pub trace_flags: u8,
    /// Trace state
    pub trace_state: Option<String>,
}

impl TraceContext {
    /// Create from current span (requires tracing-opentelemetry integration)
    pub fn current() -> Option<Self> {
        // This requires the OpenTelemetrySpanExt trait to be properly set up
        // For now, return None as a placeholder
        None
    }

    /// Create from trace and span IDs
    pub fn new(trace_id: &str, span_id: &str) -> Self {
        Self {
            trace_id: trace_id.to_string(),
            span_id: span_id.to_string(),
            trace_flags: 1,
            trace_state: None,
        }
    }

    /// Convert to W3C traceparent header
    pub fn to_traceparent(&self) -> String {
        format!(
            "00-{}-{}-{:02x}",
            self.trace_id, self.span_id, self.trace_flags
        )
    }

    /// Parse from W3C traceparent header
    pub fn from_traceparent(header: &str) -> Option<Self> {
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() < 4 {
            return None;
        }

        let trace_flags = u8::from_str_radix(parts[3], 16).ok()?;

        Some(Self {
            trace_id: parts[1].to_string(),
            span_id: parts[2].to_string(),
            trace_flags,
            trace_state: None,
        })
    }
}

/// Context propagation for distributed systems
#[derive(Debug, Clone, Default)]
pub struct PropagationContext {
    headers: HashMap<String, String>,
}

impl PropagationContext {
    /// Create a new propagation context
    pub fn new() -> Self {
        Self::default()
    }

    /// Inject current context into headers
    pub fn inject(&mut self) {
        if let Some(trace_ctx) = TraceContext::current() {
            self.headers.insert("traceparent".to_string(), trace_ctx.to_traceparent());
            if let Some(state) = trace_ctx.trace_state {
                self.headers.insert("tracestate".to_string(), state);
            }
        }
    }

    /// Create from headers
    pub fn from_headers(headers: HashMap<String, String>) -> Self {
        Self { headers }
    }

    /// Get a header value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    /// Get all headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Extract trace context
    pub fn extract_trace_context(&self) -> Option<TraceContext> {
        self.headers
            .get("traceparent")
            .and_then(|h| TraceContext::from_traceparent(h))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traceparent_roundtrip() {
        let ctx = TraceContext::new(
            "0af7651916cd43dd8448eb211c80319c",
            "b7ad6b7169203331",
        );

        let header = ctx.to_traceparent();
        let parsed = TraceContext::from_traceparent(&header).unwrap();

        assert_eq!(parsed.trace_id, ctx.trace_id);
        assert_eq!(parsed.span_id, ctx.span_id);
    }

    #[test]
    fn test_propagation_context() {
        let mut ctx = PropagationContext::new();
        ctx.headers.insert(
            "traceparent".to_string(),
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01".to_string(),
        );

        let trace = ctx.extract_trace_context().unwrap();
        assert_eq!(trace.trace_id, "0af7651916cd43dd8448eb211c80319c");
    }
}
