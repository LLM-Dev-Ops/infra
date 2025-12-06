//! Fluent configuration builder.

use crate::loader::ConfigLoader;
use crate::source::{ConfigSource, EnvSource, FileSource, MemorySource};
use crate::validation::ConfigValidator;
use infra_errors::InfraResult;
use serde::de::DeserializeOwned;
use std::path::Path;

/// Fluent configuration builder
pub struct ConfigBuilder {
    loader: ConfigLoader,
    validator: Option<ConfigValidator>,
    defaults: MemorySource,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            loader: ConfigLoader::new(),
            validator: None,
            defaults: MemorySource::new().with_priority(0),
        }
    }

    /// Add a default value
    pub fn default(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.defaults = self.defaults.set(key, value);
        self
    }

    /// Load from a file
    pub fn file(mut self, path: impl AsRef<Path>) -> Self {
        self.loader = self.loader.add_source(FileSource::new(path));
        self
    }

    /// Load from environment variables
    pub fn env(mut self, prefix: &str) -> Self {
        self.loader = self.loader.add_source(EnvSource::with_prefix(prefix));
        self
    }

    /// Add a custom source
    pub fn source<S: ConfigSource + 'static>(mut self, source: S) -> Self {
        self.loader = self.loader.add_source(source);
        self
    }

    /// Set a validator
    pub fn validator(mut self, validator: ConfigValidator) -> Self {
        self.validator = Some(validator);
        self
    }

    /// Build the configuration
    pub fn build<T: DeserializeOwned>(self) -> InfraResult<T> {
        let loader = self.loader.add_source(self.defaults);

        if let Some(validator) = &self.validator {
            let raw = ConfigLoader::new().load_raw()?;
            validator.validate(&raw)?;
        }

        loader.load()
    }

    /// Build as raw JSON
    pub fn build_raw(self) -> InfraResult<serde_json::Value> {
        let loader = self.loader.add_source(self.defaults);

        if let Some(validator) = &self.validator {
            let raw = loader.load_raw()?;
            validator.validate(&raw)?;
            return Ok(raw);
        }

        loader.load_raw()
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
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
        debug: bool,
    }

    #[test]
    fn test_builder_defaults() {
        let config: TestConfig = ConfigBuilder::new()
            .default("name", "default-app")
            .default("port", 8080)
            .default("debug", false)
            .source(MemorySource::new().set("name", "my-app"))
            .build()
            .unwrap();

        assert_eq!(config.name, "my-app");
        assert_eq!(config.port, 8080);
        assert!(!config.debug);
    }
}
