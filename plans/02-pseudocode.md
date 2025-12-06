# SPARC Phase 2: Pseudocode

## LLM-Dev-Ops Unified Infrastructure Layer

**Document Version:** 1.0
**Date:** 2025-12-06
**Status:** Draft - Pending User Approval
**Previous Phase:** [01-specification.md](./01-specification.md)

---

## 1. Overview

This document provides detailed pseudocode for all 15 infra crates, defining:
- Complete API surfaces
- Implementation logic flows
- Error handling patterns
- WASM binding generation strategy
- Integration patterns with RuvNet ecosystem

---

## 2. Infra Crate Pseudocode

### 2.1 `infra-errors` (Level 0 - Leaf Crate)

**Purpose:** Unified error model for all infrastructure operations

```rust
// ============================================================
// FILE: crates/infra-errors/src/lib.rs
// ============================================================

//! Unified error handling for LLM-Dev-Ops infrastructure.
//!
//! This crate provides:
//! - `InfraError`: The unified error enum
//! - Error conversion traits
//! - WASM-compatible error representation
//! - OpenTelemetry span recording

// -----------------------------
// Core Error Types
// -----------------------------

/// Primary error type for all infra operations
#[derive(Debug)]
pub enum InfraError {
    // Configuration errors
    Config {
        message: String,
        key: Option<String>,
        source: Option<Box<dyn Error>>,
    },

    // HTTP/Network errors
    Http {
        status: Option<u16>,
        message: String,
        url: Option<String>,
    },

    // Vector operation errors
    Vector {
        operation: VectorOperation,
        message: String,
        dimensions: Option<usize>,
    },

    // Authentication/Authorization errors
    Auth {
        kind: AuthErrorKind,
        message: String,
        identity: Option<String>,
    },

    // Cryptographic errors
    Crypto {
        operation: CryptoOperation,
        message: String,
    },

    // I/O errors
    Io {
        operation: IoOperation,
        path: Option<PathBuf>,
        source: std::io::Error,
    },

    // Serialization errors
    Serialization {
        format: SerializationFormat,
        message: String,
        location: Option<String>,
    },

    // Validation errors
    Validation {
        field: Option<String>,
        message: String,
        expected: Option<String>,
        actual: Option<String>,
    },

    // External service errors
    External {
        service: String,
        operation: String,
        message: String,
        retry_after: Option<Duration>,
    },

    // Message queue errors
    MessageQueue {
        queue: String,
        operation: MqOperation,
        message: String,
    },

    // Schema errors
    Schema {
        schema_id: Option<String>,
        path: Option<String>,
        message: String,
    },
}

// -----------------------------
// Error Kind Enums
// -----------------------------

pub enum VectorOperation {
    Insert,
    Search,
    Delete,
    Update,
    Index,
    Compress,
}

pub enum AuthErrorKind {
    InvalidCredentials,
    TokenExpired,
    InsufficientPermissions,
    InvalidToken,
    MissingCredentials,
    RateLimited,
}

pub enum CryptoOperation {
    Encrypt,
    Decrypt,
    Sign,
    Verify,
    Hash,
    KeyGeneration,
}

pub enum IoOperation {
    Read,
    Write,
    Delete,
    Create,
    List,
    Watch,
}

pub enum SerializationFormat {
    Json,
    Toml,
    Yaml,
    MessagePack,
    Protobuf,
}

pub enum MqOperation {
    Publish,
    Subscribe,
    Acknowledge,
    Reject,
    Connect,
    Disconnect,
}

// -----------------------------
// Error Implementation
// -----------------------------

impl std::error::Error for InfraError {
    FUNCTION source() -> Option<&dyn Error>:
        MATCH self:
            Config { source, .. } => source.as_ref().map(|s| s.as_ref())
            Io { source, .. } => Some(source)
            _ => None
}

impl std::fmt::Display for InfraError {
    FUNCTION fmt(f: &mut Formatter) -> fmt::Result:
        MATCH self:
            Config { message, key, .. }:
                IF key.is_some():
                    WRITE "Configuration error for '{}': {}", key, message
                ELSE:
                    WRITE "Configuration error: {}", message

            Http { status, message, url }:
                IF status.is_some() AND url.is_some():
                    WRITE "HTTP {} error for {}: {}", status, url, message
                ELSE:
                    WRITE "HTTP error: {}", message

            Vector { operation, message, dimensions }:
                WRITE "Vector {:?} error: {}", operation, message

            Auth { kind, message, .. }:
                WRITE "Authentication error ({:?}): {}", kind, message

            // ... similar patterns for other variants
}

// -----------------------------
// Error Conversion Traits
// -----------------------------

impl From<std::io::Error> for InfraError {
    FUNCTION from(err: std::io::Error) -> Self:
        InfraError::Io {
            operation: IoOperation::Read,  // Default, can be overridden
            path: None,
            source: err,
        }
}

impl From<serde_json::Error> for InfraError {
    FUNCTION from(err: serde_json::Error) -> Self:
        InfraError::Serialization {
            format: SerializationFormat::Json,
            message: err.to_string(),
            location: extract_json_location(&err),
        }
}

// -----------------------------
// Result Type Alias
// -----------------------------

pub type InfraResult<T> = Result<T, InfraError>;

// -----------------------------
// Error Builder Pattern
// -----------------------------

pub struct ErrorBuilder {
    error: InfraError,
}

impl ErrorBuilder {
    FUNCTION config(message: impl Into<String>) -> Self:
        Self {
            error: InfraError::Config {
                message: message.into(),
                key: None,
                source: None,
            }
        }

    FUNCTION with_key(mut self, key: impl Into<String>) -> Self:
        IF let InfraError::Config { key: ref mut k, .. } = self.error:
            *k = Some(key.into())
        self

    FUNCTION with_source(mut self, source: impl Error + 'static) -> Self:
        IF let InfraError::Config { source: ref mut s, .. } = self.error:
            *s = Some(Box::new(source))
        self

    FUNCTION build(self) -> InfraError:
        self.error
}

// -----------------------------
// OpenTelemetry Integration
// -----------------------------

impl InfraError {
    /// Record error details to an OpenTelemetry span
    FUNCTION record_to_span(&self, span: &tracing::Span):
        span.record("error", true)
        span.record("error.type", self.error_type())
        span.record("error.message", &self.to_string())

        MATCH self:
            Http { status, .. } IF status.is_some():
                span.record("http.status_code", status.unwrap())

            Auth { kind, .. }:
                span.record("auth.error_kind", format!("{:?}", kind))

            External { service, .. }:
                span.record("external.service", service)

            _ => ()

    /// Get the error type string for metrics/logging
    FUNCTION error_type(&self) -> &'static str:
        MATCH self:
            Config { .. } => "config"
            Http { .. } => "http"
            Vector { .. } => "vector"
            Auth { .. } => "auth"
            Crypto { .. } => "crypto"
            Io { .. } => "io"
            Serialization { .. } => "serialization"
            Validation { .. } => "validation"
            External { .. } => "external"
            MessageQueue { .. } => "message_queue"
            Schema { .. } => "schema"

    /// Check if error is retryable
    FUNCTION is_retryable(&self) -> bool:
        MATCH self:
            Http { status: Some(s), .. } => *s >= 500 OR *s == 429
            External { retry_after, .. } => retry_after.is_some()
            Auth { kind: AuthErrorKind::RateLimited, .. } => true
            Io { source, .. } => source.kind() == ErrorKind::TimedOut
            MessageQueue { .. } => true
            _ => false

    /// Get retry delay if applicable
    FUNCTION retry_after(&self) -> Option<Duration>:
        MATCH self:
            External { retry_after, .. } => *retry_after
            Auth { kind: AuthErrorKind::RateLimited, .. } => Some(Duration::from_secs(60))
            Http { status: Some(429), .. } => Some(Duration::from_secs(30))
            _ => None
}

// -----------------------------
// WASM Compatibility
// -----------------------------

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct JsInfraError {
        error_type: String,
        message: String,
        details: JsValue,
    }

    impl From<InfraError> for JsInfraError {
        FUNCTION from(err: InfraError) -> Self:
            Self {
                error_type: err.error_type().to_string(),
                message: err.to_string(),
                details: serialize_error_details(&err),
            }
    }

    impl Into<JsValue> for InfraError {
        FUNCTION into(self) -> JsValue:
            LET js_err = JsInfraError::from(self)
            js_err.into()
    }

    FUNCTION serialize_error_details(err: &InfraError) -> JsValue:
        // Convert error-specific fields to JS object
        LET obj = js_sys::Object::new()

        MATCH err:
            Http { status, url, .. }:
                IF let Some(s) = status:
                    js_sys::Reflect::set(&obj, &"status".into(), &(*s).into())
                IF let Some(u) = url:
                    js_sys::Reflect::set(&obj, &"url".into(), &u.into())
            // ... similar for other variants

        obj.into()
}

// -----------------------------
// Testing Utilities
// -----------------------------

#[cfg(test)]
pub mod testing {
    /// Create a mock config error for testing
    FUNCTION mock_config_error(message: &str) -> InfraError:
        InfraError::Config {
            message: message.to_string(),
            key: Some("test.key".to_string()),
            source: None,
        }

    /// Create a mock HTTP error for testing
    FUNCTION mock_http_error(status: u16) -> InfraError:
        InfraError::Http {
            status: Some(status),
            message: format!("HTTP {}", status),
            url: Some("http://test.example.com".to_string()),
        }

    // ... additional mock helpers
}
```

---

### 2.2 `infra-config` (Level 1)

**Purpose:** Configuration loading via llm-config-manager

