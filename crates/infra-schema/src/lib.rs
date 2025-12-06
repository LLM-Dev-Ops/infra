//! Schema validation for LLM-Dev-Ops infrastructure.
//!
//! This crate provides JSON Schema validation with detailed error reporting.

mod validator;
mod builder;
mod types;

pub use validator::{SchemaValidator, ValidationResult, ValidationErrorDetail};
pub use builder::SchemaBuilder;
pub use types::{SchemaType, Format};

use infra_errors::{InfraError, InfraResult};
use serde_json::Value;

/// Validate JSON against a schema
pub fn validate(schema: &Value, data: &Value) -> InfraResult<ValidationResult> {
    let validator = SchemaValidator::new(schema)?;
    Ok(validator.validate(data))
}

/// Validate JSON against a schema string
pub fn validate_str(schema: &str, data: &str) -> InfraResult<ValidationResult> {
    let schema: Value = serde_json::from_str(schema).map_err(|e| InfraError::Schema {
        schema_id: None,
        path: None,
        message: format!("Invalid schema JSON: {e}"),
        context: None,
    })?;

    let data: Value = serde_json::from_str(data).map_err(|e| InfraError::Schema {
        schema_id: None,
        path: None,
        message: format!("Invalid data JSON: {e}"),
        context: None,
    })?;

    validate(&schema, &data)
}

/// Quick check if data is valid against a schema
pub fn is_valid(schema: &Value, data: &Value) -> bool {
    validate(schema, data).map(|r| r.is_valid()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_valid() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" }
            },
            "required": ["name"]
        });

        let data = json!({
            "name": "John",
            "age": 30
        });

        let result = validate(&schema, &data).unwrap();
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_invalid() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        });

        let data = json!({
            "age": 30
        });

        let result = validate(&schema, &data).unwrap();
        assert!(!result.is_valid());
    }
}
