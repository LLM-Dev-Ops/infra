//! OpenTelemetry configuration.

use serde::{Deserialize, Serialize};

/// Exporter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExporterConfig {
    /// Log to stdout (for development)
    Stdout,
    /// OTLP exporter
    Otlp {
        endpoint: String,
        protocol: OtlpProtocol,
    },
    /// Jaeger exporter
    Jaeger {
        agent_endpoint: String,
    },
    /// No exporter (disabled)
    None,
}

impl Default for ExporterConfig {
    fn default() -> Self {
        Self::Stdout
    }
}

/// OTLP protocol
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OtlpProtocol {
    Grpc,
    HttpBinary,
    HttpJson,
}

impl Default for OtlpProtocol {
    fn default() -> Self {
        Self::Grpc
    }
}

/// OpenTelemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelConfig {
    /// Service name
    pub service_name: String,
    /// Service version
    pub service_version: Option<String>,
    /// Service namespace
    pub service_namespace: Option<String>,
    /// Deployment environment
    pub environment: Option<String>,
    /// Trace exporter configuration
    pub trace_exporter: ExporterConfig,
    /// Metrics exporter configuration
    pub metrics_exporter: ExporterConfig,
    /// Sample ratio (0.0 to 1.0)
    pub sample_ratio: f64,
    /// Enable console logging
    pub console_logging: bool,
    /// Log level filter
    pub log_level: String,
    /// JSON formatted logs
    pub json_logs: bool,
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            service_name: "unknown".to_string(),
            service_version: None,
            service_namespace: None,
            environment: None,
            trace_exporter: ExporterConfig::default(),
            metrics_exporter: ExporterConfig::default(),
            sample_ratio: 1.0,
            console_logging: true,
            log_level: "info".to_string(),
            json_logs: false,
        }
    }
}

impl OtelConfig {
    /// Create a new configuration builder
    pub fn builder() -> OtelConfigBuilder {
        OtelConfigBuilder::default()
    }
}

/// Builder for OtelConfig
#[derive(Debug, Default)]
pub struct OtelConfigBuilder {
    config: OtelConfig,
}

impl OtelConfigBuilder {
    /// Set service name
    pub fn service_name(mut self, name: &str) -> Self {
        self.config.service_name = name.to_string();
        self
    }

    /// Set service version
    pub fn service_version(mut self, version: &str) -> Self {
        self.config.service_version = Some(version.to_string());
        self
    }

    /// Set service namespace
    pub fn service_namespace(mut self, namespace: &str) -> Self {
        self.config.service_namespace = Some(namespace.to_string());
        self
    }

    /// Set environment
    pub fn environment(mut self, env: &str) -> Self {
        self.config.environment = Some(env.to_string());
        self
    }

    /// Set trace exporter
    pub fn trace_exporter(mut self, exporter: ExporterConfig) -> Self {
        self.config.trace_exporter = exporter;
        self
    }

    /// Set metrics exporter
    pub fn metrics_exporter(mut self, exporter: ExporterConfig) -> Self {
        self.config.metrics_exporter = exporter;
        self
    }

    /// Set sample ratio
    pub fn sample_ratio(mut self, ratio: f64) -> Self {
        self.config.sample_ratio = ratio.clamp(0.0, 1.0);
        self
    }

    /// Enable/disable console logging
    pub fn console_logging(mut self, enabled: bool) -> Self {
        self.config.console_logging = enabled;
        self
    }

    /// Set log level
    pub fn log_level(mut self, level: &str) -> Self {
        self.config.log_level = level.to_string();
        self
    }

    /// Enable JSON formatted logs
    pub fn json_logs(mut self, enabled: bool) -> Self {
        self.config.json_logs = enabled;
        self
    }

    /// Build the configuration
    pub fn build(self) -> OtelConfig {
        self.config
    }
}