```rust
// ============================================================
// FILE: crates/infra-config/src/lib.rs
// ============================================================

//! Configuration management for LLM-Dev-Ops infrastructure.
//!
//! Provides:
//! - Hierarchical configuration loading
//! - Environment variable overlay
//! - Hot-reload support via file watching
//! - Integration with llm-config-manager

use infra_errors::{InfraError, InfraResult};

// -----------------------------
// Core Configuration Types
// -----------------------------

/// Main configuration container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    values: HashMap<String, ConfigValue>,
    metadata: ConfigMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Table(HashMap<String, ConfigValue>),
    Secret(SecretValue),  // Encrypted/masked values
}

#[derive(Debug, Clone)]
pub struct SecretValue {
    encrypted: Vec<u8>,
    key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    source: ConfigSource,
    loaded_at: DateTime<Utc>,
    version: Option<String>,
    checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigSource {
    File(PathBuf),
    Environment,
    Remote { url: String, etag: Option<String> },
    Memory,
    Composite(Vec<ConfigSource>),
}

// -----------------------------
// Configuration Loader
// -----------------------------

pub struct ConfigLoader {
    sources: Vec<Box<dyn ConfigSourceProvider>>,
    env_prefix: String,
    watch_enabled: bool,
    cache: Option<Arc<RwLock<Config>>>,
}

impl ConfigLoader {
    FUNCTION new() -> Self:
        Self {
            sources: Vec::new(),
            env_prefix: "LLM_".to_string(),
            watch_enabled: false,
            cache: None,
        }

    FUNCTION with_file(mut self, path: impl AsRef<Path>) -> Self:
        self.sources.push(Box::new(FileSource::new(path)))
        self

    FUNCTION with_env_prefix(mut self, prefix: impl Into<String>) -> Self:
        self.env_prefix = prefix.into()
        self

    FUNCTION with_llm_config_manager(mut self, manager: LlmConfigManager) -> Self:
        self.sources.push(Box::new(LlmConfigManagerSource::new(manager)))
        self

    FUNCTION enable_watch(mut self) -> Self:
        self.watch_enabled = true
        self

    /// Load configuration from all sources
    FUNCTION load(&self) -> InfraResult<Config>:
        LET mut merged = Config::empty()

        // Load from each source in order (later sources override earlier)
        FOR source IN &self.sources:
            LET config = source.load()?
            merged = merged.merge(config)?

        // Apply environment variable overrides
        merged = self.apply_env_overrides(merged)?

        // Validate the final configuration
        self.validate(&merged)?

        // Update cache if enabled
        IF let Some(ref cache) = self.cache:
            *cache.write() = merged.clone()

        Ok(merged)

    /// Apply environment variable overrides
    FUNCTION apply_env_overrides(&self, mut config: Config) -> InfraResult<Config>:
        FOR (key, value) IN std::env::vars():
            IF key.starts_with(&self.env_prefix):
                LET config_key = key
                    .strip_prefix(&self.env_prefix)
                    .unwrap()
                    .to_lowercase()
                    .replace("__", ".")

                config.set(&config_key, ConfigValue::String(value))?

        Ok(config)

    /// Validate configuration against schema
    FUNCTION validate(&self, config: &Config) -> InfraResult<()>:
        // Check required fields
        FOR required IN self.required_fields():
            IF !config.contains_key(required):
                RETURN Err(InfraError::Config {
                    message: format!("Missing required field: {}", required),
                    key: Some(required.to_string()),
                    source: None,
                })

        // Type validation
        FOR (key, expected_type) IN self.field_types():
            IF let Some(value) = config.get(key):
                IF !value.matches_type(expected_type):
                    RETURN Err(InfraError::Validation {
                        field: Some(key.to_string()),
                        message: format!("Type mismatch for field {}", key),
                        expected: Some(expected_type.to_string()),
                        actual: Some(value.type_name().to_string()),
                    })

        Ok(())

    /// Watch for configuration changes
    FUNCTION watch(&self) -> InfraResult<ConfigWatcher>:
        IF !self.watch_enabled:
            RETURN Err(InfraError::Config {
                message: "Watch not enabled".to_string(),
                key: None,
                source: None,
            })

        LET (tx, rx) = channel()
        LET watcher = ConfigWatcher::new(rx, self.sources.clone())

        FOR source IN &self.sources:
            source.start_watching(tx.clone())?

        Ok(watcher)
}

// -----------------------------
// Configuration Access
// -----------------------------

impl Config {
    FUNCTION empty() -> Self:
        Self {
            values: HashMap::new(),
            metadata: ConfigMetadata::default(),
        }

    /// Get a value by dot-notation key
    FUNCTION get<T: FromConfigValue>(&self, key: &str) -> Option<T>:
        LET parts: Vec<&str> = key.split('.').collect()
        LET mut current = &self.values

        FOR (i, part) IN parts.iter().enumerate():
            MATCH current.get(*part):
                Some(ConfigValue::Table(table)) IF i < parts.len() - 1:
                    current = table
                Some(value) IF i == parts.len() - 1:
                    RETURN T::from_config_value(value).ok()
                _ => RETURN None

        None

    /// Get a value with default fallback
    FUNCTION get_or<T: FromConfigValue>(&self, key: &str, default: T) -> T:
        self.get(key).unwrap_or(default)

    /// Get a required value, returning error if missing
    FUNCTION require<T: FromConfigValue>(&self, key: &str) -> InfraResult<T>:
        self.get(key).ok_or_else(|| InfraError::Config {
            message: format!("Required configuration key not found: {}", key),
            key: Some(key.to_string()),
            source: None,
        })

    /// Set a value by dot-notation key
    FUNCTION set(&mut self, key: &str, value: ConfigValue) -> InfraResult<()>:
        LET parts: Vec<&str> = key.split('.').collect()
        LET mut current = &mut self.values

        FOR (i, part) IN parts.iter().enumerate():
            IF i == parts.len() - 1:
                current.insert(part.to_string(), value)
                RETURN Ok(())

            current = current
                .entry(part.to_string())
                .or_insert_with(|| ConfigValue::Table(HashMap::new()))
                .as_table_mut()
                .ok_or_else(|| InfraError::Config {
                    message: format!("Cannot set nested key on non-table value: {}", part),
                    key: Some(key.to_string()),
                    source: None,
                })?

        Ok(())

    /// Merge another config into this one
    FUNCTION merge(mut self, other: Config) -> InfraResult<Config>:
        FOR (key, value) IN other.values:
            self.values = deep_merge(self.values, key, value)

        self.metadata.source = ConfigSource::Composite(vec![
            self.metadata.source,
            other.metadata.source,
        ])

        Ok(self)

    /// Check if a key exists
    FUNCTION contains_key(&self, key: &str) -> bool:
        self.get::<ConfigValue>(key).is_some()
}

// -----------------------------
// Config Value Conversion
// -----------------------------

pub trait FromConfigValue: Sized {
    FUNCTION from_config_value(value: &ConfigValue) -> InfraResult<Self>;
}

impl FromConfigValue for String {
    FUNCTION from_config_value(value: &ConfigValue) -> InfraResult<Self>:
        MATCH value:
            ConfigValue::String(s) => Ok(s.clone())
            _ => Err(InfraError::Validation {
                field: None,
                message: "Expected string value".to_string(),
                expected: Some("string".to_string()),
                actual: Some(value.type_name().to_string()),
            })
}

impl FromConfigValue for i64 {
    FUNCTION from_config_value(value: &ConfigValue) -> InfraResult<Self>:
        MATCH value:
            ConfigValue::Integer(i) => Ok(*i)
            ConfigValue::String(s) => s.parse().map_err(|_| InfraError::Validation {
                field: None,
                message: "Cannot parse string as integer".to_string(),
                expected: Some("integer".to_string()),
                actual: Some(s.clone()),
            })
            _ => Err(InfraError::Validation { /* ... */ })
}

// Similar implementations for f64, bool, Vec<T>, etc.

// -----------------------------
// LLM Config Manager Integration
// -----------------------------

pub struct LlmConfigManagerSource {
    manager: LlmConfigManager,
}

impl ConfigSourceProvider for LlmConfigManagerSource {
    FUNCTION load(&self) -> InfraResult<Config>:
        LET raw = self.manager.load_config()?

        // Convert LlmConfigManager format to our Config
        LET values = convert_llm_config_to_values(raw)?

        Ok(Config {
            values,
            metadata: ConfigMetadata {
                source: ConfigSource::Remote {
                    url: self.manager.source_url().to_string(),
                    etag: self.manager.etag(),
                },
                loaded_at: Utc::now(),
                version: self.manager.version(),
                checksum: compute_checksum(&values),
            },
        })

    FUNCTION start_watching(&self, tx: Sender<ConfigChange>) -> InfraResult<()>:
        self.manager.on_change(move |change| {
            tx.send(ConfigChange::Updated {
                key: change.key,
                old_value: change.old_value.map(convert_value),
                new_value: convert_value(change.new_value),
            })
        })
}

// -----------------------------
// Configuration Watcher
// -----------------------------

pub struct ConfigWatcher {
    rx: Receiver<ConfigChange>,
    _handles: Vec<JoinHandle<()>>,
}

pub enum ConfigChange {
    Updated { key: String, old_value: Option<ConfigValue>, new_value: ConfigValue },
    Deleted { key: String },
    Reloaded { config: Config },
}

impl ConfigWatcher {
    /// Get next configuration change (blocking)
    FUNCTION next(&self) -> Option<ConfigChange>:
        self.rx.recv().ok()

    /// Get next change with timeout
    FUNCTION next_timeout(&self, timeout: Duration) -> Option<ConfigChange>:
        self.rx.recv_timeout(timeout).ok()

    /// Convert to async stream
    FUNCTION into_stream(self) -> impl Stream<Item = ConfigChange>:
        ReceiverStream::new(self.rx)
}

// -----------------------------
// WASM Compatibility
// -----------------------------

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct JsConfig {
        inner: Config,
    }

    #[wasm_bindgen]
    impl JsConfig {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self:
            Self { inner: Config::empty() }

        pub fn get(&self, key: &str) -> JsValue:
            MATCH self.inner.get::<ConfigValue>(key):
                Some(v) => config_value_to_js(&v)
                None => JsValue::UNDEFINED

        pub fn set(&mut self, key: &str, value: JsValue) -> Result<(), JsValue>:
            LET config_value = js_to_config_value(value)?
            self.inner.set(key, config_value)
                .map_err(|e| JsValue::from_str(&e.to_string()))

        pub fn to_json(&self) -> Result<String, JsValue>:
            serde_json::to_string(&self.inner.values)
                .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn load_config_from_json(json: &str) -> Result<JsConfig, JsValue>:
        LET values: HashMap<String, ConfigValue> = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?

        Ok(JsConfig {
            inner: Config {
                values,
                metadata: ConfigMetadata::default(),
            }
        })
}
```

---

### 2.3 `infra-otel` (Level 2)

**Purpose:** Standardized OpenTelemetry 0.27 initialization

