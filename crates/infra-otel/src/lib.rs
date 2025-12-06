//! OpenTelemetry integration for LLM-Dev-Ops infrastructure.
//!
//! This crate provides unified observability with tracing, metrics, and
//! distributed context propagation.

mod config;
mod context;
mod init;
mod span;
mod metrics;

pub use config::{OtelConfig, ExporterConfig};
pub use context::{TraceContext, PropagationContext};
pub use init::{init_tracing, init_metrics, shutdown};
pub use span::{SpanBuilder, SpanExt};
pub use metrics::{Counter, Gauge, Histogram, MetricsRegistry};

use infra_errors::InfraResult;

/// Initialize OpenTelemetry with default configuration
pub fn init(service_name: &str) -> InfraResult<()> {
    let config = OtelConfig::builder()
        .service_name(service_name)
        .build();

    init_with_config(config)
}

/// Initialize OpenTelemetry with custom configuration
pub fn init_with_config(config: OtelConfig) -> InfraResult<()> {
    init_tracing(&config)?;

    #[cfg(feature = "metrics")]
    init_metrics(&config)?;

    Ok(())
}

/// Create a new span
#[macro_export]
macro_rules! span {
    ($name:expr) => {
        tracing::info_span!($name)
    };
    ($name:expr, $($field:tt)*) => {
        tracing::info_span!($name, $($field)*)
    };
}

/// Record an event
#[macro_export]
macro_rules! event {
    ($level:expr, $($arg:tt)*) => {
        tracing::event!($level, $($arg)*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = OtelConfig::builder()
            .service_name("test-service")
            .service_version("1.0.0")
            .build();

        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.service_version, Some("1.0.0".to_string()));
    }
}
