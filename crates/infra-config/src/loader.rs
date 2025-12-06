//! Configuration loader.

use crate::source::ConfigSource;
use infra_errors::{InfraError, InfraResult};
use serde::de::DeserializeOwned;
use std::collections::HashMap;

/// Supported configuration formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Toml,
}

impl ConfigFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            _ => None,
        }
    }
}

/// Configuration loader with multiple sources
pub struct ConfigLoader {
    sources: Vec<Box<dyn ConfigSource>>,
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// Add a configuration source
    pub fn add_source<S: ConfigSource + 'static>(mut self, source: S) -> Self {
        self.sources.push(Box::new(source));
        self
    }

    /// Load and merge all sources into a configuration
    pub fn load<T: DeserializeOwned>(mut self) -> InfraResult<T> {
        // Sort sources by priority
        self.sources.sort_by_key(|s| s.priority());

        // Merge all sources
        let mut merged: HashMap<String, serde_json::Value> = HashMap::new();

        for source in &self.sources {
            let values = source.values()?;
            for (key, value) in values {
                merged.insert(key, value);
            }
        }

        // Convert flattened map back to nested structure
        let nested = unflatten_map(merged);

        // Deserialize
        serde_json::from_value(nested).map_err(|e| InfraError::Config {
            key: None,
            message: format!("Configuration deserialization error: {e}"),
            context: None,
        })
    }

    /// Load as raw JSON value
    pub fn load_raw(mut self) -> InfraResult<serde_json::Value> {
        self.sources.sort_by_key(|s| s.priority());

        let mut merged: HashMap<String, serde_json::Value> = HashMap::new();

        for source in &self.sources {
            let values = source.values()?;
            for (key, value) in values {
                merged.insert(key, value);
            }
        }

        Ok(unflatten_map(merged))
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a flattened map back to a nested JSON structure
fn unflatten_map(map: HashMap<String, serde_json::Value>) -> serde_json::Value {
    let mut root = serde_json::Map::new();

    for (key, value) in map {
        let parts: Vec<&str> = key.split('.').collect();
        insert_nested(&mut root, &parts, value);
    }

    serde_json::Value::Object(root)
}

/// Insert a value into a nested map structure
fn insert_nested(
    map: &mut serde_json::Map<String, serde_json::Value>,
    parts: &[&str],
    value: serde_json::Value,
) {
    if parts.is_empty() {
        return;
    }

    if parts.len() == 1 {
        map.insert(parts[0].to_string(), value);
        return;
    }

    let key = parts[0].to_string();
    let rest = &parts[1..];

    if !map.contains_key(&key) {
        map.insert(key.clone(), serde_json::Value::Object(serde_json::Map::new()));
    }

    if let Some(serde_json::Value::Object(ref mut inner)) = map.get_mut(&key) {
        insert_nested(inner, rest, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::MemorySource;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct DatabaseConfig {
        host: String,
        port: u16,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct AppConfig {
        name: String,
        database: DatabaseConfig,
    }

    #[test]
    fn test_loader_with_memory_source() {
        let config: AppConfig = ConfigLoader::new()
            .add_source(
                MemorySource::new()
                    .set("name", "my-app")
                    .set("database.host", "localhost")
                    .set("database.port", 5432),
            )
            .load()
            .unwrap();

        assert_eq!(config.name, "my-app");
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
    }

    #[test]
    fn test_source_priority() {
        let config: AppConfig = ConfigLoader::new()
            .add_source(
                MemorySource::new()
                    .set("name", "default")
                    .set("database.host", "localhost")
                    .set("database.port", 5432)
                    .with_priority(10),
            )
            .add_source(
                MemorySource::new()
                    .set("name", "override")
                    .with_priority(20),
            )
            .load()
            .unwrap();

        assert_eq!(config.name, "override");
        assert_eq!(config.database.host, "localhost");
    }

    #[test]
    fn test_unflatten_map() {
        let mut map = HashMap::new();
        map.insert("a.b.c".to_string(), serde_json::json!("value"));
        map.insert("a.b.d".to_string(), serde_json::json!(123));
        map.insert("x".to_string(), serde_json::json!("top"));

        let result = unflatten_map(map);

        assert_eq!(result["a"]["b"]["c"], "value");
        assert_eq!(result["a"]["b"]["d"], 123);
        assert_eq!(result["x"], "top");
    }
}