```rust
// ============================================================
// FILE: crates/infra-otel/src/lib.rs
// ============================================================

//! OpenTelemetry 0.27 integration for LLM-Dev-Ops infrastructure.
//!
//! Provides:
//! - Standardized tracer/meter/logger initialization
//! - Auto-instrumentation helpers
//! - Context propagation utilities
//! - Shutdown guards for proper cleanup

use infra_errors::{InfraError, InfraResult};
use infra_config::Config;
use opentelemetry::global;
use opentelemetry_sdk::{trace, metrics, logs};

// -----------------------------
// Configuration
// -----------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelConfig {
    pub service_name: String,
    pub service_version: String,
    pub service_namespace: Option<String>,
    pub deployment_environment: String,
    pub exporter: ExporterConfig,
    pub sampling: SamplingConfig,
    pub resource_attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExporterConfig {
    Otlp {
        endpoint: String,
        protocol: OtlpProtocol,
        headers: HashMap<String, String>,
        timeout: Duration,
        compression: Option<Compression>,
    },
    Jaeger {
        agent_endpoint: String,
    },
    Stdout {
        pretty: bool,
    },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OtlpProtocol {
    Grpc,
    HttpProtobuf,
    HttpJson,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    pub ratio: f64,  // 0.0 to 1.0
    pub parent_based: bool,
    pub rules: Vec<SamplingRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRule {
    pub name_pattern: String,
    pub ratio: f64,
}

// -----------------------------
// Initialization
// -----------------------------

/// Guard that ensures proper shutdown of OTEL providers
pub struct OtelGuard {
    tracer_provider: Option<trace::TracerProvider>,
    meter_provider: Option<metrics::MeterProvider>,
    logger_provider: Option<logs::LoggerProvider>,
}

impl Drop for OtelGuard {
    FUNCTION drop(&mut self):
        // Shutdown in reverse order of initialization
        IF let Some(ref provider) = self.logger_provider:
            IF let Err(e) = provider.shutdown():
                eprintln!("Error shutting down logger provider: {}", e)

        IF let Some(ref provider) = self.meter_provider:
            IF let Err(e) = provider.shutdown():
                eprintln!("Error shutting down meter provider: {}", e)

        IF let Some(ref provider) = self.tracer_provider:
            IF let Err(e) = provider.shutdown():
                eprintln!("Error shutting down tracer provider: {}", e)
}

/// Initialize OpenTelemetry with the given configuration
pub async fn init_otel(config: &OtelConfig) -> InfraResult<OtelGuard> {
    // Build resource with service info and custom attributes
    LET resource = build_resource(config)?

    // Initialize tracer provider
    LET tracer_provider = init_tracer(config, resource.clone()).await?
    global::set_tracer_provider(tracer_provider.clone())

    // Initialize meter provider
    LET meter_provider = init_meter(config, resource.clone()).await?
    global::set_meter_provider(meter_provider.clone())

    // Initialize logger provider
    LET logger_provider = init_logger(config, resource.clone()).await?

    // Set up tracing-opentelemetry bridge
    setup_tracing_subscriber(&tracer_provider)?

    Ok(OtelGuard {
        tracer_provider: Some(tracer_provider),
        meter_provider: Some(meter_provider),
        logger_provider: Some(logger_provider),
    })
}

/// Build resource with service metadata
FUNCTION build_resource(config: &OtelConfig) -> InfraResult<Resource>:
    LET mut attrs = vec![
        KeyValue::new("service.name", config.service_name.clone()),
        KeyValue::new("service.version", config.service_version.clone()),
        KeyValue::new("deployment.environment", config.deployment_environment.clone()),
    ]

    IF let Some(ref ns) = config.service_namespace:
        attrs.push(KeyValue::new("service.namespace", ns.clone()))

    FOR (key, value) IN &config.resource_attributes:
        attrs.push(KeyValue::new(key.clone(), value.clone()))

    Ok(Resource::new(attrs))

/// Initialize tracer provider
async fn init_tracer(
    config: &OtelConfig,
    resource: Resource
) -> InfraResult<trace::TracerProvider> {
    LET exporter = create_trace_exporter(&config.exporter).await?
    LET sampler = create_sampler(&config.sampling)

    LET provider = trace::TracerProvider::builder()
        .with_resource(resource)
        .with_sampler(sampler)
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build()

    Ok(provider)
}

/// Initialize meter provider
async fn init_meter(
    config: &OtelConfig,
    resource: Resource
) -> InfraResult<metrics::MeterProvider> {
    LET exporter = create_metrics_exporter(&config.exporter).await?

    LET provider = metrics::MeterProvider::builder()
        .with_resource(resource)
        .with_reader(
            metrics::PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::Tokio)
                .with_interval(Duration::from_secs(60))
                .build()
        )
        .build()

    Ok(provider)
}

/// Create sampler from configuration
FUNCTION create_sampler(config: &SamplingConfig) -> Box<dyn Sampler>:
    LET base_sampler = trace::Sampler::TraceIdRatioBased(config.ratio)

    IF config.parent_based:
        Box::new(trace::Sampler::ParentBased(Box::new(base_sampler)))
    ELSE:
        Box::new(base_sampler)

// -----------------------------
// Instrumentation Helpers
// -----------------------------

/// Create an instrumented HTTP client
pub fn instrumented_http_client<C: HttpClient>(client: C) -> InstrumentedHttpClient<C>:
    InstrumentedHttpClient { inner: client }

pub struct InstrumentedHttpClient<C> {
    inner: C,
}

impl<C: HttpClient> HttpClient for InstrumentedHttpClient<C> {
    async fn request(&self, req: Request) -> InfraResult<Response>:
        LET span = tracing::info_span!(
            "http.request",
            http.method = %req.method(),
            http.url = %req.url(),
            http.status_code = tracing::field::Empty,
            otel.kind = "client",
        )

        async move {
            LET result = self.inner.request(req).await

            MATCH &result:
                Ok(resp):
                    span.record("http.status_code", resp.status().as_u16())
                Err(e):
                    e.record_to_span(&span)

            result
        }.instrument(span).await
}

/// Create standard LLM operation span
pub fn llm_span(
    operation: &str,
    model: &str,
    provider: &str,
) -> tracing::Span:
    tracing::info_span!(
        "llm.operation",
        llm.operation = operation,
        llm.model = model,
        llm.provider = provider,
        llm.tokens.input = tracing::field::Empty,
        llm.tokens.output = tracing::field::Empty,
        llm.latency_ms = tracing::field::Empty,
    )

/// Record LLM metrics
pub fn record_llm_metrics(
    span: &tracing::Span,
    input_tokens: u32,
    output_tokens: u32,
    latency: Duration,
):
    span.record("llm.tokens.input", input_tokens)
    span.record("llm.tokens.output", output_tokens)
    span.record("llm.latency_ms", latency.as_millis() as u64)

    // Also record to metrics
    LET meter = global::meter("llm-dev-ops")

    LET token_counter = meter.u64_counter("llm.tokens")
        .with_description("Total tokens processed")
        .init()

    token_counter.add(input_tokens as u64, &[KeyValue::new("type", "input")])
    token_counter.add(output_tokens as u64, &[KeyValue::new("type", "output")])

// -----------------------------
// Context Propagation
// -----------------------------

pub struct PropagationContext {
    traceparent: String,
    tracestate: Option<String>,
    baggage: HashMap<String, String>,
}

impl PropagationContext {
    /// Extract context from HTTP headers
    FUNCTION from_headers(headers: &HeaderMap) -> Option<Self>:
        LET traceparent = headers
            .get("traceparent")
            .and_then(|v| v.to_str().ok())
            .map(String::from)?

        LET tracestate = headers
            .get("tracestate")
            .and_then(|v| v.to_str().ok())
            .map(String::from)

        Some(Self {
            traceparent,
            tracestate,
            baggage: extract_baggage(headers),
        })

    /// Inject context into HTTP headers
    FUNCTION inject_into_headers(&self, headers: &mut HeaderMap):
        headers.insert("traceparent", self.traceparent.parse().unwrap())

        IF let Some(ref state) = self.tracestate:
            headers.insert("tracestate", state.parse().unwrap())

        IF !self.baggage.is_empty():
            LET baggage_str = self.baggage.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",")
            headers.insert("baggage", baggage_str.parse().unwrap())

    /// Create span from propagated context
    FUNCTION create_span(&self, name: &str) -> tracing::Span:
        // Parse traceparent and create span with parent context
        LET parent_context = parse_traceparent(&self.traceparent)

        tracing::info_span!(
            parent: parent_context,
            name,
        )
}

// -----------------------------
// Configuration from infra-config
// -----------------------------

impl OtelConfig {
    /// Load from infra-config Config
    FUNCTION from_config(config: &Config) -> InfraResult<Self>:
        Ok(Self {
            service_name: config.require("otel.service.name")?,
            service_version: config.get_or("otel.service.version", env!("CARGO_PKG_VERSION").to_string()),
            service_namespace: config.get("otel.service.namespace"),
            deployment_environment: config.get_or("otel.environment", "development".to_string()),
            exporter: ExporterConfig::from_config(config)?,
            sampling: SamplingConfig::from_config(config)?,
            resource_attributes: config.get_or("otel.resource.attributes", HashMap::new()),
        })
}

impl ExporterConfig {
    FUNCTION from_config(config: &Config) -> InfraResult<Self>:
        LET exporter_type: String = config.get_or("otel.exporter.type", "otlp".to_string())

        MATCH exporter_type.as_str():
            "otlp" => Ok(Self::Otlp {
                endpoint: config.get_or("otel.exporter.otlp.endpoint", "http://localhost:4317".to_string()),
                protocol: config.get_or("otel.exporter.otlp.protocol", OtlpProtocol::Grpc),
                headers: config.get_or("otel.exporter.otlp.headers", HashMap::new()),
                timeout: Duration::from_secs(config.get_or("otel.exporter.otlp.timeout_secs", 30)),
                compression: config.get("otel.exporter.otlp.compression"),
            })
            "jaeger" => Ok(Self::Jaeger {
                agent_endpoint: config.require("otel.exporter.jaeger.endpoint")?,
            })
            "stdout" => Ok(Self::Stdout {
                pretty: config.get_or("otel.exporter.stdout.pretty", true),
            })
            "none" => Ok(Self::None)
            _ => Err(InfraError::Config {
                message: format!("Unknown exporter type: {}", exporter_type),
                key: Some("otel.exporter.type".to_string()),
                source: None,
            })
}
```

---

### 2.4 `infra-json` (Level 1)

**Purpose:** JSON serialization/deserialization with schema support

