//! Schema validator.

use infra_errors::{InfraError, InfraResult};
use jsonschema::Validator;
use serde_json::Value;

/// Validation error detail
#[derive(Debug, Clone)]
pub struct ValidationErrorDetail {
    /// Path in the JSON document
    pub path: String,
    /// Error message
    pub message: String,
    /// Expected value or type
    pub expected: Option<String>,
    /// Actual value
    pub actual: Option<String>,
}

impl std::fmt::Display for ValidationErrorDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the data is valid
    valid: bool,
    /// Validation errors (if any)
    errors: Vec<ValidationErrorDetail>,
}

impl ValidationResult {
    /// Create a valid result
    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
        }
    }

    /// Create an invalid result
    pub fn invalid(errors: Vec<ValidationErrorDetail>) -> Self {
        Self {
            valid: false,
            errors,
        }
    }

    /// Check if the result is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Get validation errors
    pub fn errors(&self) -> &[ValidationErrorDetail] {
        &self.errors
    }

    /// Convert to InfraResult
    pub fn into_result(self) -> InfraResult<()> {
        if self.valid {
            Ok(())
        } else {
            let messages: Vec<String> = self.errors.iter().map(|e| e.to_string()).collect();
            Err(InfraError::Schema {
                schema_id: None,
                path: None,
                message: format!("Validation failed:\n  {}", messages.join("\n  ")),
                context: None,
            })
        }
    }
}

/// Schema validator
pub struct SchemaValidator {
    compiled: Validator,
}

impl SchemaValidator {
    /// Create a new validator from a JSON schema
    pub fn new(schema: &Value) -> InfraResult<Self> {
        let compiled = jsonschema::validator_for(schema).map_err(|e| InfraError::Schema {
            schema_id: None,
            path: None,
            message: format!("Failed to compile schema: {e}"),
            context: None,
        })?;

        Ok(Self { compiled })
    }

    /// Validate data against the schema
    pub fn validate(&self, data: &Value) -> ValidationResult {
        let result = self.compiled.validate(data);
        if result.is_ok() {
            ValidationResult::valid()
        } else {
            let error_details: Vec<ValidationErrorDetail> = self.compiled
                .iter_errors(data)
                .map(|e| ValidationErrorDetail {
                    path: e.instance_path.to_string(),
                    message: e.to_string(),
                    expected: None,
                    actual: None,
                })
                .collect();

            ValidationResult::invalid(error_details)
        }
    }

    /// Check if data is valid
    pub fn is_valid(&self, data: &Value) -> bool {
        self.compiled.is_valid(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validator() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer", "minimum": 0 }
            },
            "required": ["name"]
        });

        let validator = SchemaValidator::new(&schema).unwrap();

        // Valid data
        let data = json!({ "name": "John", "age": 30 });
        assert!(validator.is_valid(&data));

        // Missing required field
        let data = json!({ "age": 30 });
        assert!(!validator.is_valid(&data));

        // Invalid type
        let data = json!({ "name": 123 });
        assert!(!validator.is_valid(&data));

        // Invalid value (negative age)
        let data = json!({ "name": "John", "age": -1 });
        assert!(!validator.is_valid(&data));
    }

    #[test]
    fn test_validation_errors() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        });

        let validator = SchemaValidator::new(&schema).unwrap();
        let data = json!({ "name": 123 });

        let result = validator.validate(&data);
        assert!(!result.is_valid());
        assert!(!result.errors().is_empty());
    }
}
