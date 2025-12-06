//! Unified configuration management for LLM-Dev-Ops infrastructure.
//!
//! This crate provides hierarchical configuration loading with environment
//! variable overlay, validation, and hot-reload capabilities.

mod loader;
mod source;
mod validation;
mod builder;

#[cfg(feature = "wasm")]
mod wasm;

pub use loader::{ConfigLoader, ConfigFormat};
pub use source::{ConfigSource, EnvSource, FileSource, MemorySource};
pub use validation::{ConfigValidator, ValidationRule, ValidationError};
pub use builder::ConfigBuilder;

#[cfg(feature = "wasm")]
pub use wasm::*;

use infra_errors::{InfraError, InfraResult};
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;

/// Load configuration from a file
pub fn load_file<T: DeserializeOwned>(path: impl AsRef<Path>) -> InfraResult<T> {
    ConfigLoader::new()
        .add_source(FileSource::new(path))
        .load()
}

/// Load configuration from a file with environment overlay
pub fn load_with_env<T: DeserializeOwned>(
    path: impl AsRef<Path>,
    prefix: &str,
) -> InfraResult<T> {
    ConfigLoader::new()
        .add_source(FileSource::new(path))
        .add_source(EnvSource::with_prefix(prefix))
        .load()
}

/// Load configuration from environment variables only
pub fn load_env<T: DeserializeOwned>(prefix: &str) -> InfraResult<T> {
    ConfigLoader::new()
        .add_source(EnvSource::with_prefix(prefix))
        .load()
}

/// Parse configuration from a string
pub fn parse<T: DeserializeOwned>(content: &str, format: ConfigFormat) -> InfraResult<T> {
    match format {
        ConfigFormat::Json => serde_json::from_str(content).map_err(|e| InfraError::Config {
            key: None,
            message: format!("JSON parse error: {e}"),
            context: None,
        }),
        ConfigFormat::Toml => toml::from_str(content).map_err(|e| InfraError::Config {
            key: None,
            message: format!("TOML parse error: {e}"),
            context: None,
        }),
    }
}

/// Serialize configuration to a string
pub fn serialize<T: Serialize>(config: &T, format: ConfigFormat) -> InfraResult<String> {
    match format {
        ConfigFormat::Json => serde_json::to_string_pretty(config).map_err(|e| InfraError::Config {
            key: None,
            message: format!("JSON serialize error: {e}"),
            context: None,
        }),
        ConfigFormat::Toml => toml::to_string_pretty(config).map_err(|e| InfraError::Config {
            key: None,
            message: format!("TOML serialize error: {e}"),
            context: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        port: u16,
    }

    #[test]
    fn test_parse_json() {
        let json = r#"{"name": "test", "port": 8080}"#;
        let config: TestConfig = parse(json, ConfigFormat::Json).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_parse_toml() {
        let toml = r#"
name = "test"
port = 8080
"#;
        let config: TestConfig = parse(toml, ConfigFormat::Toml).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.port, 8080);
    }
}