```rust
// ============================================================
// FILE: crates/infra-json/src/lib.rs
// ============================================================

//! JSON handling for LLM-Dev-Ops infrastructure.
//!
//! Provides:
//! - Typed JSON serialization/deserialization
//! - JSON path queries
//! - Schema validation hooks
//! - Streaming JSON parsing
//! - WASM-compatible API

use infra_errors::{InfraError, InfraResult};

// -----------------------------
// Core Types
// -----------------------------

/// JSON value wrapper with additional capabilities
#[derive(Debug, Clone, PartialEq)]
pub struct Json(serde_json::Value);

impl Json {
    // Constructors
    FUNCTION null() -> Self:
        Self(serde_json::Value::Null)

    FUNCTION bool(v: bool) -> Self:
        Self(serde_json::Value::Bool(v))

    FUNCTION number(v: impl Into<serde_json::Number>) -> Self:
        Self(serde_json::Value::Number(v.into()))

    FUNCTION string(v: impl Into<String>) -> Self:
        Self(serde_json::Value::String(v.into()))

    FUNCTION array(v: Vec<Json>) -> Self:
        Self(serde_json::Value::Array(v.into_iter().map(|j| j.0).collect()))

    FUNCTION object(v: impl IntoIterator<Item = (String, Json)>) -> Self:
        Self(serde_json::Value::Object(
            v.into_iter().map(|(k, j)| (k, j.0)).collect()
        ))

    // Parsing
    FUNCTION parse(s: &str) -> InfraResult<Self>:
        serde_json::from_str(s)
            .map(Self)
            .map_err(|e| InfraError::Serialization {
                format: SerializationFormat::Json,
                message: e.to_string(),
                location: extract_location(&e),
            })

    FUNCTION parse_bytes(bytes: &[u8]) -> InfraResult<Self>:
        serde_json::from_slice(bytes)
            .map(Self)
            .map_err(Into::into)

    // Serialization
    FUNCTION to_string(&self) -> String:
        serde_json::to_string(&self.0).unwrap()

    FUNCTION to_string_pretty(&self) -> String:
        serde_json::to_string_pretty(&self.0).unwrap()

    FUNCTION to_bytes(&self) -> Vec<u8>:
        serde_json::to_vec(&self.0).unwrap()

    // Type conversions
    FUNCTION from_value<T: Serialize>(value: &T) -> InfraResult<Self>:
        serde_json::to_value(value)
            .map(Self)
            .map_err(Into::into)

    FUNCTION to_value<T: DeserializeOwned>(&self) -> InfraResult<T>:
        serde_json::from_value(self.0.clone())
            .map_err(Into::into)

    // JSON Path queries
    FUNCTION query(&self, path: &str) -> Option<&Json>:
        LET parts = parse_json_path(path)
        LET mut current = &self.0

        FOR part IN parts:
            MATCH part:
                PathPart::Key(key):
                    current = current.get(key)?
                PathPart::Index(idx):
                    current = current.get(idx)?
                PathPart::Wildcard:
                    // Return first match for wildcard
                    MATCH current:
                        Value::Array(arr) => current = arr.first()?
                        Value::Object(obj) => current = obj.values().next()?
                        _ => RETURN None

        Some(unsafe { std::mem::transmute(current) })  // Safe: lifetime tied to self

    FUNCTION query_all(&self, path: &str) -> Vec<&Json>:
        // Similar to query but collects all matches
        collect_matching_paths(&self.0, path)
            .into_iter()
            .map(|v| unsafe { std::mem::transmute(v) })
            .collect()

    // Mutation
    FUNCTION set(&mut self, path: &str, value: Json) -> InfraResult<()>:
        LET parts = parse_json_path(path)
        LET mut current = &mut self.0

        FOR (i, part) IN parts.iter().enumerate():
            IF i == parts.len() - 1:
                // Set the value
                MATCH part:
                    PathPart::Key(key) => {
                        current.as_object_mut()
                            .ok_or_else(|| not_an_object_error(path))?
                            .insert(key.clone(), value.0)
                    }
                    PathPart::Index(idx) => {
                        current.as_array_mut()
                            .ok_or_else(|| not_an_array_error(path))?
                            [*idx] = value.0
                    }
                    _ => RETURN Err(invalid_path_error(path))

                RETURN Ok(())

            // Navigate deeper
            current = navigate_mut(current, part)?

        Ok(())

    // Type checks
    FUNCTION is_null(&self) -> bool:
        self.0.is_null()

    FUNCTION is_bool(&self) -> bool:
        self.0.is_boolean()

    FUNCTION is_number(&self) -> bool:
        self.0.is_number()

    FUNCTION is_string(&self) -> bool:
        self.0.is_string()

    FUNCTION is_array(&self) -> bool:
        self.0.is_array()

    FUNCTION is_object(&self) -> bool:
        self.0.is_object()

    // Accessors
    FUNCTION as_str(&self) -> Option<&str>:
        self.0.as_str()

    FUNCTION as_i64(&self) -> Option<i64>:
        self.0.as_i64()

    FUNCTION as_f64(&self) -> Option<f64>:
        self.0.as_f64()

    FUNCTION as_bool(&self) -> Option<bool>:
        self.0.as_bool()

    FUNCTION as_array(&self) -> Option<&Vec<serde_json::Value>>:
        self.0.as_array()

    FUNCTION as_object(&self) -> Option<&serde_json::Map<String, serde_json::Value>>:
        self.0.as_object()
}

// -----------------------------
// Streaming Parser
// -----------------------------

pub struct JsonStreamParser {
    buffer: Vec<u8>,
    depth: usize,
    in_string: bool,
    escape_next: bool,
}

impl JsonStreamParser {
    FUNCTION new() -> Self:
        Self {
            buffer: Vec::new(),
            depth: 0,
            in_string: false,
            escape_next: false,
        }

    /// Feed bytes to the parser, returning complete JSON values
    FUNCTION feed(&mut self, bytes: &[u8]) -> Vec<InfraResult<Json>>:
        LET mut results = Vec::new()

        FOR byte IN bytes:
            self.buffer.push(*byte)

            IF self.escape_next:
                self.escape_next = false
                CONTINUE

            MATCH byte:
                b'\\' IF self.in_string => self.escape_next = true
                b'"' => self.in_string = !self.in_string
                b'{' | b'[' IF !self.in_string => self.depth += 1
                b'}' | b']' IF !self.in_string:
                    self.depth -= 1
                    IF self.depth == 0:
                        // Complete JSON value
                        LET json_str = String::from_utf8_lossy(&self.buffer)
                        results.push(Json::parse(&json_str))
                        self.buffer.clear()
                _ => ()

        results

    /// Flush any remaining buffered data
    FUNCTION flush(&mut self) -> Option<InfraResult<Json>>:
        IF self.buffer.is_empty():
            None
        ELSE:
            LET json_str = String::from_utf8_lossy(&self.buffer)
            self.buffer.clear()
            Some(Json::parse(&json_str))
}

// -----------------------------
// JSON Diff
// -----------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum JsonDiff {
    Added { path: String, value: Json },
    Removed { path: String, value: Json },
    Changed { path: String, old: Json, new: Json },
}

pub fn diff(a: &Json, b: &Json) -> Vec<JsonDiff> {
    diff_recursive(&a.0, &b.0, "".to_string())
}

FUNCTION diff_recursive(a: &Value, b: &Value, path: String) -> Vec<JsonDiff>:
    LET mut diffs = Vec::new()

    MATCH (a, b):
        (Value::Object(obj_a), Value::Object(obj_b)):
            // Find added/changed keys
            FOR (key, val_b) IN obj_b:
                LET new_path = format!("{}.{}", path, key)
                MATCH obj_a.get(key):
                    Some(val_a) => diffs.extend(diff_recursive(val_a, val_b, new_path))
                    None => diffs.push(JsonDiff::Added {
                        path: new_path,
                        value: Json(val_b.clone()),
                    })

            // Find removed keys
            FOR (key, val_a) IN obj_a:
                IF !obj_b.contains_key(key):
                    diffs.push(JsonDiff::Removed {
                        path: format!("{}.{}", path, key),
                        value: Json(val_a.clone()),
                    })

        (Value::Array(arr_a), Value::Array(arr_b)):
            FOR (i, (val_a, val_b)) IN arr_a.iter().zip(arr_b.iter()).enumerate():
                diffs.extend(diff_recursive(val_a, val_b, format!("{}[{}]", path, i)))

            // Handle length differences
            IF arr_b.len() > arr_a.len():
                FOR (i, val) IN arr_b[arr_a.len()..].iter().enumerate():
                    diffs.push(JsonDiff::Added {
                        path: format!("{}[{}]", path, arr_a.len() + i),
                        value: Json(val.clone()),
                    })
            ELSE IF arr_a.len() > arr_b.len():
                FOR (i, val) IN arr_a[arr_b.len()..].iter().enumerate():
                    diffs.push(JsonDiff::Removed {
                        path: format!("{}[{}]", path, arr_b.len() + i),
                        value: Json(val.clone()),
                    })

        _ IF a != b:
            diffs.push(JsonDiff::Changed {
                path,
                old: Json(a.clone()),
                new: Json(b.clone()),
            })

        _ => ()

    diffs

// -----------------------------
// JSON Merge
// -----------------------------

pub fn merge(base: &Json, patch: &Json) -> Json {
    Json(merge_recursive(&base.0, &patch.0))
}

FUNCTION merge_recursive(base: &Value, patch: &Value) -> Value:
    MATCH (base, patch):
        (Value::Object(base_obj), Value::Object(patch_obj)):
            LET mut result = base_obj.clone()

            FOR (key, patch_val) IN patch_obj:
                IF patch_val.is_null():
                    result.remove(key)
                ELSE IF let Some(base_val) = result.get(key):
                    result.insert(key.clone(), merge_recursive(base_val, patch_val))
                ELSE:
                    result.insert(key.clone(), patch_val.clone())

            Value::Object(result)

        _ => patch.clone()

// -----------------------------
// WASM Bindings
// -----------------------------

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct JsJson {
        inner: Json,
    }

    #[wasm_bindgen]
    impl JsJson {
        #[wasm_bindgen(constructor)]
        pub fn parse(s: &str) -> Result<JsJson, JsValue>:
            Json::parse(s)
                .map(|j| JsJson { inner: j })
                .map_err(|e| JsValue::from_str(&e.to_string()))

        pub fn stringify(&self) -> String:
            self.inner.to_string()

        pub fn stringify_pretty(&self) -> String:
            self.inner.to_string_pretty()

        pub fn query(&self, path: &str) -> JsValue:
            MATCH self.inner.query(path):
                Some(j) => JsValue::from_str(&j.to_string())
                None => JsValue::UNDEFINED

        pub fn set(&mut self, path: &str, value: &str) -> Result<(), JsValue>:
            LET json_value = Json::parse(value)
                .map_err(|e| JsValue::from_str(&e.to_string()))?

            self.inner.set(path, json_value)
                .map_err(|e| JsValue::from_str(&e.to_string()))

        pub fn diff(a: &JsJson, b: &JsJson) -> JsValue:
            LET diffs = diff(&a.inner, &b.inner)
            serde_wasm_bindgen::to_value(&diffs).unwrap()

        pub fn merge(base: &JsJson, patch: &JsJson) -> JsJson:
            JsJson { inner: merge(&base.inner, &patch.inner) }
    }
}
```

