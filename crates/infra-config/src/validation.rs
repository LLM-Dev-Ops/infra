//! Configuration validation.

use infra_errors::{InfraError, InfraResult};
use std::collections::HashMap;

/// Validation error details
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub key: String,
    pub message: String,
    pub rule: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} (rule: {})", self.key, self.message, self.rule)
    }
}

/// Validation rule
pub enum ValidationRule {
    /// Value must be present
    Required,
    /// String must match regex
    Regex(String),
    /// Number must be in range
    Range { min: Option<f64>, max: Option<f64> },
    /// String length constraints
    Length { min: Option<usize>, max: Option<usize> },
    /// Value must be one of the allowed values
    OneOf(Vec<serde_json::Value>),
    /// Custom validation function
    Custom(Box<dyn Fn(&serde_json::Value) -> Result<(), String> + Send + Sync>),
}

impl std::fmt::Debug for ValidationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Required => write!(f, "Required"),
            Self::Regex(r) => write!(f, "Regex({r})"),
            Self::Range { min, max } => write!(f, "Range({min:?}, {max:?})"),
            Self::Length { min, max } => write!(f, "Length({min:?}, {max:?})"),
            Self::OneOf(values) => write!(f, "OneOf({values:?})"),
            Self::Custom(_) => write!(f, "Custom(fn)"),
        }
    }
}

/// Configuration validator
pub struct ConfigValidator {
    rules: HashMap<String, Vec<ValidationRule>>,
}

impl ConfigValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    /// Add a rule for a key
    pub fn rule(mut self, key: impl Into<String>, rule: ValidationRule) -> Self {
        self.rules
            .entry(key.into())
            .or_default()
            .push(rule);
        self
    }

    /// Add a required field
    pub fn required(self, key: impl Into<String>) -> Self {
        self.rule(key, ValidationRule::Required)
    }

    /// Add a range constraint
    pub fn range(self, key: impl Into<String>, min: Option<f64>, max: Option<f64>) -> Self {
        self.rule(key, ValidationRule::Range { min, max })
    }

    /// Add a length constraint
    pub fn length(self, key: impl Into<String>, min: Option<usize>, max: Option<usize>) -> Self {
        self.rule(key, ValidationRule::Length { min, max })
    }

    /// Validate a configuration value
    pub fn validate(&self, config: &serde_json::Value) -> InfraResult<()> {
        let mut errors = Vec::new();

        for (key, rules) in &self.rules {
            let value = get_nested_value(config, key);

            for rule in rules {
                if let Err(msg) = self.validate_rule(rule, value) {
                    errors.push(ValidationError {
                        key: key.clone(),
                        message: msg,
                        rule: format!("{rule:?}"),
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            Err(InfraError::Validation {
                field: None,
                message: format!("Configuration validation failed:\n  {}", messages.join("\n  ")),
                expected: None,
                actual: None,
                context: None,
            })
        }
    }

    fn validate_rule(
        &self,
        rule: &ValidationRule,
        value: Option<&serde_json::Value>,
    ) -> Result<(), String> {
        match rule {
            ValidationRule::Required => {
                if value.is_none() || value == Some(&serde_json::Value::Null) {
                    return Err("Value is required".to_string());
                }
            }
            ValidationRule::Regex(pattern) => {
                if let Some(serde_json::Value::String(s)) = value {
                    let re = regex::Regex::new(pattern)
                        .map_err(|e| format!("Invalid regex pattern: {e}"))?;
                    if !re.is_match(s) {
                        return Err(format!("Value does not match pattern: {pattern}"));
                    }
                }
            }
            ValidationRule::Range { min, max } => {
                if let Some(v) = value {
                    let num = v.as_f64().ok_or("Value is not a number")?;
                    if let Some(min_val) = min {
                        if num < *min_val {
                            return Err(format!("Value {num} is less than minimum {min_val}"));
                        }
                    }
                    if let Some(max_val) = max {
                        if num > *max_val {
                            return Err(format!("Value {num} is greater than maximum {max_val}"));
                        }
                    }
                }
            }
            ValidationRule::Length { min, max } => {
                if let Some(serde_json::Value::String(s)) = value {
                    let len = s.len();
                    if let Some(min_len) = min {
                        if len < *min_len {
                            return Err(format!("Length {len} is less than minimum {min_len}"));
                        }
                    }
                    if let Some(max_len) = max {
                        if len > *max_len {
                            return Err(format!("Length {len} is greater than maximum {max_len}"));
                        }
                    }
                }
            }
            ValidationRule::OneOf(allowed) => {
                if let Some(v) = value {
                    if !allowed.contains(v) {
                        return Err(format!("Value must be one of: {allowed:?}"));
                    }
                }
            }
            ValidationRule::Custom(f) => {
                if let Some(v) = value {
                    f(v)?;
                }
            }
        }

        Ok(())
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Get a nested value from JSON using dot notation
fn get_nested_value<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        match current.get(part) {
            Some(v) => current = v,
            None => return None,
        }
    }

    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_validation() {
        let validator = ConfigValidator::new()
            .required("name")
            .required("port");

        let config = serde_json::json!({
            "name": "test"
        });

        let result = validator.validate(&config);
        assert!(result.is_err());

        let config = serde_json::json!({
            "name": "test",
            "port": 8080
        });

        let result = validator.validate(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_range_validation() {
        let validator = ConfigValidator::new()
            .range("port", Some(1.0), Some(65535.0));

        let config = serde_json::json!({ "port": 80 });
        assert!(validator.validate(&config).is_ok());

        let config = serde_json::json!({ "port": 0 });
        assert!(validator.validate(&config).is_err());

        let config = serde_json::json!({ "port": 70000 });
        assert!(validator.validate(&config).is_err());
    }

    #[test]
    fn test_nested_value() {
        let config = serde_json::json!({
            "database": {
                "host": "localhost",
                "port": 5432
            }
        });

        let validator = ConfigValidator::new()
            .required("database.host")
            .range("database.port", Some(1.0), Some(65535.0));

        assert!(validator.validate(&config).is_ok());
    }
}
