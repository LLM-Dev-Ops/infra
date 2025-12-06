//! Configuration sources.

use infra_errors::{InfraError, InfraResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Trait for configuration sources
pub trait ConfigSource: Send + Sync {
    /// Get values as a map
    fn values(&self) -> InfraResult<HashMap<String, serde_json::Value>>;

    /// Priority (higher values override lower)
    fn priority(&self) -> i32 {
        0
    }

    /// Source name for debugging
    fn name(&self) -> &str;
}

/// File-based configuration source
pub struct FileSource {
    path: PathBuf,
}

impl FileSource {
    /// Create a new file source
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl ConfigSource for FileSource {
    fn values(&self) -> InfraResult<HashMap<String, serde_json::Value>> {
        let content = std::fs::read_to_string(&self.path).map_err(|e| InfraError::Config {
            key: None,
            message: format!("Failed to read config file '{}': {e}", self.path.display()),
            context: None,
        })?;

        let ext = self.path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let value: serde_json::Value = match ext {
            "json" => serde_json::from_str(&content).map_err(|e| InfraError::Config {
                key: None,
                message: format!("JSON parse error in '{}': {e}", self.path.display()),
                context: None,
            })?,
            "toml" => {
                let toml_value: toml::Value = toml::from_str(&content).map_err(|e| InfraError::Config {
                    key: None,
                    message: format!("TOML parse error in '{}': {e}", self.path.display()),
                    context: None,
                })?;
                toml_to_json(toml_value)
            }
            _ => {
                return Err(InfraError::Config {
                    key: None,
                    message: format!("Unsupported config format '{}' in file '{}'", ext, self.path.display()),
                    context: None,
                });
            }
        };

        flatten_json(value, "")
    }

    fn priority(&self) -> i32 {
        10
    }

    fn name(&self) -> &str {
        "file"
    }
}

/// Environment variable configuration source
pub struct EnvSource {
    prefix: String,
    separator: String,
}

impl EnvSource {
    /// Create with a prefix
    pub fn with_prefix(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_uppercase(),
            separator: "__".to_string(),
        }
    }

    /// Set the separator for nested keys
    #[must_use]
    pub fn separator(mut self, sep: &str) -> Self {
        self.separator = sep.to_string();
        self
    }
}

impl ConfigSource for EnvSource {
    fn values(&self) -> InfraResult<HashMap<String, serde_json::Value>> {
        let mut values = HashMap::new();
        let prefix_with_sep = format!("{}_", self.prefix);

        for (key, value) in std::env::vars() {
            if key.starts_with(&prefix_with_sep) {
                let config_key = key
                    .strip_prefix(&prefix_with_sep)
                    .unwrap()
                    .to_lowercase()
                    .replace(&self.separator, ".");

                // Try to parse as JSON value, fallback to string
                let json_value = if let Ok(v) = serde_json::from_str(&value) {
                    v
                } else {
                    serde_json::Value::String(value)
                };

                values.insert(config_key, json_value);
            }
        }

        Ok(values)
    }

    fn priority(&self) -> i32 {
        100 // Environment variables have highest priority
    }

    fn name(&self) -> &str {
        "environment"
    }
}

/// In-memory configuration source
pub struct MemorySource {
    values: HashMap<String, serde_json::Value>,
    priority: i32,
}

impl MemorySource {
    /// Create a new memory source
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            priority: 50,
        }
    }

    /// Set a value
    pub fn set(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.values.insert(key.into(), value.into());
        self
    }

    /// Set the priority
    #[must_use]
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

impl Default for MemorySource {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigSource for MemorySource {
    fn values(&self) -> InfraResult<HashMap<String, serde_json::Value>> {
        Ok(self.values.clone())
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    fn name(&self) -> &str {
        "memory"
    }
}

/// Convert TOML value to JSON value
fn toml_to_json(value: toml::Value) -> serde_json::Value {
    match value {
        toml::Value::String(s) => serde_json::Value::String(s),
        toml::Value::Integer(i) => serde_json::Value::Number(i.into()),
        toml::Value::Float(f) => {
            serde_json::Number::from_f64(f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        toml::Value::Boolean(b) => serde_json::Value::Bool(b),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(toml_to_json).collect())
        }
        toml::Value::Table(table) => {
            let map: serde_json::Map<String, serde_json::Value> = table
                .into_iter()
                .map(|(k, v)| (k, toml_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
    }
}

/// Flatten a JSON value into a map of dotted keys
fn flatten_json(
    value: serde_json::Value,
    prefix: &str,
) -> InfraResult<HashMap<String, serde_json::Value>> {
    let mut result = HashMap::new();

    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                let new_prefix = if prefix.is_empty() {
                    key
                } else {
                    format!("{prefix}.{key}")
                };

                if val.is_object() {
                    result.extend(flatten_json(val, &new_prefix)?);
                } else {
                    result.insert(new_prefix, val);
                }
            }
        }
        _ => {
            if !prefix.is_empty() {
                result.insert(prefix.to_string(), value);
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_source() {
        let source = MemorySource::new()
            .set("database.host", "localhost")
            .set("database.port", 5432);

        let values = source.values().unwrap();
        assert_eq!(values.get("database.host").unwrap(), "localhost");
        assert_eq!(values.get("database.port").unwrap(), 5432);
    }

    #[test]
    fn test_flatten_json() {
        let json = serde_json::json!({
            "database": {
                "host": "localhost",
                "port": 5432
            },
            "name": "test"
        });

        let flattened = flatten_json(json, "").unwrap();
        assert_eq!(flattened.get("database.host").unwrap(), "localhost");
        assert_eq!(flattened.get("database.port").unwrap(), 5432);
        assert_eq!(flattened.get("name").unwrap(), "test");
    }
}