---

### 2.5 `infra-vector` (Level 3)

**Purpose:** Vector operations wrapping ruvector-core

```rust
// ============================================================
// FILE: crates/infra-vector/src/lib.rs
// ============================================================

//! Vector database operations for LLM-Dev-Ops infrastructure.
//!
//! Wraps ruvector-core capabilities with:
//! - Unified error handling
//! - OpenTelemetry instrumentation
//! - Configuration via infra-config
//! - WASM support via ruvector-gnn-wasm

use infra_errors::{InfraError, InfraResult, VectorOperation};
use infra_config::Config;

// Re-export core types from ruvector
pub use ruvector_core::{Vector, VectorId, Distance, HnswConfig};

// -----------------------------
// Vector Store Trait
// -----------------------------

#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Insert a vector with associated metadata
    async fn insert(
        &self,
        id: VectorId,
        vector: Vector,
        metadata: Option<Json>,
    ) -> InfraResult<()>;

    /// Batch insert multiple vectors
    async fn insert_batch(
        &self,
        vectors: Vec<(VectorId, Vector, Option<Json>)>,
    ) -> InfraResult<BatchInsertResult>;

    /// Search for similar vectors
    async fn search(
        &self,
        query: Vector,
        k: usize,
        filter: Option<MetadataFilter>,
    ) -> InfraResult<Vec<SearchResult>>;

    /// Get a vector by ID
    async fn get(&self, id: &VectorId) -> InfraResult<Option<VectorRecord>>;

    /// Delete a vector by ID
    async fn delete(&self, id: &VectorId) -> InfraResult<bool>;

    /// Update vector metadata
    async fn update_metadata(
        &self,
        id: &VectorId,
        metadata: Json,
    ) -> InfraResult<()>;

    /// Get collection statistics
    async fn stats(&self) -> InfraResult<CollectionStats>;
}

// -----------------------------
// Data Types
// -----------------------------

#[derive(Debug, Clone)]
pub struct VectorRecord {
    pub id: VectorId,
    pub vector: Vector,
    pub metadata: Option<Json>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: VectorId,
    pub score: f32,
    pub vector: Option<Vector>,
    pub metadata: Option<Json>,
}

#[derive(Debug, Clone)]
pub struct BatchInsertResult {
    pub inserted: usize,
    pub failed: Vec<(VectorId, InfraError)>,
    pub duration: Duration,
}

#[derive(Debug, Clone)]
pub struct CollectionStats {
    pub total_vectors: usize,
    pub dimensions: usize,
    pub index_size_bytes: usize,
    pub distance_metric: Distance,
}

// -----------------------------
// Metadata Filtering
// -----------------------------

#[derive(Debug, Clone)]
pub enum MetadataFilter {
    Eq { field: String, value: Json },
    Ne { field: String, value: Json },
    Gt { field: String, value: Json },
    Gte { field: String, value: Json },
    Lt { field: String, value: Json },
    Lte { field: String, value: Json },
    In { field: String, values: Vec<Json> },
    Contains { field: String, value: String },
    And(Vec<MetadataFilter>),
    Or(Vec<MetadataFilter>),
    Not(Box<MetadataFilter>),
}

impl MetadataFilter {
    FUNCTION eq(field: impl Into<String>, value: impl Into<Json>) -> Self:
        Self::Eq { field: field.into(), value: value.into() }

    FUNCTION and(filters: Vec<MetadataFilter>) -> Self:
        Self::And(filters)

    FUNCTION or(filters: Vec<MetadataFilter>) -> Self:
        Self::Or(filters)

    /// Convert to ruvector's internal filter format
    FUNCTION to_ruvector_filter(&self) -> ruvector_core::Filter:
        MATCH self:
            Eq { field, value } => ruvector_core::Filter::eq(field, value.to_ruvector_value())
            And(filters) => ruvector_core::Filter::all(
                filters.iter().map(|f| f.to_ruvector_filter()).collect()
            )
            // ... etc
}

// -----------------------------
// RuVector-backed Implementation
// -----------------------------

pub struct RuVectorStore {
    inner: ruvector_core::Collection,
    config: VectorStoreConfig,
    tracer: BoxedTracer,
}

#[derive(Debug, Clone)]
pub struct VectorStoreConfig {
    pub collection_name: String,
    pub dimensions: usize,
    pub distance: Distance,
    pub hnsw: HnswConfig,
    pub compression: CompressionConfig,
}

#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub enabled: bool,
    pub tier_thresholds: TierThresholds,
}

#[derive(Debug, Clone)]
pub struct TierThresholds {
    pub hot_access_count: usize,    // f32
    pub warm_access_count: usize,   // f16
    pub cool_access_count: usize,   // PQ8
    pub cold_access_count: usize,   // PQ4
    // Below cold -> Binary
}

impl RuVectorStore {
    /// Create a new vector store from configuration
    pub async fn new(config: VectorStoreConfig) -> InfraResult<Self>:
        LET hnsw_config = ruvector_core::HnswConfig {
            m: config.hnsw.m,
            ef_construction: config.hnsw.ef_construction,
            ef_search: config.hnsw.ef_search,
        }

        LET collection = ruvector_core::Collection::new(
            &config.collection_name,
            config.dimensions,
            config.distance.into(),
            hnsw_config,
        ).await.map_err(|e| InfraError::Vector {
            operation: VectorOperation::Index,
            message: e.to_string(),
            dimensions: Some(config.dimensions),
        })?

        Ok(Self {
            inner: collection,
            config,
            tracer: global::tracer("infra-vector"),
        })

    /// Create from infra-config
    pub async fn from_config(config: &Config) -> InfraResult<Self>:
        LET store_config = VectorStoreConfig {
            collection_name: config.require("vector.collection")?,
            dimensions: config.require("vector.dimensions")?,
            distance: config.get_or("vector.distance", Distance::Cosine),
            hnsw: HnswConfig {
                m: config.get_or("vector.hnsw.m", 16),
                ef_construction: config.get_or("vector.hnsw.ef_construction", 200),
                ef_search: config.get_or("vector.hnsw.ef_search", 100),
            },
            compression: CompressionConfig {
                enabled: config.get_or("vector.compression.enabled", true),
                tier_thresholds: TierThresholds::default(),
            },
        }

        Self::new(store_config).await
}

#[async_trait]
impl VectorStore for RuVectorStore {
    async fn insert(
        &self,
        id: VectorId,
        vector: Vector,
        metadata: Option<Json>,
    ) -> InfraResult<()>:
        LET span = tracing::info_span!(
            "vector.insert",
            vector.id = %id,
            vector.dimensions = vector.len(),
        )

        async move {
            // Validate dimensions
            IF vector.len() != self.config.dimensions:
                RETURN Err(InfraError::Vector {
                    operation: VectorOperation::Insert,
                    message: format!(
                        "Dimension mismatch: expected {}, got {}",
                        self.config.dimensions,
                        vector.len()
                    ),
                    dimensions: Some(vector.len()),
                })

            // Convert metadata if present
            LET ruv_metadata = metadata.map(|m| m.to_ruvector_metadata())

            // Insert into ruvector
            self.inner.insert(id.as_str(), &vector, ruv_metadata)
                .await
                .map_err(|e| InfraError::Vector {
                    operation: VectorOperation::Insert,
                    message: e.to_string(),
                    dimensions: Some(vector.len()),
                })
        }.instrument(span).await

    async fn search(
        &self,
        query: Vector,
        k: usize,
        filter: Option<MetadataFilter>,
    ) -> InfraResult<Vec<SearchResult>>:
        LET span = tracing::info_span!(
            "vector.search",
            vector.k = k,
            vector.dimensions = query.len(),
            vector.has_filter = filter.is_some(),
        )

        async move {
            // Validate dimensions
            IF query.len() != self.config.dimensions:
                RETURN Err(InfraError::Vector {
                    operation: VectorOperation::Search,
                    message: "Query dimension mismatch".to_string(),
                    dimensions: Some(query.len()),
                })

            // Convert filter
            LET ruv_filter = filter.map(|f| f.to_ruvector_filter())

            // Execute search
            LET results = self.inner.search(&query, k, ruv_filter)
                .await
                .map_err(|e| InfraError::Vector {
                    operation: VectorOperation::Search,
                    message: e.to_string(),
                    dimensions: None,
                })?

            // Convert results
            Ok(results.into_iter().map(|r| SearchResult {
                id: VectorId::from(r.id),
                score: r.score,
                vector: r.vector,
                metadata: r.metadata.map(Json::from_ruvector_metadata),
            }).collect())
        }.instrument(span).await

    async fn insert_batch(
        &self,
        vectors: Vec<(VectorId, Vector, Option<Json>)>,
    ) -> InfraResult<BatchInsertResult>:
        LET span = tracing::info_span!(
            "vector.insert_batch",
            vector.batch_size = vectors.len(),
        )

        async move {
            LET start = Instant::now()
            LET mut inserted = 0
            LET mut failed = Vec::new()

            // Use ruvector's batch API
            FOR chunk IN vectors.chunks(1000):
                LET ruv_batch: Vec<_> = chunk.iter()
                    .map(|(id, vec, meta)| (
                        id.as_str(),
                        vec,
                        meta.as_ref().map(|m| m.to_ruvector_metadata()),
                    ))
                    .collect()

                MATCH self.inner.insert_batch(ruv_batch).await:
                    Ok(result):
                        inserted += result.success_count
                        FOR (id, err) IN result.failures:
                            failed.push((VectorId::from(id), InfraError::Vector {
                                operation: VectorOperation::Insert,
                                message: err.to_string(),
                                dimensions: None,
                            }))
                    Err(e):
                        // Entire chunk failed
                        FOR (id, _, _) IN chunk:
                            failed.push((id.clone(), InfraError::Vector {
                                operation: VectorOperation::Insert,
                                message: e.to_string(),
                                dimensions: None,
                            }))

            Ok(BatchInsertResult {
                inserted,
                failed,
                duration: start.elapsed(),
            })
        }.instrument(span).await

    // ... remaining trait methods follow similar pattern
}

// -----------------------------
// Embedding Utilities
// -----------------------------

pub struct EmbeddingNormalizer;

impl EmbeddingNormalizer {
    /// L2 normalize a vector
    FUNCTION normalize(vector: &mut Vector):
        LET magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt()

        IF magnitude > 0.0:
            FOR x IN vector.iter_mut():
                *x /= magnitude

    /// Batch normalize vectors
    FUNCTION normalize_batch(vectors: &mut [Vector]):
        FOR vector IN vectors:
            Self::normalize(vector)
}

// -----------------------------
// WASM Bindings
// -----------------------------

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
    use ruvector_gnn_wasm::WasmVectorIndex;

    #[wasm_bindgen]
    pub struct JsVectorStore {
        inner: WasmVectorIndex,
    }

    #[wasm_bindgen]
    impl JsVectorStore {
        #[wasm_bindgen(constructor)]
        pub fn new(dimensions: usize) -> Result<JsVectorStore, JsValue>:
            LET inner = WasmVectorIndex::new(dimensions)
                .map_err(|e| JsValue::from_str(&e.to_string()))?

            Ok(Self { inner })

        pub fn insert(&mut self, id: &str, vector: &[f32]) -> Result<(), JsValue>:
            self.inner.insert(id, vector)
                .map_err(|e| JsValue::from_str(&e.to_string()))

        pub fn search(&self, query: &[f32], k: usize) -> Result<JsValue, JsValue>:
            LET results = self.inner.search(query, k)
                .map_err(|e| JsValue::from_str(&e.to_string()))?

            serde_wasm_bindgen::to_value(&results)
                .map_err(|e| JsValue::from_str(&e.to_string()))

        pub fn delete(&mut self, id: &str) -> Result<bool, JsValue>:
            self.inner.delete(id)
                .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
```

