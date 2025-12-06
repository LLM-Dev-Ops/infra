//! OpenTelemetry initialization.

use crate::config::{ExporterConfig, OtelConfig};
use infra_errors::{InfraError, InfraResult};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize tracing with OpenTelemetry
pub fn init_tracing(config: &OtelConfig) -> InfraResult<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let subscriber = tracing_subscriber::registry().with(env_filter);

    match &config.trace_exporter {
        ExporterConfig::Stdout => {
            if config.json_logs {
                let fmt_layer = tracing_subscriber::fmt::layer()
                    .json()
                    .with_target(true)
                    .with_thread_ids(true);
                subscriber.with(fmt_layer).try_init().map_err(|e| {
                    InfraError::External {
                        service: "tracing".to_string(),
                        operation: "init".to_string(),
                        message: e.to_string(),
                        retry_after: None,
                        context: None,
                    }
                })?;
            } else {
                let fmt_layer = tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_thread_ids(false);
                subscriber.with(fmt_layer).try_init().map_err(|e| {
                    InfraError::External {
                        service: "tracing".to_string(),
                        operation: "init".to_string(),
                        message: e.to_string(),
                        retry_after: None,
                        context: None,
                    }
                })?;
            }
        }
        ExporterConfig::None => {
            // No-op subscriber
            subscriber.try_init().map_err(|e| {
                InfraError::External {
                    service: "tracing".to_string(),
                    operation: "init".to_string(),
                    message: e.to_string(),
                    retry_after: None,
                    context: None,
                }
            })?;
        }
        #[cfg(feature = "otlp")]
        ExporterConfig::Otlp { endpoint, protocol } => {
            use opentelemetry_otlp::WithExportConfig;
            use opentelemetry_sdk::trace::TracerProvider;

            let exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .build()
                .map_err(|e| InfraError::External {
                    service: "otlp".to_string(),
                    operation: "init".to_string(),
                    message: e.to_string(),
                    retry_after: None,
                    context: None,
                })?;

            let provider = TracerProvider::builder()
                .with_batch_exporter(exporter)
                .build();

            let tracer = provider.tracer(config.service_name.clone());
            let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

            let fmt_layer = tracing_subscriber::fmt::layer().with_target(true);

            subscriber
                .with(telemetry)
                .with(fmt_layer)
                .try_init()
                .map_err(|e| InfraError::External {
                    service: "tracing".to_string(),
                    operation: "init".to_string(),
                    message: e.to_string(),
                    retry_after: None,
                    context: None,
                })?;
        }
        #[cfg(not(feature = "otlp"))]
        ExporterConfig::Otlp { .. } => {
            return Err(InfraError::Config {
                key: Some("trace_exporter".to_string()),
                message: "OTLP exporter requires 'otlp' feature".to_string(),
                context: None,
            });
        }
        #[cfg(feature = "jaeger")]
        ExporterConfig::Jaeger { agent_endpoint } => {
            // Jaeger exporter setup would go here
            return Err(InfraError::Config {
                key: Some("trace_exporter".to_string()),
                message: "Jaeger exporter not yet implemented".to_string(),
                context: None,
            });
        }
        #[cfg(not(feature = "jaeger"))]
        ExporterConfig::Jaeger { .. } => {
            return Err(InfraError::Config {
                key: Some("trace_exporter".to_string()),
                message: "Jaeger exporter requires 'jaeger' feature".to_string(),
                context: None,
            });
        }
    }

    Ok(())
}

/// Initialize metrics
#[cfg(feature = "metrics")]
pub fn init_metrics(_config: &OtelConfig) -> InfraResult<()> {
    // Metrics initialization
    // For now, just a placeholder that can be extended
    Ok(())
}

#[cfg(not(feature = "metrics"))]
pub fn init_metrics(_config: &OtelConfig) -> InfraResult<()> {
    Ok(())
}

/// Shutdown OpenTelemetry
pub fn shutdown() {
    opentelemetry::global::shutdown_tracer_provider();
}