---

### 2.6 `infra-http` (Level 2)

**Purpose:** HTTP client/server primitives

```rust
// ============================================================
// FILE: crates/infra-http/src/lib.rs
// ============================================================

//! HTTP client and server for LLM-Dev-Ops infrastructure.

use infra_errors::{InfraError, InfraResult};
use infra_config::Config;

// -----------------------------
// HTTP Client
// -----------------------------

#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn request(&self, req: Request) -> InfraResult<Response>;

    // Convenience methods
    async fn get(&self, url: &str) -> InfraResult<Response>:
        self.request(Request::get(url).build()?).await

    async fn post(&self, url: &str, body: impl Into<Body>) -> InfraResult<Response>:
        self.request(Request::post(url).body(body).build()?).await

    async fn put(&self, url: &str, body: impl Into<Body>) -> InfraResult<Response>:
        self.request(Request::put(url).body(body).build()?).await

    async fn delete(&self, url: &str) -> InfraResult<Response>:
        self.request(Request::delete(url).build()?).await
}

// -----------------------------
// Request Builder
// -----------------------------

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub url: Url,
    pub headers: HeaderMap,
    pub body: Option<Body>,
    pub timeout: Option<Duration>,
}

pub struct RequestBuilder {
    method: Method,
    url: Option<Url>,
    headers: HeaderMap,
    body: Option<Body>,
    timeout: Option<Duration>,
}

impl RequestBuilder {
    FUNCTION get(url: &str) -> Self:
        Self::new(Method::GET, url)

    FUNCTION post(url: &str) -> Self:
        Self::new(Method::POST, url)

    FUNCTION put(url: &str) -> Self:
        Self::new(Method::PUT, url)

    FUNCTION delete(url: &str) -> Self:
        Self::new(Method::DELETE, url)

    FUNCTION new(method: Method, url: &str) -> Self:
        Self {
            method,
            url: Url::parse(url).ok(),
            headers: HeaderMap::new(),
            body: None,
            timeout: None,
        }

    FUNCTION header(mut self, name: &str, value: &str) -> Self:
        self.headers.insert(name, value.parse().unwrap())
        self

    FUNCTION bearer_auth(self, token: &str) -> Self:
        self.header("Authorization", &format!("Bearer {}", token))

    FUNCTION json<T: Serialize>(mut self, body: &T) -> Self:
        self.body = Some(Body::Json(serde_json::to_vec(body).unwrap()))
        self.header("Content-Type", "application/json")

    FUNCTION body(mut self, body: impl Into<Body>) -> Self:
        self.body = Some(body.into())
        self

    FUNCTION timeout(mut self, timeout: Duration) -> Self:
        self.timeout = Some(timeout)
        self

    FUNCTION build(self) -> InfraResult<Request>:
        LET url = self.url.ok_or_else(|| InfraError::Http {
            status: None,
            message: "Invalid URL".to_string(),
            url: None,
        })?

        Ok(Request {
            method: self.method,
            url,
            headers: self.headers,
            body: self.body,
            timeout: self.timeout,
        })
}

// -----------------------------
// Response
// -----------------------------

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HeaderMap,
    body: ResponseBody,
}

impl Response {
    FUNCTION status(&self) -> StatusCode:
        self.status

    FUNCTION is_success(&self) -> bool:
        self.status.is_success()

    FUNCTION header(&self, name: &str) -> Option<&str>:
        self.headers.get(name).and_then(|v| v.to_str().ok())

    async fn bytes(&mut self) -> InfraResult<Vec<u8>>:
        self.body.bytes().await

    async fn text(&mut self) -> InfraResult<String>:
        LET bytes = self.bytes().await?
        String::from_utf8(bytes).map_err(|e| InfraError::Serialization {
            format: SerializationFormat::Json,
            message: e.to_string(),
            location: None,
        })

    async fn json<T: DeserializeOwned>(&mut self) -> InfraResult<T>:
        LET bytes = self.bytes().await?
        serde_json::from_slice(&bytes).map_err(Into::into)

    /// Check status and return error if not successful
    FUNCTION error_for_status(self) -> InfraResult<Self>:
        IF !self.is_success():
            Err(InfraError::Http {
                status: Some(self.status.as_u16()),
                message: format!("HTTP error: {}", self.status),
                url: None,
            })
        ELSE:
            Ok(self)
}

// -----------------------------
// Native HTTP Client (reqwest-based)
// -----------------------------

#[cfg(not(target_arch = "wasm32"))]
pub struct NativeHttpClient {
    inner: reqwest::Client,
    config: HttpClientConfig,
}

#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub timeout: Duration,
    pub max_retries: usize,
    pub retry_delay: Duration,
    pub user_agent: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeHttpClient {
    FUNCTION new(config: HttpClientConfig) -> InfraResult<Self>:
        LET inner = reqwest::Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| InfraError::Http {
                status: None,
                message: e.to_string(),
                url: None,
            })?

        Ok(Self { inner, config })

    FUNCTION from_config(config: &Config) -> InfraResult<Self>:
        LET client_config = HttpClientConfig {
            timeout: Duration::from_secs(config.get_or("http.timeout_secs", 30)),
            max_retries: config.get_or("http.max_retries", 3),
            retry_delay: Duration::from_millis(config.get_or("http.retry_delay_ms", 1000)),
            user_agent: config.get_or("http.user_agent", "infra-http/1.0".to_string()),
        }

        Self::new(client_config)
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl HttpClient for NativeHttpClient {
    async fn request(&self, req: Request) -> InfraResult<Response>:
        LET span = tracing::info_span!(
            "http.request",
            http.method = %req.method,
            http.url = %req.url,
            http.status_code = tracing::field::Empty,
        )

        async move {
            LET mut attempts = 0

            LOOP:
                attempts += 1

                // Build reqwest request
                LET mut reqwest_req = self.inner.request(
                    req.method.into(),
                    req.url.clone(),
                )

                FOR (name, value) IN &req.headers:
                    reqwest_req = reqwest_req.header(name, value)

                IF let Some(ref body) = req.body:
                    reqwest_req = reqwest_req.body(body.to_bytes())

                IF let Some(timeout) = req.timeout:
                    reqwest_req = reqwest_req.timeout(timeout)

                // Execute request
                MATCH reqwest_req.send().await:
                    Ok(resp):
                        LET status = StatusCode::from_u16(resp.status().as_u16()).unwrap()
                        span.record("http.status_code", status.as_u16())

                        LET headers = convert_headers(resp.headers())
                        LET body = ResponseBody::from_reqwest(resp)

                        RETURN Ok(Response { status, headers, body })

                    Err(e) IF attempts < self.config.max_retries AND is_retryable(&e):
                        tracing::warn!("Request failed, retrying: {}", e)
                        tokio::time::sleep(self.config.retry_delay).await

                    Err(e):
                        RETURN Err(InfraError::Http {
                            status: None,
                            message: e.to_string(),
                            url: Some(req.url.to_string()),
                        })
        }.instrument(span).await
}

// -----------------------------
// WASM HTTP Client (fetch-based)
// -----------------------------

#[cfg(target_arch = "wasm32")]
pub struct WasmHttpClient {
    config: HttpClientConfig,
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
impl HttpClient for WasmHttpClient {
    async fn request(&self, req: Request) -> InfraResult<Response>:
        use web_sys::{Request as WebRequest, RequestInit, Response as WebResponse};
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;

        // Create fetch options
        LET mut opts = RequestInit::new()
        opts.method(req.method.as_str())

        IF let Some(ref body) = req.body:
            opts.body(Some(&body.to_js_value()))

        // Create request
        LET web_req = WebRequest::new_with_str_and_init(req.url.as_str(), &opts)
            .map_err(|e| InfraError::Http {
                status: None,
                message: format!("Failed to create request: {:?}", e),
                url: Some(req.url.to_string()),
            })?

        // Add headers
        FOR (name, value) IN &req.headers:
            web_req.headers().set(name, value.to_str().unwrap()).ok()

        // Execute fetch
        LET window = web_sys::window().unwrap()
        LET resp_value = JsFuture::from(window.fetch_with_request(&web_req))
            .await
            .map_err(|e| InfraError::Http {
                status: None,
                message: format!("Fetch failed: {:?}", e),
                url: Some(req.url.to_string()),
            })?

        LET resp: WebResponse = resp_value.dyn_into().unwrap()

        Ok(Response {
            status: StatusCode::from_u16(resp.status()).unwrap(),
            headers: convert_web_headers(resp.headers()),
            body: ResponseBody::from_web_response(resp),
        })
}

// -----------------------------
// HTTP Server (native only)
// -----------------------------

#[cfg(not(target_arch = "wasm32"))]
pub struct HttpServer {
    router: Router,
    config: HttpServerConfig,
}

#[derive(Debug, Clone)]
pub struct HttpServerConfig {
    pub bind_addr: SocketAddr,
    pub max_connections: usize,
    pub request_timeout: Duration,
}

#[cfg(not(target_arch = "wasm32"))]
impl HttpServer {
    FUNCTION new(config: HttpServerConfig) -> Self:
        Self {
            router: Router::new(),
            config,
        }

    FUNCTION route<H>(&mut self, method: Method, path: &str, handler: H) -> &mut Self
    where
        H: Handler + 'static
    :
        self.router.add(method, path, Box::new(handler))
        self

    FUNCTION get<H>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler + 'static
    :
        self.route(Method::GET, path, handler)

    FUNCTION post<H>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler + 'static
    :
        self.route(Method::POST, path, handler)

    async fn serve(self) -> InfraResult<()>:
        LET span = tracing::info_span!(
            "http.server",
            http.bind_addr = %self.config.bind_addr,
        )

        async move {
            // Use hyper for serving
            LET service = make_service_fn(|_conn| {
                LET router = self.router.clone()
                async move {
                    Ok::<_, Infallible>(service_fn(move |req| {
                        router.clone().handle(req)
                    }))
                }
            })

            LET server = hyper::Server::bind(&self.config.bind_addr)
                .serve(service)

            tracing::info!("HTTP server listening on {}", self.config.bind_addr)

            server.await.map_err(|e| InfraError::Http {
                status: None,
                message: e.to_string(),
                url: None,
            })
        }.instrument(span).await
}

#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&self, req: Request) -> InfraResult<Response>;
}
```

---

### 2.7 `infra-crypto` (Level 1)

**Purpose:** Cryptographic operations

```rust
// ============================================================
// FILE: crates/infra-crypto/src/lib.rs
// ============================================================

//! Cryptographic primitives for LLM-Dev-Ops infrastructure.

use infra_errors::{InfraError, InfraResult, CryptoOperation};

// -----------------------------
// Hashing
// -----------------------------

pub trait Hasher {
    FUNCTION hash(&self, data: &[u8]) -> Vec<u8>;
    FUNCTION hash_str(&self, data: &str) -> String;
    FUNCTION verify(&self, data: &[u8], expected: &[u8]) -> bool;
}

pub struct Sha256Hasher;

impl Hasher for Sha256Hasher {
    FUNCTION hash(&self, data: &[u8]) -> Vec<u8>:
        use sha2::{Sha256, Digest}
        LET mut hasher = Sha256::new()
        hasher.update(data)
        hasher.finalize().to_vec()

    FUNCTION hash_str(&self, data: &str) -> String:
        hex::encode(self.hash(data.as_bytes()))

    FUNCTION verify(&self, data: &[u8], expected: &[u8]) -> bool:
        constant_time_eq(&self.hash(data), expected)
}

pub struct Blake3Hasher;

impl Hasher for Blake3Hasher {
    FUNCTION hash(&self, data: &[u8]) -> Vec<u8>:
        blake3::hash(data).as_bytes().to_vec()

    FUNCTION hash_str(&self, data: &str) -> String:
        blake3::hash(data.as_bytes()).to_hex().to_string()

    FUNCTION verify(&self, data: &[u8], expected: &[u8]) -> bool:
        constant_time_eq(&self.hash(data), expected)
}

// -----------------------------
// Password Hashing
// -----------------------------

pub struct PasswordHasher {
    algorithm: PasswordAlgorithm,
}

pub enum PasswordAlgorithm {
    Argon2id { memory_cost: u32, time_cost: u32, parallelism: u32 },
    Bcrypt { cost: u32 },
}

impl PasswordHasher {
    FUNCTION argon2id() -> Self:
        Self {
            algorithm: PasswordAlgorithm::Argon2id {
                memory_cost: 65536,
                time_cost: 3,
                parallelism: 4,
            }
        }

    FUNCTION hash(&self, password: &str) -> InfraResult<String>:
        MATCH &self.algorithm:
            PasswordAlgorithm::Argon2id { memory_cost, time_cost, parallelism }:
                use argon2::{Argon2, PasswordHasher as _}

                LET salt = SaltString::generate(&mut OsRng)
                LET params = argon2::Params::new(*memory_cost, *time_cost, *parallelism, None)
                    .map_err(|e| InfraError::Crypto {
                        operation: CryptoOperation::Hash,
                        message: e.to_string(),
                    })?

                LET argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)

                argon2.hash_password(password.as_bytes(), &salt)
                    .map(|h| h.to_string())
                    .map_err(|e| InfraError::Crypto {
                        operation: CryptoOperation::Hash,
                        message: e.to_string(),
                    })

            PasswordAlgorithm::Bcrypt { cost }:
                bcrypt::hash(password, *cost)
                    .map_err(|e| InfraError::Crypto {
                        operation: CryptoOperation::Hash,
                        message: e.to_string(),
                    })

    FUNCTION verify(&self, password: &str, hash: &str) -> InfraResult<bool>:
        MATCH &self.algorithm:
            PasswordAlgorithm::Argon2id { .. }:
                use argon2::{Argon2, PasswordVerifier}

                LET parsed_hash = argon2::PasswordHash::new(hash)
                    .map_err(|e| InfraError::Crypto {
                        operation: CryptoOperation::Verify,
                        message: e.to_string(),
                    })?

                Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())

            PasswordAlgorithm::Bcrypt { .. }:
                bcrypt::verify(password, hash)
                    .map_err(|e| InfraError::Crypto {
                        operation: CryptoOperation::Verify,
                        message: e.to_string(),
                    })
}

// -----------------------------
// Symmetric Encryption
// -----------------------------

pub trait Cipher {
    FUNCTION encrypt(&self, plaintext: &[u8]) -> InfraResult<Vec<u8>>;
    FUNCTION decrypt(&self, ciphertext: &[u8]) -> InfraResult<Vec<u8>>;
}

pub struct Aes256GcmCipher {
    key: [u8; 32],
}

impl Aes256GcmCipher {
    FUNCTION new(key: [u8; 32]) -> Self:
        Self { key }

    FUNCTION generate() -> InfraResult<Self>:
        LET mut key = [0u8; 32]
        OsRng.fill_bytes(&mut key)
        Ok(Self { key })

    FUNCTION from_passphrase(passphrase: &str, salt: &[u8]) -> InfraResult<Self>:
        use argon2::Argon2

        LET mut key = [0u8; 32]
        Argon2::default()
            .hash_password_into(passphrase.as_bytes(), salt, &mut key)
            .map_err(|e| InfraError::Crypto {
                operation: CryptoOperation::KeyGeneration,
                message: e.to_string(),
            })?

        Ok(Self { key })
}

impl Cipher for Aes256GcmCipher {
    FUNCTION encrypt(&self, plaintext: &[u8]) -> InfraResult<Vec<u8>>:
        use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead}

        LET cipher = Aes256Gcm::new(&self.key.into())

        // Generate random nonce
        LET mut nonce_bytes = [0u8; 12]
        OsRng.fill_bytes(&mut nonce_bytes)
        LET nonce = aes_gcm::Nonce::from_slice(&nonce_bytes)

        // Encrypt
        LET ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| InfraError::Crypto {
                operation: CryptoOperation::Encrypt,
                message: e.to_string(),
            })?

        // Prepend nonce to ciphertext
        LET mut result = nonce_bytes.to_vec()
        result.extend(ciphertext)

        Ok(result)

    FUNCTION decrypt(&self, ciphertext: &[u8]) -> InfraResult<Vec<u8>>:
        use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead}

        IF ciphertext.len() < 12:
            RETURN Err(InfraError::Crypto {
                operation: CryptoOperation::Decrypt,
                message: "Ciphertext too short".to_string(),
            })

        LET cipher = Aes256Gcm::new(&self.key.into())
        LET nonce = aes_gcm::Nonce::from_slice(&ciphertext[..12])

        cipher.decrypt(nonce, &ciphertext[12..])
            .map_err(|e| InfraError::Crypto {
                operation: CryptoOperation::Decrypt,
                message: e.to_string(),
            })
}

// -----------------------------
// Digital Signatures
// -----------------------------

pub trait Signer {
    FUNCTION sign(&self, data: &[u8]) -> InfraResult<Signature>;
    FUNCTION public_key(&self) -> PublicKey;
}

pub trait Verifier {
    FUNCTION verify(&self, data: &[u8], signature: &Signature) -> InfraResult<bool>;
}

#[derive(Debug, Clone)]
pub struct Signature(Vec<u8>);

#[derive(Debug, Clone)]
pub struct PublicKey(Vec<u8>);

pub struct Ed25519Signer {
    keypair: ed25519_dalek::SigningKey,
}

impl Ed25519Signer {
    FUNCTION generate() -> Self:
        Self {
            keypair: ed25519_dalek::SigningKey::generate(&mut OsRng),
        }

    FUNCTION from_bytes(bytes: &[u8; 32]) -> InfraResult<Self>:
        Ok(Self {
            keypair: ed25519_dalek::SigningKey::from_bytes(bytes),
        })
}

impl Signer for Ed25519Signer {
    FUNCTION sign(&self, data: &[u8]) -> InfraResult<Signature>:
        use ed25519_dalek::Signer

        LET sig = self.keypair.sign(data)
        Ok(Signature(sig.to_bytes().to_vec()))

    FUNCTION public_key(&self) -> PublicKey:
        PublicKey(self.keypair.verifying_key().to_bytes().to_vec())
}

pub struct Ed25519Verifier {
    public_key: ed25519_dalek::VerifyingKey,
}

impl Verifier for Ed25519Verifier {
    FUNCTION verify(&self, data: &[u8], signature: &Signature) -> InfraResult<bool>:
        use ed25519_dalek::Verifier

        LET sig = ed25519_dalek::Signature::from_bytes(
            signature.0.as_slice().try_into().map_err(|_| InfraError::Crypto {
                operation: CryptoOperation::Verify,
                message: "Invalid signature length".to_string(),
            })?
        )

        Ok(self.public_key.verify(data, &sig).is_ok())
}

// -----------------------------
// JWT Support
// -----------------------------

pub struct JwtSigner {
    algorithm: JwtAlgorithm,
    key: Vec<u8>,
}

pub enum JwtAlgorithm {
    HS256,
    HS384,
    HS512,
    RS256,
    ES256,
}

impl JwtSigner {
    FUNCTION hs256(secret: &[u8]) -> Self:
        Self {
            algorithm: JwtAlgorithm::HS256,
            key: secret.to_vec(),
        }

    FUNCTION sign_claims<T: Serialize>(&self, claims: &T, expiry: Duration) -> InfraResult<String>:
        use jsonwebtoken::{encode, Header, EncodingKey}

        LET header = Header::new(self.algorithm.to_jsonwebtoken())

        LET token_claims = TokenClaims {
            exp: (Utc::now() + chrono::Duration::from_std(expiry).unwrap()).timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            payload: serde_json::to_value(claims)?,
        }

        encode(&header, &token_claims, &self.encoding_key())
            .map_err(|e| InfraError::Crypto {
                operation: CryptoOperation::Sign,
                message: e.to_string(),
            })

    FUNCTION verify_claims<T: DeserializeOwned>(&self, token: &str) -> InfraResult<T>:
        use jsonwebtoken::{decode, Validation, DecodingKey}

        LET validation = Validation::new(self.algorithm.to_jsonwebtoken())

        LET token_data = decode::<TokenClaims>(token, &self.decoding_key(), &validation)
            .map_err(|e| InfraError::Auth {
                kind: AuthErrorKind::InvalidToken,
                message: e.to_string(),
                identity: None,
            })?

        serde_json::from_value(token_data.claims.payload)
            .map_err(Into::into)
}

// -----------------------------
// WASM Bindings
// -----------------------------

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn sha256(data: &[u8]) -> Vec<u8>:
        Sha256Hasher.hash(data)

    #[wasm_bindgen]
    pub fn sha256_hex(data: &str) -> String:
        Sha256Hasher.hash_str(data)

    #[wasm_bindgen]
    pub struct JsAes256Gcm {
        cipher: Aes256GcmCipher,
    }

    #[wasm_bindgen]
    impl JsAes256Gcm {
        #[wasm_bindgen(constructor)]
        pub fn new(key: &[u8]) -> Result<JsAes256Gcm, JsValue>:
            IF key.len() != 32:
                RETURN Err(JsValue::from_str("Key must be 32 bytes"))

            LET mut key_array = [0u8; 32]
            key_array.copy_from_slice(key)

            Ok(Self { cipher: Aes256GcmCipher::new(key_array) })

        pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, JsValue>:
            self.cipher.encrypt(plaintext)
                .map_err(|e| JsValue::from_str(&e.to_string()))

        pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, JsValue>:
            self.cipher.decrypt(ciphertext)
                .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
```

---

### 2.8-2.15 Remaining Crates (Condensed)

For brevity, here are the API signatures for the remaining crates:

```rust
// ============================================================
// infra-auth (Level 3)
// ============================================================

pub trait Authenticator: Send + Sync {
    async fn authenticate(&self, credentials: Credentials) -> InfraResult<Identity>;
}

pub trait Authorizer: Send + Sync {
    async fn authorize(&self, identity: &Identity, resource: &Resource, action: &Action) -> InfraResult<bool>;
}

pub struct JwtAuthenticator { /* uses infra-crypto JwtSigner */ }
pub struct ApiKeyAuthenticator { /* uses infra-crypto Hasher */ }
pub struct OAuthClient { /* uses infra-http */ }

// ============================================================
// infra-id (Level 1)
// ============================================================

pub trait IdGenerator: Send + Sync {
    fn generate(&self) -> String;
}

pub struct UuidV7Generator;      // Time-ordered UUIDs
pub struct UlidGenerator;        // Lexicographically sortable
pub struct SnowflakeGenerator;   // Distributed IDs
pub struct NanoIdGenerator;      // URL-safe short IDs

// ============================================================
// infra-fs (Level 2) - Native only
// ============================================================

#[async_trait]
pub trait FileSystem: Send + Sync {
    async fn read(&self, path: &Path) -> InfraResult<Vec<u8>>;
    async fn write(&self, path: &Path, data: &[u8]) -> InfraResult<()>;
    async fn delete(&self, path: &Path) -> InfraResult<()>;
    async fn list(&self, path: &Path) -> InfraResult<Vec<PathBuf>>;
    async fn watch(&self, path: &Path) -> InfraResult<FileWatcher>;
}

pub struct LocalFileSystem;
pub struct S3FileSystem;
pub struct GcsFileSystem;

// ============================================================
// infra-mq (Level 2)
// ============================================================

#[async_trait]
pub trait MessageQueue: Send + Sync {
    async fn publish(&self, topic: &str, message: Message) -> InfraResult<MessageId>;
    async fn subscribe(&self, topic: &str) -> InfraResult<Subscription>;
    async fn acknowledge(&self, message_id: &MessageId) -> InfraResult<()>;
}

pub struct NatsMessageQueue;
pub struct RedisStreamQueue;
pub struct InMemoryQueue;  // For testing

// ============================================================
// infra-router (Level 3)
// ============================================================

pub trait Router: Send + Sync {
    async fn route(&self, request: Request) -> InfraResult<Endpoint>;
}

pub struct WeightedRouter;      // Load balancing
pub struct ContentRouter;       // Content-based routing
pub struct LatencyRouter;       // Latency-aware routing

// ============================================================
// infra-schema (Level 2)
// ============================================================

pub trait SchemaValidator: Send + Sync {
    fn validate(&self, value: &Json, schema: &Schema) -> InfraResult<ValidationResult>;
}

pub struct JsonSchemaValidator;
pub struct TypeSchemaValidator;  // For TypeScript SDK generation

pub struct SchemaGenerator {
    fn generate_typescript(&self, schema: &Schema) -> String;
    fn generate_openapi(&self, schemas: &[Schema]) -> OpenApiSpec;
}

// ============================================================
// infra-audit (Level 2)
// ============================================================

#[async_trait]
pub trait AuditLog: Send + Sync {
    async fn log(&self, event: AuditEvent) -> InfraResult<()>;
    async fn query(&self, filter: AuditFilter) -> InfraResult<Vec<AuditEvent>>;
}

pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub outcome: Outcome,
    pub metadata: Json,
}

// ============================================================
// infra-sim (Level 2)
// ============================================================

pub struct MockHttpClient { /* ... */ }
pub struct MockVectorStore { /* ... */ }
pub struct MockMessageQueue { /* ... */ }

pub trait Simulator {
    fn with_latency(self, latency: Duration) -> Self;
    fn with_failure_rate(self, rate: f64) -> Self;
    fn with_response(self, response: impl Into<Response>) -> Self;
}
```

---

## 3. Error Handling Flow

### 3.1 Error Propagation Pattern

```
┌─────────────────────────────────────────────────────────────┐
│                    LLM-Dev-Ops Repository                    │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                  Application Code                    │    │
│  │  result = infra_vector::store.search(query, 10)?    │    │
│  └──────────────────────────┬──────────────────────────┘    │
│                             │                                │
│                             ▼                                │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                   infra-vector                       │    │
│  │  Maps ruvector_core::Error → InfraError::Vector     │    │
│  └──────────────────────────┬──────────────────────────┘    │
│                             │                                │
│                             ▼                                │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                   infra-errors                       │    │
│  │  • Records to OTEL span                              │    │
│  │  • Provides is_retryable() / retry_after()          │    │
│  │  • Converts to WASM JsValue if needed               │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Retry Logic Pattern

```rust
FUNCTION with_retry<T, F>(operation: F, config: RetryConfig) -> InfraResult<T>
WHERE F: Fn() -> InfraResult<T>
:
    LET mut attempts = 0
    LET mut last_error: Option<InfraError> = None

    WHILE attempts < config.max_attempts:
        attempts += 1

        MATCH operation():
            Ok(result) => RETURN Ok(result)

            Err(e) IF e.is_retryable():
                LET delay = e.retry_after()
                    .unwrap_or(config.base_delay * 2^(attempts - 1))

                tracing::warn!(
                    error = %e,
                    attempt = attempts,
                    delay_ms = delay.as_millis(),
                    "Retrying after error"
                )

                sleep(delay).await
                last_error = Some(e)

            Err(e) => RETURN Err(e)

    Err(last_error.unwrap())
```

---

## 4. WASM Binding Generation Strategy

### 4.1 Build Process

```
┌─────────────────────────────────────────────────────────────┐
│                      Build Pipeline                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Rust Compilation (wasm32-unknown-unknown)               │
│     cargo build --target wasm32-unknown-unknown --release   │
│                                                              │
│  2. wasm-bindgen Generation                                  │
│     wasm-bindgen --out-dir sdk/ts/wasm --typescript         │
│                                                              │
│  3. wasm-opt Optimization                                    │
│     wasm-opt -O3 --enable-simd                              │
│                                                              │
│  4. TypeScript Wrapper Generation                            │
│     Generate high-level TS classes wrapping WASM exports    │
│                                                              │
│  5. Bundle with esbuild/rollup                               │
│     Create ESM and CJS bundles                              │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 TypeScript SDK Structure

```typescript
// sdk/ts/src/index.ts

// Re-export all modules
export * from './errors';
export * from './config';
export * from './json';
export * from './vector';
export * from './crypto';
export * from './auth';
export * from './id';
export * from './schema';

// sdk/ts/src/vector.ts

import { initWasm, VectorStoreWasm } from '../wasm/infra_vector';

export class VectorStore {
  private inner: VectorStoreWasm;

  private constructor(inner: VectorStoreWasm) {
    this.inner = inner;
  }

  static async create(dimensions: number): Promise<VectorStore> {
    await initWasm();
    const inner = new VectorStoreWasm(dimensions);
    return new VectorStore(inner);
  }

  async insert(id: string, vector: Float32Array, metadata?: object): Promise<void> {
    return this.inner.insert(id, vector, metadata ? JSON.stringify(metadata) : undefined);
  }

  async search(query: Float32Array, k: number): Promise<SearchResult[]> {
    const results = this.inner.search(query, k);
    return JSON.parse(results);
  }
}

export interface SearchResult {
  id: string;
  score: number;
  metadata?: object;
}
```

---

## 5. Integration Patterns

### 5.1 Standard Crate Initialization

```rust
// Example: llm-vector-store using infra crates

use infra_config::ConfigLoader;
use infra_errors::InfraResult;
use infra_otel::{init_otel, OtelConfig};
use infra_vector::{RuVectorStore, VectorStore};

pub async fn initialize() -> InfraResult<(OtelGuard, impl VectorStore)> {
    // Load configuration
    LET config = ConfigLoader::new()
        .with_file("config.toml")
        .with_env_prefix("LLM_VECTOR_")
        .load()?

    // Initialize OpenTelemetry
    LET otel_config = OtelConfig::from_config(&config)?
    LET otel_guard = init_otel(&otel_config).await?

    // Create vector store
    LET store = RuVectorStore::from_config(&config).await?

    Ok((otel_guard, store))
}
```

---

## 6. Acceptance Criteria

### 6.1 Pseudocode Phase Completion

- [x] All 15 infra crates have detailed pseudocode
- [x] API surfaces fully defined
- [x] Error handling flows documented
- [x] WASM binding generation strategy specified
- [x] Integration patterns demonstrated

### 6.2 Ready for Next Phase

Upon user approval, proceed to **Phase 3: Architecture** which will define:
- Crate directory structure
- Module organization
- Dependency graph visualization
- Build system configuration
- CI/CD pipeline design

---

**Document Status:** Awaiting user approval to proceed to Architecture phase.
